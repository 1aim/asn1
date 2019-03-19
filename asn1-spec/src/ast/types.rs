use crate::ast::*;

#[derive(Clone, Debug, Derefable, Eq, Hash, PartialEq, PartialOrd, Ord)]
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

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Variation)]
pub enum RawType {
    Builtin(BuiltinType),
    ParameterizedReference(ReferenceType, Vec<Parameter>),
    Referenced(ReferenceType),
    ReferencedFromObject(FieldReference),
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

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Variation)]
pub enum BuiltinType {
    Boolean,
    BitString(BTreeMap<String, Number>),
    CharacterString(CharacterStringType),
    Choice(ChoiceType),
    Enumeration(
        Vec<EnumerationType>,
        Option<ExceptionIdentification>,
        Option<Vec<EnumerationType>>,
    ),
    Integer(BTreeMap<String, Number>),
    Null,
    ObjectClassField(DefinedObjectClass, Vec<Field>),
    ObjectIdentifier,
    OctetString,
    Prefixed(Prefix, Box<Type>),
    Sequence(Vec<ComponentType>),
    SequenceOf(Box<Type>),
    Set(Set),
    SetOf(Box<Type>),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ReferenceType {
    pub module: Option<String>,
    pub item: String,
}

impl ReferenceType {
    pub fn new(module: Option<String>, item: String) -> Self {
        Self { module, item }
    }

    pub fn is_internal(&self) -> bool {
        self.module.is_none()
    }
}

impl fmt::Display for ReferenceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.module.is_some() {
            write!(f, "{}.", self.module.as_ref().unwrap())?;
        }

        write!(f, "{}", self.item)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Variation)]
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

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ChoiceType {
    pub alternatives: Vec<Type>,
    pub extension: Option<ExtensionAndException>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Variation)]
pub enum EnumerationType {
    NamedNumber((String, Number)),
    Name(String),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Variation)]
pub enum DefinedObjectClass {
    External(String, String),
    Internal(String),
    AbstractSyntax,
    TypeIdentifier,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Prefix {
    encoding: Option<String>,
    class: Option<Class>,
    number: Number,
}

impl Prefix {
    pub fn new(encoding: Option<String>, class: Option<Class>, number: Number) -> Self {
        Self {
            encoding,
            class,
            number,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum ComponentType {
    Type {
        ty: Type,
        optional: bool,
        default: Option<Value>,
    },
    ComponentsOf(Type),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Variation)]
pub enum Set {
    Extensible(ExtensionAndException, bool),
    Concrete(Vec<ComponentType>),

}