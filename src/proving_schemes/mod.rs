mod bulletproof;
mod transcript_protocol;

pub const MAX_GENERATORS: usize = 256;

pub trait ProvingScheme {
    type Scalar;
    type Commit;
    type Proof;
    type PolynomialPoint;

    // Return a `ProvingScheme` by instantiating a starting amount of generators
    fn instantiate_generators() -> Self;

    // Increase the generators' quantity
    fn add_new_generator(&mut self);

    /// Generate a polynomial and its commitment from slice of bytes
    fn compute_commitment(&self, children: &[Self::Scalar]) -> Self::Commit;

    /// Convert a slice of bytes into a scalar (field element)
    fn from_bytes_to_scalar(input: &[u8]) -> Self::Scalar;

    /// Convert a point (group element) into a scalar (field element)
    fn from_commitment_to_scalar(input: &Self::Commit) -> Self::Scalar;

    /// Convert a compressed commitment in a byte array
    fn commitment_to_bytes(com: Self::Commit) -> [u8; 32];

    /// Compute the proof that the point (position, evaluation) is a node's child
    fn prove(
        &self,
        com: &Self::Commit,
        polynomial_point: &Self::PolynomialPoint,
    ) -> Self::Proof;

    /// Verify that points the point (position, evaluation) is a node's child
    fn verify(
        &self,
        proof: &Self::Proof,
        children_count: usize,
        polynomial_point: &Self::PolynomialPoint,
    ) -> bool;
}
