use crate::ast::*;

#[derive(Clone, Debug, Derefable)]
pub struct Type {
    #[deref]
    pub raw_type: RawType,
    pub name: Option<String>,
    pub constraints: Option<Vec<Constraint>>,
}

impl From<RawType> for Type {
    fn from(raw_type: RawType) -> Self {
        Type {
            raw_type,
            name: None,
            constraints: None,
        }
    }
}

#[derive(Clone, Debug, Variation)]
pub enum RawType {
    Builtin(BuiltinType),
    Referenced(ReferenceType),
    Parameterized(ReferenceType, Vec<Parameter>),
}

impl From<BuiltinType> for RawType {
    fn from(builtin: BuiltinType) -> Self {
        RawType::Builtin(builtin)
    }
}

impl From<ReferenceType> for RawType {
    fn from(reference: ReferenceType) -> Self {
        RawType::Referenced(reference)
    }
}

#[derive(Clone, Debug, Variation)]
pub enum BuiltinType {
    Boolean,
    CharacterString(CharacterStringType),
    Choice(ChoiceType),
    Enumeration(
        Vec<EnumerationType>,
        Option<ExceptionIdentification>,
        Option<Vec<EnumerationType>>,
    ),
    Integer(HashMap<String, NumberOrDefinedValue>),
    Null,
    ObjectClassField(DefinedObjectClass, Vec<Field>),
    ObjectIdentifier,
    OctetString,
    Prefixed(Prefix, Box<Type>),
    Sequence(Vec<ComponentType>),
    SequenceOf(Box<Type>),
    SetOf(Box<Type>),
}

#[derive(Clone, Debug, Variation)]
pub enum ReferenceType {
    External(String, String),
    Internal(String),
}

#[derive(Clone, Debug, Variation)]
pub enum CharacterStringType {
    Bmp,
    General,
    Graphic,
    Ia5,
    Iso646,
    Numeric,
    Printable,
    T61,
    Teletex,
    Universal,
    Unrestricted,
    Utf8,
    Videotex,
    Visible,
}

#[derive(Clone, Debug)]
pub struct ChoiceType {
    pub alternatives: Vec<Type>,
    pub extension: Option<ExtensionAndException>,
}

#[derive(Clone, Debug, Variation)]
pub enum EnumerationType {
    NamedNumber((String, NumberOrDefinedValue)),
    Name(String),
}

#[derive(Clone, Debug, Variation)]
pub enum DefinedObjectClass {
    External(String, String),
    Internal(String),
    AbstractSyntax,
    TypeIdentifier,
}

#[derive(Clone, Debug)]
pub struct Prefix {
    encoding: Option<String>,
    class: Option<Class>,
    number: NumberOrDefinedValue,
}

impl Prefix {
    pub fn new(
        encoding: Option<String>,
        class: Option<Class>,
        number: NumberOrDefinedValue,
    ) -> Self {
        Self {
            encoding,
            class,
            number,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ComponentType {
    Type {
        ty: Type,
        optional: bool,
        default: Option<Value>,
    },
    ComponentsOf(Type),
}
