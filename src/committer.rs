pub trait Committer {
    type Point;
    type Scalar;

    //number of bytes should be less then prover max filed exponent
    fn from_bytes_to_scalar(&self, input: &[u8]) -> &Self::Scalar;
    fn from_point_to_scalar(&self, input: &Self::Point) -> &Self::Scalar;

    fn commit(&self, input: Vec<&Self::Scalar>) -> &Self;

    fn consume_commit(&self) -> Self::Point;

    fn prove(&self) -> ();

    fn verify(&self) -> ();
}
