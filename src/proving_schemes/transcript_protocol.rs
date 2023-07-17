use std::borrow::Borrow;

use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar};
use merlin::Transcript;

/// Trait providing methods to append different types to a transcript
pub trait TranscriptProtocol {
    fn append_usize(&mut self, label: &'static [u8], value: usize);
    fn append_points_vector<I>(&mut self, label: &'static [u8], vec: I)
    where
        I: IntoIterator,
        I::Item: Borrow<CompressedRistretto>;
    fn append_point(
        &mut self,
        label: &'static [u8],
        point: &CompressedRistretto,
    );
    fn append_scalar(&mut self, label: &'static [u8], scalar: &Scalar);
    fn challenge_scalar(&mut self, label: &'static [u8]) -> Scalar;
}

impl TranscriptProtocol for Transcript {
    fn append_usize(&mut self, label: &'static [u8], value: usize) {
        let bytes = value.to_be_bytes();
        // Strip leading zero to obtain the same number of bytes regardless of the size of usize
        let cut_position = bytes
            .iter()
            .position(|byte| *byte != 0)
            .unwrap_or(bytes.len() - 1);
        self.append_message(label, &bytes[cut_position..]);
    }

    fn append_points_vector<I>(&mut self, label: &'static [u8], vec: I)
    where
        I: IntoIterator,
        I::Item: Borrow<CompressedRistretto>,
    {
        for point in vec {
            self.append_point(label, point.borrow());
        }
    }

    fn append_point(
        &mut self,
        label: &'static [u8],
        point: &CompressedRistretto,
    ) {
        self.append_message(label, point.as_bytes());
    }

    fn append_scalar(&mut self, label: &'static [u8], scalar: &Scalar) {
        self.append_message(label, scalar.as_bytes())
    }

    fn challenge_scalar(&mut self, label: &'static [u8]) -> Scalar {
        let mut buf = [0; 64];
        self.challenge_bytes(label, &mut buf);
        Scalar::from_bytes_mod_order_wide(&buf)
    }
}
