use crate::error::{Error, Result};
use serde::{
    ser::{self, Impossible},
    Serialize,
};
use std::io::Write;

/// Serializer used solely to encode octet strings properly.
pub(crate) struct ObjectIdentifierSerializer<W> {
    pub output: W,
    first_components: Vec<u128>,
    encoded_len: usize,
}

impl<W: Write> ObjectIdentifierSerializer<W> {
    pub fn new(output: W) -> Self {
        Self {
            output,
            first_components: Vec::new(),
            encoded_len: 0,
        }
    }

    pub fn output(mut self) -> Result<W> {
        if self.encoded_len == 2 {
            let first = self.first_components[0];
            let second = self.first_components[1];

            encode_component(first * 40 + second, &mut self.output)?;
        }

        Ok(self.output)
    }
}

fn encode_component<W: Write>(mut v: u128, writer: &mut W) -> Result<()> {
    let mut bytes: Vec<u8> = Vec::new();

    while v != 0 {
        bytes.push((v & 0x7f) as u8);
        v >>= 7;
    }

    for byte in bytes.iter().skip(1).rev() {
        let octet = (0x80 | byte) as u8;
        writer.write(&[octet])?;
    }

    let final_octet = bytes[0] as u8;
    writer.write(&[final_octet])?;

    Ok(())
}

impl<'a, W: Write> ser::Serializer for &'a mut ObjectIdentifierSerializer<W> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, _: bool) -> Result<Self::Ok> {
        unreachable!()
    }

    fn serialize_i8(self, _: i8) -> Result<Self::Ok> {
        unreachable!()
    }

    fn serialize_i16(self, _: i16) -> Result<()> {
        unreachable!()
    }

    fn serialize_i32(self, _: i32) -> Result<()> {
        unreachable!()
    }

    fn serialize_i64(self, _: i64) -> Result<()> {
        unreachable!()
    }

    fn serialize_i128(self, _: i128) -> Result<()> {
        unreachable!()
    }

    fn serialize_u8(self, _: u8) -> Result<()> {
        unreachable!()
    }

    fn serialize_u16(self, _: u16) -> Result<()> {
        unreachable!()
    }

    fn serialize_u32(self, _: u32) -> Result<()> {
        unreachable!()
    }

    fn serialize_u64(self, _: u64) -> Result<()> {
        unreachable!()
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        self.encoded_len += 1;

        match self.first_components.len() {
            0..=1 => self.first_components.push(v),
            2 => {
                let first = self.first_components[0];
                let second = self.first_components[1];
                encode_component((first * 40) + second, &mut self.output)?;
                // Also encode the current element.
                encode_component(v, &mut self.output)?;
                // Just pushing an extra element to go to the next branch of the
                // match statement.
                self.first_components.push(40);
            }
            _ => {
                encode_component(v, &mut self.output)?;
            }
        }

        Ok(())
    }

    fn serialize_f32(self, _: f32) -> Result<()> {
        unreachable!()
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        unreachable!()
    }

    fn serialize_char(self, _: char) -> Result<()> {
        unreachable!()
    }

    fn serialize_str(self, _: &str) -> Result<()> {
        unreachable!()
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<()> {
        unreachable!()
    }

    fn serialize_none(self) -> Result<()> {
        unreachable!()
    }

    fn serialize_some<T>(self, _: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_unit(self) -> Result<()> {
        unreachable!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        unreachable!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        unreachable!()
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        unreachable!()
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        unreachable!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unreachable!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unreachable!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        unreachable!()
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        unreachable!()
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unreachable!()
    }
}
