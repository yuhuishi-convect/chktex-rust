use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

/// Integration test comparing Rust chktex output against the upstream C binary.
///
/// Set `CHKTEX_ORACLE` and `CHKTEX_UPSTREAM_DIR`, or run `tools/setup-oracle.sh`
/// to clone/build upstream and write `target/oracle.env`.
const DEFAULT_ORACLE: &str = "/tmp/chktex-upstream/chktex/chktex/chktex";
const DEFAULT_UPSTREAM_DIR: &str = "/tmp/chktex-upstream/chktex/chktex";

#[test]
#[ignore = "requires CHKTEX_ORACLE pointing at an upstream C chktex binary"]
fn action_outputs_match_oracle() {
    let oracle = oracle_path();

    for args in [["--version"], ["--help"], ["--license"]] {
        let c_output = run(&oracle, args);
        let rust_output = run(env!("CARGO_BIN_EXE_chktex"), args);
        assert_outputs_equal(&c_output, &rust_output);
    }
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

    let args: &[&OsStr] = &[
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

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; tracks upstream tests/run-tests.sh inclusion behavior"]
fn upstream_inclusion_fixture_matches_expected() {
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let expected = fs::read(upstream_dir.join("tests/main.expected")).expect("read main.expected");

    let args: &[&OsStr] = &[
        OsStr::new("-mall"),
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-v5"),
        OsStr::new("-q"),
        OsStr::new("tests/main.tex"),
    ];

    let rust_output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .current_dir(&upstream_dir)
        .args(args)
        .output()
        .expect("run rust inclusion fixture");
    assert!(
        rust_output.status.success(),
        "rust chktex failed: {}",
        String::from_utf8_lossy(&rust_output.stderr)
    );
    assert_eq!(rust_output.stderr, b"");
    assert_eq!(normalize_upstream_test_paths(&rust_output.stdout), expected);
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; tracks upstream tests/run-tests.sh config lookup behavior"]
fn upstream_config_lookup_fixture_matches_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    ensure_upstream_config_fixtures(&upstream_dir);
    let fixture_input = b"%\n";
    let sub_config = upstream_dir.join("tests/sub");

    let c_output = run_with_stdin(
        &oracle,
        |command| {
            command
                .args(["-mall", "-v0", "-q"])
                .env("XDG_CONFIG_HOME", &sub_config)
                .env_remove("HOME");
        },
        fixture_input,
    );

    let rust_output = run_with_stdin(
        env!("CARGO_BIN_EXE_chktex"),
        |command| {
            command
                .args(["-mall", "-v0", "-q"])
                .env("XDG_CONFIG_HOME", &sub_config)
                .env_remove("HOME");
        },
        fixture_input,
    );

    assert_outputs_equal(&c_output, &rust_output);

    let tests_dir = upstream_dir.join("tests");
    let config_cases = [
        (
            "HOME/.config/chktexrc",
            vec![("HOME", tests_dir.join("sub1"))],
            "loaded chktex/tests/sub1/.config/chktexrc stdin",
        ),
        (
            "HOME/.chktexrc",
            vec![("HOME", tests_dir.join("sub2"))],
            "loaded chktex/tests/sub2/.chktexrc stdin",
        ),
        (
            "LOGDIR/.chktexrc",
            vec![("LOGDIR", tests_dir.join("sub2"))],
            "loaded chktex/tests/sub2/.chktexrc stdin",
        ),
        (
            "CHKTEXRC directory",
            vec![("CHKTEXRC", tests_dir.join("sub2"))],
            "loaded chktex/tests/sub2/.chktexrc stdin",
        ),
    ];

    for (name, envs, expected) in config_cases {
        let output = run_rust_config_probe(&envs, None, fixture_input);
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(expected),
            "{name} lookup failed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = run_rust_config_probe(&[], Some(&tests_dir.join("sub2")), fixture_input);
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("loaded chktex/tests/sub2/.chktexrc stdin"),
        "CWD .chktexrc lookup failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; tracks upstream tests/run-tests.sh command-line rc behavior"]
fn upstream_command_line_rc_setting_fixture_matches_expected() {
    let output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .args(["-d", "4", "-STabSize=7"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("run rust command-line rc fixture");

    let combined = [output.stdout, output.stderr].concat();
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&combined).contains("TabSize:\n\t7"),
        "TabSize override not reflected in debug output: {}",
        String::from_utf8_lossy(&combined)
    );
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks focused debug output parity"]
fn debug_flag_output_matches_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!("chktex-rust-debug-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create debug fixture dir");
    let fixture = dir.join("input.tex");
    fs::write(&fixture, b"Here(warn)\n").expect("write debug fixture");

    let args: &[&OsStr] = &[
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-q"),
        OsStr::new("-v0"),
        OsStr::new("-d8"),
        fixture.as_os_str(),
    ];

    let c_output = run_os(&oracle, args);
    let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), args);
    assert_eq!(c_output.status.code(), rust_output.status.code());
    assert_eq!(c_output.stdout, rust_output.stdout);
    assert_eq!(
        normalize_paths(&normalize_oracle_stderr(&c_output.stderr), &dir, &oracle),
        normalize_paths(
            &normalize_oracle_stderr(&rust_output.stderr),
            &dir,
            Path::new(env!("CARGO_BIN_EXE_chktex"))
        )
    );
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks -d1 warning table parity"]
fn debug_warning_table_matches_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir =
        std::env::temp_dir().join(format!("chktex-rust-debug-warnings-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create debug warning table fixture dir");
    let fixture = dir.join("input.tex");
    fs::write(&fixture, b"Here(warn)\n").expect("write debug warning table fixture");

    for extra_args in [
        vec![OsStr::new("-d1")],
        vec![
            OsStr::new("-d1"),
            OsStr::new("-n36"),
            OsStr::new("-m14"),
            OsStr::new("-e22"),
            OsStr::new("-w19"),
        ],
    ] {
        let mut args: Vec<&OsStr> = vec![
            OsStr::new("-r"),
            OsStr::new("-g0"),
            OsStr::new("-l"),
            rc.as_os_str(),
            OsStr::new("-q"),
            OsStr::new("-v0"),
        ];
        args.extend(extra_args);
        args.push(fixture.as_os_str());

        let c_output = run_os(&oracle, &args);
        let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), &args);
        assert_outputs_equal(&c_output, &rust_output);
    }
}

#[test]
#[ignore = "requires CHKTEX_ORACLE; checks focused -d2 resource summary parity"]
fn debug_resource_summary_matches_oracle() {
    let oracle = oracle_path();
    let dir =
        std::env::temp_dir().join(format!("chktex-rust-debug-summary-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create debug summary fixture dir");
    let fixture = dir.join("input.tex");
    let rc = dir.join("minimal.chktexrc");
    fs::write(&fixture, b"Here(warn)\n").expect("write debug summary fixture");
    fs::write(
        &rc,
        b"TabSize = 7\nOutFormat { \"%f:%l:%c:%n:%m!n\" }\nUserWarn { warn }\n",
    )
    .expect("write minimal rc");

    let args: &[&OsStr] = &[
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-q"),
        OsStr::new("-v0"),
        OsStr::new("-d2"),
        fixture.as_os_str(),
    ];

    let c_output = run_os(&oracle, args);
    let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), args);
    assert_eq!(c_output.status.code(), rust_output.status.code());
    assert_eq!(c_output.stdout, rust_output.stdout);
    assert_eq!(
        normalize_paths(&normalize_oracle_stderr(&c_output.stderr), &dir, &oracle),
        normalize_paths(
            &normalize_oracle_stderr(&rust_output.stderr),
            &dir,
            Path::new(env!("CARGO_BIN_EXE_chktex"))
        )
    );
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks default -d2 resource summary parity"]
fn debug_default_resource_summary_matches_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!(
        "chktex-rust-debug-default-summary-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create default debug summary fixture dir");
    let fixture = dir.join("input.tex");
    fs::write(&fixture, b"Here(warn)\n").expect("write default debug summary fixture");

    let args: &[&OsStr] = &[
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-q"),
        OsStr::new("-v0"),
        OsStr::new("-d2"),
        fixture.as_os_str(),
    ];

    let c_output = run_os(&oracle, args);
    let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), args);
    assert_outputs_equal(&c_output, &rust_output);
}

