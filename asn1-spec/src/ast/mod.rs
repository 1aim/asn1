pub mod module;
pub mod oid;
pub mod types;

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::FromStr;

use derefable::Derefable;
use pest::iterators::{FlatPairs, Pair};
use pest::Parser;
use variation::Variation;

use super::Result;

pub use self::module::*;
pub use self::oid::*;
pub use self::types::*;

type ElementSet = Vec<Vec<Element>>;

#[derive(Parser)]
#[grammar = "asn1.pest"]
struct Asn1Parser;

pub(crate) struct Ast<'a>(Peekable<FlatPairs<'a, Rule>>);

impl<'a> Ast<'a> {
    /// Parse asn1 module into an Abstract Syntax Tree (AST) represented by the `Module` struct.
    pub fn parse(source: &'a str) -> Result<Module> {
        Self::new(Rule::ModuleDefinition, source)?.parse_module()
    }

    /// Copies the lexer output and parses the module's identifying information into an Abstract
    /// Syntax Tree (AST) represented by the `ModuleIdentifier` struct.
    pub fn parse_header(source: &'a str) -> Result<ModuleIdentifier> {
        let mut ast = Self::new(Rule::ModuleHeaderOnly, source)?;
        ast.take(Rule::ModuleHeaderOnly);

        ast.parse_module_identifier()
    }

    fn new(rule: Rule, source: &'a str) -> Result<Self> {
        let iter = Asn1Parser::parse(rule, source)?;

        Ok(Self(iter.flatten().peekable()))
    }

