use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[test]
#[ignore = "requires CHKTEX_ORACLE pointing at an upstream C chktex binary"]
fn version_output_matches_oracle_shape() {
    let oracle = oracle_path();

    let c_output = run(&oracle, ["--version"]);
    let rust_output = run(env!("CARGO_BIN_EXE_chktex"), ["--version"]);

    assert_eq!(c_output.status.success(), rust_output.status.success());
    assert_contains(&rust_output.stdout, b"ChkTeX v");
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR"]
fn fixture_output_matches_oracle_when_available() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let fixture = upstream_dir.join("Test.tex");
    let rc = upstream_dir.join("chktexrc");

    assert!(
        fixture.is_file(),
        "missing upstream fixture: {}",
        fixture.display()
    );
    assert!(rc.is_file(), "missing upstream rc: {}", rc.display());

    let args = [
        OsStr::new("-mall"),
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-v5"),
        OsStr::new("-q"),
        fixture.as_os_str(),
    ];

    let c_output = run_os(&oracle, args);
    let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), args);

    assert_outputs_equal(&c_output, &rust_output);
}

fn oracle_path() -> PathBuf {
    std::env::var_os("CHKTEX_ORACLE")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .expect("CHKTEX_ORACLE must point at an upstream C chktex binary")
}

fn upstream_dir() -> PathBuf {
    std::env::var_os("CHKTEX_UPSTREAM_DIR")
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
        .expect("CHKTEX_UPSTREAM_DIR must point at an upstream chktex source/build directory")
}

fn run<I, S>(program: impl AsRef<OsStr>, args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_os(program, args)
}

fn run_os<I, S>(program: impl AsRef<OsStr>, args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(program)
        .args(args)
        .output()
        .expect("run command")
}

fn assert_outputs_equal(c_output: &Output, rust_output: &Output) {
    assert_eq!(c_output.status.code(), rust_output.status.code());

    if c_output.stdout != rust_output.stdout {
        write_debug_file("oracle.stdout", &c_output.stdout);
        write_debug_file("rust.stdout", &rust_output.stdout);
        panic!("stdout differs; wrote oracle.stdout and rust.stdout");
    }

    if c_output.stderr != rust_output.stderr {
        write_debug_file("oracle.stderr", &c_output.stderr);
        write_debug_file("rust.stderr", &rust_output.stderr);
        panic!("stderr differs; wrote oracle.stderr and rust.stderr");
    }
}

fn assert_contains(haystack: &[u8], needle: &[u8]) {
    assert!(
        haystack
            .windows(needle.len())
            .any(|window| window == needle),
        "expected {:?} to contain {:?}",
        String::from_utf8_lossy(haystack),
        String::from_utf8_lossy(needle)
    );
}

fn write_debug_file(name: &str, contents: &[u8]) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create debug output directory");
    }
    fs::write(path, contents).expect("write debug output");
}
