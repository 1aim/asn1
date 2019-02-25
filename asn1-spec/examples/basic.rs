use asn1_parser::Asn1;

fn main() {
    let mut module = Asn1::new(
        include_str!("../definitions/UsefulDefinitions.asn1"),
        "./definitions",
    )
    .unwrap_or_else(|e| panic!("{}", e));

    module.build();
}
