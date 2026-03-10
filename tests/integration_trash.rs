#![allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn zut(home: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("zut").unwrap();
    cmd.env("HOME", home);
    cmd.env("ZUT_PERSONALITY", "0");
    cmd
}

#[test]
fn trash_single_file() {
    let home = TempDir::new().unwrap();
    let file = home.path().join("test.txt");
    fs::write(&file, "hello").unwrap();

    zut(home.path())
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("→"));

    assert!(!file.exists());
    assert!(home.path().join(".zut/metadata.json").exists());
}

#[test]
fn trash_multiple_files() {
    let home = TempDir::new().unwrap();
    let f1 = home.path().join("a.txt");
    let f2 = home.path().join("b.txt");
    fs::write(&f1, "a").unwrap();
    fs::write(&f2, "b").unwrap();

    zut(home.path())
        .args([f1.as_os_str(), f2.as_os_str()])
        .assert()
        .success();

    assert!(!f1.exists());
    assert!(!f2.exists());
}

#[test]
fn trash_directory() {
    let home = TempDir::new().unwrap();
    let dir = home.path().join("mydir");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("file.txt"), "content").unwrap();

    zut(home.path()).arg(&dir).assert().success();

    assert!(!dir.exists());
}

#[test]
fn trash_missing_file_error() {
    let home = TempDir::new().unwrap();

    zut(home.path())
        .arg("/nonexistent/file.txt")
        .assert()
        .success()
        .stderr(predicate::str::contains("No such file"));
}

#[test]
fn trash_missing_file_force_silent() {
    let home = TempDir::new().unwrap();

    zut(home.path())
        .args(["-f", "/nonexistent/file.txt"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn trash_protected_path_root() {
    let home = TempDir::new().unwrap();

    zut(home.path())
        .arg("/")
        .assert()
        .success()
        .stderr(predicate::str::contains("Non. Juste non."));
}

#[test]
fn undo_last() {
    let home = TempDir::new().unwrap();
    let file = home.path().join("restore-me.txt");
    fs::write(&file, "data").unwrap();

    zut(home.path()).arg(&file).assert().success();
    assert!(!file.exists());

    zut(home.path())
        .arg("--undo")
        .assert()
        .success()
        .stdout(predicate::str::contains("←"));

    assert!(file.exists());
    assert_eq!(fs::read_to_string(&file).unwrap(), "data");
}

#[test]
fn undo_by_name() {
    let home = TempDir::new().unwrap();
    let f1 = home.path().join("first.txt");
    let f2 = home.path().join("second.txt");
    fs::write(&f1, "1").unwrap();
    fs::write(&f2, "2").unwrap();

    zut(home.path()).arg(&f1).assert().success();
    zut(home.path()).arg(&f2).assert().success();

    zut(home.path())
        .args(["--undo", "first.txt"])
        .assert()
        .success();

    assert!(f1.exists());
    assert!(!f2.exists());
}

#[test]
fn undo_empty_trash() {
    let home = TempDir::new().unwrap();

    zut(home.path())
        .arg("--undo")
        .assert()
        .failure()
        .stderr(predicate::str::contains("corbeille est vide"));
}

#[test]
fn purge_all() {
    let home = TempDir::new().unwrap();
    let file = home.path().join("gone.txt");
    fs::write(&file, "bye").unwrap();

    zut(home.path()).arg(&file).assert().success();

    zut(home.path())
        .args(["--purge", "-f"])
        .assert()
        .success()
        .stdout(predicate::str::contains("supprimés"));

    let trash = home.path().join(".zut/trash");
    assert!(fs::read_dir(&trash).unwrap().next().is_none());
}

#[test]
fn purge_older_nothing_old() {
    let home = TempDir::new().unwrap();
    let file = home.path().join("recent.txt");
    fs::write(&file, "new").unwrap();

    zut(home.path()).arg(&file).assert().success();

    zut(home.path())
        .args(["--purge", "--older", "1h", "-f"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rien à purger"));
}

#[test]
fn purge_invalid_duration() {
    let home = TempDir::new().unwrap();

    zut(home.path())
        .args(["--purge", "--older", "xyz", "-f"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalide"));
}