#[test]
#[ignore = "requires CHKTEX_ORACLE; checks focused -d4 resource list dump parity"]
fn debug_resource_list_dump_matches_oracle() {
    let oracle = oracle_path();
    let dir = std::env::temp_dir().join(format!("chktex-rust-debug-list-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create debug list fixture dir");
    let fixture = dir.join("input.tex");
    let rc = dir.join("minimal.chktexrc");
    fs::write(&fixture, b"Here(warn)\n").expect("write debug list fixture");
    fs::write(
        &rc,
        b"TabSize = 7\nOutFormat { \"%f:%l:%c:%n:%m!n\" }\nUserWarn { warn }\nSilent { \\foo bar }\n",
    )
    .expect("write minimal rc");

    let args: &[&OsStr] = &[
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-q"),
        OsStr::new("-v0"),
        OsStr::new("-d4"),
        fixture.as_os_str(),
    ];

    let c_output = run_os(&oracle, args);
    let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), args);
    assert_outputs_equal(&c_output, &rust_output);
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks default -d4 resource list dump parity"]
fn debug_default_resource_list_dump_matches_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!(
        "chktex-rust-debug-default-list-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create default debug list fixture dir");
    let fixture = dir.join("input.tex");
    fs::write(&fixture, b"Here(warn)\n").expect("write default debug list fixture");

    let args: &[&OsStr] = &[
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-q"),
        OsStr::new("-v0"),
        OsStr::new("-d4"),
        fixture.as_os_str(),
    ];

    let c_output = run_os(&oracle, args);
    let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), args);
    assert_outputs_equal(&c_output, &rust_output);
}

#[test]
#[ignore = "requires CHKTEX_ORACLE; checks custom case-insensitive debug resource parity"]
fn debug_custom_case_lists_match_oracle() {
    let oracle = oracle_path();
    let dir = std::env::temp_dir().join(format!("chktex-rust-debug-case-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create custom debug fixture dir");
    let fixture = dir.join("input.tex");
    let rc = dir.join("custom.chktexrc");
    fs::write(&fixture, b"Here(warn)\n").expect("write custom debug fixture");
    fs::write(
        &rc,
        b"Silent { \\foo }\n[ \\Bar ]\nAbbrev { abc }\n[ Def ]\nVerbEnvir { custom }\nMathEnvir { calc }\nWipeArg { \\foo:{} }\nNoCharNext { \\left:{}$ }\nOutFormat { \"%f:%l:%c:%n:%m!n\" }\n",
    )
    .expect("write custom debug rc");

    for debug_flag in ["-d2", "-d4"] {
        let args: &[&OsStr] = &[
            OsStr::new("-r"),
            OsStr::new("-g0"),
            OsStr::new("-l"),
            rc.as_os_str(),
            OsStr::new("-q"),
            OsStr::new("-v0"),
            OsStr::new(debug_flag),
            fixture.as_os_str(),
        ];

        let c_output = run_os(&oracle, args);
        let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), args);
        assert_output_run_equal(&c_output, &rust_output, &dir, &dir, &oracle);
    }
}

#[test]
#[ignore = "requires CHKTEX_ORACLE; checks combined -d2/-d4 resource debug parity"]
fn debug_combined_resource_bits_match_oracle() {
    let oracle = oracle_path();
    let dir = std::env::temp_dir().join(format!(
        "chktex-rust-debug-combined-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    fs::create_dir_all(&dir).expect("create combined debug fixture dir");
    let fixture = dir.join("input.tex");
    let rc = dir.join("combined.chktexrc");
    fs::write(&fixture, b"Here(warn)\n").expect("write combined debug fixture");
    fs::write(
        &rc,
        b"TabSize = 7\nOutFormat { \"%f:%l:%c:%n:%m!n\" }\nUserWarn { warn }\nSilent { \\foo bar }\n",
    )
    .expect("write combined debug rc");

    let args: &[&OsStr] = &[
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-q"),
        OsStr::new("-v0"),
        OsStr::new("-d6"),
        fixture.as_os_str(),
    ];

    let c_output = run_os(&oracle, args);
    let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), args);
    assert_outputs_equal(&c_output, &rust_output);
}

