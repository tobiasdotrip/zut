#![allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn zut(home: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("zut").unwrap();
    cmd.env("HOME", home);
    cmd.env("ZUT_PERSONALITY", "0");
    cmd
}

#[test]
fn help_flag() {
    let home = TempDir::new().unwrap();
    zut(home.path())
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("corbeille"));
}

#[test]
fn version_flag() {
    let home = TempDir::new().unwrap();
    zut(home.path())
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("zut"));
}

#[test]
fn list_empty() {
    let home = TempDir::new().unwrap();
    zut(home.path()).arg("--list").assert().success();
}

#[test]
fn list_with_content() {
    let home = TempDir::new().unwrap();
    let file = home.path().join("listed.txt");
    std::fs::write(&file, "data").unwrap();

    zut(home.path()).arg(&file).assert().success();

    zut(home.path())
        .arg("--list")
        .assert()
        .success()
        .stdout(predicate::str::contains("listed.txt"))
        .stdout(predicate::str::contains("fichier"));
}

#[test]
fn stats_empty() {
    let home = TempDir::new().unwrap();
    zut(home.path())
        .arg("--stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Fichiers dans la corbeille : 0"));
}

#[test]
fn mutually_exclusive_flags() {
    let home = TempDir::new().unwrap();
    zut(home.path())
        .args(["--list", "--purge"])
        .assert()
        .failure();
}

#[test]
fn rm_compat_flags() {
    let home = TempDir::new().unwrap();
    let dir = home.path().join("rmdir");
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("f.txt"), "x").unwrap();

    zut(home.path())
        .args(["-rf", dir.to_str().unwrap()])
        .assert()
        .success();

    assert!(!dir.exists());
}

#[test]
fn no_args_shows_help_hint() {
    let home = TempDir::new().unwrap();
    zut(home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("--help"));
}

#[test]
fn exit_code_on_invalid_args() {
    let home = TempDir::new().unwrap();
    zut(home.path())
        .arg("--invalid-flag")
        .assert()
        .failure()
        .code(2);
}
