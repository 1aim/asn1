mod imports;
mod structs;

use std::collections::HashSet;
use std::fmt::Write as _;
use std::io::Write;

use self::imports::*;
use self::structs::*;
use crate::{parser::*, semantics::SemanticChecker, Result};

pub trait Backend: Default {
    fn generate_type(&mut self, ty: &Type) -> Result<String>;
    fn generate_value(&mut self, value: &Value) -> Result<String>;
    fn generate_value_assignment(&mut self, name: &str, ty: &Type, value: &Value)
        -> Result<String>;
    fn generate_sequence(&mut self, name: &str, fields: &[ComponentType]) -> Result<String>;
    fn generate_builtin(&mut self, builtin: &BuiltinType) -> Result<String>;
    fn write_prelude<W: Write>(&self, writer: &mut W) -> Result<()>;
    fn write_footer<W: Write>(&self, writer: &mut W) -> Result<()>;
}

#[derive(Default)]
pub struct Rust {
    consts: Vec<String>,
    structs: Vec<Struct>,
    prelude: HashSet<Import>,
    indentation: usize,
}

impl Rust {
    fn generate_const(&mut self, public: bool, name: &str, ty: &str, value: &str) {
        use heck::ShoutySnakeCase;
        let visibility = if public { "pub" } else { "" };
        self.consts.push(format!(
            "{} const {}: {} = {};",
            visibility,
            name.to_shouty_snake_case(),
            ty,
            value
        ));
    }
}

impl Backend for Rust {
    /// As Rust doesn't allow you to have anonymous structs `generate_sequence` returns the name
    /// of the struct and stores the definition seperately.
    fn generate_sequence(&mut self, name: &str, fields: &[ComponentType]) -> Result<String> {
        let mut generated_struct = Struct::new(name);

        for field in fields {
            // Unwrap currently needed as i haven't created the simplified AST without
            // `ComponentsOf` yet.
            let (ty, optional, default) = field.as_type().unwrap();
            let field = FieldBuilder::new(&**ty.name.as_ref().unwrap(), &*self.generate_type(&ty)?)
                .optional(*optional)
                .default_value(default.clone().and_then(|v| self.generate_value(&v).ok()))
                .build();

            generated_struct.add_field(field);
        }

        self.structs.push(generated_struct);

        Ok(String::from(name))
    }

    fn generate_type(&mut self, ty: &Type) -> Result<String> {
        match ty.raw_type {
            RawType::Builtin(ref builtin) => self.generate_builtin(builtin),
            RawType::Referenced(ref reference) if reference.is_internal() => {
                Ok(reference.item.clone())
            }
            ref raw => {
                warn!("UNKNOWN TYPE: {:?}", raw);
                Ok(String::from("UNIMPLEMENTED"))
            }
        }
    }

    fn generate_builtin(&mut self, builtin: &BuiltinType) -> Result<String> {
        let output = match builtin {
            BuiltinType::Boolean => String::from("bool"),
            BuiltinType::ObjectIdentifier => {
                self.prelude.insert(Import::new(
                    Visibility::Private,
                    ["asn1", "core", "ObjectIdentifier"]
                        .into_iter()
                        .map(ToString::to_string)
                        .collect(),
                ));
                String::from("ObjId")
            }
            BuiltinType::OctetString => String::from("Vec<u8>"),
            BuiltinType::Integer(_) => String::from("isize"),
            ref builtin => {
                warn!("UNKNOWN BUILTIN TYPE: {:?}", builtin);
                String::from("UNIMPLEMENTED")
            }
        };

        Ok(output)
    }

    fn write_prelude<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write(
            itertools::join(self.prelude.iter().map(ToString::to_string), "\n").as_bytes(),
        )?;

        writer.write(itertools::join(self.consts.iter(), "\n").as_bytes())?;
        Ok(())
    }

    fn write_footer<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write(
            itertools::join(self.structs.iter().map(ToString::to_string), "\n").as_bytes(),
        )?;

        Ok(())
    }

    fn generate_value(&mut self, value: &Value) -> Result<String> {
        match value {
            _ => Ok(String::from("UNIMPLEMENTED")),
        }
    }

    fn generate_value_assignment(
        &mut self,
        name: &str,
        ty: &Type,
        value: &Value,
    ) -> Result<String> {
        let generated_value = match value {
            _ => String::from("UNIMPLEMENTED"),
        };

        let type_output = self.generate_type(ty)?;

        self.generate_const(false, name, &type_output, &generated_value);
        Ok(String::from(name))
    }
}

pub struct CodeGenerator<W: Write, B: Backend> {
    backend: B,
    table: SemanticChecker,
    writer: W,
}

impl<W: Write, B: Backend> CodeGenerator<W, B> {
    pub fn new(table: SemanticChecker, writer: W) -> Self {
        Self {
            backend: B::default(),
            table,
            writer,
        }
    }

    pub fn generate(mut self) -> Result<W> {
        for (name, (ty, value)) in self.table.values.iter() {
            self.backend.generate_value_assignment(name, ty, value)?;
        }

        for (name, ty) in self.table.types.iter() {
            match &ty.raw_type {
                RawType::Builtin(BuiltinType::Sequence(components)) => {
                    self.backend.generate_sequence(&name, &components)?;
                }
                _ => {}
            }
        }

        self.backend.write_prelude(&mut self.writer)?;
        writeln!(self.writer)?;
        self.backend.write_footer(&mut self.writer)?;

        Ok(self.writer)
    }
}
