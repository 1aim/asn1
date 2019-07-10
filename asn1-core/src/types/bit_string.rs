// use std::{fmt, error::Error};
use std::ops;

use bit_vec::BitVec;
use serde::{
    self,
    Deserialize,
    // de::{ SeqAccess, Visitor, },
    Serialize,
    Serializer
};


/// A representation of the `BIT STRING` ASN.1 data type. Uses `bit_vec::BitVec`
/// internally. Please refer to the `BitVec` documentation for using
/// `BitString`.
///
/// # Example
/// Use a bit string type to model binary data whose format and length are
/// unspecified, or specified elsewhere, and whose length in bits is not
/// necessarily a multiple of eight.
/// ```asn1
/// G3FacsimilePage ::= BIT STRING
/// -- a sequence of bits conforming to Rec. ITU-T T.4.
/// image G3FacsimilePage ::= '100110100100001110110'B
/// trailer BIT STRING ::= '0123456789ABCDEF'H
/// body1 G3FacsimilePage ::= '1101'B
/// body2 G3FacsimilePage ::= '1101000'B
/// ```
/// **Note** that `body1` and `body2` are distinct abstract values because trailing
/// 0 bits are significant (due to there being no "NamedBitList" in the
/// definition of G3FacsimilePage).
/// # Example
/// Use a bit string type with a size constraint to model the values of a fixed
/// sized bit field.
/// ```asn1
/// BitField ::= BIT STRING (SIZE (12))
/// map1 BitField ::= '100110100100'B
/// map2 BitField ::= '9A4'H
/// map3 BitField ::= '1001101001'B -- Illegal - violates size constraint.
/// ```
/// # Example
/// Use a bit string type to model the values of a bit map, an ordered
/// collection of logical variables indicating whether a particular condition
/// holds for each of a correspondingly ordered collection of objects.
/// ```asn1
/// DaysOfTheWeek ::= BIT STRING { sunday(0), monday (1), tuesday(2),
///     wednesday(3), thursday(4), friday(5), saturday(6) } (SIZE (0..7))
///
/// sunnyDaysLastWeek1 DaysOfTheWeek ::= {sunday, monday, wednesday}
/// sunnyDaysLastWeek2 DaysOfTheWeek ::= '1101'B
/// sunnyDaysLastWeek3 DaysOfTheWeek ::= '1101000'B
/// sunnyDaysLastWeek4 DaysOfTheWeek ::= '11010000'B -- Illegal
/// ```
/// **Note** that if the bit string value is less than 7 bits long, then the
/// missing bits indicate a cloudy day for those days, hence the first three
/// values above have the same abstract value.
/// # Example
/// Use a bit string type to model the values of a bit map, a fixed-size ordered
/// collection of logical variables indicating whether a particular condition
/// holds for each of a correspondingly ordered collection of objects.
/// ```asn1
/// DaysOfTheWeek ::= BIT STRING { sunday(0), monday (1), tuesday(2),
///     wednesday(3), thursday(4), friday(5), saturday(6) } (SIZE (7))
///
/// sunnyDaysLastWeek1 DaysOfTheWeek ::= {sunday, monday, wednesday}
/// sunnyDaysLastWeek2 DaysOfTheWeek ::= '1101'B -- Illegal -- violates size constraint.
/// sunnyDaysLastWeek3 DaysOfTheWeek ::= '1101000'B
/// sunnyDaysLastWeek4 DaysOfTheWeek ::= '11010000'B -- Illegal -- violates size constraint.
/// ```
/// **Note** that the first and third values have the same abstract value.
/// # Example
/// Use a bit string type with named bits to model the values of a collection of
/// related logical variables.
/// ```asn1
/// PersonalStatus ::= BIT STRING { married(0), employed(1), veteran(2), collegeGraduate(3) }
/// jane PersonalStatus ::= { married, employed, collegeGraduate }
/// alice PersonalStatus ::= '110100'B
/// ```
/// **Note** that `jane` and `alice` have the same abstract values.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename="ASN.1#BitString")]
pub struct BitString(BitVec);

impl BitString {
    /// Instantiates a new empty instance of `BitString`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Instantiate a new instance of `BitString` from a byte slice.
    pub fn from_bytes(input: &[u8]) -> Self {
        Self(BitVec::from_bytes(input))
    }
}

impl ops::Deref for BitString {
    type Target = BitVec;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for BitString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<BitVec> for BitString {
    fn from(bit_vec: BitVec) -> Self {
        BitString(bit_vec)
    }
}

/*
struct BitStringVisitor;

impl<'de> Visitor<'de> for BitStringVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a bit string")
    }

    fn visit_u8<E: Error>(self, v: u8) -> Result<Self::Value, E> {
        unimplemented!()
    }

    fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(v.to_vec())
    }

    fn visit_borrowed_bytes<E: Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(v.to_vec())
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut visitor: S) -> Result<Self::Value, S::Error> {
        let mut values = Vec::new();
        while let Some(value) = visitor.next_element()? {
            values.push(value);
        }

        Ok(values)
    }
}
*/

impl Serialize for BitString {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut bytes = self.0.to_bytes();

        if let Some(last) = bytes.last() {
            let zeroes = last.trailing_zeros();
            bytes.insert(0, zeroes as u8);
        } else {
            // If there is no last, then the vec is empty and we put in a single
            // zero octet. (X.690 8.6.2.3).
            bytes.push(0);
        }

        serializer.serialize_newtype_struct("ASN.1#BitString", &bytes)
    }
}
