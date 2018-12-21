#[macro_use]
extern crate pest_derive;

use std::iter::Peekable;

use pest::{iterators::FlatPairs, Parser};

pub type Result<T> = std::result::Result<T, failure::Error>;

pub struct Asn1<'a> {
    input: Peekable<FlatPairs<'a, Rule>>
}

impl<'a> Asn1<'a> {

    pub fn new(source: &'a str) -> Result<Self> {
        let parsed = Asn1Parser::parse(Rule::ModuleDefinition, source)?;
        let input = parsed.flatten().peekable();

        Ok(Self { input })
    }

    pub fn print_ast(&self) {
        for pair in self.input.clone() {
            println!("RULE: {:?}, STR: {:?}", pair.as_rule(), pair.as_str());
        }
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

        let rules = Asn1Parser::parse(Rule::ModuleDefinition, input).unwrap_or_else(|e| panic!("{}", e)).flatten();

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
