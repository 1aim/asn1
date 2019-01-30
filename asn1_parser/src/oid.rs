use std::collections::HashMap;

use derefable::Derefable;
use unwrap_to::unwrap_to;
use variation::Variation;

#[derive(Clone, Debug, Derefable, Default, Hash, PartialEq, PartialOrd, Eq)]
pub struct ObjectIdentifier(#[deref(mutable)] Vec<ObjIdComponent>);

impl ObjectIdentifier {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_components(components: Vec<ObjIdComponent>) -> Self {
        Self(components)
    }

    pub fn is_absolute(&self) -> bool {
        !self.is_relative()
    }

    pub fn is_relative(&self) -> bool {
        for component in &self.0 {
            if component.is_reserved_name_form() {
                continue;
            } else if component.is_name() {
                return true;
            }
        }

        false
    }

    pub fn name_forms(&self) -> impl Iterator<Item = &String> {
        self.0
            .iter()
            .filter(|c| c.is_name())
            .map(|c| &*unwrap_to!(c => ObjIdComponent::Name))
    }

    pub fn replace(&mut self, map: &HashMap<String, ObjectIdentifier>) {
        for (name, id) in map {
            while let Ok(index) = self.0.binary_search(&ObjIdComponent::Name(name.clone())) {
                self.0.splice(index..(index+1), id.0.iter().cloned());
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialOrd, Ord, Variation)]
pub enum ObjIdComponent {
    Name(String),
    Number(i64),
    NameAndNumber(String, i64),
}

impl ObjIdComponent {
    pub fn is_reserved_name_form(&self) -> bool {
        match self {
            ObjIdComponent::NameAndNumber(name, _)| ObjIdComponent::Name(name) => match &**name {
                "ITU-T" | "itu-t" | "ccitt" | "ISO" | "iso" | "Joint-ISO-ITU-T"
                | "joint-iso-itu-t" | "joint-iso-ccitt" => true,
                _ => false,
            },
            _ => false,
        }
    }
}

impl PartialEq for ObjIdComponent {
    fn eq(&self, rhs: &Self) -> bool {
        if self.is_reserved_name_form() {
            let self_name = match self {
                ObjIdComponent::Name(name) | ObjIdComponent::NameAndNumber(name, _) => name,
                _ => unreachable!(),
            };

            if rhs.is_reserved_name_form() {
                let rhs_name = match self {
                    ObjIdComponent::Name(name) | ObjIdComponent::NameAndNumber(name, _) => name,
                    _ => unreachable!(),
                };

                self_name.to_lowercase() == rhs_name.to_lowercase()
            } else {
                false
            }
        } else {
            match (self, rhs) {
                (ObjIdComponent::Name(lhs), ObjIdComponent::Name(rhs)) => lhs == rhs,
                (ObjIdComponent::Number(lhs), ObjIdComponent::Number(rhs)) => lhs == rhs,
                (ObjIdComponent::NameAndNumber(lhs_s, lhs_n), ObjIdComponent::NameAndNumber(rhs_s, rhs_n)) => {
                    lhs_s == rhs_s && lhs_n == rhs_n
                }
                _ => false,
            }
        }
    }
}


