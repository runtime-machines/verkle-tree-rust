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

    // Odd gens length
    let gens = compute_gens(5);
    let (g_vec, h_vec) = split_gens(&gens);
    assert_eq!(g_vec.len(), 3);
    assert_eq!(h_vec.len(), 3);
}
