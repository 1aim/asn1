use std::collections::HashMap;
use std::iter::Peekable;

use pest::iterators::{FlatPairs, Pair};

use super::{Result, Rule};

pub(crate) struct Ast<'a>(Peekable<FlatPairs<'a, Rule>>);

impl<'a> Ast<'a> {
    pub fn new(input: Peekable<FlatPairs<'a, Rule>>) -> Self {
        Ast(input)
    }

    fn peek(&mut self, rule: Rule) -> bool {
        self.rule_peek().map(|r| r == rule).unwrap_or(false)
    }

    fn rule_peek(&mut self) -> Option<Rule> {
        self.0.peek().map(|x| x.as_rule())
    }

    fn next_rule(&mut self) -> Option<Rule> {
        self.0.next().map(|x| x.as_rule())
    }

    fn next(&mut self) -> Option<Pair<Rule>> {
        self.0.next()
    }

    /// Takes the next pair, and asserts that's its `Rule` matches `rule`.
    ///
    /// # Panics
    /// If `rule` doesn't match the next rule in the iterator.
    fn take(&mut self, rule: Rule) -> Pair<Rule> {
        let pair = self.0.next().unwrap();
        assert_eq!(rule, pair.as_rule());

        pair
    }

    /// Look at the next pair and checks if its `Rule` matches `rule`, consumes the pair if it
    /// matches. Useful for checking for optional items.
    fn look(&mut self, rule: Rule) -> Option<Pair<Rule>> {
        if self.peek(rule) {
            Some(self.take(rule))
        } else {
            None
        }
    }

    pub fn parse_module(&mut self) -> Result<Module> {
        self.take(Rule::ModuleDefinition);

        let identifier = self.parse_module_identifier()?;
        let tag = self.parse_tag_default();
        // let extension = self.parse_extension_default();

        let (exports, imports, assignments) = if self.look(Rule::ModuleBody).is_some() {
            let exports = self.parse_exports();
            let imports = self.parse_imports();
            let assignments = self.parse_assignments();

            (exports, imports, assignments)
        } else {
            (Exports::All, HashMap::new(), Vec::new())
        };

        Ok(Module {
            identifier: identifier,
            tag,
            extension: None,
            exports,
            imports,
            assignments,
        })
    }

    pub fn parse_module_identifier(&mut self) -> Result<ModuleIdentifier> {
        self.take(Rule::ModuleIdentifier);

        let mut module_identifier = ModuleIdentifier::new(self.parse_reference_identifier());

        if self.look(Rule::DefinitiveIdentification).is_some() {
            self.take(Rule::DefinitiveOID);

            while self.look(Rule::DefinitiveObjIdComponent).is_some() {
                let pair = self.next().unwrap();

                let component = match pair.as_rule() {
                    Rule::NameForm => DefinitiveObjIdComponent::Name(pair.as_str().to_owned()),
                    Rule::DefinitiveNumberForm => {
                        DefinitiveObjIdComponent::Number(pair.as_str().parse()?)
                    }
                    Rule::DefinitiveNameAndNumberForm => {
                        let name = self.parse_identifier();
                        let number = self.take(Rule::DefinitiveNumberForm).as_str().parse()?;
                        DefinitiveObjIdComponent::NameAndNumber(name, number)
                    }
                    _ => unreachable!(),
                };

                module_identifier.identification.push(component);
            }
        }

        Ok(module_identifier)
    }

    pub fn parse_tag_default(&mut self) -> Tag {
        if let Some(pair) = self.look(Rule::TagDefault) {
            let raw = pair.as_str();

            if raw.contains("AUTOMATIC") {
                Tag::Automatic
            } else if raw.contains("IMPLICIT") {
                Tag::Implicit
            } else {
                Tag::Explicit
            }
        } else {
            Tag::Explicit
        }
    }

    pub fn parse_exports(&mut self) -> Exports {
        if self.look(Rule::Exports).is_some() && self.look(Rule::SymbolsExported).is_some() {
            Exports::Symbols(self.parse_symbol_list())
        } else {
            Exports::All
        }
    }