#[test]
#[ignore = "requires CHKTEX_ORACLE; checks all debug bit interactions"]
fn debug_all_bits_match_oracle() {
    let oracle = oracle_path();
    let dir = std::env::temp_dir().join(format!(
        "chktex-rust-debug-all-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    let project_dir = dir.join("project");
    let recursive_dir = dir.join("recursive");
    let nested_dir = recursive_dir.join("nested");
    fs::create_dir_all(&project_dir).expect("create all-debug project dir");
    fs::create_dir_all(&nested_dir).expect("create all-debug recursive dir");
    fs::write(project_dir.join("main.tex"), b"\\input{deep}\nHere(warn)\n")
        .expect("write all-debug main fixture");
    fs::write(nested_dir.join("deep.tex"), b"Here(warn)\n").expect("write all-debug child fixture");

    let rc = dir.join("all-debug.chktexrc");
    fs::write(
        &rc,
        format!(
            "TabSize = 7\nOutFormat {{ \"%f:%l:%c:%n:%m!n\" }}\nUserWarn {{ warn }}\nSilent {{ \\\\foo bar }}\nTeXInputs {{ {}// }}\n",
            recursive_dir.display()
        ),
    )
    .expect("write all-debug rc");

    let args: &[&OsStr] = &[
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-q"),
        OsStr::new("-v0"),
        OsStr::new("-d31"),
        OsStr::new("main.tex"),
    ];

    let c_output = Command::new(&oracle)
        .current_dir(&project_dir)
        .args(args)
        .output()
        .expect("run oracle all-debug fixture");
    let rust_output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .current_dir(&project_dir)
        .args(args)
        .output()
        .expect("run rust all-debug fixture");
    assert_outputs_equal(&c_output, &rust_output);
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks warning/error/message exit status parity"]
fn warning_exit_statuses_match_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!("chktex-rust-exit-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create exit-status fixture dir");
    let warning_fixture = dir.join("warning.tex");
    let error_fixture = dir.join("error.tex");
    let error_then_warning_fixture = dir.join("error_then_warning.tex");
    let warning_then_error_fixture = dir.join("warning_then_error.tex");
    fs::write(&warning_fixture, b"Here(warn)\n").expect("write warning fixture");
    fs::write(&error_fixture, b"\\hat\n").expect("write error fixture");
    fs::write(&error_then_warning_fixture, b"\\hat\nHere(warn)\n")
        .expect("write error then warning fixture");
    fs::write(&warning_then_error_fixture, b"Here(warn)\n\\hat\n")
        .expect("write warning then error fixture");

    let cases: [(&[&str], &Path); 9] = [
        (&[], warning_fixture.as_path()),
        (&["-wall"], warning_fixture.as_path()),
        (&["-mall"], warning_fixture.as_path()),
        (&["-e36"], warning_fixture.as_path()),
        (&["-n36"], warning_fixture.as_path()),
        (&[], error_fixture.as_path()),
        (&["-mall"], error_fixture.as_path()),
        (&[], error_then_warning_fixture.as_path()),
        (&[], warning_then_error_fixture.as_path()),
    ];

    for (extra_args, fixture) in cases {
        let c_output = run_with_fixture(&oracle, &rc, extra_args, fixture);
        let rust_output = run_with_fixture(env!("CARGO_BIN_EXE_chktex"), &rc, extra_args, fixture);
        assert_eq!(
            c_output.status.code(),
            rust_output.status.code(),
            "exit mismatch for args {:?} fixture {}",
            extra_args,
            fixture.display()
        );
    }

    let multi_file_cases: [(&Path, &Path); 2] = [
        (error_fixture.as_path(), warning_fixture.as_path()),
        (warning_fixture.as_path(), error_fixture.as_path()),
    ];
    for (first, second) in multi_file_cases {
        let args: Vec<&OsStr> = vec![
            OsStr::new("-r"),
            OsStr::new("-g0"),
            OsStr::new("-l"),
            rc.as_os_str(),
            OsStr::new("-q"),
            first.as_os_str(),
            second.as_os_str(),
        ];
        let c_output = run_os(&oracle, &args);
        let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), &args);
        assert_eq!(
            c_output.status.code(),
            rust_output.status.code(),
            "multi-file exit mismatch for {} then {}",
            first.display(),
            second.display()
        );
    }
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks EOF warning parity"]
fn eof_warning_controls_match_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!("chktex-rust-eof-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create EOF fixture dir");
    let env_fixture = dir.join("env.tex");
    let math_fixture = dir.join("math.tex");
    let bracket_fixture = dir.join("bracket.tex");
    let display_cmd_fixture = dir.join("display_cmd.tex");
    let inline_cmd_fixture = dir.join("inline_cmd.tex");
    let left_fixture = dir.join("left.tex");
    let context_fixture = dir.join("context.tex");
    let math_context_fixture = dir.join("math_context.tex");
    let context_math_fixture = dir.join("context_math.tex");
    fs::write(
        &env_fixture,
        b"\\begin{foo}\n\\begin{document}\n\\begin{bar}\n",
    )
    .expect("write env EOF fixture");
    fs::write(&math_fixture, b"$unclosed\n").expect("write math EOF fixture");
    fs::write(&bracket_fixture, b"(\n").expect("write bracket EOF fixture");
    fs::write(&display_cmd_fixture, b"\\[\n").expect("write display command EOF fixture");
    fs::write(&inline_cmd_fixture, b"\\(\n").expect("write inline command EOF fixture");
    fs::write(&left_fixture, b"\\left(\n").expect("write left delimiter fixture");
    fs::write(&context_fixture, b"\\startfoo\n").expect("write ConTeXt EOF fixture");
    fs::write(&math_context_fixture, b"$\\startfoo\n").expect("write math ConTeXt EOF fixture");
    fs::write(&context_math_fixture, b"\\startfoo\n$unclosed\n")
        .expect("write ConTeXt math EOF fixture");

    let cases: [(&[&str], &Path); 16] = [
        (&[], env_fixture.as_path()),
        (&["-n15"], env_fixture.as_path()),
        (&["-m15"], env_fixture.as_path()),
        (&["-e15"], env_fixture.as_path()),
        (&[], math_fixture.as_path()),
        (&["-n16"], math_fixture.as_path()),
        (&[], bracket_fixture.as_path()),
        (&["-n17"], bracket_fixture.as_path()),
        (&[], display_cmd_fixture.as_path()),
        (&[], inline_cmd_fixture.as_path()),
        (&[], left_fixture.as_path()),
        (&[], context_fixture.as_path()),
        (&["-e48"], context_fixture.as_path()),
        (&["-n48"], context_fixture.as_path()),
        (&[], math_context_fixture.as_path()),
        (&[], context_math_fixture.as_path()),
    ];

    for (extra_args, fixture) in cases {
        let c_output = run_with_fixture(&oracle, &rc, extra_args, fixture);
        let rust_output = run_with_fixture(env!("CARGO_BIN_EXE_chktex"), &rc, extra_args, fixture);
        assert_outputs_equal(&c_output, &rust_output);
    }
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks option behavior parity"]
fn resource_switches_match_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!("chktex-rust-switches-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create switch fixture dir");

    let header_fixture = dir.join("header.tex");
    fs::write(
        &header_fixture,
        b"Here(warn)\n\\begin{document}\nHere(warn)\n",
    )
    .expect("write header fixture");

    let verb_fixture = dir.join("verb.tex");
    fs::write(&verb_fixture, b"\\verb|Here(warn)|\nHere(warn)\n").expect("write verb fixture");

    let pipe_fixture = dir.join("pipe.tex");
    fs::write(&pipe_fixture, b"Here(warn)\n").expect("write pipe fixture");

    let cases: [(&[&str], &Path); 6] = [
        (&[], header_fixture.as_path()),
        (&["-H0"], header_fixture.as_path()),
        (&["-H1"], header_fixture.as_path()),
        (&["-x0"], verb_fixture.as_path()),
        (&["-x1"], verb_fixture.as_path()),
        (&["-V0"], pipe_fixture.as_path()),
    ];

    for (extra_args, fixture) in cases {
        let c_output = run_with_fixture(&oracle, &rc, extra_args, fixture);
        let rust_output = run_with_fixture(env!("CARGO_BIN_EXE_chktex"), &rc, extra_args, fixture);
        assert_outputs_equal(&c_output, &rust_output);
    }
}

