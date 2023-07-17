use merlin::Transcript;

mod bulletproof;
mod transcript_protocol;

pub trait ProvingScheme<Point, Proof> {
    fn prove<Protocol>(&self, transcript: &mut Transcript) -> Proof;
    fn verify(
        &self,
        gens: &[Point],
        transcript: &mut Transcript,
        proof: &Proof,
    ) -> bool;
}
