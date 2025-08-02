//! Integration tests for the CLI.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;
use std::fs;

#[test]
fn test_keygen_json() {
    let mut cmd = Command::cargo_bin("hesha").unwrap();
    cmd.arg("keygen")
        .assert()
        .success()
        .stdout(predicate::str::contains("private_key"))
        .stdout(predicate::str::contains("public_key"));
}

#[test]
fn test_keygen_base64() {
    let mut cmd = Command::cargo_bin("hesha").unwrap();
    cmd.args(&["keygen", "--format", "base64"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Private key:"))
        .stdout(predicate::str::contains("Public key:"));
}

#[test]
fn test_keygen_hex() {
    let mut cmd = Command::cargo_bin("hesha").unwrap();
    cmd.args(&["keygen", "--format", "hex"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Private key:"))
        .stdout(predicate::str::contains("Public key:"));
}

#[test]
fn test_inspect_jwt() {
    // Create a sample JWT (this would normally come from an issuer)
    let jwt = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.test.signature";
    
    let mut cmd = Command::cargo_bin("hesha").unwrap();
    cmd.args(&["inspect", jwt])
        .assert()
        .failure(); // Will fail because it's not a valid JWT
}

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("hesha").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hesha Protocol CLI"));
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("hesha").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("hesha"));
}