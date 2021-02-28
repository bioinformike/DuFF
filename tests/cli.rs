use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

// Not currently functional, so not spending time documenting right now.

// Search dir tests
#[test]
fn search_dir_dne() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("DuFF")?;

    cmd.arg("-d").arg("this/path/is/not/real").arg("-f").arg("/tmp/duff_out");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));

    Ok(())
}

#[test]
fn search_dir_no_perm() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("DuFF")?;

    cmd.arg("-d").arg("/").arg("-f").arg("/tmp/duff_out");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Permission denied"));

    Ok(())
}

#[test]
fn search_dir_not_dir() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("DuFF")?;

    cmd.arg("-d").arg(".").arg("-f").arg("/tmp/duff_out");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Permission denied"));

    Ok(())
}

