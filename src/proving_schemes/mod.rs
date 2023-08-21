mod bulletproof;
mod transcript_protocol;

pub const MAX_GENERATORS: usize = 256;

pub trait ProvingScheme {
    type Scalar;
    type Commit;
    type Proof;

    // Return a `ProvingScheme` by instantiating a starting amount of generators
    fn instantiate_generators() -> Self;

    // Increase the generators' quantity
    fn add_new_generator(&mut self);

    /// Generate a polynomial and its commitment from slice of bytes
    fn compute_commitment(
        &self,
        children: &[Self::Scalar],
    ) -> (Vec<Self::Scalar>, Self::Commit);

    /// Convert a slice of bytes into a scalar (field element)
    fn from_bytes_to_scalar(input: &[u8]) -> Self::Scalar;

    /// Convert a point (group element) into a scalar (field element)
    fn from_commitment_to_scalar(input: &Self::Commit) -> Self::Scalar;

    /// Compute the proof that the evaluation at a given position is a node's child
    fn prove(
        &self,
        polynomial_coefficients: &[Self::Scalar],
        position: u128,
        evaluation: Self::Scalar,
    ) -> Self::Proof;

    /// Verify that points the evaluation at a given position is a node's child
    fn verify(
        &self,
        commitment: &Self::Commit,
        proof: &Self::Proof,
        children_count: usize,
        position: u128,
        evaluation: Self::Scalar,
    ) -> bool;
}