    pub fn parse_symbol_list(&mut self) -> Vec<Symbol> {
        self.take(Rule::SymbolList);
        let mut symbols = Vec::new();

        while self.look(Rule::Symbol).is_some() {
            let pair = self.next().unwrap();
            let raw = pair.as_str().to_owned();

            let symbol = match pair.as_rule() {
                Rule::ReferenceIdentifier => Symbol::Reference(raw),
                Rule::Identifier => Symbol::Value(raw),
                Rule::ParameterizedReference => Symbol::Parameterized(raw),
                _ => unreachable!(),
            };

            symbols.push(symbol);
        }

        symbols
    }

    pub fn parse_imports(&mut self) -> HashMap<ModuleReference, Vec<Symbol>> {
        let mut imports = HashMap::new();

        if self.look(Rule::Imports).is_some() {
            while self.look(Rule::SymbolsFromModule).is_some() {
                let symbol_list = self.parse_symbol_list();
                self.take(Rule::GlobalModuleReference);
                let module_name = self.take(Rule::ReferenceIdentifier).as_str().to_owned();

                let identification = if self.look(Rule::AssignedIdentifier).is_some() {
                    let identification = match self.rule_peek().unwrap() {
                        Rule::ObjectIdentifierValue => AssignedIdentifier::ObjectIdentifier(
                            self.parse_object_identifier_value(),
                        ),
                        Rule::DefinedValue => {
                            AssignedIdentifier::Defined(self.parse_defined_value())
                        }
                        _ => unreachable!(),
                    };

                    Some(identification)
                } else {
                    None
                };

                imports.insert(
                    ModuleReference::new(module_name, identification),
                    symbol_list,
                );
            }
        }

        imports
    }

    pub fn parse_object_identifier_value(&mut self) -> Vec<ObjIdComponent> {
        self.take(Rule::ObjectIdentifierValue);
        let mut components = Vec::new();

        while self.look(Rule::ObjIdComponents).is_some() {
            use self::DefinitiveObjIdComponent::*;

            let component = match self.rule_peek().unwrap() {
                Rule::Identifier => ObjIdComponent::Definitive(Name(self.parse_identifier())),
                Rule::NumberForm => ObjIdComponent::Definitive(Number(
                    self.take(Rule::NumberForm).as_str().parse().unwrap(),
                )),
                Rule::DefinedValue => ObjIdComponent::DefinedValue(self.parse_defined_value()),
                Rule::NameAndNumberForm => {
                    self.take(Rule::NameAndNumberForm);
                    let name = self.parse_identifier();
                    let number = self.take(Rule::NumberForm).as_str().parse().unwrap();

                    ObjIdComponent::Definitive(NameAndNumber(name, number))
                }
                _ => unreachable!(),
            };

            components.push(component)
        }

        components
    }

    fn parse_defined_value(&mut self) -> DefinedValue {
        use self::SimpleDefinedValue::*;
        self.take(Rule::DefinedValue);

        match self.rule_peek().unwrap() {
            Rule::ExternalTypeReference => {
                self.take(Rule::ExternalTypeReference);
                let parent = self.parse_reference_identifier();
                let child = self.parse_reference_identifier();

                DefinedValue::Simple(Reference(parent, child))
            }

            Rule::Identifier => DefinedValue::Simple(Value(self.parse_identifier())),
            Rule::ParameterizedReference => unimplemented!(),
            _ => unreachable!(),
        }
    }

