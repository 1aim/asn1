use clap::{clap_app, crate_description, crate_version};

fn main() {
    let matches = clap_app!(casn1 =>
        (version: crate_version!())
        (author: "Aaron P. <theaaronepower@gmail.com> + Contributors")
        (about: crate_description!())
        (@arg dependencies: -d --dependencies
            +takes_value
            "Specify the dependency directory. Will automatically parse the headers of \
            the files, and import them if necessary. Default: \"./definitions\"")
        (@arg input: ... "ASN.1 files to parse.")
    )
    .get_matches();

    let directory = matches.value_of("dependencies").unwrap_or("./definitions");

    let mut module = asn1_spec::Asn1::new(matches.value_of("input").unwrap())
        .dependencies(directory)
        .build()
        .unwrap_or_else(|e| panic!("{}", e));

    println!("{}", module);
}
