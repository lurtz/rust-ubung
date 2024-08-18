use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn denon_control_prints_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("denon-control")?;

    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage"));

    Ok(())
}

#[test]
fn denon_control_fails_to_connect() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("denon-control")?;

    cmd.arg("--address").arg("localhost");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("using receiver: localhost"))
        .stderr(predicate::str::contains("Connection refused"));

    Ok(())
}