    fn parse_assignments(&mut self) -> Vec<Assignment> {
        let mut assignments = Vec::new();

        while self.look(Rule::Assignment).is_some() {
            let assignment = match self.rule_peek().unwrap() {
                Rule::TypeAssignment => {
                    self.take(Rule::TypeAssignment);

                    let ident = self.parse_reference_identifier();
                    let r#type = self.parse_type();

                    Assignment::Type(ident, r#type)
                }
                Rule::ValueAssignment => {
                    self.take(Rule::ValueAssignment);

                    let ident = self.take(Rule::valuereference).as_str().to_owned();
                    let value_type = self.parse_type();
                    let value = self.parse_value();

                    Assignment::Value(ident, value_type, value)
                }
                Rule::ObjectClassAssignment => unimplemented!(),
                Rule::ObjectAssignment => unimplemented!(),
                Rule::ObjectSetAssignment => unimplemented!(),
                _ => unreachable!(),
            };

            assignments.push(assignment)
        }

        assignments
    }

    fn parse_type(&mut self) -> Type {
        self.take(Rule::Type);

        match self.rule_peek().unwrap() {
            Rule::UnconstrainedType => self.parse_unconstrained_type().into(),
            Rule::ConstrainedType => {
                self.take(Rule::ConstrainedType);

                if self.peek(Rule::TypeWithConstraint) {
                    unimplemented!()
                } else {
                    let raw_type = self.parse_unconstrained_type();
                    self.take(Rule::Constraint);
                    self.take(Rule::ConstraintSpec);

                    if self.peek(Rule::GeneralConstraint) {
                        unimplemented!("GeneralConstraint");
                    } else {
                        let is_extendable =
                            self.take(Rule::ElementSetSpecs).as_str().contains("...");
                        let constraint = self.parse_element_set_spec();

                        Type {
                            raw_type,
                            constraint,
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn parse_element_set_spec(&mut self) -> Vec<Vec<Element>> {
        self.take(Rule::ElementSetSpec);
        self.take(Rule::Unions);
        let mut constraint = Vec::new();

        while self.look(Rule::Intersections).is_some() {
            let mut intersections = Vec::new();
            while self.look(Rule::IntersectionElements).is_some() {
                intersections.push(self.parse_elements());
                self.look(Rule::IntersectionMark);
            }

            constraint.push(intersections);
            self.look(Rule::UnionMark);
        }

        constraint
    }

    fn parse_unconstrained_type(&mut self) -> RawType {
        self.take(Rule::UnconstrainedType);

        if self.look(Rule::BuiltinType).is_some() {
            let pair = self.next().unwrap();
            match pair.as_rule() {
                Rule::IntegerType => {
                    let mut named_numbers = HashMap::new();

                    if self.look(Rule::NamedNumberList).is_some() {
                        while self.look(Rule::NamedNumber).is_some() {
                            let ident = self.parse_identifier();

                            let value = match self.rule_peek().unwrap() {
                                Rule::SignedNumber => {
                                    NumberOrDefinedValue::Number(self.parse_signed_number())
                                }
                                Rule::DefinedValue => {
                                    NumberOrDefinedValue::DefinedValue(self.parse_defined_value())
                                }
                                _ => unreachable!(),
                            };

                            named_numbers.insert(ident, value);
                        }
                    }

                    RawType::Builtin(BuiltinType::Integer(named_numbers))
                }
                Rule::ObjectClassFieldType => unimplemented!(),
                Rule::ObjectIdentifierType => RawType::Builtin(BuiltinType::ObjectIdentifier),
                Rule::OctetStringType => RawType::Builtin(BuiltinType::OctetString),
                Rule::SequenceType => {
                    RawType::Builtin(BuiltinType::Sequence(self.parse_component_type_lists()))
                }
                Rule::SequenceOfType => {
                    if self.peek(Rule::Type) {
                        RawType::Builtin(BuiltinType::SequenceOf(Box::new(SequenceOfType::Type(self.parse_type()))))
                    } else {
                        let (ident, r#type) = self.parse_named_type();
                        RawType::Builtin(BuiltinType::SequenceOf(Box::new(SequenceOfType::Named(ident, r#type))))
                    }
                },
                Rule::SetType => unimplemented!(),
                Rule::SetOfType => unimplemented!(),
                Rule::PrefixedType => unimplemented!(),
                _ => unreachable!(),
            }
        } else {
            self.take(Rule::ReferencedType);

            match self.next_rule().unwrap() {
                Rule::DefinedType => match self.rule_peek().unwrap() {
                    Rule::ExternalTypeReference => {
                        self.take(Rule::ExternalTypeReference);
                        let module = self.parse_reference_identifier();
                        let subtype = self.parse_reference_identifier();

                        ReferenceType::Defined(DefinedType::External(module, subtype)).into()
                    }

                    Rule::ReferenceIdentifier => ReferenceType::Defined(DefinedType::Internal(
                        self.parse_reference_identifier(),
                    ))
                    .into(),

                    _ => unreachable!(),
                },
                Rule::TypeFromObject => unimplemented!(),
                Rule::ValueSetFromObjects => unimplemented!(),
                _ => unreachable!(),
            }
        }
    }

    fn parse_value(&mut self) -> Value {
        self.take(Rule::Value);

        match self.rule_peek().unwrap() {
            Rule::BuiltinValue => self.parse_builtin_value().into(),
            Rule::ReferencedValue => unimplemented!(),
            Rule::ObjectClassFieldType => unimplemented!(),
            _ => unreachable!(),
        }
    }

    fn parse_builtin_value(&mut self) -> BuiltinValue {
        self.take(Rule::BuiltinValue);

        match self.rule_peek().unwrap() {
            Rule::IntegerValue => {
                self.take(Rule::IntegerValue);

                let value = match self.rule_peek().unwrap() {
                    Rule::SignedNumber => IntegerValue::Literal(self.parse_signed_number()),
                    Rule::Identifier => IntegerValue::Identifier(self.parse_identifier()),
                    _ => unreachable!(),
                };

                BuiltinValue::Integer(value)
            }
            Rule::ObjectIdentifierValue => {
                BuiltinValue::ObjectIdentifier(self.parse_object_identifier_value())
            }

            e => unreachable!("Unexpected Rule {:?}", e),
        }
    }

    fn parse_signed_number(&mut self) -> i64 {
        self.take(Rule::SignedNumber).as_str().parse().unwrap()
    }

    fn parse_identifier(&mut self) -> String {
        self.take(Rule::Identifier).as_str().to_owned()
    }

    fn parse_reference_identifier(&mut self) -> String {
        self.take(Rule::ReferenceIdentifier).as_str().to_owned()
    }

    fn parse_component_type_lists(&mut self) -> Vec<ComponentType> {
        self.take(Rule::ComponentTypeLists);

        self.parse_component_type_list()
    }

    fn parse_component_type_list(&mut self) -> Vec<ComponentType> {
        self.take(Rule::ComponentTypeList);

        let mut component_types = Vec::new();

        while self.peek(Rule::ComponentType) {
            component_types.push(self.parse_component_type());
        }

        component_types
    }

    fn parse_component_type(&mut self) -> ComponentType {
        let raw = self.take(Rule::ComponentType).as_str().to_owned();

        if raw.contains("COMPONENTS") {
            ComponentType::ComponentsOf(self.parse_type())
        } else {
            let (identifier, value_type) = self.parse_named_type();
            let optional = raw.contains("OPTIONAL");
            let default = match raw.contains("DEFAULT") {
                true => Some(self.parse_value()),
                false => None,
            };

            ComponentType::Type {
                identifier,
                value_type,
                optional,
                default,
            }
        }
    }

    fn parse_named_type(&mut self) -> (String, Type) {
        self.take(Rule::NamedType);

        (self.parse_identifier(), self.parse_type())
    }

    fn parse_elements(&mut self) -> Element {
        self.take(Rule::Elements);

        match self.rule_peek().unwrap() {
            Rule::ElementSetSpec => Element::ElementSet(self.parse_element_set_spec()),
            Rule::SubtypeElements => {
                self.take(Rule::SubtypeElements);
                match self.rule_peek().unwrap() {
                    Rule::Value => Element::SubType(SubTypeElement::Value(self.parse_value())),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Module {
    identifier: ModuleIdentifier,
    tag: Tag,
    extension: Option<()>,
    exports: Exports,
    imports: HashMap<ModuleReference, Vec<Symbol>>,
    assignments: Vec<Assignment>,
}

#[derive(Debug)]
pub struct ModuleIdentifier {
    pub name: String,
    pub identification: Vec<DefinitiveObjIdComponent>,
    // iri: Option<Iri>
}

impl ModuleIdentifier {
    fn new(name: String) -> Self {
        Self {
            name,
            identification: Vec::new(),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum DefinitiveObjIdComponent {
    Name(String),
    Number(i64),
    NameAndNumber(String, i64),
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ObjIdComponent {
    Definitive(DefinitiveObjIdComponent),
    DefinedValue(DefinedValue),
}

#[derive(Debug)]
pub enum Tag {
    Explicit,
    Implicit,
    Automatic,
}

#[derive(Debug)]
pub enum Exports {
    All,
    Symbols(Vec<Symbol>),
}

#[derive(Debug)]
pub enum Symbol {
    Reference(String),
    Value(String),
    Parameterized(String),
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum SimpleDefinedValue {
    /// An external type reference e.g. `foo.bar`
    Reference(String, String),
    /// Identifier pointing to a value
    Value(String),
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum DefinedValue {
    Simple(SimpleDefinedValue),
    /// Paramaterized value
    Parameterized(SimpleDefinedValue, Vec<()>),
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ModuleReference {
    name: String,
    identification: Option<AssignedIdentifier>,
}

impl ModuleReference {
    pub(crate) fn new(name: String, identification: Option<AssignedIdentifier>) -> Self {
        Self {
            name,
            identification,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum AssignedIdentifier {
    ObjectIdentifier(Vec<ObjIdComponent>),
    Defined(DefinedValue),
}

// First argument is always the identifier.
#[derive(Debug)]
pub enum Assignment {
    Type(String, Type),
    Value(String, Type, Value),
}

#[derive(Debug)]
pub struct Type {
    raw_type: RawType,
    // First `Vec` is a list of unions, the second a list of intersections.
    constraint: Vec<Vec<Element>>,
}

impl From<RawType> for Type {
    fn from(raw_type: RawType) -> Self {
        Type {
            raw_type,
            constraint: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum RawType {
    Builtin(BuiltinType),
    Referenced(ReferenceType),
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

#[derive(Debug)]
pub enum BuiltinType {
    Integer(HashMap<String, NumberOrDefinedValue>),
    ObjectClassField,
    ObjectIdentifier,
    OctetString,
    Sequence(Vec<ComponentType>),
    SequenceOf(Box<SequenceOfType>),
    SetType,
    SetOf,
    Prefixed,
}

#[derive(Debug)]
pub enum ReferenceType {
    Defined(DefinedType),
}

impl From<DefinedType> for ReferenceType {
    fn from(def: DefinedType) -> Self {
        ReferenceType::Defined(def)
    }
}

#[derive(Debug)]
pub enum DefinedType {
    External(String, String),
    Internal(String),
}

#[derive(Debug)]
pub enum Value {
    Builtin(BuiltinValue),
    Reference,
    ObjectClassField,
}

impl From<BuiltinValue> for Value {
    fn from(builtin: BuiltinValue) -> Self {
        Value::Builtin(builtin)
    }
}

#[derive(Debug)]
pub enum BuiltinValue {
    Integer(IntegerValue),
    ObjectIdentifier(Vec<ObjIdComponent>),
}

#[derive(Debug)]
pub enum IntegerValue {
    Literal(i64),
    Identifier(String),
}

#[derive(Debug)]
pub enum ComponentType {
    Type {
        identifier: String,
        value_type: Type,
        optional: bool,
        default: Option<Value>,
    },
    ComponentsOf(Type),
}

#[derive(Debug)]
pub enum NumberOrDefinedValue {
    Number(i64),
    DefinedValue(DefinedValue),
}

#[derive(Debug)]
pub enum Element {
    SubType(SubTypeElement),
    ElementSet(Vec<Vec<Element>>),
}

#[derive(Debug)]
pub enum SubTypeElement {
    Value(Value),
}

#[derive(Debug)]
pub enum SequenceOfType {
    Type(Type),
    Named(String, Type),
}

