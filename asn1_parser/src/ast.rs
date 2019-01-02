use std::collections::HashMap;
use std::iter::Peekable;

use pest::iterators::{FlatPairs, Pair};

use super::{Result, Rule};
use crate::module::Module;

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
            let imports = self.parse_imports()?;

            (exports, imports, ())
        } else {
            (Exports::All, HashMap::new(), ())
        };

        Ok(Module {
            identifier: identifier,
            tag,
            extension: None,
            exports,
            imports,
        })
    }

    pub fn parse_module_identifier(&mut self) -> Result<ModuleIdentifier> {
        self.take(Rule::ModuleIdentifier);

        let name = self.take(Rule::ReferenceIdentifier).as_str();
        let mut module_identifier = ModuleIdentifier::new(name.to_owned());

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
                        let name = self.take(Rule::Identifier).as_str().to_owned();
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

    pub fn parse_imports(&mut self) -> Result<HashMap<ModuleReference, Vec<Symbol>>> {
        let mut imports = HashMap::new();

        if self.look(Rule::Imports).is_some() {
            while self.look(Rule::SymbolsFromModule).is_some() {
                let symbol_list = self.parse_symbol_list();
                self.take(Rule::GlobalModuleReference);
                let module_name = self.take(Rule::ReferenceIdentifier).as_str().to_owned();

                let identification = if self.look(Rule::AssignedIdentifier).is_some() {
                    let identification = match self.rule_peek().unwrap() {
                        Rule::ObjectIdentifierValue => AssignedIdentifier::ObjectIdentifier(
                            self.parse_object_identifier_value()?,
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

        Ok(imports)
    }

    pub fn parse_object_identifier_value(&mut self) -> Result<Vec<ObjIdComponent>> {
        self.take(Rule::ObjectIdentifierValue);
        let mut components = Vec::new();

        while self.look(Rule::ObjIdComponents).is_some() {
            use self::DefinitiveObjIdComponent::*;

            let component = match self.rule_peek().unwrap() {
                Rule::Identifier => ObjIdComponent::Definitive(Name(
                    self.take(Rule::Identifier).as_str().to_owned(),
                )),
                Rule::NumberForm => ObjIdComponent::Definitive(Number(
                    self.take(Rule::NumberForm).as_str().parse()?,
                )),
                Rule::DefinedValue => ObjIdComponent::DefinedValue(self.parse_defined_value()),
                Rule::NameAndNumberForm => {
                    self.take(Rule::NameAndNumberForm);
                    let name = self.take(Rule::Identifier).as_str().to_owned();
                    let number = self.take(Rule::NumberForm).as_str().parse()?;

                    ObjIdComponent::Definitive(NameAndNumber(name, number))
                }
                _ => unreachable!(),
            };

            components.push(component)
        }

        Ok(components)
    }

    fn parse_defined_value(&mut self) -> DefinedValue {
        use self::SimpleDefinedValue::*;
        self.take(Rule::DefinedValue);

        match self.rule_peek().unwrap() {
            Rule::ExternalTypeReference => {
                self.take(Rule::ExternalTypeReference);
                let parent = self.take(Rule::ReferenceIdentifier).as_str().to_owned();
                let child = self.take(Rule::ReferenceIdentifier).as_str().to_owned();

                DefinedValue::Simple(Reference(parent, child))
            }

            Rule::Identifier => {
                DefinedValue::Simple(Value(self.take(Rule::Identifier).as_str().to_owned()))
            }
            Rule::ParameterizedReference => unimplemented!(),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Module {
    pub identifier: ModuleIdentifier,
    pub tag: Tag,
    pub extension: Option<()>,
    pub exports: Exports,
    pub imports: HashMap<ModuleReference, Vec<Symbol>>,
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
