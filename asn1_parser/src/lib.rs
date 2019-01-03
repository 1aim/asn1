#[macro_use]
extern crate pest_derive;

mod ast;

use pest::Parser;

use self::ast::{Ast, Module};

pub type Result<T> = std::result::Result<T, failure::Error>;

pub struct Asn1;

impl Asn1 {
    pub fn new(source: &str) -> Result<Module> {
        let parsed = Asn1Parser::parse(Rule::ModuleDefinition, source)?;
        let input = parsed.flatten().peekable();

        /*
        for pair in input.clone() {
            println!("RULE: {:?}, STR: {:?}", pair.as_rule(), pair.as_str());
        }
        */

        Ast::new(input).parse_module()
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
