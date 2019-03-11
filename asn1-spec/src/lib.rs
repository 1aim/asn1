#[macro_use]
extern crate pest_derive;

mod ast;
mod registry;
mod semantics;

use std::{fs, path::PathBuf};

use crate::{ast::*, semantics::*};

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

        let fixed_tree = SemanticChecker::new(ast, self.dependencies);

        Ok(format!("{:?} parsed successfully", self.path))
    }
}
