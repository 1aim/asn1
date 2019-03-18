use std::{collections::BTreeMap, fs, iter::FromIterator, path::PathBuf};

use derefable::Derefable;
use unwrap_to::unwrap_to;

use crate::{ast::*, oid::ObjectIdentifier, Result};

#[derive(Debug, Default, Derefable)]
pub struct TypeRegistry {
    #[deref(mutable)]
    map: BTreeMap<String, Type>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}

impl FromIterator<(String, Type)> for TypeRegistry {
    fn from_iter<I: IntoIterator<Item = (String, Type)>>(iter: I) -> Self {
        Self {
            map: BTreeMap::from_iter(iter),
        }
    }
}

#[derive(Debug, Default, Derefable)]
pub struct ValueRegistry {
    #[deref(mutable)]
    map: BTreeMap<String, (Type, Value)>,
}

impl ValueRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resolve_object_identifiers(&mut self) {
        debug!("Resolving OIDs to full path.");
        let total_length = self
            .map
            .iter()
            .filter(|(_, (_, v))| v.is_object_identifier())
            .count();

        trace!("Total number of OIDs: {}", total_length);

        let mut absolute_oids: BTreeMap<String, ObjectIdentifier> = self
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

        trace!("Number of initial Absolute OIDs: {}", absolute_oids.len());

        while total_length > absolute_oids.len() {
            for (name, object_identifier) in self
                .get_object_identifiers_mut()
                .filter(|(_, o)| o.is_relative())
            {
                trace!("Attempting to canonicalise {}", object_identifier);
                object_identifier.replace(&absolute_oids);
                if object_identifier.is_absolute() {
                    trace!("{} is now absolute.", object_identifier);
                    absolute_oids.insert(name.clone(), object_identifier.clone());
                    trace!("New number of Absolute OIDs: {}, remaining {}", absolute_oids.len(), total_length - absolute_oids.len());
                }
            }
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

impl FromIterator<(String, (Type, Value))> for ValueRegistry {
    fn from_iter<I: IntoIterator<Item = (String, (Type, Value))>>(iter: I) -> Self {
        Self {
            map: BTreeMap::from_iter(iter),
        }
    }
}

pub struct ModuleRegistry {
    available_modules: BTreeMap<ModuleIdentifier, PathBuf>
}

impl ModuleRegistry {
    pub fn new(dependencies: Option<PathBuf>) -> Result<Self> {
        let mut available_modules = BTreeMap::new();

        if let Some(ref dependencies) = dependencies {
            for entry in fs::read_dir(&dependencies)? {
                let entry = entry?;

                if entry.file_type()?.is_dir() {
                    continue;
                }

                let path = entry.path();

                if path.extension().map(|x| x != "asn1").unwrap_or(true) {
                    continue;
                }

                let header = Ast::parse_header(&fs::read_to_string(&path)?)?;

                available_modules.insert(header, path.to_owned());
            }
        }

        Ok(Self {
            available_modules
        })
    }
}

#[derive(Debug, Default, Derefable)]
pub struct ValueSetRegistry {
    #[deref(mutable)]
    map: BTreeMap<String, (Type, ElementSetSpec)>,
}

impl ValueSetRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}
