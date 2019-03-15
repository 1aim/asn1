use std::str::FromStr;

use variation::Variation;

use crate::ast::*;

#[derive(Clone, Debug, Variation)]
pub enum ObjectClass {
    Def(ClassDefinition),
    Defined(DefinedObjectClass),
    Parameterized(String, Option<()>),
}

#[derive(Clone, Debug)]
pub struct ClassDefinition {
    fields: Vec<FieldSpec>,
    syntax: Option<Vec<Token>>,
}

impl ClassDefinition {
    pub fn new(fields: Vec<FieldSpec>, syntax: Option<Vec<Token>>) -> Self {
        Self { fields, syntax }
    }
}

#[derive(Clone, Debug, Variation)]
pub enum FieldSpec {
    FixedTypeValue(String, Type, bool, Optionality<Value>),
    VariableTypeValue(String, Vec<Field>, Optionality<Value>),
    FixedValueSet(String, Type, Optionality<ElementSetSpec>),
    ObjectField(String, DefinedObjectClass, Optionality<Object>),
    Type(String, Optionality<Type>),
    ObjectSet(String, DefinedObjectClass, Optionality<(ElementSet, bool)>),
}

#[derive(Clone, Debug)]
pub enum Optionality<T> {
    Optional,
    Default(T),
    None,
}

#[derive(Clone, Debug, Variation)]
pub enum DefinedObjectSet {
    External(String, String),
    Internal(String),
}

#[derive(Clone, Debug)]
pub struct Field {
    name: String,
    kind: FieldType,
}

impl Field {
    pub fn new(name: String, kind: FieldType) -> Self {
        Self { name, kind }
    }
}

#[derive(Clone, Debug, Variation)]
pub enum FieldType {
    Type,
    Value,
    ValueSet,
    Object,
    ObjectSet,
}

#[derive(Clone, Debug, Variation)]
pub enum ObjectSetElements {
    Defined(DefinedObjectSet),
}

#[derive(Clone, Debug, Variation)]
pub enum Class {
    Universal,
    Application,
    Private,
}

impl FromStr for Class {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "UNIVERSAL" => Ok(Class::Universal),
            "APPLICATION" => Ok(Class::Application),
            "PRIVATE" => Ok(Class::Private),
            _ => Err(()),
        }
    }
}
#[derive(Clone, Debug, Variation)]
pub enum Object {
    Def(Vec<ObjectDefn>),
    Defined(ReferenceType, Option<Vec<Parameter>>)
}

#[derive(Clone, Debug, Variation)]
pub enum ObjectDefn {
    Setting(Setting),
    Literal(String),
}

#[derive(Clone, Debug, Variation)]
pub enum Setting {
    Type(Type),
    Value(Value),
    ValueSet(ElementSetSpec),
    Object(Object),
    ObjectSet(ElementSet),
}


