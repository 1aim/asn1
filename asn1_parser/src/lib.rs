#[macro_use]
extern crate pest_derive;

mod ast;

use std::{fs, path::Path};

use pest::Parser;

use self::ast::{Ast, Module};

pub type Result<T> = std::result::Result<T, failure::Error>;

pub struct Asn1;

impl Asn1 {
    pub fn new<A: AsRef<Path>>(source: &str, dependency_folder: A) -> Result<Module> {
        let parsed = Asn1Parser::parse(Rule::ModuleDefinition, source)?;
        let input = parsed.flatten().peekable();

        let module = Ast::parse(input)?;

        for entry in fs::read_dir(dependency_folder)? {
            let entry = entry?;

            if entry.file_type()?.is_dir() {
                continue;
            }

            let path = entry.path();

            if path.extension().map(|x| x != "asn1").unwrap_or(true) {
                continue;
            }

            let source = fs::read_to_string(path)?;
            let parsed = Asn1Parser::parse(Rule::ModuleDefinition, &source)?;
            let input = parsed.flatten().peekable();
            let header = Ast::parse_header(input)?;

        }

        Ok(module)
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

        let rules = Asn1Parser::parse(Rule::ModuleDefinition, input)
            .unwrap_or_else(|e| panic!("{}", e))
            .flatten();

        println!("{:#?}", rules);
    }

    #[test]
    fn pkcs12() {
        let input = include_str!("../tests/pkcs12.asn1");

        let rules = Asn1Parser::parse(Rule::ModuleDefinition, input)
            .unwrap_or_else(|e| panic!("{}", e))
            .flatten();

        println!("{:#?}", rules);
    }
}
