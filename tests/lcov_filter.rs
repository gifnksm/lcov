use std::path::PathBuf;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_lcov-filter")
}

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn run_filter(args: &[&str]) -> String {
    let output = Command::new(bin())
        .args(args)
        .output()
        .expect("failed to run lcov-filter");
    assert!(
        output.status.success(),
        "lcov-filter failed.\nSTDOUT:\n{}\nSTDERR:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn remap_path_prefix_basic() {
    let report = fixture("lcov/tests/fixtures/report.run.info");
    let out = run_filter(&[
        "--remap-path-prefix",
        "/home/nksm=/mnt/nksm",
        report.to_str().unwrap(),
    ]);

    assert!(out.contains("SF:/mnt/nksm/rhq/github.com/gifnksm/lcov/tests/fixtures/src/div.c"));
    assert!(out.contains("SF:/mnt/nksm/rhq/github.com/gifnksm/lcov/tests/fixtures/src/fizzbuzz.c"));
    assert!(!out.contains("SF:/home/nksm/"));
}

#[test]
fn remap_path_regex_numeric_groups() {
    let report = fixture("lcov/tests/fixtures/report.run.info");
    let out = run_filter(&[
        "--remap-path-regex",
        "^/home/([^/]+)/(.+)=/mnt/$1/src/$2",
        report.to_str().unwrap(),
    ]);

    assert!(out.contains("SF:/mnt/nksm/src/rhq/github.com/gifnksm/lcov/tests/fixtures/src/div.c"));
    assert!(out.contains("SF:/mnt/nksm/src/rhq/github.com/gifnksm/lcov/tests/fixtures/src/fizzbuzz.c"));
    assert!(!out.contains("SF:/home/nksm/"));
}

#[test]
fn remap_path_regex_python_style_groups() {
    let report = fixture("lcov/tests/fixtures/report.run.info");
    let out = run_filter(&[
        "--remap-path-regex",
        "^/home/(?P<user>[^/]+)/(?P<rest>.+)=/mnt/\\g<user>/src/\\g<rest>",
        report.to_str().unwrap(),
    ]);

    assert!(out.contains("SF:/mnt/nksm/src/rhq/github.com/gifnksm/lcov/tests/fixtures/src/div.c"));
    assert!(out.contains("SF:/mnt/nksm/src/rhq/github.com/gifnksm/lcov/tests/fixtures/src/fizzbuzz.c"));
    assert!(!out.contains("SF:/home/nksm/"));
}

