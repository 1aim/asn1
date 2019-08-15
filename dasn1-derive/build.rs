use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

use walkdir::WalkDir;

const TESTS_DIRECTORY: &str = "tests/";
const TESTS_FILE: &str = "tests.rs";

/// This build script concats all files in the tests/ directory except
/// `TESTS_FILE` into TESTS_FILE. This is a workaround [compiletest-rs not being
/// functional on macOS](https://github.com/laumann/compiletest-rs/issues/179).
/// Until that is fixed this is enough to have a directory of full tests.
///
/// TODO: Generate modules from directory names instead of concating into a
/// single module as currently you're required to use wildcard imports to ensure
/// you have your dependency and that it doesn't conflict with another test's
/// imports.
fn main() -> Result<(), Box<std::error::Error>> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(TESTS_FILE);
    let mut tests_file = File::create(&dest_path).unwrap();

    for entry in WalkDir::new(TESTS_DIRECTORY)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() || entry.path().display().to_string().contains(TESTS_FILE) {
            continue;
        }

        let contents = match fs::read_to_string(entry.path()) {
            Ok(contents) => contents,
            _ => continue,
        };

        writeln!(tests_file, "{}", contents)?;
    }

    Ok(())
}
