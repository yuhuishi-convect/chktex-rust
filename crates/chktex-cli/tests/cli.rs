use std::process::Command;

#[test]
fn version_flag_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .arg("--version")
        .output()
        .expect("run chktex");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("ChkTeX v"));
    assert!(output.stderr.is_empty());
}

#[test]
fn help_flag_matches_upstream_status_and_channel() {
    let output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .arg("--help")
        .output()
        .expect("run chktex");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage of ChkTeX"));
}
