use std::io;

/// Trait defining an encoder for ASN.1.
pub trait Decoder: Clone {
	/// Whether the encoding is canonical or not.
	const CANONICAL: bool;
}

/// Trait for a type to be encodable for a given encoder.
pub trait Decode<T> {
	/// Encode self for the given encoder, writes the encoded output to the
	/// passed writer.
	fn decode<R>(&mut self, reader: &mut R) -> crate::Result<T>
		where R: io::Read + ?Sized;
}
