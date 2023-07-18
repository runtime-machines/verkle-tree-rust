mod polynomial;

use bulletproofs::{BulletproofGens, InnerProductProof};
use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_POINT,
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
    traits::MultiscalarMul,
};
use merlin::Transcript;

use self::polynomial::Polynomial;

use super::{
    transcript_protocol::TranscriptProtocol, ProvingScheme, MAX_GENERATORS,
};

struct BulletproofPS {
    gens: Vec<RistrettoPoint>,
}

type Node = (Polynomial, CompressedRistretto);

struct InnerProductProofCom {
    proof: InnerProductProof,
    com: CompressedRistretto,
}

// TODO: manage generators more efficiently

impl ProvingScheme<Node, CompressedRistretto, Scalar, InnerProductProofCom>
    for BulletproofPS
{
    fn instantiate_generators() -> BulletproofPS {
        let padded_length = (MAX_GENERATORS * 2).next_power_of_two();
        let bp_gens = BulletproofGens::new(padded_length, 1);
        BulletproofPS {
            gens: bp_gens.share(0).G(padded_length).cloned().collect(),
        }
    }

    fn compute_commitment(&self, bytes: &[[u8; 32]]) -> Node {
        let points = compute_points(bytes);
        // Lagrange interpolation
        let polynomial = Polynomial::lagrange(&points);

        // Commit
        let commitment = RistrettoPoint::multiscalar_mul(
            &polynomial.0,
            &self.gens[..points.len()],
        )
        .compress();

        (polynomial, commitment)
    }

    fn prove<BulletproofPS>(
        &self,
        (polynomial, _): &Node,
        point: &(u64, [u8; 32]),
    ) -> InnerProductProofCom {
        let mut transcript = Transcript::new(b"InnerProductNode");
        let q = (transcript.challenge_scalar(b"w")) * RISTRETTO_BASEPOINT_POINT;

        let n = polynomial.0.len();

        let g_h_factors = vec![Scalar::one(); n];

        let (g_vec, h_vec) = split_gens(&self.gens[..(n * 2)]);

        let a_vec = &polynomial.0;
        let b_vec = compute_b_vec(n, point.0);

        let com = RistrettoPoint::multiscalar_mul(
            a_vec.iter().chain(&b_vec),
            g_vec[..n].iter().chain(&h_vec[..n]),
        )
        .compress();

        let proof = InnerProductProof::create(
            &mut transcript,
            &q,
            &g_h_factors,
            &g_h_factors,
            g_vec[..n].to_vec(),
            h_vec[..n].to_vec(),
            a_vec.to_vec(),
            b_vec,
        );

        InnerProductProofCom { proof, com }
    }

    // TODO: ErrorHandling
    // TODO: correct to share generators?
    fn verify(
        &self,
        InnerProductProofCom { proof, com }: &InnerProductProofCom,
        children_count: usize,
        _point: &(u64, [u8; 32]),
    ) -> bool {
        let mut transcript = Transcript::new(b"InnerProductNode");
        let q = (transcript.challenge_scalar(b"w")) * RISTRETTO_BASEPOINT_POINT;

        let n = children_count.next_power_of_two();

        let vec_one = vec![Scalar::one(); n];

        // TODO: handle error
        let com = com.decompress().unwrap();

        let p = com + q;

        let (g_vec, h_vec) = split_gens(&self.gens[..(n * 2)]);

        proof
            .verify(
                n,
                &mut transcript,
                &vec_one,
                &vec_one,
                &p,
                &q,
                &g_vec,
                &h_vec,
            )
            .is_ok()
    }

    fn commitment_to_bytes(com: CompressedRistretto) -> [u8; 32] {
        com.to_bytes()
    }

    fn add_new_generator(&self) {
        todo!()
    }
}

fn compute_points(bytes: &[[u8; 32]]) -> Vec<(Scalar, Scalar)> {
    let points: Vec<_> = bytes
        .iter()
        .enumerate()
        .map(|(index, &byte)| point_into_scalar_point(&(index as u64, byte)))
        .collect();

    padding_points(&points)
}

fn point_into_scalar_point(
    &(position, evaluation): &(u64, [u8; 32]),
) -> (Scalar, Scalar) {
    (
        Scalar::from(position),
        Scalar::from_bytes_mod_order(evaluation),
    )
}

fn padding_points(points: &[(Scalar, Scalar)]) -> Vec<(Scalar, Scalar)> {
    points
        .iter()
        .copied()
        .chain(
            (points.len()..points.len().next_power_of_two())
                .map(|index| (Scalar::from(index as u64), Scalar::zero())),
        )
        .collect()
}

fn split_gens(
    gens: &[RistrettoPoint],
) -> (Vec<RistrettoPoint>, Vec<RistrettoPoint>) {
    let (g_vec, h_vec) = gens.split_at((gens.len() + 1) / 2);
    (g_vec.to_vec(), h_vec.to_vec())
}

fn compute_b_vec(length: usize, position: u64) -> Vec<Scalar> {
    // TODO: handle types error
    let length: u32 = length.try_into().unwrap();
    (0..length)
        .map(|index| Scalar::from(position.pow(index)))
        .collect()
}

#[cfg(test)]
mod test {
    use curve25519_dalek::scalar::Scalar;

    use crate::proving_schemes::{
        bulletproof::polynomial::Polynomial, ProvingScheme,
    };

    use super::BulletproofPS;

    fn generate_bytes() -> Vec<[u8; 32]> {
        vec![
            [
                0x90, 0x76, 0x33, 0xfe, 0x1c, 0x4b, 0x66, 0xa4, 0xa2, 0x8d,
                0x2d, 0xd7, 0x67, 0x83, 0x86, 0xc3, 0x53, 0xd0, 0xde, 0x54,
                0x55, 0xd4, 0xfc, 0x9d, 0xe8, 0xef, 0x7a, 0xc3, 0x1f, 0x35,
                0xbb, 0x05,
            ],
            [
                0x6c, 0x33, 0x74, 0xa1, 0x89, 0x4f, 0x62, 0x21, 0x0a, 0xaa,
                0x2f, 0xe1, 0x86, 0xa6, 0xf9, 0x2c, 0xe0, 0xaa, 0x75, 0xc2,
                0x77, 0x95, 0x81, 0xc2, 0x95, 0xfc, 0x08, 0x17, 0x9a, 0x73,
                0x94, 0x0c,
            ],
            [
                0xef, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c,
                0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x10,
            ],
        ]
    }

    fn generate_positions() -> Vec<(Scalar, Scalar)> {
        generate_bytes()
            .iter()
            .enumerate()
            .map(|(position, &evaluation)| {
                (
                    Scalar::from(position as u128),
                    Scalar::from_bytes_mod_order(evaluation),
                )
            })
            .collect()
    }

    #[test]
    fn correct_polynomial_construction() {
        let bytes = generate_positions();

        let polynomial = Polynomial::lagrange(&bytes);

        assert_eq!(polynomial.eval(&Scalar::from(0_u128)), bytes[0].1);
        assert_eq!(polynomial.eval(&Scalar::from(1_u128)), bytes[1].1);
        assert_eq!(polynomial.eval(&Scalar::from(2_u128)), bytes[2].1);
    }

    #[test]
    fn correct_prove_verification() {
        let scheme = BulletproofPS::instantiate_generators();
        let bytes: Vec<[u8; 32]> = generate_bytes();

        let node = scheme.compute_commitment(&bytes);

        let proof = scheme.prove::<BulletproofPS>(&node, &(1, bytes[1]));

        assert!(scheme.verify(&proof, bytes.len(), &(1, bytes[1])));
    }
}