#[test]
#[ignore = "requires CHKTEX_ORACLE; checks CmdLine rc option parity"]
fn cmdline_resource_options_match_oracle() {
    let oracle = oracle_path();
    let dir = std::env::temp_dir().join(format!("chktex-rust-cmdline-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create CmdLine fixture dir");

    let suppressed_rc = dir.join("suppressed.chktexrc");
    fs::write(&suppressed_rc, b"CmdLine { -n36 }\n").expect("write suppressed rc");

    let formatted_rc = dir.join("formatted.chktexrc");
    fs::write(
        &formatted_rc,
        b"CmdLine { -v0 }\nOutFormat { \"%f:%l:%c:%n:%m!n\" }\n",
    )
    .expect("write formatted rc");

    let fixture = dir.join("input.tex");
    fs::write(&fixture, b"Here(warn)\n").expect("write CmdLine fixture");

    let cases: [(&Path, &[&str]); 3] = [
        (suppressed_rc.as_path(), &[]),
        (suppressed_rc.as_path(), &["-w36"]),
        (formatted_rc.as_path(), &[]),
    ];

    for (rc, extra_args) in cases {
        let c_output = run_with_custom_rc(&oracle, rc, extra_args, &fixture);
        let rust_output =
            run_with_custom_rc(env!("CARGO_BIN_EXE_chktex"), rc, extra_args, &fixture);
        assert_outputs_equal(&c_output, &rust_output);
    }
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks CmdLine action option parity"]
fn cmdline_resource_action_options_match_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let base_rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!(
        "chktex-rust-cmdline-actions-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create CmdLine action fixture dir");

    let fixture = dir.join("input.tex");
    fs::write(&fixture, b"Here(warn)\n").expect("write CmdLine action fixture");

    let cases = [
        ("version.chktexrc", b"CmdLine { --version }\n".as_slice()),
        ("help.chktexrc", b"CmdLine { --help }\n".as_slice()),
        ("license.chktexrc", b"CmdLine { --license }\n".as_slice()),
    ];

    for (name, contents) in cases {
        let rc = dir.join(name);
        fs::write(&rc, contents).expect("write CmdLine action rc");
        let args: &[&OsStr] = &[
            OsStr::new("-g0"),
            OsStr::new("-l"),
            base_rc.as_os_str(),
            OsStr::new("-l"),
            rc.as_os_str(),
            OsStr::new("-q"),
            fixture.as_os_str(),
        ];

        let c_output = run_os(&oracle, args);
        let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), args);
        assert_output_run_equal(&c_output, &rust_output, &dir, &dir, &oracle);
    }
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks CmdLine accumulation across rc files"]
fn cmdline_resource_accumulates_across_rc_files() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let base_rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!(
        "chktex-rust-cmdline-accumulate-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create CmdLine accumulation fixture dir");

    let suppress_rc = dir.join("suppress.chktexrc");
    fs::write(&suppress_rc, b"CmdLine { -n36 }\n").expect("write suppression rc");
    let reset_rc = dir.join("reset.chktexrc");
    fs::write(&reset_rc, b"CmdLine { -r -q -v0 }\n").expect("write reset rc");
    let fixture = dir.join("input.tex");
    fs::write(&fixture, b"Here(warn)\n").expect("write CmdLine accumulation fixture");

    let args: &[&OsStr] = &[
        OsStr::new("-g0"),
        OsStr::new("-l"),
        base_rc.as_os_str(),
        OsStr::new("-l"),
        suppress_rc.as_os_str(),
        OsStr::new("-l"),
        reset_rc.as_os_str(),
        fixture.as_os_str(),
    ];

    let c_output = run_os(&oracle, args);
    let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), args);
    assert_outputs_equal(&c_output, &rust_output);
}

#[test]
#[ignore = "requires CHKTEX_ORACLE; checks CmdLine reset runtime option parity"]
fn cmdline_resource_reset_clears_prior_runtime_options() {
    let oracle = oracle_path();
    let dir =
        std::env::temp_dir().join(format!("chktex-rust-cmdline-reset-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create CmdLine reset fixture dir");

    let rc = dir.join("reset.chktexrc");
    fs::write(
        &rc,
        b"CmdLine { -r }\nOutFormat { \"%f:%l:%c:%n:%m!n\" }\nUserWarn { warn }\n",
    )
    .expect("write CmdLine reset rc");
    let fixture = dir.join("input.tex");
    fs::write(&fixture, b"Here(warn)\n").expect("write CmdLine reset fixture");

    let args: &[&OsStr] = &[
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-q"),
        OsStr::new("-v0"),
        fixture.as_os_str(),
    ];

    let c_output = run_os(&oracle, args);
    let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), args);
    assert_outputs_equal(&c_output, &rust_output);
}

