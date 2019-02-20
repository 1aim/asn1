#[macro_use]
extern crate pest_derive;

mod ast;
mod oid;
mod registry;

use std::{
    collections::HashMap,
    fs,
    mem,
    path::{Path, PathBuf},
};

use pest::Parser;
use unwrap_to::unwrap_to;

use crate::{ast::*, registry::*};

pub type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug)]
pub struct Asn1 {
    definitions: HashMap<ModuleIdentifier, PathBuf>,
    dependencies: HashMap<String, Self>,
    dependency_directory: PathBuf,
    module: Module,
    types: TypeRegistry,
    values: ValueRegistry,
    // value_sets: ValueRegistry,
    // object_sets: ValueRegistry,
    // objects: ValueRegistry,
    // classes: ValueRegistry,
}

impl Asn1 {
    pub fn new<A: AsRef<Path>>(path: A, dependency_directory: A) -> Result<Self> {
        let source = fs::read_to_string(path)?;
        let parsed = Asn1Parser::parse(Rule::ModuleDefinition, &source)?;
        let input = parsed.flatten().peekable();

        let module = Ast::parse(input)?;
        let mut definitions = HashMap::new();
        let dependencies = HashMap::new();
        let values = ValueRegistry::new();
        let types = TypeRegistry::new();

        for entry in fs::read_dir(dependency_directory.as_ref())? {
            let entry = entry?;

            if entry.file_type()?.is_dir() {
                continue;
            }

            let path = entry.path();

            if path.extension().map(|x| x != "asn1").unwrap_or(true) {
                continue;
            }

            let source = fs::read_to_string(&path)?;
            let parsed = Asn1Parser::parse(Rule::ModuleHeaderOnly, &source)?;
            let input = parsed.flatten().peekable();
            let header = Ast::parse_header(input)?;

            definitions.insert(header, path.to_owned());
        }

        let dependency_directory = dependency_directory.as_ref().to_owned();

        Ok(Self {
            definitions,
            dependencies,
            dependency_directory,
            module,
            types,
            values,
        })
    }

    pub fn build(&mut self) {
        for assignment in mem::replace(&mut self.module.assignments, Vec::new()) {
            match assignment.kind {
                AssignmentType::Type(ty) => {
                    self.types.insert(assignment.name, ty);
                }
                AssignmentType::Value(ty, value) => {
                    self.values.insert(assignment.name, (ty, value));
                }
                AssignmentType::ValueSet(ty, value) => {
                    //self.value_sets.insert(assignment.name, (ty, value));
                }
                AssignmentType::Object(class, object) => {
                    //self.objects.insert(assignment.name, (class, object));
                }
                AssignmentType::ObjectClass(class) => {
                    //self.classes.insert(assignment.name, class);
                }
                AssignmentType::ObjectSet(class, set) => {
                    //self.object_sets.insert(assignment.name, class);
                }
            }
        }

        let iter = self
            .module
            .imports
            .iter()
            .filter(|(k, _)| !k.identification_uses_defined_value())
            .filter_map(|(k, v)| k.into_identifier().map(|k| (k, v)));

        for (module, imported_symbols) in iter {
            let module: Asn1 = match self.definitions.get(&module) {
                Some(path) => {
                    let mut module = Asn1::new(path, &self.dependency_directory).unwrap();
                    module.build();
                    module
                }
                None => panic!("Unknown import {:?}", module),
            };

            let (available_types, available_values) = match module.module.exports {
                Exports::All => (module.types, module.values),
                Exports::Symbols(symbols) => (
                    module
                        .types
                        .clone()
                        .into_iter()
                        .filter(|(k, _)| symbols.contains(k))
                        .collect(),
                    module
                        .values
                        .clone()
                        .into_iter()
                        .filter(|(k, _)| symbols.contains(k))
                        .collect(),
                ),
            };

            for symbol in imported_symbols {
                let symbol = symbol.clone();
                if let Some(value) = available_types.get(&symbol) {
                    self.types.insert(symbol, value.clone());
                } else if let Some(value) = available_values.get(&symbol) {
                    self.values.insert(symbol, value.clone());
                }
            }
        }

        self.resolve_type_aliases();
        self.values.resolve_object_identifiers();
        self.resolve_defined_values();
    }

    pub fn resolve_type_aliases(&mut self) {
        for t in self
            .values
            .iter_mut()
            .map(|(_, (t, _))| t)
            .filter(|t| t.is_referenced())
        {
            let reference = unwrap_to!(t.raw_type => RawType::Referenced);

            if reference.is_internal() {
                let name = unwrap_to!(reference => ReferenceType::Internal);
                if let Some(original_type) = self.types.get(name) {
                    // TODO: How do constraints work across type alias?
                    *t = original_type.clone();
                }
            } else {
                unimplemented!("External reference types not available yet");
            }
        }
    }

    pub fn resolve_defined_values(&mut self) {
        let frozen_map = self.values.clone();
        let get_value = |defined_value: &mut DefinedValue| {
            let simple = match defined_value {
                DefinedValue::Simple(v) => v,
                DefinedValue::Parameterized(_, _) => {
                    unimplemented!("Parameterized defined values are not currently supported")
                }
            };

            let original_value = match simple {
                // TODO: Replace with some form of type checking.
                SimpleDefinedValue::Value(ref_value) => {
                    let value = frozen_map.get(&*ref_value).map(|(_, v)| v);

                    match value {
                        Some(v) => v,
                        None => panic!("Couldn't find {:?} value", ref_value),
                    }
                }
                SimpleDefinedValue::Reference(_, _) => {
                    unimplemented!("External defines are not currently supported")
                }
            };

            original_value.clone()
        };

        for value in self
            .values
            .iter_mut()
            .map(|(_, (_, v))| v)
            .filter(|v| v.is_defined())
        {
            let def = match value {
                Value::Defined(def) => def,
                _ => unreachable!(),
            };

            *value = get_value(def);
        }

        let defined_values_imports = self
            .module
            .imports
            .iter_mut()
            .map(|(k, _)| k)
            .filter_map(ModuleReference::as_identification_mut)
            .filter(|a| a.is_defined());

        for value in defined_values_imports {
            let def = match value {
                AssignedIdentifier::Defined(def) => def,
                _ => unreachable!(),
            };

            *value = AssignedIdentifier::ObjectIdentifier(get_value(def).into_object_identifier());
        }
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

        Asn1Parser::parse(Rule::ModuleDefinition, input)
            .unwrap_or_else(|e| panic!("{}", e));

    }

    #[test]
    fn pkcs12() {
        let input = include_str!("../tests/pkcs12.asn1");

        Asn1Parser::parse(Rule::ModuleDefinition, input)
            .unwrap_or_else(|e| panic!("{}", e));
    }
}
