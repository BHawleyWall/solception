#![allow(unused_imports, unused_variables, dead_code)]

use assert_cmd::prelude::*;
use predicates::prelude::*;
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

#[test]
fn binary_entrypoint_is_invokable() {
    assert_cmd::Command::cargo_bin("solc")
        .expect(
            "The binary should pass compilation and be available at the expected command label.  \
             This could fail if the code in the `src` directory could not compile, or if this \
             test code is not correct.  Check the Git blame for both paths to determine the cause \
             of the failure.",
        )
        .assert()
        .success()
        .stdout(predicate::eq("Hello, world!\n"));
}