#[test]
#[ignore = "requires CHKTEX_ORACLE; checks CmdLine reset output option ordering"]
fn cmdline_resource_reset_allows_later_output_option() {
    let oracle = oracle_path();
    let dir = std::env::temp_dir().join(format!(
        "chktex-rust-cmdline-output-reset-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    let oracle_dir = dir.join("oracle");
    let rust_dir = dir.join("rust");
    fs::create_dir_all(&oracle_dir).expect("create oracle CmdLine output dir");
    fs::create_dir_all(&rust_dir).expect("create rust CmdLine output dir");

    for work_dir in [&oracle_dir, &rust_dir] {
        fs::write(
            work_dir.join("reset.chktexrc"),
            b"CmdLine { -o first.txt -r -o second.txt -q -v0 }\nOutFormat { \"%f:%l:%c:%n:%m!n\" }\n",
        )
        .expect("write CmdLine output reset rc");
        fs::write(work_dir.join("input.tex"), b"Here(warn)\n")
            .expect("write CmdLine output reset fixture");
    }

    let args: &[&OsStr] = &[
        OsStr::new("-g0"),
        OsStr::new("-l"),
        OsStr::new("reset.chktexrc"),
        OsStr::new("input.tex"),
    ];
    let c_output = Command::new(&oracle)
        .current_dir(&oracle_dir)
        .args(args)
        .output()
        .expect("run oracle CmdLine output reset fixture");
    let rust_output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .current_dir(&rust_dir)
        .args(args)
        .output()
        .expect("run rust CmdLine output reset fixture");

    assert_output_run_equal(&c_output, &rust_output, &oracle_dir, &rust_dir, &oracle);
    assert!(!oracle_dir.join("first.txt").exists());
    assert!(!rust_dir.join("first.txt").exists());
    assert_eq!(
        normalize_paths(
            &fs::read(oracle_dir.join("second.txt")).unwrap(),
            &oracle_dir,
            &oracle
        ),
        normalize_paths(
            &fs::read(rust_dir.join("second.txt")).unwrap(),
            &rust_dir,
            Path::new(env!("CARGO_BIN_EXE_chktex"))
        )
    );
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks CmdSpaceStyle rc parity"]
fn cmd_space_style_matches_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!("chktex-rust-cmdspace-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create CmdSpaceStyle fixture dir");

    let fixture = dir.join("cmdspace.tex");
    fs::write(
        &fixture,
        b"\\etc. in text\n\\UFO! Right there\n\\UFO. Right there\n",
    )
    .expect("write CmdSpaceStyle fixture");

    for style in ["Ignore", "InterWord", "InterSentence", "Both"] {
        let setting = format!("CmdSpaceStyle={style}");
        let args = ["-S", setting.as_str()];
        let c_output = run_with_fixture(&oracle, &rc, &args, &fixture);
        let rust_output = run_with_fixture(env!("CARGO_BIN_EXE_chktex"), &rc, &args, &fixture);
        assert_outputs_equal(&c_output, &rust_output);
    }
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks TeXInputs include search parity"]
fn tex_inputs_include_search_matches_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!("chktex-rust-texinputs-{}", std::process::id()));
    let project_dir = dir.join("project");
    let lib_dir = dir.join("lib");
    let recursive_dir = dir.join("recursive");
    let nested_dir = recursive_dir.join("nested");
    fs::create_dir_all(&project_dir).expect("create project dir");
    fs::create_dir_all(&lib_dir).expect("create lib dir");
    fs::create_dir_all(&nested_dir).expect("create recursive dir");

    fs::write(
        project_dir.join("main.tex"),
        b"\\input{shared}\n\\input{deep}\n\\input{dotted.name}\n",
    )
    .expect("write main fixture");
    fs::write(lib_dir.join("shared.tex"), b"Here(warn)\n").expect("write shared fixture");
    fs::write(lib_dir.join("dotted.name.tex"), b"Here(warn)\n")
        .expect("write dotted extension fixture");
    fs::write(nested_dir.join("deep.tex"), b"Here(warn)\n").expect("write deep fixture");

    let texinputs_rc = dir.join("texinputs.chktexrc");
    fs::write(
        &texinputs_rc,
        format!(
            "TeXInputs {{ {} {}// }}\n",
            lib_dir.display(),
            recursive_dir.display()
        ),
    )
    .expect("write TeXInputs rc");

    let args: &[&OsStr] = &[
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-l"),
        texinputs_rc.as_os_str(),
        OsStr::new("-v0"),
        OsStr::new("-q"),
        OsStr::new("main.tex"),
    ];

    let c_output = Command::new(&oracle)
        .current_dir(&project_dir)
        .args(args)
        .output()
        .expect("run oracle TeXInputs fixture");
    let rust_output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .current_dir(&project_dir)
        .args(args)
        .output()
        .expect("run rust TeXInputs fixture");

    assert_outputs_equal(&c_output, &rust_output);
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks -d16 recursive include trace parity"]
fn runtime_debug_recursive_tex_inputs_matches_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!("chktex-rust-runtime-{}", std::process::id()));
    let project_dir = dir.join("project");
    let recursive_dir = dir.join("recursive");
    let nested_dir = recursive_dir.join("nested");
    fs::create_dir_all(&project_dir).expect("create runtime debug project dir");
    fs::create_dir_all(&nested_dir).expect("create runtime debug recursive dir");

    fs::write(project_dir.join("main.tex"), b"\\input{deep}\n")
        .expect("write runtime debug main fixture");
    fs::write(nested_dir.join("deep.tex"), b"Here(warn)\n")
        .expect("write runtime debug child fixture");
    let texinputs_rc = dir.join("texinputs.chktexrc");
    fs::write(
        &texinputs_rc,
        format!("TeXInputs {{ {}// }}\n", recursive_dir.display()),
    )
    .expect("write runtime debug TeXInputs rc");

    let args: &[&OsStr] = &[
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-l"),
        texinputs_rc.as_os_str(),
        OsStr::new("-v0"),
        OsStr::new("-q"),
        OsStr::new("-d16"),
        OsStr::new("main.tex"),
    ];

    let c_output = Command::new(&oracle)
        .current_dir(&project_dir)
        .args(args)
        .output()
        .expect("run oracle runtime debug fixture");
    let rust_output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .current_dir(&project_dir)
        .args(args)
        .output()
        .expect("run rust runtime debug fixture");

    assert_outputs_equal(&c_output, &rust_output);
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks failed recursive include trace parity"]
fn runtime_debug_failed_recursive_tex_inputs_matches_oracle() {
    use std::os::unix::fs::PermissionsExt;

    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!(
        "chktex-rust-runtime-failed-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    let oracle_dir = dir.join("oracle");
    let rust_dir = dir.join("rust");
    let oracle_project = oracle_dir.join("project");
    let rust_project = rust_dir.join("project");
    let oracle_blocked = oracle_dir.join("blocked");
    let rust_blocked = rust_dir.join("blocked");
    fs::create_dir_all(&oracle_project).expect("create oracle project dir");
    fs::create_dir_all(&rust_project).expect("create rust project dir");
    fs::create_dir_all(&oracle_blocked).expect("create oracle blocked dir");
    fs::create_dir_all(&rust_blocked).expect("create rust blocked dir");

    for project_dir in [&oracle_project, &rust_project] {
        fs::write(project_dir.join("main.tex"), b"\\input{missing}\n")
            .expect("write failed runtime main fixture");
    }
    let oracle_rc = oracle_dir.join("texinputs.chktexrc");
    let rust_rc = rust_dir.join("texinputs.chktexrc");
    fs::write(
        &oracle_rc,
        format!("TeXInputs {{ {}// }}\n", oracle_blocked.display()),
    )
    .expect("write oracle failed runtime rc");
    fs::write(
        &rust_rc,
        format!("TeXInputs {{ {}// }}\n", rust_blocked.display()),
    )
    .expect("write rust failed runtime rc");
    fs::set_permissions(&oracle_blocked, fs::Permissions::from_mode(0o000))
        .expect("make oracle blocked dir unreadable");
    fs::set_permissions(&rust_blocked, fs::Permissions::from_mode(0o000))
        .expect("make rust blocked dir unreadable");

    let c_output = Command::new(&oracle)
        .current_dir(&oracle_project)
        .args([
            OsStr::new("-r"),
            OsStr::new("-g0"),
            OsStr::new("-l"),
            rc.as_os_str(),
            OsStr::new("-l"),
            oracle_rc.as_os_str(),
            OsStr::new("-q"),
            OsStr::new("-v0"),
            OsStr::new("-d16"),
            OsStr::new("main.tex"),
        ])
        .output()
        .expect("run oracle failed runtime fixture");
    let rust_output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .current_dir(&rust_project)
        .args([
            OsStr::new("-r"),
            OsStr::new("-g0"),
            OsStr::new("-l"),
            rc.as_os_str(),
            OsStr::new("-l"),
            rust_rc.as_os_str(),
            OsStr::new("-q"),
            OsStr::new("-v0"),
            OsStr::new("-d16"),
            OsStr::new("main.tex"),
        ])
        .output()
        .expect("run rust failed runtime fixture");

    fs::set_permissions(&oracle_blocked, fs::Permissions::from_mode(0o700))
        .expect("restore oracle blocked dir");
    fs::set_permissions(&rust_blocked, fs::Permissions::from_mode(0o700))
        .expect("restore rust blocked dir");

    assert_output_run_equal(&c_output, &rust_output, &oracle_dir, &rust_dir, &oracle);
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks missing include parity"]
fn missing_include_matches_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!(
        "chktex-rust-missing-include-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create missing include fixture dir");
    let fixture = dir.join("main.tex");
    fs::write(&fixture, b"\\input{missing}\nHere(warn)\n").expect("write missing include fixture");

    for extra_args in [
        Vec::<&OsStr>::new(),
        vec![OsStr::new("-n27")],
        vec![OsStr::new("-m27")],
        vec![OsStr::new("-w27")],
        vec![OsStr::new("-e27")],
    ] {
        let mut args: Vec<&OsStr> = vec![
            OsStr::new("-r"),
            OsStr::new("-g0"),
            OsStr::new("-l"),
            rc.as_os_str(),
            OsStr::new("-q"),
        ];
        args.extend(extra_args);
        args.push(fixture.as_os_str());

        let c_output = run_os(&oracle, &args);
        let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), &args);
        assert_eq!(c_output.status.code(), rust_output.status.code());
        assert_eq!(c_output.stdout, rust_output.stdout);
        assert_eq!(
            normalize_paths(&normalize_oracle_stderr(&c_output.stderr), &dir, &oracle),
            normalize_paths(
                &normalize_oracle_stderr(&rust_output.stderr),
                &dir,
                Path::new(env!("CARGO_BIN_EXE_chktex"))
            )
        );
    }
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks include scanning edge parity"]
fn include_scanning_edges_match_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!(
        "chktex-rust-include-edges-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    fs::create_dir_all(&dir).expect("create include edge fixture dir");

    let unbraced = dir.join("unbraced.tex");
    fs::write(&unbraced, b"\\input child").expect("write unbraced include fixture");
    fs::write(dir.join("child.tex"), b"Here(warn)\n").expect("write child fixture");

    let multiple = dir.join("multiple.tex");
    fs::write(&multiple, b"\\input{one}\\input{two}\\input{three}\n")
        .expect("write multiple include fixture");
    for name in ["one", "two", "three"] {
        fs::write(dir.join(format!("{name}.tex")), format!("{name}(warn)\n"))
            .expect("write multi include child");
    }

    let dotted = dir.join("dotted.tex");
    fs::write(&dotted, b"\\input{child.foo}\n").expect("write dotted include fixture");
    fs::write(dir.join("child.foo.tex"), b"Here(warn)\n").expect("write dotted include child");

    let nested_missing = dir.join("nested_missing.tex");
    let subdir = dir.join("sub");
    fs::create_dir(&subdir).expect("create nested include subdir");
    fs::write(&nested_missing, b"\\input{sub/child}\n").expect("write nested missing fixture");
    fs::write(subdir.join("child.tex"), b"\\input{grand}\n").expect("write nested missing child");
    fs::write(subdir.join("grand.tex"), b"Here(warn)\n").expect("write ignored nested grandchild");

    for fixture in [unbraced, multiple, dotted, nested_missing] {
        let args: &[&OsStr] = &[
            OsStr::new("-r"),
            OsStr::new("-g0"),
            OsStr::new("-l"),
            rc.as_os_str(),
            OsStr::new("-q"),
            OsStr::new("-v0"),
            fixture.as_os_str(),
        ];

        let c_output = run_os(&oracle, args);
        let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), args);
        assert_output_run_equal(&c_output, &rust_output, &dir, &dir, &oracle);
    }
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks non-quiet banner and summary parity"]
fn non_quiet_banner_and_summary_match_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!("chktex-rust-summary-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create summary fixture dir");

    let clean_fixture = dir.join("clean.tex");
    let warning_fixture = dir.join("warning.tex");
    let error_fixture = dir.join("error.tex");
    fs::write(&clean_fixture, b"Clean text\n").expect("write clean fixture");
    fs::write(&warning_fixture, b"Here(warn)\n").expect("write warning fixture");
    fs::write(&error_fixture, b"\\hat\n").expect("write error fixture");

    for fixture in [clean_fixture, warning_fixture, error_fixture] {
        let mut command_args: Vec<&OsStr> = vec![
            OsStr::new("-r"),
            OsStr::new("-g0"),
            OsStr::new("-l"),
            rc.as_os_str(),
            OsStr::new("-v0"),
        ];
        command_args.push(fixture.as_os_str());

        let c_output = run_os(&oracle, &command_args);
        let rust_output = run_os(env!("CARGO_BIN_EXE_chktex"), &command_args);
        assert_outputs_equal(&c_output, &rust_output);
    }
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks output file and backup parity"]
fn output_file_backup_matches_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!("chktex-rust-output-{}", std::process::id()));
    let oracle_dir = dir.join("oracle");
    let rust_dir = dir.join("rust");
    fs::create_dir_all(&oracle_dir).expect("create oracle output dir");
    fs::create_dir_all(&rust_dir).expect("create rust output dir");

    for work_dir in [&oracle_dir, &rust_dir] {
        fs::write(work_dir.join("input.tex"), b"Here(warn)\n").expect("write output fixture");
        fs::write(work_dir.join("report.txt"), b"old output\n").expect("write prior report");
    }

    let args: &[&OsStr] = &[
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-v0"),
        OsStr::new("-o"),
        OsStr::new("report.txt"),
        OsStr::new("input.tex"),
    ];

    let c_output = Command::new(&oracle)
        .current_dir(&oracle_dir)
        .args(args)
        .output()
        .expect("run oracle output fixture");
    let rust_output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .current_dir(&rust_dir)
        .args(args)
        .output()
        .expect("run rust output fixture");

    assert_eq!(c_output.status.code(), rust_output.status.code());
    assert_eq!(c_output.stdout, rust_output.stdout);
    assert_eq!(
        normalize_paths(
            &normalize_oracle_stderr(&c_output.stderr),
            &oracle_dir,
            &oracle
        ),
        normalize_paths(
            &normalize_oracle_stderr(&rust_output.stderr),
            &rust_dir,
            Path::new(env!("CARGO_BIN_EXE_chktex"))
        )
    );
    assert_eq!(
        normalize_paths(
            &fs::read(oracle_dir.join("report.txt")).unwrap(),
            &oracle_dir,
            &oracle
        ),
        normalize_paths(
            &fs::read(rust_dir.join("report.txt")).unwrap(),
            &rust_dir,
            Path::new(env!("CARGO_BIN_EXE_chktex"))
        )
    );
    assert_eq!(
        fs::read(oracle_dir.join("report.txt.bak")).unwrap(),
        fs::read(rust_dir.join("report.txt.bak")).unwrap()
    );
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks explicit output backup disabling"]
fn output_file_no_backup_matches_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!(
        "chktex-rust-output-no-backup-{}",
        std::process::id()
    ));
    let oracle_dir = dir.join("oracle");
    let rust_dir = dir.join("rust");
    fs::create_dir_all(&oracle_dir).expect("create oracle output dir");
    fs::create_dir_all(&rust_dir).expect("create rust output dir");

    for work_dir in [&oracle_dir, &rust_dir] {
        fs::write(work_dir.join("input.tex"), b"Here(warn)\n").expect("write output fixture");
        fs::write(work_dir.join("report.txt"), b"old output\n").expect("write prior report");
    }

    let args: &[&OsStr] = &[
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-v0"),
        OsStr::new("-b0"),
        OsStr::new("-o"),
        OsStr::new("report.txt"),
        OsStr::new("input.tex"),
    ];

    let c_output = Command::new(&oracle)
        .current_dir(&oracle_dir)
        .args(args)
        .output()
        .expect("run oracle output fixture");
    let rust_output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .current_dir(&rust_dir)
        .args(args)
        .output()
        .expect("run rust output fixture");

    assert_eq!(c_output.status.code(), rust_output.status.code());
    assert_eq!(c_output.stdout, rust_output.stdout);
    assert_eq!(
        normalize_paths(
            &normalize_oracle_stderr(&c_output.stderr),
            &oracle_dir,
            &oracle
        ),
        normalize_paths(
            &normalize_oracle_stderr(&rust_output.stderr),
            &rust_dir,
            Path::new(env!("CARGO_BIN_EXE_chktex"))
        )
    );
    assert_eq!(
        normalize_paths(
            &fs::read(oracle_dir.join("report.txt")).unwrap(),
            &oracle_dir,
            &oracle
        ),
        normalize_paths(
            &fs::read(rust_dir.join("report.txt")).unwrap(),
            &rust_dir,
            Path::new(env!("CARGO_BIN_EXE_chktex"))
        )
    );
    assert!(!oracle_dir.join("report.txt.bak").exists());
    assert!(!rust_dir.join("report.txt.bak").exists());
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks pre-existing output backup overwrite"]
fn output_file_existing_backup_matches_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!(
        "chktex-rust-output-existing-backup-{}",
        std::process::id()
    ));
    let oracle_dir = dir.join("oracle");
    let rust_dir = dir.join("rust");
    fs::create_dir_all(&oracle_dir).expect("create oracle output dir");
    fs::create_dir_all(&rust_dir).expect("create rust output dir");

    for work_dir in [&oracle_dir, &rust_dir] {
        fs::write(work_dir.join("input.tex"), b"Here(warn)\n").expect("write output fixture");
        fs::write(work_dir.join("report.txt"), b"old output\n").expect("write prior report");
        fs::write(work_dir.join("report.txt.bak"), b"old backup\n").expect("write prior backup");
    }

    let args: &[&OsStr] = &[
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-v0"),
        OsStr::new("-o"),
        OsStr::new("report.txt"),
        OsStr::new("input.tex"),
    ];

    let c_output = Command::new(&oracle)
        .current_dir(&oracle_dir)
        .args(args)
        .output()
        .expect("run oracle output fixture");
    let rust_output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .current_dir(&rust_dir)
        .args(args)
        .output()
        .expect("run rust output fixture");

    assert_output_run_equal(&c_output, &rust_output, &oracle_dir, &rust_dir, &oracle);
    assert_eq!(
        normalize_paths(
            &fs::read(oracle_dir.join("report.txt")).unwrap(),
            &oracle_dir,
            &oracle
        ),
        normalize_paths(
            &fs::read(rust_dir.join("report.txt")).unwrap(),
            &rust_dir,
            Path::new(env!("CARGO_BIN_EXE_chktex"))
        )
    );
    assert_eq!(
        fs::read(oracle_dir.join("report.txt.bak")).unwrap(),
        fs::read(rust_dir.join("report.txt.bak")).unwrap()
    );
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks backup rename failures"]
fn output_file_backup_rename_failure_matches_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!(
        "chktex-rust-output-backup-failure-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    let oracle_dir = dir.join("oracle");
    let rust_dir = dir.join("rust");
    fs::create_dir_all(&oracle_dir).expect("create oracle output dir");
    fs::create_dir_all(&rust_dir).expect("create rust output dir");

    for work_dir in [&oracle_dir, &rust_dir] {
        fs::write(work_dir.join("input.tex"), b"Here(warn)\n").expect("write output fixture");
        fs::write(work_dir.join("report.txt"), b"old output\n").expect("write prior report");
        fs::create_dir(work_dir.join("report.txt.bak")).expect("create backup collision dir");
    }

    let args: &[&OsStr] = &[
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-v0"),
        OsStr::new("-o"),
        OsStr::new("report.txt"),
        OsStr::new("input.tex"),
    ];

    let c_output = Command::new(&oracle)
        .current_dir(&oracle_dir)
        .args(args)
        .output()
        .expect("run oracle backup failure fixture");
    let rust_output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .current_dir(&rust_dir)
        .args(args)
        .output()
        .expect("run rust backup failure fixture");

    assert_output_run_equal(&c_output, &rust_output, &oracle_dir, &rust_dir, &oracle);
    assert_eq!(
        fs::read(oracle_dir.join("report.txt")).unwrap(),
        b"old output\n"
    );
    assert_eq!(
        fs::read(rust_dir.join("report.txt")).unwrap(),
        b"old output\n"
    );
    assert!(oracle_dir.join("report.txt.bak").is_dir());
    assert!(rust_dir.join("report.txt.bak").is_dir());
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks output open failures"]
fn output_file_directory_failure_matches_oracle() {
    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!(
        "chktex-rust-output-dir-failure-{}",
        std::process::id()
    ));
    let oracle_dir = dir.join("oracle");
    let rust_dir = dir.join("rust");
    fs::create_dir_all(oracle_dir.join("outdir")).expect("create oracle output dir");
    fs::create_dir_all(rust_dir.join("outdir")).expect("create rust output dir");

    for work_dir in [&oracle_dir, &rust_dir] {
        fs::write(work_dir.join("input.tex"), b"Here(warn)\n").expect("write output fixture");
    }

    let args: &[&OsStr] = &[
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-v0"),
        OsStr::new("-o"),
        OsStr::new("outdir"),
        OsStr::new("input.tex"),
    ];

    let c_output = Command::new(&oracle)
        .current_dir(&oracle_dir)
        .args(args)
        .output()
        .expect("run oracle output failure fixture");
    let rust_output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .current_dir(&rust_dir)
        .args(args)
        .output()
        .expect("run rust output failure fixture");

    assert_output_run_equal(&c_output, &rust_output, &oracle_dir, &rust_dir, &oracle);
    assert!(oracle_dir.join("outdir").is_dir());
    assert!(rust_dir.join("outdir").is_dir());
    assert!(!oracle_dir.join("outdir.bak").exists());
    assert!(!rust_dir.join("outdir.bak").exists());
}

