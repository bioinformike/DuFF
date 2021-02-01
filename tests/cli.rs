use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;


// Size tests
#[test]
fn path_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("DuFF")?;

    cmd.arg("-d").arg("this/path/is/not/real");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("No such file or directory"));

    Ok(())

}

