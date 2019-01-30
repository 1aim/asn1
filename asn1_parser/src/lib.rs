#[macro_use]
extern crate pest_derive;

mod ast;
mod oid;

use std::{collections::HashMap, fs, mem, path::Path};

use derefable::Derefable;
use pest::Parser;
use unwrap_to::unwrap_to;

use self::ast::*;
use crate::oid::ObjectIdentifier;

pub type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug)]
pub struct Asn1 {
    module: Module,
    definitions: Vec<ModuleIdentifier>,
}

impl Asn1 {
    pub fn new<A: AsRef<Path>>(source: &str, dependency_folder: A) -> Result<Self> {
        let parsed = Asn1Parser::parse(Rule::ModuleDefinition, source)?;
        let input = parsed.flatten().peekable();

        let module = Ast::parse(input)?;
        let mut definitions = Vec::new();

        for entry in fs::read_dir(dependency_folder)? {
            let entry = entry?;

            if entry.file_type()?.is_dir() {
                continue;
            }

            let path = entry.path();

            if path.extension().map(|x| x != "asn1").unwrap_or(true) {
                continue;
            }

            let source = fs::read_to_string(path)?;
            let parsed = Asn1Parser::parse(Rule::ModuleDefinition, &source)?;
            let input = parsed.flatten().peekable();
            let header = Ast::parse_header(input)?;

            definitions.push(header);
        }

        Ok(Self {
            module,
            definitions,
        })
    }

    pub fn build(&mut self) {
        let mut type_registry = TypeRegistry::new();
        let mut value_registry = ValueRegistry::new();

        // for (module, symbols) in &self.module.imports {
        //     if module.has_identification() {
        //     }
        // }

        for assignment in mem::replace(&mut self.module.assignments, Vec::new()) {
            match assignment {
                Assignment::Type(name, r#type) => {
                    type_registry.insert(name, r#type);
                }
                Assignment::Value(name, r#type, value) => {
                    value_registry.insert(name, (r#type, value));
                }
                c => println!("{:?}", c),
            }
        }

        // println!("{:#?}", type_registry);

        value_registry.resolve_type_aliases(&type_registry);
        value_registry.resolve_object_identifiers();
        value_registry.resolve_defined_values();
        println!("{:#?}", value_registry);
    }
}

#[derive(Debug, Default, Derefable)]
struct TypeRegistry {
    #[deref(mutable)]
    map: HashMap<String, Type>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Default, Derefable)]
struct ValueRegistry {
    #[deref(mutable)]
    map: HashMap<String, (Type, Value)>,
}

impl ValueRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resolve_object_identifiers(&mut self) {
        let total_length = self.map.iter().filter(|(_, (_, v))| v.is_object_identifier()).count();

        let mut absolute_oids: HashMap<String, ObjectIdentifier> = self
            .map
            .clone()
            .into_iter()
            .filter(|(_, (_, v))| {
                v.is_object_identifier() && unwrap_to!(v => Value::ObjectIdentifier).is_absolute()
            })
            .map(|(k, (_, v))| {
                (
                    k,
                    match v {
                        Value::ObjectIdentifier(s) => s,
                        _ => unreachable!(),
                    },
                )
            })
            .collect();

        while total_length > absolute_oids.len() {
            for (name, object_identifier) in self
                .get_object_identifiers_mut()
                .filter(|(_, o)| o.is_relative())
            {
                object_identifier.replace(&absolute_oids);
                if object_identifier.is_absolute() {
                    absolute_oids.insert(name.clone(), object_identifier.clone());
                }
            }
        }
    }

    pub fn resolve_type_aliases(&mut self, registry: &TypeRegistry) {
        for t in self.map.iter_mut().map(|(_, (t, _))| t).filter(|t| t.is_referenced()) {
            let reference = unwrap_to!(t.raw_type => RawType::Referenced);

            if reference.is_internal() {
                let name = unwrap_to!(reference => ReferenceType::Internal);
                if let Some(original_type) = registry.get(name) {
                    // TODO: How do constraints work across type alias?
                    *t = original_type.clone();
                }
            } else {
                unimplemented!("External reference types not available yet");
            }
        }
    }

    pub fn resolve_defined_values(&mut self) {
        let frozen_map = self.map.clone();

        for value in self.map.iter_mut().map(|(_, (_, v))| v).filter(|v| v.is_defined()) {
            let def = match value {
                Value::Defined(DefinedValue::Simple(v)) => v,
                Value::Defined(DefinedValue::Parameterized(_, _)) => unimplemented!("Parameterized defined values are not currently supported"),
                _ => unreachable!(),
            };

            let original_value = match def {
                // TODO: Replace with some form of type checking.
                SimpleDefinedValue::Value(v) => frozen_map.get(v).map(|(_, v)| v).expect("Couldn't find defined value"),
                SimpleDefinedValue::Reference(_, _) => unimplemented!("External defines are not currently supported"),
            };

            *value = original_value.clone();
        }
    }

    fn get_object_identifiers_mut(
        &mut self,
    ) -> impl Iterator<Item = (&String, &mut ObjectIdentifier)> {
        self.map
            .iter_mut()
            .filter_map(|(k, (_, v))| v.as_object_identifier_mut().map(|v| (k, v)))
    }
}

#[derive(Parser)]
#[grammar = "asn1.pest"]
struct Asn1Parser;

#[cfg(test)]
mod tests {
    use pest::Parser;

    use super::Asn1Parser;
    use super::Rule;

    #[test]
    fn basic_definition() {
        let input = include_str!("../tests/basic.asn1");

        let rules = Asn1Parser::parse(Rule::ModuleDefinition, input)
            .unwrap_or_else(|e| panic!("{}", e))
            .flatten();

        println!("{:#?}", rules);
    }

    #[test]
    fn pkcs12() {
        let input = include_str!("../tests/pkcs12.asn1");

        let rules = Asn1Parser::parse(Rule::ModuleDefinition, input)
            .unwrap_or_else(|e| panic!("{}", e))
            .flatten();

        println!("{:#?}", rules);
    }
}
