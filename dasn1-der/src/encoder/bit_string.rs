use serde::{
    ser::{self, Impossible},
    Serialize,
};

use crate::error::{Error, Result};

/// Serializer used solely to encode bit strings properly.
#[derive(Default)]
pub(crate) struct BitStringSerializer {
    pub output: Vec<u8>,
    pub last: u8,
}

impl BitStringSerializer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'a> ser::Serializer for &'a mut BitStringSerializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.last = v;
        self.output.push(v);
        Ok(())
    }

    fn serialize_bool(self, _: bool) -> Result<Self::Ok> { unreachable!() }
    fn serialize_i8(self, _: i8) -> Result<Self::Ok> { unreachable!() }
    fn serialize_i16(self, _: i16) -> Result<()> { unreachable!() }
    fn serialize_i32(self, _: i32) -> Result<()> { unreachable!() }
    fn serialize_i64(self, _: i64) -> Result<()> { unreachable!() }
    fn serialize_i128(self, _: i128) -> Result<()> { unreachable!() }
    fn serialize_u16(self, _: u16) -> Result<()> { unreachable!() }
    fn serialize_u32(self, _: u32) -> Result<()> { unreachable!() }
    fn serialize_u64(self, _: u64) -> Result<()> { unreachable!() }
    fn serialize_u128(self, _: u128) -> Result<()> { unreachable!() }
    fn serialize_f32(self, _: f32) -> Result<()> { unreachable!() }
    fn serialize_f64(self, _v: f64) -> Result<()> { unreachable!() }
    fn serialize_char(self, _: char) -> Result<()> { unreachable!() }
    fn serialize_str(self, _: &str) -> Result<()> { unreachable!() }
    fn serialize_bytes(self, _: &[u8]) -> Result<()> { unreachable!() }
    fn serialize_none(self) -> Result<()> { unreachable!() }
    fn serialize_some<T>(self, _: &T) -> Result<()> where T: ?Sized + Serialize, { unreachable!() }
    fn serialize_unit(self) -> Result<()> { unreachable!() }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> { unreachable!() }
    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str,) -> Result<()> { unreachable!() }
    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<()> where T: ?Sized + Serialize, { unreachable!() }
    fn serialize_newtype_variant<T>(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _value: &T,) -> Result<()> where T: ?Sized + Serialize, { unreachable!() }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> { unreachable!() }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> { unreachable!() }
    fn serialize_tuple_struct(self, _name: &'static str, _len: usize,) -> Result<Self::SerializeTupleStruct> { unreachable!() }
    fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize,) -> Result<Self::SerializeTupleVariant> { unreachable!() }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> { unreachable!() }
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> { unreachable!() }
    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize,) -> Result<Self::SerializeStructVariant> { unreachable!() }
}

