pub trait Committer {
    type Commit;

    //number of bytes should be less then prover max filed exponent
    fn from_bytes_to_scalar(&self, input: &[u8]) -> &[u8];

    fn commit(&self, input: Vec<&[u8]>) -> &Self;

    fn consume_commit(&self) -> Self::Commit;

    fn prove(&self) -> ();

    fn verify(&self) -> ();
}
