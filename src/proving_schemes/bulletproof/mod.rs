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

use super::{transcript_protocol::TranscriptProtocol, ProvingScheme};

struct BulletproofPS {
    gens: Vec<RistrettoPoint>,
    polynomial: Polynomial,
    commitment: CompressedRistretto,
}

struct InnerProduct {
    proof: InnerProductProof,
    com: RistrettoPoint,
    result: Scalar,
}

impl ProvingScheme<RistrettoPoint, CompressedRistretto, InnerProduct>
    for BulletproofPS
{
    fn compute_commitment(&self, bytes: &[[u8; 32]]) -> BulletproofPS {
        // Compute generators
        let gens = compute_gens(bytes.len());

        let points: Vec<_> = bytes
            .iter()
            .enumerate()
            .map(|(index, &byte)| {
                (
                    Scalar::from_bytes_mod_order(byte),
                    Scalar::from(index as u64),
                )
            })
            .collect();

        // Lagrange interpolation
        let polynomial = Polynomial::lagrange(&points);

        // Commit
        let commitment =
            RistrettoPoint::multiscalar_mul(&self.polynomial.0, &gens)
                .compress();

        BulletproofPS {
            gens,
            polynomial,
            commitment,
        }
    }

    fn prove<Protocol>(&self) -> InnerProduct {
        let mut transcript = Transcript::new(b"InnerProductNode");
        let n = self.polynomial.0.len();
        let vec_one = vec![Scalar::one(); n];
        let q = (transcript.challenge_scalar(b"w")) * RISTRETTO_BASEPOINT_POINT;
        let (g_vec, h_vec) = split_gens(&self.gens);

        let proof = InnerProductProof::create(
            &mut transcript,
            &q,
            &vec_one,
            &vec_one,
            g_vec[..n].to_vec(),
            h_vec[..n].to_vec(),
            self.polynomial.0.clone(),
            vec_one.to_vec(),
        );

        let result = inner_product(&self.polynomial.0, &vec_one);

        let com = RistrettoPoint::multiscalar_mul(
            self.polynomial.0.iter().chain(vec_one.iter()),
            g_vec.iter().chain(h_vec.iter()),
        );

        InnerProduct { proof, result, com }
    }

    // TODO: ErrorHandling
    fn verify(
        InnerProduct { proof, result, com }: &InnerProduct,
        children: &[CompressedRistretto],
    ) -> bool {
        let mut transcript = Transcript::new(b"InnerProductNode");
        let number_of_children = children.len();
        let vec_one = vec![Scalar::one(); number_of_children];
        let q = (transcript.challenge_scalar(b"w")) * RISTRETTO_BASEPOINT_POINT;
        let p = com + q * result;
        let (g_vec, h_vec) = split_gens(&compute_gens(number_of_children));

        proof
            .verify(
                number_of_children,
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

    fn get_commitment(&self) -> CompressedRistretto {
        self.commitment
    }

    fn commitment_to_bytes(com: CompressedRistretto) -> [u8; 32] {
        com.to_bytes()
    }
}

fn compute_gens(children_count: usize) -> Vec<RistrettoPoint> {
    let children_count = ((children_count + 1) / 2) * 2;
    let bp_gens = BulletproofGens::new(1, children_count);
    (0..children_count)
        .flat_map(|share| bp_gens.share(share).G(1))
        .copied()
        .collect()
}

fn split_gens(
    gens: &[RistrettoPoint],
) -> (Vec<RistrettoPoint>, Vec<RistrettoPoint>) {
    let (g_vec, h_vec) = gens.split_at((gens.len() + 1) / 2);
    (g_vec.to_vec(), h_vec.to_vec())
}

#[test]
fn correct_compute_gens() {
    // Even gens length
    let gens = compute_gens(6);
    let (g_vec, h_vec) = split_gens(&gens);
    assert_eq!(g_vec.len(), 3);
    assert_eq!(h_vec.len(), 3);

    // Odd gens length
    let gens = compute_gens(5);
    let (g_vec, h_vec) = split_gens(&gens);
    assert_eq!(g_vec.len(), 3);
    assert_eq!(h_vec.len(), 3);
}
