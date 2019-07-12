mod constant;
mod imports;
mod structs;

use std::{collections::HashSet, io::Write, mem};

use failure::Fallible as Result;

use self::{constant::Constant, imports::*, structs::*};
use crate::{
    parser::*,
    registry::{GlobalSymbolTable, SymbolTable},
    semantics::SemanticChecker,
};

pub trait Backend: Default {
    fn generate_type(&mut self, ty: &Type) -> Result<String>;
    fn generate_value(&mut self, value: &Value) -> Result<String>;
    fn generate_value_assignment(&mut self, name: String, ty: Type, value: Value) -> Result<()>;
    fn generate_sequence(&mut self, name: &str, fields: &[ComponentType]) -> Result<String>;
    fn generate_sequence_of(&mut self, name: &str, ty: &Type) -> Result<String>;
    fn generate_builtin(&mut self, builtin: &BuiltinType) -> Result<String>;
    fn write_prelude<W: Write>(&mut self, writer: &mut W) -> Result<()>;
    fn write_footer<W: Write>(&self, writer: &mut W) -> Result<()>;
}

#[derive(Default)]
pub struct Rust {
    consts: HashSet<Constant>,
    structs: Vec<Struct>,
    prelude: HashSet<Import>,
}

impl Backend for Rust {
    /// As Rust doesn't allow you to have anonymous structs,
    /// `generate_sequence` returns the name of the struct and
    /// stores the definition seperately.
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

    fn generate_sequence_of(&mut self, name: &str, ty: &Type) -> Result<String> {
        let inner_type = match ty.raw_type {
            RawType::Referenced(ref reference) => &reference.item,
            _ => unimplemented!(),
        };

        Ok(format!("pub type {} = Vec<{}>;", name, inner_type))
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
                    ["asn1", "types", "ObjectIdentifier"]
                        .into_iter()
                        .map(ToString::to_string)
                        .collect(),
                ));

                String::from("ObjectIdentifier")
            }

            BuiltinType::OctetString => {
                self.prelude.insert(Import::new(
                    Visibility::Private,
                    ["asn1", "types", "OctetString"]
                        .into_iter()
                        .map(ToString::to_string)
                        .collect(),
                ));

                String::from("ObjectIdentifier")
            }
            BuiltinType::Integer(_) => String::from("isize"),
            ref builtin => {
                warn!("UNKNOWN BUILTIN TYPE: {:?}", builtin);
                String::from("UNIMPLEMENTED")
            }
        };

        Ok(output)
    }

    fn write_prelude<W: Write>(&mut self, writer: &mut W) -> Result<()> {
        let prelude = mem::replace(&mut self.prelude, HashSet::new());
        writer.write(itertools::join(prelude.iter().map(ToString::to_string), "\n").as_bytes())?;

        /*
        let consts = mem::replace(&mut self.consts, HashSet::new());
        writer.write(
            itertools::join(
                consts.into_iter().filter_map(|c| c.generate(self).ok()),
                "\n",
            )
            .as_bytes(),
        )?;
        */
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

    fn generate_value_assignment(&mut self, name: String, ty: Type, value: Value) -> Result<()> {
        self.consts
            .insert(Constant::new(Visibility::Public, name, ty, value));
        Ok(())
    }
}

pub struct CodeGenerator<W: Write, B: Backend> {
    backend: B,
    table: GlobalSymbolTable,
    writer: W,
}

impl<W: Write, B: Backend> CodeGenerator<W, B> {
    pub fn new(table: GlobalSymbolTable, writer: W) -> Self {
        Self {
            backend: B::default(),
            table,
            writer,
        }
    }

    pub fn generate(mut self) -> Result<W> {
        let table = self.table;

        for (name, (ty, value)) in table.values.clone().into_iter() {
            self.backend.generate_value_assignment(name, ty, value)?;
        }

        for (name, ty) in table.types.iter() {
            match &ty.raw_type {
                RawType::Builtin(BuiltinType::Sequence(components)) => {
                    self.backend.generate_sequence(&name, &components)?;
                }
                RawType::Builtin(BuiltinType::SequenceOf(ty)) => {
                    write!(
                        self.writer,
                        "{}\n",
                        self.backend.generate_sequence_of(&name, &ty)?
                    )?;
                }
                _ => {}
            }
        }

        self.backend.write_prelude(&mut self.writer)?;
        write!(self.writer, "\n\n")?;
        self.backend.write_footer(&mut self.writer)?;

        Ok(self.writer)
    }
}
