use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "asn1.pest"]
pub struct Asn1Parser;

