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
        let vec_one = vec![Scalar::one(); self.polynomial.0.len()];
        let q = (transcript.challenge_scalar(b"w")) * RISTRETTO_BASEPOINT_POINT;
        // TODO: split in the middle
        let (g_vec, h_vec) = self.gens.split_at(self.gens.len() / 2);

        InnerProductProof::create(
            transcript,
            &q,
            &vec_one,
            &vec_one,
            g_vec.to_vec(),
            h_vec.to_vec(),
            self.polynomial.0,
            vec_one,
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
        // TODO: split in the middle
        let (g_vec, h_vec) = self.gens.split_at(self.gens.len() / 2);
        proof
            .verify(
                number_of_children,
                transcript,
                &vec_one,
                &vec_one,
                p,
                &q,
                g_vec,
                h_vec,
            )
            .is_ok()
    }
}

fn compute_gens(children_count: usize) -> Vec<RistrettoPoint> {
    let bp_gens = BulletproofGens::new(1, children_count);
    (0..children_count)
        .flat_map(|share| bp_gens.share(share).G(1))
        .copied()
        .collect()
}

fn hash_com_to_scalar(com: &CompressedRistretto) -> Scalar {
    Scalar::hash_from_bytes::<Sha512>(com.as_bytes())
}
