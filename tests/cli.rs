use assert_cmd::Command;

use tempfile::{Builder, tempdir};

#[test]
fn rmon_runs_executable() {
    let temp_dir = tempdir().unwrap();
    let obj_path = temp_dir.into_path().join("hello.rmx");

    rmachine::build_exe("testdata/hello.s", &obj_path).unwrap();

    let mut cmd = Command::cargo_bin("rmon").unwrap();

    cmd.arg(&obj_path).assert().stdout("Hello World!\\n");
}

#[test]
fn rmon_debug_flag_shows_debug_prompt() {
    let mut cmd = Command::cargo_bin("rmon").unwrap();

    cmd.arg("--debug")
        .assert()
        .success()
        .stdout(predicates::str::contains(">"));
}

#[test]
fn rasm_run_builds_object_file_with_no_extension() {
    let temp = Builder::new().suffix(".s").tempfile().unwrap();
    std::fs::write(temp.path(), "li a0, 42").unwrap();

    let mut cmd = Command::cargo_bin("rasm").unwrap();
    cmd.arg(temp.path()).assert().success();

    let output = temp.path().with_extension("");
    assert!(output.exists(), "Assembled file should exist at {output:?}",);
}

#[test]
fn rdis_dissembles_object_file() {
    let temp = Builder::new().suffix(".s").tempfile().unwrap();
    std::fs::write(temp.path(), "li a0, 42").unwrap();

    let mut cmd = Command::cargo_bin("rasm").unwrap();
    cmd.arg(temp.path()).assert().success();

    let mut cmd = Command::cargo_bin("rdis").unwrap();
    cmd.arg(temp.path().with_extension(""))
        .assert()
        .success()
        .stdout(predicates::str::contains("addi   a0, zero, 0x2a"));
}