#[test]
#[ignore = "requires CHKTEX_ORACLE and CHKTEX_UPSTREAM_DIR; checks output permission failures"]
fn output_file_permission_failure_matches_oracle() {
    use std::os::unix::fs::PermissionsExt;

    let oracle = oracle_path();
    let upstream_dir = upstream_dir();
    let rc = upstream_dir.join("chktexrc");
    let dir = std::env::temp_dir().join(format!(
        "chktex-rust-output-permission-failure-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    let oracle_dir = dir.join("oracle");
    let rust_dir = dir.join("rust");
    let oracle_blocked = oracle_dir.join("blocked");
    let rust_blocked = rust_dir.join("blocked");
    fs::create_dir_all(&oracle_blocked).expect("create oracle blocked dir");
    fs::create_dir_all(&rust_blocked).expect("create rust blocked dir");

    for work_dir in [&oracle_dir, &rust_dir] {
        fs::write(work_dir.join("input.tex"), b"Here(warn)\n").expect("write output fixture");
    }
    fs::set_permissions(&oracle_blocked, fs::Permissions::from_mode(0o500))
        .expect("make oracle dir read-only");
    fs::set_permissions(&rust_blocked, fs::Permissions::from_mode(0o500))
        .expect("make rust dir read-only");

    let args: &[&OsStr] = &[
        OsStr::new("-r"),
        OsStr::new("-g0"),
        OsStr::new("-l"),
        rc.as_os_str(),
        OsStr::new("-v0"),
        OsStr::new("-o"),
        OsStr::new("blocked/report.txt"),
        OsStr::new("input.tex"),
    ];

    let c_output = Command::new(&oracle)
        .current_dir(&oracle_dir)
        .args(args)
        .output()
        .expect("run oracle output permission fixture");
    let rust_output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .current_dir(&rust_dir)
        .args(args)
        .output()
        .expect("run rust output permission fixture");

    fs::set_permissions(&oracle_blocked, fs::Permissions::from_mode(0o700))
        .expect("restore oracle dir permissions");
    fs::set_permissions(&rust_blocked, fs::Permissions::from_mode(0o700))
        .expect("restore rust dir permissions");

    assert_output_run_equal(&c_output, &rust_output, &oracle_dir, &rust_dir, &oracle);
    assert!(!oracle_blocked.join("report.txt").exists());
    assert!(!rust_blocked.join("report.txt").exists());
}

/// Runs the Rust binary on the canonical test fixture using local resources.
/// This tests our rc parsing and basic checking without needing the oracle.
#[test]
fn rust_binary_handles_test_fixture() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir.parent().unwrap().parent().unwrap();
    let fixture = project_root.join("tests/fixtures/upstream/Test.tex");
    let rc = project_root.join("tests/fixtures/upstream/chktexrc");

    let output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .args(["-mall", "-r", "-g0", "-l"])
        .arg(&rc)
        .args(["-v5", "-q"])
        .arg(&fixture)
        .output()
        .expect("run chktex on Test.tex");

    // Must exit successfully
    assert!(
        output.status.success(),
        "chktex exited with code {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    // Must produce output
    assert!(!output.stdout.is_empty(), "chktex should produce output");

    // Must not produce any stderr
    assert!(
        output.stderr.is_empty(),
        "chktex stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn rust_binary_version_and_help_match_upstream_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .arg("--version")
        .output()
        .expect("run chktex --version");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("ChkTeX v"));

    let output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .arg("--help")
        .output()
        .expect("run chktex --help");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage of ChkTeX"));

    let output = Command::new(env!("CARGO_BIN_EXE_chktex"))
        .arg("--license")
        .output()
        .expect("run chktex --license");
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("GNU General Public License"));
}

// ====== Helpers ======

fn ensure_upstream_config_fixtures(upstream_dir: &Path) {
    let tests = upstream_dir.join("tests");
    let write = |path: &Path, body: &str| {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create upstream config fixture dir");
        }
        fs::write(path, body).expect("write upstream config fixture");
    };

    write(
        &tests.join("sub/chktexrc"),
        "OutFormat\n{\n\"loaded chktex/tests/sub %f!n\"\n}\n",
    );
    write(
        &tests.join("sub1/.config/chktexrc"),
        "OutFormat\n{\n\"loaded chktex/tests/sub1/.config/chktexrc %f!n\"\n}\n",
    );
    write(
        &tests.join("sub2/.chktexrc"),
        "OutFormat\n{\n\"loaded chktex/tests/sub2/.chktexrc %f!n\"\n}\n",
    );
}

