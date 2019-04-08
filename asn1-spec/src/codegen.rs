use std::io::Write;
use std::fmt::Write as _;

use crate::{ast::*, Result, semantics::SemanticChecker};

pub trait Backend {
    fn new() -> Self;
    fn generate_type(&self, ty: &Type) -> Result<String>;
    fn generate_sequence(&self, name: &str, fields: &[ComponentType]) -> Result<String>;
}

pub struct Rust;

impl Backend for Rust {
    fn new() -> Self {
        Self
    }

    fn generate_sequence(&self, name: &str, fields: &[ComponentType]) -> Result<String> {
        let mut output = String::new();

        writeln!(output, "struct {name} {{", name=name)?;

        for field in fields {
            // Unwrap currently needed as i haven't created the simplified AST without
            // `ComponentsOf` yet.
            let (ty, optional, default) = field.as_type().unwrap();
            let ty_output = self.generate_type(&ty)?;
            let output_type = if *optional {
                format!("Option<{}>", ty_output)
            } else {
                ty_output
            };

            writeln!(output, "{field_name} {ty},", field_name=ty.name.as_ref().unwrap(), ty=self.generate_type(ty)?)?;
        }

        writeln!(output, "}}")?;

        Ok(output)
    }

    fn generate_type(&self, ty: &Type) -> Result<String> {
        Ok(String::from("UNIMPLEMENTED"))
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
            backend: B::new(),
            table,
            writer,
        }
    }

    pub fn generate(mut self) -> Result<W> {
        for (name, ty) in self.table.types.iter() {
            let output = match &ty.raw_type {
                RawType::Builtin(BuiltinType::Sequence(components)) => self.backend.generate_sequence(&name, &components),
                _ => Ok(String::new()),
            }?;

            self.writer.write(output.as_bytes())?;
        }

        Ok(self.writer)
    }
}
