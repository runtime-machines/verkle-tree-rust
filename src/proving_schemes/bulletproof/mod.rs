mod polynomial;

use bulletproofs::{inner_product, BulletproofGens, InnerProductProof};
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

/// Represent the `ProvingScheme` based on the Bulletproof protocol
struct BulletproofPS {
    gens: Vec<RistrettoPoint>,
}

/// Represent the commitment
type Node = (Polynomial, CompressedRistretto);

type ScalarPolynomialPoint = (Scalar, Scalar);

/// Represent the info related to the IPA such as;
/// - the proof
/// - the vector commitment of the vector a
/// - the value of the inner product
struct InnerProduct {
    proof: InnerProductProof,
    com: CompressedRistretto,
    value: Scalar,
}

// TODO: handle errors
// TODO: manage generators more efficiently

impl ProvingScheme for BulletproofPS {
    type Scalar = Scalar;
    type Commit = Node;
    type Proof = InnerProduct;
    type PolynomialPoint = (u128, [u8; 32]);

    fn instantiate_generators() -> BulletproofPS {
        BulletproofPS {
            gens: create_gens(MAX_GENERATORS * 2),
        }
    }

    fn compute_commitment(&self, bytes: &[[u8; 32]]) -> Node {
        let points = compute_scalar_polynomial_points(bytes);

        let polynomial = Polynomial::lagrange(&points);

        let commitment = RistrettoPoint::multiscalar_mul(
            &polynomial.0,
            &self.gens[..points.len()],
        )
        .compress();

        (polynomial, commitment)
    }

    fn prove(
        &self,
        (polynomial, _): &Node,
        polynomial_point: &Self::PolynomialPoint,
    ) -> InnerProduct {
        let mut transcript = Transcript::new(b"InnerProductNode");
        let q = (transcript.challenge_scalar(b"w")) * RISTRETTO_BASEPOINT_POINT;

        let n = polynomial.0.len();

        let g_h_factors = vec![Scalar::one(); n];

        let (g_vec, h_vec) = split_gens(&self.gens[..(n * 2)]);

        let a_vec = &polynomial.0;
        let b_vec = compute_b_vec(n, polynomial_point.0);

        let com =
            RistrettoPoint::multiscalar_mul(a_vec, &g_vec[..n]).compress();

        let value = inner_product(a_vec, &b_vec);

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

        InnerProduct { proof, com, value }
    }

    fn verify(
        &self,
        InnerProduct { proof, com, value }: &InnerProduct,
        children_count: usize,
        polynomial_point: &Self::PolynomialPoint,
    ) -> bool {
        let mut transcript = Transcript::new(b"InnerProductNode");
        let q = (transcript.challenge_scalar(b"w")) * RISTRETTO_BASEPOINT_POINT;

        let n = children_count.next_power_of_two();

        let g_h_factors = vec![Scalar::one(); n];

        let (g_vec, h_vec) = split_gens(&self.gens[..(n * 2)]);

        let a_com = com.decompress().unwrap();
        let b_vec = compute_b_vec(n, polynomial_point.0);
        let b_com = RistrettoPoint::multiscalar_mul(b_vec, &h_vec[..n]);
        let p = a_com + b_com + (q * value);

        proof
            .verify(
                n,
                &mut transcript,
                &g_h_factors,
                &g_h_factors,
                &p,
                &q,
                g_vec,
                h_vec,
            )
            .is_ok()
    }

    fn commitment_to_bytes(com: Node) -> [u8; 32] {
        com.1.to_bytes()
    }

    fn add_new_generator(&mut self) {
        self.gens = create_gens(self.gens.len() + 1);
    }
}

fn create_gens(gens_capacity: usize) -> Vec<RistrettoPoint> {
    let padded_length = gens_capacity.next_power_of_two();
    let bp_gens = BulletproofGens::new(padded_length, 1);
    bp_gens.share(0).G(padded_length).cloned().collect()
}

fn compute_scalar_polynomial_points(
    bytes: &[[u8; 32]],
) -> Vec<ScalarPolynomialPoint> {
    let scalar_polynomial_points: Vec<_> = bytes
        .iter()
        .enumerate()
        .map(|(index, &byte)| {
            create_scalar_polynomial_point(index as u128, byte)
        })
        .collect();

    padding_scalar_polynomial_points(&scalar_polynomial_points)
}

fn create_scalar_polynomial_point(
    position: u128,
    evaluation: [u8; 32],
) -> ScalarPolynomialPoint {
    (
        Scalar::from(position),
        Scalar::from_bytes_mod_order(evaluation),
    )
}

fn padding_scalar_polynomial_points(
    scalar_polynomial_points: &[ScalarPolynomialPoint],
) -> Vec<ScalarPolynomialPoint> {
    scalar_polynomial_points
        .iter()
        .copied()
        .chain(
            (scalar_polynomial_points.len()
                ..scalar_polynomial_points.len().next_power_of_two())
                .map(|index| (Scalar::from(index as u64), Scalar::zero())),
        )
        .collect()
}

fn split_gens(
    gens: &[RistrettoPoint],
) -> (&[RistrettoPoint], &[RistrettoPoint]) {
    let (g_vec, h_vec) = gens.split_at((gens.len() + 1) / 2);
    (g_vec, h_vec)
}

fn compute_b_vec(length: usize, position: u128) -> Vec<Scalar> {
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

    use super::{BulletproofPS, ScalarPolynomialPoint};

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

    fn generate_positions() -> Vec<ScalarPolynomialPoint> {
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

        let proof = scheme.prove(&node, &(1, bytes[1]));

        assert!(scheme.verify(&proof, bytes.len(), &(1, bytes[1])));
    }
}
