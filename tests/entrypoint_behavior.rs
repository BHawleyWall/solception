#![allow(unused_imports, unused_variables, dead_code)]

use assert_cmd::prelude::*;
use predicates::prelude::*;
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

fn test_command() -> assert_cmd::Command {
    assert_cmd::Command::cargo_bin("solc").expect(
        "The binary should pass compilation and be available at the expected command label.  \
             This could fail if the code in the `src` directory could not compile, or if this \
             test code is not correct.  Check the Git blame for both paths to determine the cause \
             of the failure.",
    )
}

#[test]
fn binary_entrypoint_is_invokable() {
    test_command()
        .assert()
        .append_context(
            "entrypoint",
            "The binary should be available at the expected command label.",
        )
        .failure()
        .code(2);
}

#[test]
fn invoking_without_any_argument_prints_short_help() {
    test_command()
        .assert()
        .append_context(
            "help",
            "Invoking with no arguments should print a useful help message.",
        )
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "error: the following required arguments were not provided:",
        ));
}