    fn next(&mut self) -> Option<Pair<Rule>> {
        self.0.next()
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

    /// Takes the next pair, and asserts that's its `Rule` matches `rule`.
    ///
    /// # Panics
    /// If `rule` doesn't match the next rule in the iterator.
    fn take(&mut self, rule: Rule) -> Pair<Rule> {
        let pair = self.0.next().unwrap();
        let expected = pair.as_rule();
        if rule != expected {
            eprintln!("Parse Error: {:?} != {:?}", expected, rule);
            eprintln!("===================LINE==================");
            eprintln!("{}", pair.as_str());
            eprintln!("===================REST==================");
            eprintln!("{:#?}", &self.0.clone().collect::<Vec<_>>()[..5]);
            eprintln!("=========================================");
            panic!("Parse Error: {:?} != {:?}", expected, rule);
            //::std::process::exit(-1);
        }

        pair
    }

    /// Look at the next pair and checks if its `Rule` matches `rule`, consumes the pair if it
    /// matches. Useful for checking for optional wrapper rules.
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
        //let extension = self.parse_extension_default();

        let (exports, imports, assignments) = if self.look(Rule::ModuleBody).is_some() {
            let exports = self.parse_exports();
            let imports = self.parse_imports();
            let assignments = self.parse_assignments();

            (exports, imports, assignments)
        } else {
            (Exports::All, Vec::new(), Vec::new())
        };

        self.take(Rule::EOI);

        Ok(Module {
            identifier,
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
                    Rule::NameForm => ObjIdComponent::Name(self.parse_identifier()),
                    Rule::DefinitiveNumberForm => ObjIdComponent::Number(pair.as_str().parse()?),
                    Rule::DefinitiveNameAndNumberForm => {
                        let name = self.parse_identifier();
                        let number = self.take(Rule::DefinitiveNumberForm).as_str().parse()?;
                        ObjIdComponent::NameAndNumber(name, number)
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
        if self.look(Rule::Exports).is_some() && self.peek(Rule::SymbolList) {
            Exports::Symbols(self.parse_symbol_list())
        } else {
            Exports::All
        }
    }

    pub fn parse_symbol_list(&mut self) -> Vec<String> {
        self.take(Rule::SymbolList);
        let mut symbols = Vec::new();

        while self.look(Rule::Symbol).is_some() {
            match self.rule_peek().unwrap() {
                // TODO(Aaron): Support parameterization
                Rule::Reference => {
                    symbols.push(self.parse_reference());
                }
                Rule::ParameterizedReference => {
                    self.take(Rule::ParameterizedReference);
                    symbols.push(self.parse_reference());
                }
                r => unreachable!("Unexpected rule: {:?}", r),
            }
        }

        symbols
    }

    pub fn parse_imports(&mut self) -> Vec<(ModuleReference, Vec<String>)> {
        let mut imports = Vec::new();

        if self.look(Rule::Imports).is_some() {
            while self.look(Rule::SymbolsFromModule).is_some() {
                let symbol_list = self.parse_symbol_list();
                self.take(Rule::GlobalModuleReference);
                let module_name = self.parse_reference_identifier();

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

                imports.push((
                    ModuleReference::new(module_name, identification),
                    symbol_list,
                ));
            }
        }

        imports
    }

    pub fn parse_object_identifier_value(&mut self) -> ObjectIdentifier {
        self.take(Rule::ObjectIdentifierValue);
        let mut components = Vec::new();

        while self.look(Rule::ObjIdComponents).is_some() {
            let component = match self.rule_peek().unwrap() {
                Rule::Identifier => ObjIdComponent::Name(self.parse_identifier()),
                Rule::NumberForm => {
                    ObjIdComponent::Number(self.take(Rule::NumberForm).as_str().parse().unwrap())
                }
                Rule::NameAndNumberForm => {
                    self.take(Rule::NameAndNumberForm);
                    let name = self.parse_identifier();
                    let number = self.take(Rule::NumberForm).as_str().parse().unwrap();

                    ObjIdComponent::NameAndNumber(name, number)
                }
                _ => unreachable!(),
            };

            components.push(component)
        }

        ObjectIdentifier::from_components(components)
    }

    fn parse_defined_value(&mut self) -> DefinedValue {
        use self::SimpleDefinedValue::*;
        self.take(Rule::DefinedValue);

        match self.rule_peek().unwrap() {
            Rule::ExternalTypeReference => {
                let (parent, child) = self.parse_external_type_reference();
                DefinedValue::Simple(Reference(parent, child))
            }

            Rule::valuereference => DefinedValue::Simple(Value(self.parse_value_reference())),
            Rule::ParameterizedValue => unimplemented!(),
            _ => unreachable!(),
        }
    }

    fn parse_assignments(&mut self) -> Vec<Assignment> {
        let mut assignments = Vec::new();

        while self.look(Rule::Assignment).is_some() {
            let assignment_type = self.next_rule().unwrap();
            let ident = self.parse_reference();

            let parameter_list = if self.look(Rule::ParameterList).is_some() {
                let mut parameters = Vec::new();

                while self.look(Rule::Parameter).is_some() {
                    let governor = if self.look(Rule::ParamGovernor).is_some() {
                        match self.rule_peek().unwrap() {
                            Rule::Governor => {
                                self.take(Rule::Governor);
                                let g = match self.rule_peek().unwrap() {
                                    Rule::Type => ParamGovernor::Type(self.parse_type()),
                                    Rule::DefinedObjectClass => {
                                        ParamGovernor::Class(self.parse_defined_object_class())
                                    }
                                    _ => unreachable!(),
                                };

                                Some(g)
                            }
                            Rule::Reference => {
                                Some(ParamGovernor::Reference(self.parse_reference()))
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        None
                    };

                    parameters.push((governor, self.parse_reference()));
                }

                Some(parameters)
            } else {
                None
            };

            let kind = match assignment_type {
                Rule::TypeAssignment => AssignmentType::Type(self.parse_type()),
                Rule::ValueAssignment => {
                    AssignmentType::Value(self.parse_type(), self.parse_value())
                }
                Rule::ValueSetAssignment => {
                    AssignmentType::ValueSet(self.parse_type(), self.parse_value_set())
                }
                Rule::ObjectClassAssignment => {
                    AssignmentType::ObjectClass(self.parse_object_class())
                }
                Rule::ObjectAssignment => {
                    AssignmentType::Object(self.parse_defined_object_class(), self.parse_object())
                }
                Rule::ObjectSetAssignment => AssignmentType::ObjectSet(
                    self.parse_defined_object_class(),
                    self.parse_object_set(),
                ),
                _ => unreachable!(),
            };

            assignments.push(Assignment::new(ident, kind, parameter_list))
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
                    let is_set = self.take(Rule::TypeWithConstraint).as_str().contains("SET");

                    let constraint = if self.peek(Rule::Constraint) {
                        self.parse_constraint()
                    } else {
                        self.parse_size_constraint()
                    };

                    let inner_type = if self.peek(Rule::NamedType) {
                        self.parse_named_type()
                    } else {
                        self.parse_type()
                    };

                    let raw_type = if is_set {
                        RawType::Builtin(BuiltinType::SetOf(Box::new(inner_type)))
                    } else {
                        RawType::Builtin(BuiltinType::SequenceOf(Box::new(inner_type)))
                    };

                    Type {
                        raw_type,
                        name: None,
                        constraints: Some(vec![constraint]),
                    }
                } else {
                    let raw_type = self.parse_unconstrained_type();
                    let mut constraints = Vec::new();

                    while self.peek(Rule::Constraint) {
                        constraints.push(self.parse_constraint());
                    }

                    Type {
                        raw_type,
                        name: None,
                        constraints: Some(constraints),
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn parse_constraint(&mut self) -> Constraint {
        self.take(Rule::Constraint);
        self.take(Rule::ConstraintSpec);

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
            let is_extendable = self.take(Rule::ElementSetSpecs).as_str().contains("...");

            Constraint::ElementSet(self.parse_element_set_spec(), is_extendable)
        }
    }

    fn parse_size_constraint(&mut self) -> Constraint {
        self.take(Rule::SizeConstraint);
        self.parse_constraint()
    }

    fn parse_element_set_specs(&mut self) -> ElementSetSpec {
        let has_ellipsis = self.take(Rule::ElementSetSpecs).as_str().contains("...");
        let set = self.parse_element_set_spec();

        let extensible = if has_ellipsis {
            if self.peek(Rule::ElementSetSpec) {
                let with = self.parse_element_set_spec();

                Extensible::YesWith(with)
            } else {
                Extensible::Yes
            }
        } else {
            Extensible::No
        };

        ElementSetSpec { set, extensible }
    }

    fn parse_element_set_spec(&mut self) -> ElementSet {
        let mut element_set = Vec::new();

        if self.look(Rule::ElementSetSpec).is_none() {
            return element_set;
        }

        self.take(Rule::Unions);

        while self.look(Rule::Intersections).is_some() {
            let mut intersections = Vec::new();
            while self.look(Rule::IntersectionElements).is_some() {
                intersections.push(self.parse_elements());
                self.look(Rule::IntersectionMark);
            }

            element_set.push(intersections);
            self.look(Rule::UnionMark);
        }

        element_set
    }

    fn parse_unconstrained_type(&mut self) -> RawType {
        self.take(Rule::UnconstrainedType);

        if self.look(Rule::BuiltinType).is_some() {
            let pair = self.next().unwrap();
            match pair.as_rule() {
                Rule::BooleanType => RawType::Builtin(BuiltinType::Boolean),

                Rule::CharacterStringType => {
                    let pair = self.next().unwrap();
                    let char_type = if pair.as_rule() == Rule::UnrestrictedCharacterStringType {
                        CharacterStringType::Unrestricted
                    } else {
                        match pair.as_str() {
                            "BMPString" => CharacterStringType::Bmp,
                            "GeneralString" => CharacterStringType::General,
                            "GraphicString" => CharacterStringType::Graphic,
                            "IA5String" => CharacterStringType::Ia5,
                            "ISO646String" => CharacterStringType::Iso646,
                            "NumericString" => CharacterStringType::Numeric,
                            "PrintableString" => CharacterStringType::Printable,
                            "TeletexString" => CharacterStringType::Teletex,
                            "T61String" => CharacterStringType::T61,
                            "UniversalString" => CharacterStringType::Universal,
                            "UTF8String" => CharacterStringType::Utf8,
                            "VideotexString" => CharacterStringType::Videotex,
                            "VisibleString" => CharacterStringType::Visible,
                            _ => unreachable!(),
                        }
                    };

                    RawType::Builtin(BuiltinType::CharacterString(char_type))
                }

                Rule::ChoiceType => {
                    self.take(Rule::AlternativeTypeLists);
                    self.take(Rule::AlternativeTypeList);
                    let mut alternatives = Vec::new();

                    while self.peek(Rule::NamedType) {
                        alternatives.push(self.parse_named_type());
                    }

                    let extension = self.parse_extension_and_exception();

                    RawType::Builtin(BuiltinType::Choice(ChoiceType {
                        alternatives,
                        extension,
                    }))
                }

                Rule::EnumeratedType => {
                    self.take(Rule::Enumerations);

                    let enumerations = self.parse_enumeration();

                    let exception_spec = if self.peek(Rule::ExceptionSpec) {
                        Some(self.parse_exception_spec())
                    } else {
                        None
                    };

                    let extended_enumerations = if self.peek(Rule::Enumeration) {
                        Some(self.parse_enumeration())
                    } else {
                        None
                    };

                    RawType::Builtin(BuiltinType::Enumeration(
                        enumerations,
                        exception_spec,
                        extended_enumerations,
                    ))
                }

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

                        RawType::Builtin(BuiltinType::Prefixed(
                            Prefix::new(encoding, class, number),
                            r#type,
                        ))
                    } else {
                        unimplemented!("Encoding prefixed types are not supported currently.")
                    }
                }

                Rule::SequenceType => {
                    RawType::Builtin(BuiltinType::Sequence(self.parse_component_type_lists()))
                }

                Rule::SequenceOfType => {
                    if self.peek(Rule::Type) {
                        RawType::Builtin(BuiltinType::SequenceOf(Box::new(self.parse_type())))
                    } else {
                        RawType::Builtin(BuiltinType::SequenceOf(Box::new(self.parse_named_type())))
                    }
                }

                Rule::SetType => unimplemented!(),
                Rule::SetOfType => {
                    if self.peek(Rule::Type) {
                        RawType::Builtin(BuiltinType::SetOf(Box::new(self.parse_type())))
                    } else {
                        RawType::Builtin(BuiltinType::SetOf(Box::new(self.parse_named_type())))
                    }
                }

                r => unreachable!("Unexpected rule: {:?}", r),
            }
        } else {
            self.take(Rule::ReferencedType);

            match self.next_rule().unwrap() {
                Rule::DefinedType => match self.rule_peek().unwrap() {
                    Rule::ExternalTypeReference => {
                        let (module, subtype) = self.parse_external_type_reference();

                        ReferenceType::External(module, subtype).into()
                    }

                    Rule::typereference => {
                        ReferenceType::Internal(self.parse_type_reference()).into()
                    }

                    Rule::ParameterizedType => {
                        self.take(Rule::ParameterizedType);

                        let reference = self.parse_simple_defined_type();
                        let parameters = self.parse_actual_parameter_list();

                        RawType::Parameterized(reference, parameters)
                    }
                    Rule::ParameterizedValueSet => unimplemented!(),

                    r => unreachable!("Unexpected rule: {:?}", r),
                },
                Rule::TypeFromObject => unimplemented!(),
                Rule::ValueSetFromObjects => unimplemented!(),
                _ => unreachable!(),
            }
        }
    }

    fn parse_simple_defined_type(&mut self) -> ReferenceType {
        self.take(Rule::SimpleDefinedType);

        match self.rule_peek().unwrap() {
            Rule::ExternalTypeReference => {
                let (parent, child) = self.parse_external_type_reference();

                ReferenceType::External(parent, child).into()
            }
            Rule::typereference => ReferenceType::Internal(self.parse_type_reference()).into(),
            _ => unreachable!(),
        }
    }

    fn parse_actual_parameter_list(&mut self) -> Vec<Parameter> {
        self.take(Rule::ActualParameterList);
        let mut parameters = Vec::new();

        while self.look(Rule::ActualParameter).is_some() {
            let parameter = match self.rule_peek().unwrap() {
                Rule::Type => Parameter::Type(self.parse_type()),
                Rule::Value => Parameter::Value(self.parse_value()),
                Rule::ValueSet => Parameter::ValueSet(self.parse_value_set()),
                Rule::DefinedObjectClass => {
                    Parameter::ObjectClass(self.parse_defined_object_class())
                }
                Rule::Object => Parameter::Object(self.parse_object()),
                Rule::ObjectSet => Parameter::ObjectSet(self.parse_object_set()),
                _ => unreachable!(),
            };

            parameters.push(parameter);
        }

        parameters
    }

    fn parse_value_set(&mut self) -> ElementSetSpec {
        self.take(Rule::ValueSet);

        self.parse_element_set_specs()
    }

    fn parse_value(&mut self) -> Value {
        self.take(Rule::Value);

        match self.rule_peek().unwrap() {
            Rule::BuiltinValue => self.parse_builtin_value(),
            Rule::ReferencedValue => self.parse_referenced_value(),
            Rule::ObjectClassFieldType => unimplemented!(),
            _ => unreachable!(),
        }
    }

    fn parse_builtin_value(&mut self) -> Value {
        self.take(Rule::BuiltinValue);

        match self.rule_peek().unwrap() {
            Rule::IntegerValue => {
                self.take(Rule::IntegerValue);

                let value = match self.rule_peek().unwrap() {
                    Rule::SignedNumber => IntegerValue::Literal(self.parse_signed_number()),
                    Rule::Identifier => IntegerValue::Identifier(self.parse_identifier()),
                    _ => unreachable!(),
                };

                Value::Integer(value)
            }

            Rule::ObjectIdentifierValue => {
                Value::ObjectIdentifier(self.parse_object_identifier_value())
            }

            Rule::SequenceValue => self.parse_sequence_value(),
            Rule::EnumeratedValue => self.parse_enumerated_value(),
            Rule::BooleanValue => self.parse_boolean_value(),

            e => unreachable!("Unexpected Rule {:?}", e),
        }
    }

    fn parse_referenced_value(&mut self) -> Value {
        self.take(Rule::ReferencedValue);

        match self.rule_peek().unwrap() {
            Rule::DefinedValue => Value::Defined(self.parse_defined_value()),
            Rule::ValueFromObject => {
                let s = self
                    .take(Rule::ValueFromObject)
                    .as_str()
                    .split(".")
                    .map(String::from)
                    .collect();

                Value::Object(s)
            }
            _ => unreachable!(),
        }
    }

    fn parse_signed_number(&mut self) -> i64 {
        self.take(Rule::SignedNumber).as_str().parse().unwrap()
    }

    fn parse_identifier(&mut self) -> String {
        self.parse_to_str(Rule::Identifier)
    }

    fn parse_value_reference(&mut self) -> String {
        self.parse_to_str(Rule::valuereference)
    }

    fn parse_reference(&mut self) -> String {
        const VALID_RULES: [Rule; 8] = [
            Rule::Reference,
            // These rules are also allowed, so that this can be called in parse_assignment.
            Rule::EncodingIdentifier,
            Rule::ReferenceIdentifier,
            Rule::typereference,
            Rule::valuereference,
            Rule::objectclassreference,
            Rule::objectreference,
            Rule::objectsetreference,
        ];

        let pair = self.next().unwrap();

        let is_valid = VALID_RULES.into_iter().any(|rule| pair.as_rule() == *rule);

        if is_valid {
            pair.as_str().to_owned()
        } else {
            panic!("{:?} != {:?}", pair.as_rule(), VALID_RULES);
        }
    }

    fn parse_reference_identifier(&mut self) -> String {
        self.parse_to_str(Rule::ReferenceIdentifier)
    }

    fn parse_type_reference(&mut self) -> String {
        self.parse_to_str(Rule::typereference)
    }

    fn parse_encoding_identifier(&mut self) -> String {
        self.parse_to_str(Rule::EncodingIdentifier)
    }

    fn parse_module_reference(&mut self) -> String {
        self.parse_to_str(Rule::modulereference)
    }

    fn parse_object_reference(&mut self) -> String {
        self.parse_to_str(Rule::objectreference)
    }

    fn parse_object_set_reference(&mut self) -> String {
        self.parse_to_str(Rule::objectsetreference)
    }

    fn parse_to_str(&mut self, rule: Rule) -> String {
        self.take(rule).as_str().to_owned()
    }

    fn parse_value_field_reference(&mut self) -> String {
        self.parse_field_reference(Rule::valuefieldreference)
    }

    fn parse_value_set_field_reference(&mut self) -> String {
        self.parse_field_reference(Rule::valuesetfieldreference)
    }

    fn parse_object_field_reference(&mut self) -> String {
        self.parse_field_reference(Rule::objectfieldreference)
    }

    fn parse_object_set_field_reference(&mut self) -> String {
        self.parse_field_reference(Rule::objectsetfieldreference)
    }

    fn parse_type_field_reference(&mut self) -> String {
        self.parse_field_reference(Rule::typefieldreference)
    }

    fn parse_field_reference(&mut self, rule: Rule) -> String {
        self.take(rule).as_str().trim_matches('&').to_owned()
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
            let ty = self.parse_named_type();
            let optional = raw.contains("OPTIONAL");
            let default = match raw.contains("DEFAULT") {
                true => Some(self.parse_value()),
                false => None,
            };

            ComponentType::Type {
                ty,
                optional,
                default,
            }
        }
    }

    fn parse_named_type(&mut self) -> Type {
        self.take(Rule::NamedType);
        let ident = self.parse_identifier();
        let mut ty = self.parse_type();
        ty.name = Some(ident);

        ty
    }

    fn parse_elements(&mut self) -> Element {
        self.take(Rule::Elements);
        match self.rule_peek().unwrap() {
            Rule::ElementSetSpec => Element::ElementSet(self.parse_element_set_spec()),
            Rule::SubtypeElements => {
                self.take(Rule::SubtypeElements);
                let subtype = match self.rule_peek().unwrap() {
                    Rule::Value => SubTypeElement::Value(self.parse_value()),
                    Rule::Type => SubTypeElement::Type(self.parse_type()),
                    Rule::SizeConstraint => SubTypeElement::Size(self.parse_size_constraint()),
                    Rule::ValueRange => {
                        self.take(Rule::ValueRange);
                        let is_low_inclusive =
                            self.take(Rule::LowerEndpoint).as_str().contains('<');
                        let low_value = if self.take(Rule::LowerEndValue).as_str().contains("MIN") {
                            RangeValue::Min(is_low_inclusive)
                        } else {
                            RangeValue::Value(self.parse_value(), is_low_inclusive)
                        };

                        let is_high_inclusive =
                            self.take(Rule::UpperEndpoint).as_str().contains('<');
                        let high_value = if self.take(Rule::UpperEndValue).as_str().contains("MAX")
                        {
                            RangeValue::Max(is_high_inclusive)
                        } else {
                            RangeValue::Value(self.parse_value(), is_high_inclusive)
                        };

                        SubTypeElement::Range(low_value, high_value)
                    }
                    Rule::InnerTypeConstraints => {
                        self.take(Rule::InnerTypeConstraints);

                        if self.peek(Rule::Constraint) {
                            SubTypeElement::Constraint(self.parse_constraint())
                        } else {
                            self.take(Rule::MultipleTypeConstraints);

                            if self.look(Rule::FullSpecification).is_some() {
                                SubTypeElement::FullSpec(self.parse_type_constraints())
                            } else {
                                self.take(Rule::PartialSpecification);
                                SubTypeElement::PartialSpec(self.parse_type_constraints())
                            }
                        }
                    }
                    e => unreachable!("{:?}", e),
                };

                Element::SubType(subtype)
            }
            Rule::ObjectSetElements => {
                self.take(Rule::ObjectSetElements);

                match self.rule_peek().unwrap() {
                    Rule::Object => Element::Object(self.parse_object()),
                    Rule::DefinedObjectSet => Element::ObjectSet(ObjectSetElements::Defined(
                        self.parse_defined_object_set(),
                    )),
                    Rule::ObjectSetFromObjects => unimplemented!("ObjectSetFromObjects"),
                    Rule::ParameterizedObjectSet => unimplemented!("ParamterizedObjectSet"),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    fn parse_type_constraints(&mut self) -> HashMap<String, ComponentConstraint> {
        self.take(Rule::TypeConstraints);
        let mut map = HashMap::new();

        while self.look(Rule::NamedConstraint).is_some() {
            let name = self.parse_identifier();
            self.take(Rule::ComponentConstraint);

            let constraint = if self.peek(Rule::Constraint) {
                Some(self.parse_constraint())
            } else {
                None
            };

            let presence = if self.peek(Rule::PresenceConstraint) {
                let p = match self.next().unwrap().as_str() {
                    "PRESENT" => Presence::Present,
                    "ABSENT" => Presence::Absent,
                    "OPTIONAL" => Presence::Optional,
                    _ => unreachable!(),
                };

                Some(p)
            } else {
                None
            };

            map.insert(name, ComponentConstraint::new(constraint, presence));
        }

        map
    }

    fn parse_defined_object_class(&mut self) -> DefinedObjectClass {
        self.take(Rule::DefinedObjectClass);

        match self.rule_peek().unwrap() {
            Rule::ExternalObjectClassReference => {
                self.take(Rule::ExternalObjectClassReference);

                DefinedObjectClass::External(
                    self.parse_reference_identifier(),
                    self.parse_encoding_identifier(),
                )
            }
            Rule::EncodingIdentifier => {
                DefinedObjectClass::Internal(self.parse_encoding_identifier())
            }
            Rule::UsefulObjectClassReference => {
                if self
                    .take(Rule::UsefulObjectClassReference)
                    .as_str()
                    .contains("ABSTRACT-SYNTAX")
                {
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

        while self.peek(Rule::PrimitiveFieldName) {
            field_names.push(self.parse_primitive_field_name());
        }

        field_names
    }

    fn parse_primitive_field_name(&mut self) -> Field {
        self.take(Rule::PrimitiveFieldName);

        let rule = self.rule_peek().unwrap();
        let kind = match rule {
            Rule::typefieldreference => FieldType::Type,
            Rule::valuefieldreference => FieldType::Value,
            Rule::valuesetfieldreference => FieldType::ValueSet,
            Rule::objectfieldreference => FieldType::Object,
            Rule::objectsetfieldreference => FieldType::ObjectSet,
            _ => unreachable!(),
        };

        Field::new(self.parse_field_reference(rule), kind)
    }

    fn parse_defined_object_set(&mut self) -> DefinedObjectSet {
        self.take(Rule::DefinedObjectSet);

        match self.rule_peek().unwrap() {
            Rule::ExternalObjectSetReference => {
                self.take(Rule::ExternalObjectSetReference);

                DefinedObjectSet::External(
                    self.parse_module_reference(),
                    self.parse_object_set_reference(),
                )
            }

            Rule::objectsetreference => {
                DefinedObjectSet::Internal(self.parse_object_set_reference())
            }

            _ => unreachable!(),
        }
    }

    fn parse_object_set(&mut self) -> (ElementSet, bool) {
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
                    //panic!("{:#?}", &self.0.clone().collect::<Vec<_>>()[..5]);
                    while self.look(Rule::DefinedSyntaxToken).is_some() {
                        let token = if self.look(Rule::Setting).is_some() {
                            let setting = match self.rule_peek().unwrap() {
                                Rule::Type => Setting::Type(self.parse_type()),
                                Rule::Value => Setting::Value(self.parse_value()),
                                Rule::ValueSet => Setting::ValueSet(self.parse_value_set()),
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
            }
            Rule::ObjectFromObject => unimplemented!("ObjectFromObject"),
            Rule::ParameterizedObject => unimplemented!("ParameterizedObject"),
            _ => unreachable!(),
        }
    }

    fn parse_sequence_value(&mut self) -> Value {
        self.take(Rule::SequenceValue);

        Value::Sequence(self.parse_component_value_list())
    }

    fn parse_component_value_list(&mut self) -> Vec<NamedValue> {
        self.take(Rule::ComponentValueList);

        let mut values = Vec::new();

        while self.look(Rule::NamedValue).is_some() {
            values.push(NamedValue(self.parse_identifier(), self.parse_value()));
        }

        values
    }

    fn parse_enumerated_value(&mut self) -> Value {
        self.take(Rule::EnumeratedValue);

        Value::Enumerated(self.parse_identifier())
    }

    fn parse_object_class(&mut self) -> ObjectClass {
        self.take(Rule::ObjectClass);

        match self.rule_peek().unwrap() {
            Rule::ObjectClassDefn => ObjectClass::Def(self.parse_object_class_defn()),
            Rule::ParameterizedObjectClass => unimplemented!("ParameterizedObjectClass"),
            Rule::DefinedObjectClass => ObjectClass::Defined(self.parse_defined_object_class()),
            _ => unreachable!(),
        }
    }

    fn parse_object_class_defn(&mut self) -> ClassDef {
        self.take(Rule::ObjectClassDefn);
        let mut fields = Vec::new();

        while self.look(Rule::FieldSpec).is_some() {
            let field = match self.rule_peek().unwrap() {
                Rule::FixedTypeValueFieldSpec => self.parse_fixed_type_value_field_spec(),
                Rule::VariableTypeValueFieldSpec => self.parse_variable_type_value_field_spec(),
                Rule::FixedTypeValueSetFieldSpec => self.parse_fixed_type_value_set_field_spec(),
                Rule::VariableTypeValueSetFieldSpec => {
                    self.parse_variable_type_value_set_field_spec()
                }
                Rule::ObjectFieldSpec => self.parse_object_field_spec(),
                Rule::TypeFieldSpec => self.parse_type_field_spec(),
                Rule::ObjectSetFieldSpec => self.parse_object_set_field_spec(),
                _ => unreachable!(),
            };

            fields.push(field);
        }

        let syntax = if self.look(Rule::WithSyntaxSpec).is_some() {
            self.take(Rule::SyntaxList);
            Some(self.parse_token_or_group_spec())
        } else {
            None
        };

        ClassDef { fields, syntax }
    }

    fn parse_token_or_group_spec(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.look(Rule::TokenOrGroupSpec).is_some() {
            match self.next_rule().unwrap() {
                Rule::RequiredToken => {
                    let token = match self.rule_peek().unwrap() {
                        Rule::Literal => Token::Literal(self.parse_literal()),
                        Rule::PrimitiveFieldName => Token::Field(self.parse_primitive_field_name()),
                        _ => unreachable!(),
                    };

                    tokens.push(token);
                }
                Rule::OptionalGroup => {
                    tokens.push(Token::OptionalGroup(self.parse_token_or_group_spec()));
                }
                _ => unreachable!(),
            }
        }

        tokens
    }

    fn parse_fixed_type_value_field_spec(&mut self) -> FieldSpec {
        let is_unique = self
            .take(Rule::FixedTypeValueFieldSpec)
            .as_str()
            .contains("UNIQUE");
        let ident = self.parse_value_field_reference();
        let ty = self.parse_type();
        let optionality = self.parse_value_optionality_spec();

        FieldSpec::FixedTypeValue(ident, ty, is_unique, optionality)
    }

    fn parse_variable_type_value_field_spec(&mut self) -> FieldSpec {
        self.take(Rule::VariableTypeValueFieldSpec);

        let ident = self.parse_value_field_reference();
        let field_name = self.parse_field_name();
        let optionality = self.parse_value_optionality_spec();

        FieldSpec::VariableTypeValue(ident, field_name, optionality)
    }

    fn parse_fixed_type_value_set_field_spec(&mut self) -> FieldSpec {
        self.take(Rule::FixedTypeValueSetFieldSpec);

        let ident = self.parse_value_set_field_reference();
        let ty = self.parse_type();
        let optionality = self.parse_value_set_optionality_spec();

        FieldSpec::FixedValueSet(ident, ty, optionality)
    }

    fn parse_variable_type_value_set_field_spec(&mut self) -> FieldSpec {
        self.take(Rule::VariableTypeValueSetFieldSpec);
        let ident = self.parse_value_set_field_reference();
        let field = self.parse_field_name();
        let optionality = self.parse_value_optionality_spec();

        FieldSpec::VariableTypeValue(ident, field, optionality)
    }

    fn parse_object_field_spec(&mut self) -> FieldSpec {
        self.take(Rule::ObjectFieldSpec);
        let ident = self.parse_object_field_reference();
        let class = self.parse_defined_object_class();
        let optionality = self.parse_object_optionality_spec();

        FieldSpec::ObjectField(ident, class, optionality)
    }

    fn parse_type_field_spec(&mut self) -> FieldSpec {
        self.take(Rule::TypeFieldSpec);
        let ident = self.parse_type_field_reference();
        let optionality = self.parse_type_optionality_spec();

        FieldSpec::Type(ident, optionality)
    }

    fn parse_object_set_field_spec(&mut self) -> FieldSpec {
        self.take(Rule::ObjectSetFieldSpec);

        let ident = self.parse_object_set_field_reference();
        let class = self.parse_defined_object_class();
        let optionality = self.parse_object_set_optionality_spec();

        FieldSpec::ObjectSet(ident, class, optionality)
    }

    fn parse_value_optionality_spec(&mut self) -> Optionality<Value> {
        self.parse_optionality_spec(Rule::ValueOptionalitySpec, &Self::parse_value)
    }

    fn parse_value_set_optionality_spec(&mut self) -> Optionality<ElementSetSpec> {
        self.parse_optionality_spec(Rule::ValueSetOptionalitySpec, &Self::parse_value_set)
    }

    fn parse_object_optionality_spec(&mut self) -> Optionality<Object> {
        self.parse_optionality_spec(Rule::ObjectOptionalitySpec, &Self::parse_object)
    }

    fn parse_type_optionality_spec(&mut self) -> Optionality<Type> {
        self.parse_optionality_spec(Rule::TypeOptionalitySpec, &Self::parse_type)
    }

    fn parse_object_set_optionality_spec(&mut self) -> Optionality<(ElementSet, bool)> {
        self.parse_optionality_spec(Rule::ObjectSetOptionalitySpec, &Self::parse_object_set)
    }

    fn parse_optionality_spec<T>(
        &mut self,
        rule: Rule,
        parse_fn: &Fn(&mut Self) -> T,
    ) -> Optionality<T> {
        if !self.peek(rule) {
            Optionality::None
        } else {
            let pair = self.take(rule);

            if pair.as_str().contains("OPTIONAL") {
                Optionality::Optional
            } else {
                Optionality::Default((parse_fn)(self))
            }
        }
    }

    fn parse_external_type_reference(&mut self) -> (String, String) {
        self.take(Rule::ExternalTypeReference);

        (self.parse_module_reference(), self.parse_type_reference())
    }

    fn parse_external_value_reference(&mut self) -> (String, String) {
        self.take(Rule::ExternalValueReference);

        (self.parse_module_reference(), self.parse_value_reference())
    }

    fn parse_extension_and_exception(&mut self) -> Option<ExtensionAndException> {
        if !self.peek(Rule::ExtensionAndException) {
            return None;
        }

        self.take(Rule::ExtensionAndException);

        if self.peek(Rule::ExceptionSpec) {
            Some(ExtensionAndException::Exception(
                self.parse_exception_spec(),
            ))
        } else {
            Some(ExtensionAndException::Extension)
        }
    }

    fn parse_boolean_value(&mut self) -> Value {
        Value::Boolean(self.take(Rule::BooleanValue).as_str().contains("TRUE"))
    }

    fn parse_enumeration(&mut self) -> Vec<EnumerationType> {
        self.take(Rule::Enumeration);

        let mut enumerations = Vec::new();

        while self.look(Rule::EnumerationItem).is_some() {
            let variant = if self.peek(Rule::NamedNumber) {
                EnumerationType::NamedNumber(self.parse_named_number())
            } else {
                EnumerationType::Name(self.parse_identifier())
            };

            enumerations.push(variant);
        }

        enumerations
    }

    fn parse_named_number(&mut self) -> (String, NumberOrDefinedValue) {
        self.take(Rule::NamedNumber);

        let name = self.parse_identifier();

        let number = if self.peek(Rule::SignedNumber) {
            NumberOrDefinedValue::Number(self.parse_signed_number())
        } else {
            NumberOrDefinedValue::DefinedValue(self.parse_defined_value())
        };

        (name, number)
    }

    fn parse_exception_spec(&mut self) -> ExceptionIdentification {
        self.take(Rule::ExceptionSpec);
        self.take(Rule::ExceptionIdentification);

        match self.rule_peek().unwrap() {
            Rule::SignedNumber => ExceptionIdentification::Number(self.parse_signed_number()),
            Rule::DefinedValue => ExceptionIdentification::Reference(self.parse_defined_value()),
            Rule::Type => {
                ExceptionIdentification::Arbitrary(Box::new(self.parse_type()), self.parse_value())
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Variation)]
pub enum SimpleDefinedValue {
    /// An external type reference e.g. `foo.bar`
    Reference(String, String),
    /// Identifier pointing to a value
    Value(String),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Variation)]
pub enum DefinedValue {
    Simple(SimpleDefinedValue),
    /// Paramaterized value
    Parameterized(SimpleDefinedValue, Vec<()>),
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub name: String,
    pub kind: AssignmentType,
    pub parameters: Option<Vec<(Option<ParamGovernor>, String)>>,
}

impl Assignment {
    fn new(
        name: String,
        kind: AssignmentType,
        parameters: Option<Vec<(Option<ParamGovernor>, String)>>,
    ) -> Self {
        Self {
            name,
            kind,
            parameters,
        }
    }
}

// First argument is always the identifier.
#[derive(Clone, Debug, Variation)]
pub enum AssignmentType {
    Type(Type),
    Value(Type, Value),
    ValueSet(Type, ElementSetSpec),
    Object(DefinedObjectClass, Object),
    ObjectClass(ObjectClass),
    ObjectSet(DefinedObjectClass, (ElementSet, bool)),
}

#[derive(Clone, Debug, Variation)]
pub enum Constraint {
    General(GeneralConstraint),
    ElementSet(ElementSet, bool),
}

#[derive(Clone, Debug, Variation)]
pub enum GeneralConstraint {
    Table(DefinedObjectSet, Vec<Vec<String>>),
    ObjectSet(ElementSet, bool),
}

#[derive(Clone, Debug, Variation)]
pub enum Value {
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

#[derive(Clone, Debug, Variation)]
pub enum Element {
    SubType(SubTypeElement),
    ElementSet(ElementSet),
    Object(Object),
    ObjectSet(ObjectSetElements),
}

#[derive(Clone, Debug, Variation)]
pub enum SubTypeElement {
    Value(Value),
    Type(Type),
    Size(Constraint),
    Range(RangeValue, RangeValue),
    Constraint(Constraint),
    FullSpec(HashMap<String, ComponentConstraint>),
    PartialSpec(HashMap<String, ComponentConstraint>),
}

#[derive(Clone, Debug, Variation)]
pub enum ObjectClass {
    Def(ClassDef),
    Defined(DefinedObjectClass),
    Parameterized(String, Option<()>),
}

#[derive(Clone, Debug)]
pub struct ClassDef {
    fields: Vec<FieldSpec>,
    syntax: Option<Vec<Token>>,
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
    fn new(name: String, kind: FieldType) -> Self {
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

#[derive(Clone, Debug)]
pub struct NamedValue(String, Value);

#[derive(Clone, Debug, Variation)]
pub enum Extensible {
    Yes,
    YesWith(ElementSet),
    No,
}

#[derive(Clone, Debug)]
pub struct ElementSetSpec {
    set: ElementSet,
    extensible: Extensible,
}

#[derive(Clone, Debug, Variation)]
pub enum ParamGovernor {
    Type(Type),
    Class(DefinedObjectClass),
    Reference(String),
}

#[derive(Clone, Debug, Variation)]
pub enum ExtensionAndException {
    Extension,
    Exception(ExceptionIdentification),
}

#[derive(Clone, Debug, Variation)]
pub enum ExceptionIdentification {
    Number(i64),
    Reference(DefinedValue),
    Arbitrary(Box<Type>, Value),
}

#[derive(Clone, Debug, Variation)]
pub enum RangeValue {
    Min(bool),
    Max(bool),
    Value(Value, bool),
}

#[derive(Clone, Debug, Variation)]
pub enum Token {
    Literal(String),
    Field(Field),
    OptionalGroup(Vec<Token>),
}

#[derive(Clone, Debug, Variation)]
pub enum Parameter {
    Type(Type),
    Value(Value),
    ValueSet(ElementSetSpec),
    ObjectClass(DefinedObjectClass),
    Object(Object),
    ObjectSet((ElementSet, bool)),
}

#[derive(Clone, Debug, Variation)]
pub enum Presence {
    Absent,
    Optional,
    Present,
}

#[derive(Clone, Debug)]
pub struct ComponentConstraint {
    constraint: Option<Constraint>,
    presence: Option<Presence>,
}

impl ComponentConstraint {
    fn new(constraint: Option<Constraint>, presence: Option<Presence>) -> Self {
        Self {
            constraint,
            presence,
        }
    }
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use super::Asn1Parser;
    use super::Rule;

    #[test]
    fn basic_definition() {
        let input = include_str!("../tests/basic.asn1");

        Asn1Parser::parse(Rule::ModuleDefinition, input).unwrap_or_else(|e| panic!("{}", e));
    }

    #[test]
    fn pkcs12() {
        let input = include_str!("../tests/pkcs12.asn1");

        Asn1Parser::parse(Rule::ModuleDefinition, input).unwrap_or_else(|e| panic!("{}", e));
    }
}
