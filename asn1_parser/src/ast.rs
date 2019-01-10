use std::collections::HashMap;
use std::iter::Peekable;
use std::str::FromStr;

use pest::iterators::{FlatPairs, Pair};

use super::{Result, Rule};

pub(crate) struct Ast<'a>(Peekable<FlatPairs<'a, Rule>>);

impl<'a> Ast<'a> {
    /// Parse asn1 module into an Abstract Syntax Tree (AST) represented by the `Module` struct.
    pub fn parse(input: Peekable<FlatPairs<'a, Rule>>) -> Result<Module> {
        for pair in input.clone() {
            //println!("RULE: {:?}, STR: {:?}", pair.as_rule(), pair.as_str());
        }

        Ast(input).parse_module()
    }

    /// Copies the lexer output and parses the module's identifying information into an Abstract
    /// Syntax Tree (AST) represented by the `ModuleIdentifier` struct.
    pub fn parse_header(input: Peekable<FlatPairs<'a, Rule>>) -> Result<ModuleIdentifier> {
        let mut ast = Ast(input);

        ast.take(Rule::ModuleDefinition);

        ast.parse_module_identifier()
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

    fn next_string(&mut self) -> Option<String> {
        self.0.next().map(|x| x.as_str().to_owned())
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

    fn parse_module(&mut self) -> Result<Module> {
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

    fn parse_module_identifier(&mut self) -> Result<ModuleIdentifier> {
        self.take(Rule::ModuleIdentifier);

        let mut module_identifier = ModuleIdentifier::new(self.parse_reference_identifier());

        if self.look(Rule::DefinitiveIdentification).is_some() {
            self.take(Rule::DefinitiveOID);

            while self.look(Rule::DefinitiveObjIdComponent).is_some() {
                let pair = self.next().unwrap();

                let component = match pair.as_rule() {
                    Rule::NameForm => DefinitiveObjIdComponent::Name(self.parse_identifier()),
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
            let assignment = match self.next_rule().unwrap() {
                Rule::TypeAssignment => {
                    let ident = self.parse_reference_identifier();
                    let r#type = self.parse_type();

                    Assignment::Type(ident, r#type)
                }
                Rule::ValueAssignment => {
                    let ident = self.take(Rule::valuereference).as_str().to_owned();
                    let value_type = self.parse_type();
                    let value = self.parse_value();

                    Assignment::Value(ident, value_type, value)
                }
                Rule::ObjectClassAssignment => unimplemented!(),
                Rule::ObjectAssignment => {
                    let name = self.parse_object_reference();
                    let class = self.parse_defined_object_class();
                    let object = self.parse_object();

                    Assignment::Object(name, class, object)
                },
                Rule::ObjectSetAssignment => {
                    let name = self.parse_object_set_reference();
                    let class = self.parse_defined_object_class();
                    let set = self.parse_object_set();

                    Assignment::ObjectSet(name, class, set)
                },
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

                    let constraint = Some(self.parse_constraint());

                    Type {
                        raw_type,
                        constraint,
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn parse_constraint(&mut self) -> Constraint {
        if self.look(Rule::GeneralConstraint).is_some() {
            match self.rule_peek().unwrap() {
                Rule::TableConstraint => {
                    self.take(Rule::TableConstraint);

                    if self.look(Rule::ComponentRelationConstraint).is_some() {
                        let object_set = self.parse_defined_object_set();
                        let mut components = Vec::new();

                        while self.look(Rule::AtNotation).is_some() {
                            let mut component_ids = Vec::new();

                            if self.peek(Rule::Level) {
                                unimplemented!("Leveled constraints currently not supported");
                            } else {
                                self.take(Rule::ComponentIdList);

                                while self.peek(Rule::Identifier) {
                                    component_ids.push(self.parse_identifier());
                                }
                            }

                            components.push(component_ids);
                        }

                        Constraint::General(GeneralConstraint::Table(object_set, components))
                    } else {
                        let (set, extendable) = self.parse_object_set();
                        Constraint::General(GeneralConstraint::ObjectSet(set, extendable))
                    }
                }
                Rule::ContentsConstraint => unimplemented!(),
                Rule::UserDefinedConstraint => unimplemented!(),
                _ => unreachable!(),
            }
        } else {
            let is_extendable =
                self.take(Rule::ElementSetSpecs).as_str().contains("...");

            Constraint::ElementSet(self.parse_element_set_spec(), is_extendable)
        }
    }

    fn parse_element_set_spec(&mut self) -> Vec<Vec<Element>> {
        let mut constraint = Vec::new();

        if self.look(Rule::ElementSetSpec).is_none() {
            return constraint
        }

        self.take(Rule::Unions);

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
                Rule::ObjectClassFieldType => {
                    let class = self.parse_defined_object_class();

                    let field_name = self.parse_field_name();

                    RawType::Builtin(BuiltinType::ObjectClassField(class, field_name))
                }
                Rule::ObjectIdentifierType => RawType::Builtin(BuiltinType::ObjectIdentifier),
                Rule::OctetStringType => RawType::Builtin(BuiltinType::OctetString),
                Rule::SequenceType => {
                    RawType::Builtin(BuiltinType::Sequence(self.parse_component_type_lists()))
                }
                Rule::SequenceOfType => {
                    if self.peek(Rule::Type) {
                        RawType::Builtin(BuiltinType::SequenceOf(Box::new(TypeKind::Type(self.parse_type()))))
                    } else {
                        let (ident, r#type) = self.parse_named_type();
                        RawType::Builtin(BuiltinType::SequenceOf(Box::new(TypeKind::Named(ident, r#type))))
                    }
                }
                Rule::SetType => unimplemented!(),
                Rule::SetOfType => {
                    if self.peek(Rule::Type) {
                        RawType::Builtin(BuiltinType::SetOf(Box::new(TypeKind::Type(self.parse_type()))))
                    } else {
                        let (ident, r#type) = self.parse_named_type();
                        RawType::Builtin(BuiltinType::SetOf(Box::new(TypeKind::Named(ident, r#type))))
                    }
                }
                Rule::PrefixedType => {
                    if self.look(Rule::TaggedType).is_some() {
                        self.take(Rule::Tag);

                        let encoding = if self.look(Rule::EncodingReference).is_some() {
                            Some(self.parse_encoding_reference())
                        } else {
                            None
                        };

                        let class = self.look(Rule::Class).and_then(|r| r.as_str().parse().ok());

                        let pair = self.take(Rule::ClassNumber).as_str().to_owned();
                        let number = if self.peek(Rule::DefinedValue) {
                            NumberOrDefinedValue::DefinedValue(self.parse_defined_value())
                        } else {
                            NumberOrDefinedValue::Number(pair.as_str().parse().unwrap())
                        };

                        let r#type = Box::new(self.parse_type());

                        RawType::Builtin(BuiltinType::Prefixed(Prefix::new(encoding, class, number), r#type))
                    } else {
                        unimplemented!("Encoding prefixed types are not supported currently.")
                    }
                }
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

            Rule::SequenceValue => BuiltinValue::Sequence(self.parse_sequence_value()),
            Rule::EnumeratedValue => self.parse_enumerated_value(),

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

    fn parse_encoding_identifier(&mut self) -> String {
        self.take(Rule::EncodingIdentifier).as_str().to_owned()
    }

    fn parse_module_reference(&mut self) -> String {
        self.take(Rule::modulereference).as_str().to_owned()
    }

    fn parse_object_reference(&mut self) -> String {
        self.take(Rule::objectreference).as_str().to_owned()
    }

    fn parse_object_set_reference(&mut self) -> String {
        self.take(Rule::objectsetreference).as_str().to_owned()
    }

    fn parse_encoding_reference(&mut self) -> String {
        self.take(Rule::encodingreference).as_str().to_owned()
    }

    fn parse_literal(&mut self) -> String {
        self.take(Rule::Literal).as_str().to_owned()
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
                    Rule::Type => Element::SubType(SubTypeElement::Type(self.parse_type())),
                    e => unreachable!("{:?}", e),
                }
            }
            Rule::ObjectSetElements => {
                self.take(Rule::ObjectSetElements);

                match self.rule_peek().unwrap() {
                    Rule::Object => unimplemented!("Object"),
                    Rule::DefinedObjectSet => Element::ObjectSet(ObjectSetElements::Defined(self.parse_defined_object_set())),
                    Rule::ObjectSetFromObjects => unimplemented!("ObjectSetFromObjects"),
                    Rule::ParameterizedObjectSet => unimplemented!("ParamterizedObjectSet"),
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
    }

    fn parse_defined_object_class(&mut self) -> DefinedObjectClass {
        self.take(Rule::DefinedObjectClass);

        match self.rule_peek().unwrap() {
            Rule::ExternalObjectClassReference => {
                self.take(Rule::ExternalObjectClassReference);

                DefinedObjectClass::External(self.parse_reference_identifier(), self.parse_encoding_identifier())
            }
            Rule::EncodingIdentifier => DefinedObjectClass::Internal(self.parse_encoding_identifier()),
            Rule::UsefulObjectClassReference => {
                if self.take(Rule::UsefulObjectClassReference).as_str().contains("ABSTRACT-SYNTAX") {
                    DefinedObjectClass::AbstractSyntax
                } else {
                    DefinedObjectClass::TypeIdentifier
                }
            }
            _ => unreachable!(),
        }
    }

    fn parse_field_name(&mut self) -> Vec<Field> {
        self.take(Rule::FieldName);
        let mut field_names = Vec::new();

        while self.look(Rule::PrimitiveFieldName).is_some() {
            let kind = match self.rule_peek().unwrap() {
                Rule::typefieldreference => FieldType::Type,
                Rule::valuefieldreference => FieldType::Value,
                Rule::valuesetfieldreference => FieldType::ValueSet,
                Rule::objectfieldreference => FieldType::Object,
                Rule::objectsetfieldreference => FieldType::ObjectSet,
                _ => unreachable!(),
            };

            field_names.push(Field::new(self.next_string().unwrap().trim_matches('&').to_owned(), kind));
        }

        field_names
    }

    fn parse_defined_object_set(&mut self) -> DefinedObjectSet {
        self.take(Rule::DefinedObjectSet);

        match self.rule_peek().unwrap() {
            Rule::ExternalObjectSetReference => {
                self.take(Rule::ExternalObjectSetReference);

                DefinedObjectSet::External(self.parse_module_reference(), self.parse_object_set_reference())
            }

            Rule::objectsetreference => DefinedObjectSet::Internal(self.parse_object_set_reference()),

            _ => unreachable!(),

        }
    }

    fn parse_object_set(&mut self) -> (Vec<Vec<Element>>, bool) {
        self.take(Rule::ObjectSet);

        let is_extendable = self.take(Rule::ObjectSetSpec).as_str().contains("...");

        (self.parse_element_set_spec(), is_extendable)
    }

    fn parse_object(&mut self) -> Object {
        self.take(Rule::Object);

        match self.rule_peek().unwrap() {
            Rule::DefinedObject => unimplemented!("DefinedObject"),
            Rule::ObjectDefn => {
                self.take(Rule::ObjectDefn);

                let mut tokens = Vec::new();

                if self.look(Rule::DefinedSyntax).is_some() {
                    while self.look(Rule::DefinedSyntaxToken).is_some() {
                        let token = if self.look(Rule::Setting).is_some() {
                            let setting = match self.rule_peek().unwrap() {
                                Rule::Type => Setting::Type(self.parse_type()),
                                Rule::Value => Setting::Value(self.parse_value()),
                                Rule::ValueSet => unimplemented!("ValueSet"),
                                Rule::Object => Setting::Object(self.parse_object()),
                                Rule::ObjectSet => Setting::ObjectSet(self.parse_object_set().0),
                                _ => unreachable!(),
                            };

                            ObjectDefn::Setting(setting)
                        } else {
                            ObjectDefn::Literal(self.parse_literal())
                        };

                        tokens.push(token);
                    }

                } else {
                    unimplemented!("Default Syntax is not currently suppported")
                }

                Object::Def(tokens)
            },
            Rule::ObjectFromObject => unimplemented!("ObjectFromObject"),
            Rule::ParameterizedObject => unimplemented!("ParameterizedObject"),
            _ => unreachable!(),
        }
    }

    fn parse_sequence_value(&mut self) -> Vec<NamedValue> {
        self.take(Rule::SequenceValue);

        self.parse_component_value_list()
    }

    fn parse_component_value_list(&mut self) -> Vec<NamedValue> {
        self.take(Rule::ComponentValueList);

        let mut values = Vec::new();

        while self.look(Rule::NamedValue).is_some() {
            values.push(NamedValue(self.parse_identifier(), self.parse_value()));
        }

        values
    }

    fn parse_enumerated_value(&mut self) -> BuiltinValue {
        self.take(Rule::EnumeratedValue);

        BuiltinValue::Enumerated(self.parse_identifier())

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
    Object(String, DefinedObjectClass, Object),
    ObjectSet(String, DefinedObjectClass, (Vec<Vec<Element>>, bool)),
}

#[derive(Debug)]
pub struct Type {
    raw_type: RawType,
    // First `Vec` is a list of unions, the second a list of intersections.
    constraint: Option<Constraint>,
}

impl From<RawType> for Type {
    fn from(raw_type: RawType) -> Self {
        Type {
            raw_type,
            constraint: None,
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
    ObjectClassField(DefinedObjectClass, Vec<Field>),
    ObjectIdentifier,
    OctetString,
    Sequence(Vec<ComponentType>),
    SequenceOf(Box<TypeKind>),
    SetType,
    SetOf(Box<TypeKind>),
    Prefixed(Prefix, Box<Type>),
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
pub enum Constraint {
    General(GeneralConstraint),
    ElementSet(Vec<Vec<Element>>, bool),
}

#[derive(Debug)]
pub enum GeneralConstraint {
    Table(DefinedObjectSet, Vec<Vec<String>>),
    ObjectSet(Vec<Vec<Element>>, bool),
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
    Sequence(Vec<NamedValue>),
    Enumerated(String),
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
    ObjectSet(ObjectSetElements),
}

#[derive(Debug)]
pub enum SubTypeElement {
    Value(Value),
    Type(Type),
}

#[derive(Debug)]
pub enum TypeKind {
    Type(Type),
    Named(String, Type),
}

#[derive(Debug)]
pub enum DefinedObjectClass {
    External(String, String),
    Internal(String),
    AbstractSyntax,
    TypeIdentifier,
}

#[derive(Debug)]
pub enum DefinedObjectSet {
    External(String, String),
    Internal(String),
}

#[derive(Debug)]
pub struct Field {
    name: String,
    kind: FieldType,
}

impl Field {
    fn new(name: String, kind: FieldType) -> Self {
        Self { name, kind }
    }
}

#[derive(Debug)]
pub enum FieldType {
    Type,
    Value,
    ValueSet,
    Object,
    ObjectSet,
}

#[derive(Debug)]
pub enum ObjectSetElements {
    Defined(DefinedObjectSet)
}

#[derive(Debug)]
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
            _ => Err(())
        }
    }
}

#[derive(Debug)]
pub struct Prefix {
    encoding: Option<String>,
    class: Option<Class>,
    number: NumberOrDefinedValue,
}

impl Prefix {
    fn new(encoding: Option<String>, class: Option<Class>, number: NumberOrDefinedValue) -> Self {
        Self { encoding, class, number }
    }
}

#[derive(Debug)]
pub enum Object {
    Def(Vec<ObjectDefn>),
}

#[derive(Debug)]
pub enum ObjectDefn {
    Setting(Setting),
    Literal(String)
}

#[derive(Debug)]
pub enum Setting {
    Type(Type),
    Value(Value),
    Object(Object),
    ObjectSet(Vec<Vec<Element>>),
}

#[derive(Debug)]
pub struct NamedValue(String, Value);
