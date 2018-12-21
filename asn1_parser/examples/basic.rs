use asn1_parser::Asn1;

fn main() {
    let parser = Asn1::new(include_str!("../tests/pkcs12.asn1")).unwrap();

    parser.print_ast();
}
