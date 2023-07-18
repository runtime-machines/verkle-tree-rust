pub trait Committer {
    type Commit;
    type Scalar;

    //number of bytes should be less then prover max filed exponent
    fn from_bytes_to_scalar(&self, input: &[u8]) -> &Self::Scalar;

    fn commit(&self, input: Vec<&Self::Scalar>) -> &Self;

    fn consume_commit(&self) -> Self::Commit;

    fn prove(&self) -> ();

    fn verify(&self) -> ();
}
