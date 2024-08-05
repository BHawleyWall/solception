use predicates::prelude::*;

const MARINADE_STAKING_PROGRAM_ID: &str = "MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD";
const RANDOM_DEVNET_PROGRAM_WITH_FEW_DEPLOYMENTS: &str =
    "HP3G4ptUEd6C4urhjM5sV57RgoarNCPNrWu7vWrCkYWg";

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

/*
 * This test currently takes ~7 hours to complete against the devnet public
 * RPC node.
 *
 * Against a private full-history node, this test take ~13 minutes to
 * complete when no other tests are running.  Performance as part of the full
 * integ suite depends on the node's load at the time of the test run, but
 * averages around 30 minutes.
 *
 * This duration is because the blocking RpcClient respects the HTTP 429
 * headers from the node, resulting in optimal performance within the
 * constraints of the node's rate limiting due to the headers communicating
 * the actual delay before the next token refresh on the three applicable
 * throttle limits effecting the client.
 */
#[test]
fn invoking_with_verbose_flag_once_prints_warn_level_logs() {
    test_command()
        .arg("--verbose")
        .arg(MARINADE_STAKING_PROGRAM_ID)
        .assert()
        .append_context(
            "verbosity",
            "Invoking with the `--verbose` flag once should print WARN log events.",
        )
        .success()
        .stderr(predicate::str::contains("WARN"))
        .stderr(predicate::str::contains(
            "solception::adapters::gateways::solana::interface",
        ))
        .stderr(predicate::str::contains(
            "The number of transactions for program_id:",
        ))
        .stderr(predicate::str::contains(
            " exceeds 1000.  This may take a long time to retrieve all transaction details, \
             depending on the chosen cluster RPC node\'s rate limits!",
        ));
}

#[test]
fn invoking_with_verbose_flag_twice_prints_info_level_logs() {
    test_command()
        .arg("-vv")
        .arg(RANDOM_DEVNET_PROGRAM_WITH_FEW_DEPLOYMENTS)
        .assert()
        .append_context(
            "verbosity",
            "Invoking with the `--verbose` flag twice should print INFO log events.",
        )
        .success()
        .stdout(predicate::str::contains("INFO"))
        .stdout(predicate::str::contains(
            "This may take some time, depending on the number of transactions and the chosen \
             cluster RPC node's rate limits.",
        ));
}

#[test]
fn invoking_with_verbose_flag_thrice_prints_debug_level_logs() {
    test_command()
        .arg("-vvv")
        .arg(RANDOM_DEVNET_PROGRAM_WITH_FEW_DEPLOYMENTS)
        .assert()
        .append_context(
            "verbosity",
            "Invoking with the `--verbose` flag thrice should print DEBUG log events.",
        )
        .success()
        .stdout(predicate::str::contains("INFO"))
        .stdout(predicate::str::contains(
            "This may take some time, depending on the number of transactions and the chosen \
             cluster RPC node's rate limits.",
        ))
        .stdout(predicate::str::contains("DEBUG"))
        .stdout(predicate::str::contains("reqwest::connect"))
        .stdout(predicate::str::contains("starting new connection:"))
        .stdout(predicate::str::contains("hyper::client::connect::dns"))
        .stdout(predicate::str::contains("resolving host="))
        .stdout(predicate::str::contains("rustls::client::hs"))
        .stdout(predicate::str::contains("No cached session for DnsName"));
}

#[test]
fn invoking_with_verbose_flag_four_times_prints_trace_level_logs() {
    test_command()
        .arg("-vvvv")
        .arg(RANDOM_DEVNET_PROGRAM_WITH_FEW_DEPLOYMENTS)
        .assert()
        .append_context(
            "verbosity",
            "Invoking with the `--verbose` flag four times should print TRACE log events.",
        )
        .success()
        .stdout(predicate::str::contains("INFO"))
        .stdout(predicate::str::contains(
            "This may take some time, depending on the number of transactions and the chosen \
             cluster RPC node's rate limits.",
        ))
        .stdout(predicate::str::contains("TRACE"))
        .stdout(predicate::str::contains(
            "solception::adapters::gateways::telemetry::interface",
        ))
        .stdout(predicate::str::contains(
            "Initializing tracing with debug level: TRACE",
        ))
        .stdout(predicate::str::contains("hyper::client::pool"))
        .stdout(predicate::str::contains(
            "checkout waiting for idle connection:",
        ))
        .stdout(predicate::str::contains("DEBUG"))
        .stdout(predicate::str::contains("reqwest::connect"))
        .stdout(predicate::str::contains("starting new connection:"))
        .stdout(predicate::str::contains("hyper::client::connect::http"))
        .stdout(predicate::str::contains("Http::connect; scheme="));
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
        .arg(RANDOM_DEVNET_PROGRAM_WITH_FEW_DEPLOYMENTS)
        .assert()
        .append_context(
            "valid-argument",
            "Invoking with a valid Solana program ID should succeed.",
        )
        .success()
        .stdout(predicate::str::contains("2024-08-03T17:11:30+00:00"));
}

/*
 * This test currently takes ~7 hours to complete against the devnet public
 * RPC node.
 *
 * Against a private full-history node, this test take ~13 minutes to
 * complete when no other tests are running.  Performance as part of the full
 * integ suite depends on the node's load at the time of the test run, but
 * averages around 30 minutes.
 *
 * This duration is because the blocking RpcClient respects the HTTP 429
 * headers from the node, resulting in optimal performance within the
 * constraints of the node's rate limiting due to the headers communicating
 * the actual delay before the next token refresh on the three applicable
 * throttle limits effecting the client.
 */
#[test]
fn invoking_with_an_extreme_history_correctly_paginates_the_full_available_history() {
    test_command()
        .arg(MARINADE_STAKING_PROGRAM_ID)
        .assert()
        .append_context(
            "valid-argument",
            "Invoking with a valid Solana program ID should succeed.",
        )
        .success()
        .stdout(predicate::str::contains("2022-04-24T11:02:50+00:00"));
}
