use variation::Variation;

use super::oid::ObjectIdentifier;

#[derive(Clone, Debug, Variation)]
pub enum Value {
    BitString(BitString),
    Boolean(bool),
    Integer(IntegerValue),
    ObjectIdentifier(ObjectIdentifier),
    Sequence(Vec<NamedValue>),
    Enumerated(String),
    Defined(DefinedValue),
    Object(Vec<String>),
    ObjectClassField,
}

#[derive(Clone, Debug, Variation)]
pub enum IntegerValue {
    Literal(i64),
    Identifier(String),
}

#[derive(Clone, Debug, Variation)]
pub enum NumberOrDefinedValue {
    Number(i64),
    DefinedValue(DefinedValue),
}

#[derive(Clone, Debug)]
pub struct NamedValue(pub String, pub Value);

#[derive(Clone, Debug, Hash, PartialEq, Eq, Variation)]
pub enum DefinedValue {
    Simple(SimpleDefinedValue),
    /// Paramaterized value
    Parameterized(SimpleDefinedValue, Vec<()>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Variation)]
pub enum SimpleDefinedValue {
    /// An external type reference e.g. `foo.bar`
    Reference(String, String),
    /// Identifier pointing to a value
    Value(String),
}

#[derive(Clone, Debug, Variation)]
pub enum BitString {
    Literal(String),
    List(Vec<String>),
    Containing(Box<Value>),
}
