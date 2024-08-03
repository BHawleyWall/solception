#![allow(unused_imports, unused_variables, dead_code)]

use assert_cmd::prelude::*;
use predicates::prelude::*;
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

const MARINADE_STAKING_PROGRAM_ID: &str = "MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD";

fn test_command() -> assert_cmd::Command {
    assert_cmd::Command::cargo_bin("solc").expect(
        "The binary should pass compilation and be available at the expected command label.  This \
         could fail if the code in the `src` directory could not compile, or if this test code is \
         not correct.  Check the Git blame for both paths to determine the cause of the failure.",
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

#[test]
fn invoking_with_help_flag_prints_help() {
    test_command()
        .arg("--help")
        .assert()
        .append_context(
            "help",
            "Invoking with the `--help` flag should print the full help message.",
        )
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Arguments:"))
        .stdout(predicate::str::contains("Options:"));
}

#[test]
fn invoking_with_version_flag_prints_version() {
    test_command()
        .arg("--version")
        .assert()
        .append_context(
            "version",
            "Invoking with the `--version` flag should print the version of this crate.",
        )
        .success()
        .stdout(predicate::str::contains("solception"));
}

#[test]
fn invoking_with_invalid_argument_prints_error() {
    test_command()
        .arg("--invalid-argument")
        .assert()
        .append_context(
            "invalid-argument",
            "Invoking with an invalid argument should print an error message.",
        )
        .failure()
        .code(2)
        .stderr(predicate::str::contains("error: unexpected argument"))
        .stderr(predicate::str::contains("Usage:"));
}

#[test]
fn invoking_with_valid_program_id_succeeds() {
    test_command()
        .arg(MARINADE_STAKING_PROGRAM_ID)
        .assert()
        .append_context(
            "valid-argument",
            "Invoking with a valid Solana program ID should succeed.",
        )
        .success();
}