fn run_with_stdin<F>(program: impl AsRef<OsStr>, setup: F, input: &[u8]) -> Output
where
    F: FnOnce(&mut Command),
{
    use std::io::Write;

    let mut command = Command::new(program.as_ref());
    setup(&mut command);
    command
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = command.spawn().expect("spawn process with stdin");
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input);
    }
    child
        .wait_with_output()
        .expect("wait for process with stdin")
}

fn oracle_path() -> PathBuf {
    std::env::var_os("CHKTEX_ORACLE")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .or_else(|| {
            let path = PathBuf::from(DEFAULT_ORACLE);
            path.is_file().then_some(path)
        })
        .unwrap_or_else(|| {
            panic!(
                "Set CHKTEX_ORACLE to an upstream C chktex binary, or build one at {DEFAULT_ORACLE}"
            )
        })
}

fn upstream_dir() -> PathBuf {
    std::env::var_os("CHKTEX_UPSTREAM_DIR")
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
        .or_else(|| {
            let path = PathBuf::from(DEFAULT_UPSTREAM_DIR);
            path.is_dir().then_some(path)
        })
        .unwrap_or_else(|| {
            panic!("Set CHKTEX_UPSTREAM_DIR to an upstream checkout, or use {DEFAULT_UPSTREAM_DIR}")
        })
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos()
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

fn run_rust_config_probe(
    envs: &[(&str, PathBuf)],
    current_dir: Option<&Path>,
    input: &[u8],
) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_chktex"));
    command
        .args(["-mall", "-v0", "-q"])
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("HOME")
        .env_remove("LOGDIR")
        .env_remove("CHKTEXRC")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    if let Some(current_dir) = current_dir {
        command.current_dir(current_dir);
    }
    for (key, value) in envs {
        command.env(key, value);
    }
    command
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(input);
            }
            child.wait_with_output()
        })
        .expect("run rust config lookup fixture")
}

