#[cfg(feature = "devtools")]
#[test]
fn derive_contracts() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/pass/*.rs");
    tests.compile_fail("tests/ui/fail/*.rs");
}

#[cfg(not(feature = "devtools"))]
#[test]
fn derive_is_noop_without_devtools() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/pass-production/*.rs");
}
