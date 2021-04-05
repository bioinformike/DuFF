use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use std::path::PathBuf;
// Not currently functional, so not spending time documenting right now.

// Search dir tests
#[test]
fn search_dir_dne() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("duff").unwrap();

    cmd.arg("-d").arg("/this/path/is/not/real").arg("-o").arg("/tmp/duff_out");
    cmd.env("exit", "1")
        .assert()
        .failure()
        .stderr(predicate::str::starts_with(
            "There was an error with the specified directory"));

    Ok(())
}

/*#[test]
fn search_dir_no_perm() {
    let mut cmd = Command::cargo_bin("duff").unwrap();

    cmd.arg("-d").arg("/").arg("-o").arg("/tmp/duff_out");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Permission denied"));


}*/

#[test]
fn search_dir_not_dir() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("duff")?;

    cmd.arg("-d").arg(".").arg("-o").arg("/tmp/duff_out");
    cmd.env("exit", "1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Permission denied"));

    Ok(())
}


#[test]
fn ext_filtering() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("duff")?;

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/data/ext_test");

    let mut out = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    out.push("tests/outputs/");

    cmd.arg("-d")
        .arg(d)
        .arg("-o")
        .arg(out)
        .arg("-e")
        .arg(".a");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test_ext.a"));

    Ok(())
}
