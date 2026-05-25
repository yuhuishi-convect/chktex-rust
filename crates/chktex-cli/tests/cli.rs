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
fn help_flag_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .arg("--help")
        .output()
        .expect("run chktex");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage: chktex"));
    assert!(output.stderr.is_empty());
}
