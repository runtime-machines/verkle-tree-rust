mod bulletproof;
mod transcript_protocol;

pub const MAX_GENERATORS: usize = 256;

pub trait ProvingScheme {
    type Scalar;
    type Commit;
    type Proof;

    // Instantiate the `ProvingScheme` instantiating a starting amount of generators
    fn instantiate_generators() -> Self;

    // Increase the generators' quantity
    fn add_new_generator(&self);

    /// Generate a polynomial and its commitment from slice of points
    fn compute_commitment(&self, bytes: &[[u8; 32]]) -> Self::Commit;

    /// Convert a compressed commitment in a byte array
    fn commitment_to_bytes(com: Self::Commit) -> [u8; 32];

    /// Compute the proof that the point (position, evaluation) is a node's child
    fn prove(&self, com: &Self::Commit, point: &(u64, [u8; 32]))
        -> Self::Proof;

    /// Verify that points the point (position, evaluation) is a node's child
    fn verify(
        &self,
        proof: &Self::Proof,
        children_count: usize,
        point: &(u64, [u8; 32]),
    ) -> bool;
}
