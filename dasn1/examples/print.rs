fn main() {
    let x = vec![0x0A, 0x3B, 0x5F, 0x29, 0x1C, 0xD0];

    println!("{:?}", hex::encode(&asn1::der::to_vec(&x).unwrap()));
}
