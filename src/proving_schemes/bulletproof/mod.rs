mod polynomial;

use bulletproofs::{BulletproofGens, InnerProductProof};
use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_POINT,
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
    traits::MultiscalarMul,
};
use merlin::Transcript;
use sha2::Sha512;

use self::polynomial::Polynomial;

use super::{transcript_protocol::TranscriptProtocol, ProvingScheme};

struct BulletproofPS {
    gens: Vec<RistrettoPoint>,
    polynomial: Polynomial,
    commitment: RistrettoPoint,
}

impl ProvingScheme<RistrettoPoint, InnerProductProof> for BulletproofPS {
    fn prove<Protocol>(
        &self,
        transcript: &mut Transcript,
    ) -> InnerProductProof {
        let n = self.polynomial.0.len();
        let vec_one = vec![Scalar::one(); n];
        let q = (transcript.challenge_scalar(b"w")) * RISTRETTO_BASEPOINT_POINT;
        let (g_vec, h_vec) = split_gens(&self.gens);

        InnerProductProof::create(
            transcript,
            &q,
            &vec_one,
            &vec_one,
            g_vec[..n].to_vec(),
            h_vec[..n].to_vec(),
            self.polynomial.0.clone(),
            vec_one.to_vec(),
        )
    }

    // TODO: ErrorHandling
    fn verify(
        &self,
        gens: &[RistrettoPoint],
        transcript: &mut Transcript,
        proof: &InnerProductProof,
    ) -> bool {
        let number_of_children = todo!();
        let vec_one = vec![Scalar::one(); self.polynomial.0.len()];
        let q = (transcript.challenge_scalar(b"w")) * RISTRETTO_BASEPOINT_POINT;
        let p = todo!();
        let (g_vec, h_vec) = split_gens(gens);
        proof
            .verify(
                number_of_children,
                transcript,
                &vec_one,
                &vec_one,
                p,
                &q,
                &g_vec,
                &h_vec,
            )
            .is_ok()
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

fn hash_com_to_scalar(com: &CompressedRistretto) -> Scalar {
    Scalar::hash_from_bytes::<Sha512>(com.as_bytes())
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
