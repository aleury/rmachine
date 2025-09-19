use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn rmachine_runs_executable() {
    let temp_dir = tempdir().unwrap();
    let obj_path = temp_dir.into_path().join("hello");

    rmachine::build_exe("testdata/hello.s", &obj_path).unwrap();

    let mut cmd = Command::cargo_bin("rmon").unwrap();

    cmd.arg(&obj_path).assert().stdout("Hello World!\\n");
}

#[test]
fn rmachine_debug_flag_shows_debug_prompt() {
    let mut cmd = Command::cargo_bin("rmon").unwrap();

    cmd.arg("--debug")
        .assert()
        .success()
        .stdout(predicates::str::contains(">"));
}
