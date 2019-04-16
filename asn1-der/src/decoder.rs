use core::{Decode, Decoder as Super, Result};
#[derive(Copy, Clone, Debug, Default)]
pub struct Decoder;

impl Super for Decoder {
    const CANONICAL: bool = true;
}

impl Decoder {
	pub fn from_bytes<T: Decode>(bytes: &[u8]) -> Result<T> {

	}
}