fn run_with_fixture(
    program: impl AsRef<OsStr>,
    rc: &Path,
    extra_args: &[&str],
    fixture: &Path,
) -> Output {
    let mut command = Command::new(program);
    command.args(["-r", "-g0", "-l"]).arg(rc).arg("-q");
    command.args(extra_args).arg(fixture);
    command.output().expect("run exit-status fixture")
}

fn run_with_custom_rc(
    program: impl AsRef<OsStr>,
    rc: &Path,
    extra_args: &[&str],
    fixture: &Path,
) -> Output {
    let mut command = Command::new(program);
    command.args(["-g0", "-l"]).arg(rc).arg("-q");
    command.args(extra_args).arg(fixture);
    command.output().expect("run custom rc fixture")
}

fn assert_outputs_equal(c_output: &Output, rust_output: &Output) {
    assert_eq!(c_output.status.code(), rust_output.status.code());

    if c_output.stdout != rust_output.stdout {
        write_debug_file("oracle.stdout", &c_output.stdout);
        write_debug_file("rust.stdout", &rust_output.stdout);

        // Warning-level comparison for debugging
        let oracle_warnings = extract_warning_counts(&c_output.stdout);
        let rust_warnings = extract_warning_counts(&rust_output.stdout);

        eprintln!("\n=== WARNING COUNTS ===");
        eprintln!("{:>5} {:>8} {:>8} {:>8}", "W#", "Oracle", "Rust", "Diff");
        eprintln!("{}", "-".repeat(33));

        let mut all_keys: Vec<i32> = oracle_warnings
            .keys()
            .chain(rust_warnings.keys())
            .copied()
            .collect();
        all_keys.sort();
        all_keys.dedup();
        for key in &all_keys {
            let o = oracle_warnings.get(key).copied().unwrap_or(0);
            let r = rust_warnings.get(key).copied().unwrap_or(0);
            if o != r {
                eprintln!("{:>5} {:>8} {:>8} {:>+8}", key, o, r, r as i64 - o as i64);
            }
        }
        eprintln!("{}", "-".repeat(33));

        panic!("stdout differs; wrote oracle.stdout and rust.stdout");
    }

    let c_stderr = normalize_oracle_stderr(&c_output.stderr);
    let rust_stderr = normalize_oracle_stderr(&rust_output.stderr);
    if c_stderr != rust_stderr {
        write_debug_file("oracle.stderr", &c_output.stderr);
        write_debug_file("rust.stderr", &rust_output.stderr);
        panic!("stderr differs; wrote oracle.stderr and rust.stderr");
    }
}

fn assert_output_run_equal(
    c_output: &Output,
    rust_output: &Output,
    oracle_dir: &Path,
    rust_dir: &Path,
    oracle: &Path,
) {
    assert_eq!(c_output.status.code(), rust_output.status.code());
    assert_eq!(c_output.stdout, rust_output.stdout);
    assert_eq!(
        normalize_paths(
            &normalize_oracle_stderr(&c_output.stderr),
            oracle_dir,
            oracle
        ),
        normalize_paths(
            &normalize_oracle_stderr(&rust_output.stderr),
            rust_dir,
            Path::new(env!("CARGO_BIN_EXE_chktex"))
        )
    );
}

fn normalize_oracle_stderr(stderr: &[u8]) -> Vec<u8> {
    stderr
        .split_inclusive(|byte| *byte == b'\n')
        .filter(|line| !line.ends_with(b": WARNING -- Could not find global resource file.\n"))
        .flatten()
        .copied()
        .collect()
}

fn normalize_paths(bytes: &[u8], work_dir: &Path, program: &Path) -> Vec<u8> {
    String::from_utf8_lossy(bytes)
        .replace(&work_dir.to_string_lossy().to_string(), "$WORKDIR")
        .replace(&program.to_string_lossy().to_string(), "$PROGRAM")
        .into_bytes()
}

/// Extract warning counts per warning number from chktex output.
fn extract_warning_counts(output: &[u8]) -> std::collections::HashMap<i32, usize> {
    let mut counts = std::collections::HashMap::new();
    for line in output.split(|&b| b == b'\n') {
        if line.is_empty() {
            continue;
        }
        // Format: "Message N in ..." or "Warning N in ..." or "Error N in ..."
        let line_str = String::from_utf8_lossy(line);
        if let Some(rest) = line_str
            .strip_prefix("Message ")
            .or_else(|| line_str.strip_prefix("Warning "))
            .or_else(|| line_str.strip_prefix("Error "))
        {
            if let Some(num_str) = rest.split(' ').next() {
                if let Ok(num) = num_str.parse::<i32>() {
                    *counts.entry(num).or_insert(0) += 1;
                }
            }
        }
    }
    counts
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

fn normalize_upstream_test_paths(output: &[u8]) -> Vec<u8> {
    String::from_utf8_lossy(output)
        .replace("Message 22 in tests/", "Message 22 in ")
        .into_bytes()
}
