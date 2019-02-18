extern crate cargo_readme;

use std::fs::File;
use std::io::Read;
use std::path::Path;

#[test]
fn test_readme_identical() {
    let mut source = File::open("src/lib.rs").expect("failed to open source file");
    let mut template = File::open("README.tpl").expect("failed to open template file");
    let mut expected = cargo_readme::generate_readme(
        Path::new("."),
        &mut source,
        Some(&mut template),
        true,
        true,
        true,
    )
    .expect("failed to generate readme");
    expected.push('\n');

    let mut readme = String::new();
    let mut file = File::open("README.md").expect("failed to open README.md");
    file.read_to_string(&mut readme)
        .expect("failed to read README.md");
    for (l, r) in readme.lines().zip(expected.lines()) {
        assert_eq!(l, r);
    }
    assert_eq!(readme, expected);
}
