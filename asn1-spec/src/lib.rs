#[macro_use] extern crate log;

mod ast;
mod codegen;
mod registry;
mod semantics;

use std::{fs, path::PathBuf};

use self::{ast::*, semantics::*, codegen::*};

pub type Result<T> = std::result::Result<T, failure::Error>;

pub struct Asn1 {
    path: PathBuf,
    dependencies: Option<PathBuf>,
}

impl Asn1 {
    pub fn new<I: Into<PathBuf>>(path: I) -> Self {
        Self {
            path: path.into(),
            dependencies: None,
        }
    }

    pub fn dependencies<I: Into<PathBuf>>(mut self, path: I) -> Self {
        self.dependencies = Some(path.into());
        self
    }

    pub fn build(self) -> Result<String> {
        let source = fs::read_to_string(&self.path)?;
        let ast = Ast::parse(&source)?;

        let mut fixed_tree = SemanticChecker::new(ast);
        fixed_tree.build()?;

        let mut output = Vec::new();

        let mut codegen = CodeGenerator::<Vec<u8>, Rust>::new(fixed_tree, output);
        let output = codegen.generate()?;

        Ok(String::from_utf8(output).unwrap())
    }
}
