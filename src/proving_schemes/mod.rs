mod bulletproof;
mod transcript_protocol;

pub trait ProvingScheme<Point, CompressedPoint, Proof> {
    /// Compute the commitment of a slice of bytes interpreted as little endian Scalars
    fn compute_commitment(&self, bytes: &[[u8; 32]]) -> Self;

    /// Return the commpressed commitment of the node
    fn get_commitment(&self) -> CompressedPoint;

    /// Convert a compressed commitment in a byte array
    fn commitment_to_bytes(com: CompressedPoint) -> [u8; 32];

    /// Compute the proof of a node
    fn prove<Protocol>(&self) -> Proof;

    /// Verify the proof of a node
    fn verify(proof: &Proof, children: &[CompressedPoint]) -> bool;
}
