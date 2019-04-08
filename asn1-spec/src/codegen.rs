use std::io::Write;
use std::collections::HashSet;
use std::fmt::Write as _;

use crate::{ast::*, Result, semantics::SemanticChecker};

pub trait Backend: Default {
    fn generate_type(&mut self, ty: &Type) -> Result<String>;
    fn generate_sequence(&mut self, name: &str, fields: &[ComponentType]) -> Result<String>;
    fn generate_builtin(&mut self, builtin: &BuiltinType) -> Result<String>;
    fn write_prelude<W: Write>(&self, writer: &mut W) -> Result<()>;
    fn write_footer<W: Write>(&self, writer: &mut W) -> Result<()>;
}

#[derive(Default)]
pub struct Rust {
    structs: Vec<String>,
    prelude: HashSet<String>,
}

impl Backend for Rust {
    /// As Rust doesn't allow you to have anonymous structs `generate_sequence` returns the name
    /// of the struct and stores the definition seperately.
    fn generate_sequence(&mut self, name: &str, fields: &[ComponentType]) -> Result<String> {
        let mut output = String::new();

        writeln!(output, "struct {name} {{", name=name)?;

        for field in fields {
            // Unwrap currently needed as i haven't created the simplified AST without
            // `ComponentsOf` yet.
            let (ty, optional, default) = field.as_type().unwrap();

            let ty_output = self.generate_type(&ty)?;
            let type_output = if *optional {
                format!("Option<{}>", ty_output)
            } else {
                ty_output
            };

            write!(output, "{}", " ".repeat(4))?;

            writeln!(output, "{}: {},", ty.name.as_ref().unwrap(), type_output)?;
        }

        writeln!(output, "}}")?;

        self.structs.push(output);

        Ok(String::from(name))
    }

    fn generate_type(&mut self, ty: &Type) -> Result<String> {
        match ty.raw_type {
            RawType::Builtin(ref builtin) => self.generate_builtin(builtin),
            RawType::Referenced(ref reference) if reference.is_internal() => Ok(reference.item.clone()),
            _ => Ok(String::from("UNIMPLEMENTED"))
        }
    }

    fn generate_builtin(&mut self, builtin: &BuiltinType) -> Result<String> {
        let output = match builtin {
            BuiltinType::Boolean => String::from("bool"),
            BuiltinType::ObjectIdentifier => {
                self.prelude.insert(String::from("use asn1::core::ObjId;\n"));
                String::from("ObjId")
            },
            BuiltinType::OctetString => String::from("Vec<u8>"),
            BuiltinType::Integer(_) => String::from("isize"),
            _ => String::from("UNIMPLEMENTED"),
        };

        Ok(output)
    }

    fn write_prelude<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write(self.prelude.iter().cloned().collect::<Vec<_>>().join("\n").as_bytes())?;
        Ok(())
    }

    fn write_footer<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write(self.structs.join("\n").as_bytes())?;

        Ok(())
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
        for (name, ty) in self.table.types.iter() {
            match &ty.raw_type {
                RawType::Builtin(BuiltinType::Sequence(components)) => {
                    self.backend.generate_sequence(&name, &components)?;
                }
                _ => {},
            }
        }

        self.backend.write_prelude(&mut self.writer)?;
        writeln!(self.writer)?;
        self.backend.write_footer(&mut self.writer)?;

        Ok(self.writer)
    }
}
