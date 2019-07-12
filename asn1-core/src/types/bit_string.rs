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


/// A representation of the `BIT STRING` ASN.1 data type. `BitString` is
/// a wrapper around the `bit_vec::BitVec` type. Please refer to the [`BitVec`]
/// documentation for using `BitString` in Rust. The following is documentation
/// on how to use a `BIT STRING` in ASN.1.
///
/// A bit string has a tag which is universal class, number 3.
///
/// The first and final bits in a bit string are called the leading and trailing
/// bits respectively.
/// * **Note** — This terminology is used in specifying the value notation and
/// in defining encoding rules.
///
/// [`BitVec`]: https://docs.rs/bit-vec/0.6.1/bit_vec/struct.BitVec.html
///
/// # Notation
/// The following describes the notation used for defining `BIT STRING` types
/// and values.
///
/// ## Type
/// ```asn1-notation
/// BitStringType ::=
///     BIT STRING
///   | BIT STRING "{" NamedBitList "}"
///
/// NamedBitList ::=
///     NamedBit
///   | NamedBitList "," NamedBit
///
/// NamedBit ::=
///     identifier "(" number ")"
///   | identifier "(" DefinedValue ")"
/// ```
/// The `DefinedValue` shall be a reference to a non-negative value of type
/// integer.
///
/// The value of each `number` or `DefinedValue` appearing in the
/// `NamedBitList` shall be different, and is the number of a distinguished
/// bit in a bitstring value. The leading bit of the bit string is identified by
/// the `number` zero, with succeeding bits having successive values.
/// * **Note 1** — The order of the `NamedBit` production sequences in the
/// `NamedBitList` is not significant.
/// * **Note 2** — Since an `identifier` that
/// appears within the `NamedBitList` cannot be used to specify the value
/// associated with a `NamedBit`, the `DefinedValue` can never be misinterpreted
/// as an `IntegerValue`. Therefore in the following case:
/// ```asn1
/// a INTEGER ::= 1
/// T1 ::= INTEGER { a(2) }
/// T2 ::= BIT STRING { a(3), b(a) }
/// ```
/// The last occurrence of `a` in `T2` denotes the value `1`, as it cannot be a
/// reference to the second nor the third occurrence of `a`.
///
/// The presence of a `NamedBitList` has no effect on the set of abstract
/// values of this type. Values containing 1 bits other than the named bits are
/// permitted.
///
/// When a `NamedBitList` is used in defining a bitstring type ASN.1 encoding
/// rules are free to add (or remove) arbitrarily any trailing 0 bits to (or
/// from) values that are being encoded or decoded. Application designers should
/// therefore ensure that different semantics are not associated with such
/// values which differ only in the number of trailing 0 bits.
///
/// ## Value
/// ```asn1-notation
/// BitStringValue ::=
///     bstring
///   | hstring
///   | "{" IdentifierList "}"
///   | "{" "}"
///   | CONTAINING Value
///
/// IdentifierList ::=
///     identifier
///   | IdentifierList "," identifier
/// ```
/// Each `identifier` in `BitStringValue` shall be the same as an `identifier`
/// in the `BitStringType` production sequence with which the value
/// is associated.
///
/// If the bit string has named bits, the `BitStringValue` notation denotes a
/// bit string value with ones in the bit positions specified by the numbers
/// corresponding to the `identifier`s, and with all other bits zero.
/// * **Note** — For a `BitStringType` that has a `NamedBitList`, the `"{" "}"`
///   production sequence in `BitStringValue` is used to denote the bit string
///   which contains no one bits.
///
/// When using the `bstring` notation, the leading bit of the bitstring value is
/// on the left, and the trailing bit of the bitstring value is on the right.
///
/// When using the `hstring` notation, the most significant bit of each
/// hexadecimal digit corresponds to the leftmost bit in the bitstring.
/// * **Note** — This notation does not, in any way, constrain the way encoding
///   rules place a bitstring into octets for transfer.
///
/// The `hstring` notation shall not be used unless the bitstring value consists
/// of a multiple of four bits. The following are alternative notations for the
/// same bitstring value. If the type was defined using a `NamedBitList`, the
/// (single) trailing zero does not form part of the value, which is thus 15
/// bits in length. If the type was defined without a `NamedBitList`, the
/// trailing zero does form part of the value, which is thus 16 bits in length.
/// ```asn1
/// 'A98A'H
/// '1010100110001010'B
/// ```
///
/// The `CONTAINING` alternative can only be used if there is a contents
/// constraint on the bitstring type which includes CONTAINING. The `Value`
/// shall then be value notation for a value of the `Type` in the
/// `ContentsConstraint` (see Rec. ITU-T X.682 | ISO/IEC 8824-3, clause 11).
/// * **Note** — This value notation can never appear in a subtype constraint
/// because Rec. ITU-T X.682 | ISO/IEC 8824-3, clause 11.3 forbids further
/// constraints after a `ContentsConstraint`, and the above text forbids its use
/// unless the governor has a `ContentsConstraint`.
///
/// The `CONTAINING` alternative shall be used if there is a contents constraint
/// on the bitstring type which does not containENCODED BY.
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
/// * **Note** — that `body1` and `body2` are distinct abstract values because
/// trailing 0 bits are significant (due to there being no `NamedBitList` in the
/// definition of G3FacsimilePage).
///
/// # Fixed size example
/// Use a bit string type with a size constraint to model the values of a fixed
/// sized bit field.
/// ```asn1
/// BitField ::= BIT STRING (SIZE (12))
/// map1 BitField ::= '100110100100'B
/// map2 BitField ::= '9A4'H
/// map3 BitField ::= '1001101001'B -- Illegal - violates size constraint.
/// ```
/// # Bit map example
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
/// * **Note** — If the bit string value is less than 7 bits long, then the
/// missing bits indicate a cloudy day for those days, hence the first three
/// values above have the same abstract value.
///
/// # Fixed size bit map example
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
/// * **Note** — The first and third values have the same abstract value.
/// # Example
/// Use a bit string type with named bits to model the values of a collection of
/// related logical variables.
/// ```asn1
/// PersonalStatus ::= BIT STRING { married(0), employed(1), veteran(2), collegeGraduate(3) }
/// jane PersonalStatus ::= { married, employed, collegeGraduate }
/// alice PersonalStatus ::= '110100'B
/// ```
/// * **Note** — that `jane` and `alice` have the same abstract values.
///
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
