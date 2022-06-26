#[test]
fn test_pkgbuild_version() {
    version_sync::assert_contains_regex!("dist/pkgbuild/PKGBUILD", "^pkgver={version}$");
}
