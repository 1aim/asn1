use asn1_parser::Asn1;

fn main() {
    let module =
        Asn1::new(include_str!("../tests/pkcs12.asn1"), "./definitions").unwrap_or_else(|e| panic!("{}", e));

    println!("{:#?}", module);
}
