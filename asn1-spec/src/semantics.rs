use std::{collections::HashMap, mem};

use failure::ensure;
use unwrap_to::unwrap_to;

use crate::{ast::*, registry::*, Result};

#[derive(Debug)]
pub struct SemanticChecker {
    imports: HashMap<ModuleReference, Vec<String>>,
    module: Module,
    types: TypeRegistry,
    values: ValueRegistry,
    // value_sets: ValueRegistry,
    // object_sets: ValueRegistry,
    // objects: ValueRegistry,
    // classes: ValueRegistry,
}

impl SemanticChecker {
    pub fn new(module: Module) -> Self {
        let imports = HashMap::with_capacity(module.imports.len());
        let values = ValueRegistry::new();
        let types = TypeRegistry::new();

        Self {
            imports,
            module,
            types,
            values,
        }
    }

    pub fn build(&mut self) -> Result<()>  {
        debug!("Building Module {:?}", self.module.identifier);
        self.resolve_imports()?;
        self.resolve_assignments()?;
        self.resolve_type_aliases();
        self.values.resolve_object_identifiers();
        self.resolve_defined_values();
        Ok(())
    }

    pub fn resolve_assignments(&mut self) -> Result<()> {
        for assignment in mem::replace(&mut self.module.assignments, Vec::new()) {
            ensure!(!self.contains_assignment(&assignment.name), "{:?} was already defined.", assignment.name);

            debug!("ASSIGNMENT: {:#?}", assignment);

            match assignment.kind {
                AssignmentType::Type(ty) => {
                    self.types.insert(assignment.name, ty);
                }
                AssignmentType::Value(ty, value) => {
                    self.values.insert(assignment.name, (ty, value));
                }
                AssignmentType::ValueSet(ty, elements) => {
                    ensure!(elements.set.len() != 0, "{:?} is empty, empty element sets are not allowed.", assignment.name);

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

        Ok(())
    }

    fn contains_assignment(&self, name: &String) -> bool {
        self.types.contains_key(name) && self.values.contains_key(name)
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
            let mut def = match value {
                AssignedIdentifier::Defined(def) => def,
                _ => unreachable!(),
            };

            *value = AssignedIdentifier::ObjectIdentifier(get_value(&mut def).into_object_identifier());
        }
    }

    pub fn resolve_imports(&mut self) -> Result<()> {
        for (reference, items) in mem::replace(&mut self.module.imports, Vec::new()) {
            ensure!(!self.imports.contains_key(&reference), "{:?} was already imported.", reference);

            self.imports.insert(reference, items);
        }

        Ok(())
    }
}


