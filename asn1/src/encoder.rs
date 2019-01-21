use std::io;

/// Trait defining an encoder for ASN.1
pub trait Encoder: Clone {
	/// Whether the encoding is canonical or not.
	const CANONICAL: bool;
}

/// Trait for a type to be encodable for a given encoder.
pub trait Encode<T> {
	/// Encode self for the given encoder, writes the encoded output to the
	/// passed writer.
	fn encode<W>(&mut self, writer: &mut W, value: T) -> io::Result<()>
		where W: io::Write + ?Sized;
}
