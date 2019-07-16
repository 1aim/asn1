use core::identifier::Class;
use serde::{
    ser,
    Serialize,
};

use crate::error::{Error, Result};
use super::Serializer;

/// Serializer used solely to encode octet strings properly.
pub(crate) struct PrefixSerializer {
    pub output: Serializer<Vec<u8>>,
    pub class: Option<Class>,
    pub tag: Option<usize>,
}

impl PrefixSerializer {
    pub fn new(implicit: bool) -> Self {
        let mut ser = Serializer::new(Vec::new());
        ser.implicit = implicit;
        Self {
            output: ser,
            class: None,
            tag: None,
        }
    }
}

impl<'a> ser::Serializer for &'a mut PrefixSerializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = super::Sequence<'a, Vec<u8>>;
    type SerializeTuple = super::Sequence<'a, Vec<u8>>;
    type SerializeTupleStruct = super::Sequence<'a, Vec<u8>>;
    type SerializeTupleVariant = super::Sequence<'a, Vec<u8>>;
    type SerializeMap = super::Sequence<'a, Vec<u8>>;
    type SerializeStruct = super::Sequence<'a, Vec<u8>>;
    type SerializeStructVariant = super::Sequence<'a, Vec<u8>>;

    fn serialize_u8(self, v: u8) -> Result<()> {
        if self.class.is_none() {
            self.class = Some(Class::from(v));
            Ok(())
        } else {
            self.output.serialize_u8(v)
        }
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        self.output.serialize_u128(v)
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.output.serialize_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.output.serialize_i8(v)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.output.serialize_i16(v)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.output.serialize_i32(v)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output.serialize_i64(v)
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        self.output.serialize_i128(v)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.output.serialize_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.output.serialize_u32(v)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        if self.tag.is_none() {
            self.tag = Some(v as usize);
            Ok(())
        } else {
            self.output.serialize_u64(v)
        }
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.output.serialize_f32(v)
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output.serialize_f64(v)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.output.serialize_char(v)
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.output.serialize_str(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.output.serialize_bytes(v)
    }

    fn serialize_none(self) -> Result<()> {
        self.output.serialize_none()
    }

    fn serialize_some<T>(self, v: &T) -> Result<()> where T: ?Sized + Serialize, {
        self.output.serialize_some(v)
    }

    fn serialize_unit(self) -> Result<()> {
        self.output.serialize_unit()
    }
    fn serialize_unit_struct(self, n: &'static str) -> Result<()> {
        self.output.serialize_unit_struct(n)
    }

    fn serialize_unit_variant(self, n: &'static str, v_i: u32, v: &'static str) -> Result<()> {
        self.output.serialize_unit_variant(n, v_i, v)
    }

    fn serialize_newtype_struct<T>(self, n: &'static str, v: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        self.output.serialize_newtype_struct(n, v)
    }

    fn serialize_newtype_variant<T>(self, n: &'static str, v_i: u32, vn: &'static str, v: &T) -> Result<()> where T: ?Sized + Serialize, {
        self.output.serialize_newtype_variant(n, v_i, vn, v)
    }

    fn serialize_seq(self, l: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output.serialize_seq(l)
    }

    fn serialize_tuple(self, l: usize) -> Result<Self::SerializeTuple> {
        self.output.serialize_seq(Some(l))
    }

    fn serialize_tuple_struct(self, n: &'static str, l: usize,) -> Result<Self::SerializeTupleStruct> {
        self.output.serialize_tuple_struct(n, l)
    }

    fn serialize_tuple_variant(self, n: &'static str, v_i: u32, v: &'static str, l: usize) -> Result<Self::SerializeTupleVariant> {
        self.output.serialize_tuple_variant(n, v_i, v, l)
    }

    fn serialize_map(self, l: Option<usize>) -> Result<Self::SerializeMap> {
        self.output.serialize_map(l)
    }

    fn serialize_struct(self, n: &'static str, l: usize) -> Result<Self::SerializeStruct> {
        self.output.serialize_struct(n, l)
    }

    fn serialize_struct_variant(self, n: &'static str, v_i: u32, v: &'static str, l: usize) -> Result<Self::SerializeStructVariant> {
        self.output.serialize_struct_variant(n, v_i, v, l)
    }
}
