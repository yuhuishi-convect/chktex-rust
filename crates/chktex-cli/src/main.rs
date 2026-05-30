use std::{
    fs,
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use chktex_core::{
    PACKAGE_NAME,
    checker::{CheckerConfig, check_document},
    cli::{CliAction, CliOptions, WarningSelector, WarningSeverity, parse_args},
    diagnostic::{DiagnosticKind, FormatOptions, format_diagnostic_bytes},
    resource::{ResourceSet, parse_resource},
};

fn main() -> ExitCode {
    let args = std::env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned());

    let options = match parse_args(args) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("{PACKAGE_NAME}: {err}");
            return ExitCode::from(1);
        }
    };

    match options.action {
        CliAction::Version => {
            print_version_stdout();
            ExitCode::SUCCESS
        }
        CliAction::Help => {
            print_help();
            ExitCode::from(1)
        }
        CliAction::License => {
            print_license();
            ExitCode::SUCCESS
        }
        CliAction::Check => run_check(&options),
    }
}

fn run_check(options: &CliOptions) -> ExitCode {
    let mut options = options.clone();
    let mut resources = match load_resources(&options) {
        Ok(resources) => resources,
        Err(err) => {
            eprintln!("{PACKAGE_NAME}: {err}");
            return ExitCode::from(1);
        }
    };
    if let Err(err) = apply_cmdline_resource_options(&mut options, &mut resources) {
        eprintln!("{PACKAGE_NAME}: {err}");
        return ExitCode::from(1);
    }
    match options.action {
        CliAction::Version => {
            print_version_stdout();
            return ExitCode::SUCCESS;
        }
        CliAction::Help => {
            print_help();
            return ExitCode::from(1);
        }
        CliAction::License => {
            print_license();
            return ExitCode::SUCCESS;
        }
        CliAction::Check => {}
    }
    let mut config = CheckerConfig::from_resources(&resources);
    config.no_line_suppression = options.no_line_suppression;
    if let Some(header_errors) = options.header_errors {
        config.header_errors = header_errors;
    }
    if let Some(wipe_verb) = options.wipe_verb {
        config.wipe_verb = wipe_verb;
    }
    apply_warning_options(&mut config, &options);

    let output_format = select_format(&options, &resources);
    let sv_override = severity_override(&options);
    let tex_inputs = tex_inputs(&resources);
    let runtime_debug = options
        .debug_level
        .is_some_and(|debug_level| debug_level & 16 != 0);
    let mut output = Vec::new();
    let mut stats = OutputStats::default();
    let mut exit_status = 0u8;

    if !options.quiet {
        print_banner_stderr();
    }
    if let Some(debug_level) = options.debug_level {
        write_debug_resources(&options, &resources, &config, &output_format, debug_level);
    }

    if options.files.is_empty() {
        let mut input = Vec::new();
        if let Err(err) = io::stdin().read_to_end(&mut input) {
            eprintln!("{PACKAGE_NAME}: failed to read stdin: {err}");
            return ExitCode::from(1);
        }
        let name = options.pseudoname.as_deref().unwrap_or("stdin");
        exit_status = combine_exit_status(
            exit_status,
            write_diagnostics(
                &mut output,
                name,
                &input,
                &config,
                &output_format,
                sv_override,
                &mut stats,
            ),
        );
    } else {
        for file in &options.files {
            let name = options
                .pseudoname
                .as_deref()
                .map(String::from)
                .unwrap_or_else(|| file.to_string_lossy().into_owned());
            match write_file_diagnostics(
                &mut output,
                file,
                &name,
                &config,
                &output_format,
                sv_override,
                options.input_files.unwrap_or(true),
                &tex_inputs,
                runtime_debug,
                &mut stats,
            ) {
                Ok(status) => exit_status = combine_exit_status(exit_status, status),
                Err(err) => {
                    eprintln!("{PACKAGE_NAME}: {err}");
                    return ExitCode::from(1);
                }
            }
        }
    }

    if let Some(path) = &options.output {
        if let Err(err) = write_output_file(path, &output, options.backup.unwrap_or(true)) {
            eprintln!("{}: {err}", program_invocation_name());
            return ExitCode::from(1);
        }
    } else if let Err(err) = io::stdout().write_all(&output) {
        eprintln!("{PACKAGE_NAME}: failed to write stdout: {err}");
        return ExitCode::from(1);
    }

    if !options.quiet {
        print_summary(&stats);
    }

    ExitCode::from(exit_status)
}

fn write_debug_resources(
    options: &CliOptions,
    resources: &ResourceSet,
    config: &CheckerConfig,
    output_format: &FormatOptions,
    debug_level: i64,
) {
    if debug_level & 1 != 0 {
        write_debug_warning_table(config);
    }

    if debug_level & 6 != 0 {
        write_debug_resource_tables(resources, debug_level & 2 != 0, debug_level & 4 != 0);
    }

    if debug_level & 8 != 0 {
        eprintln!("Outputformat:\n\t{}", output_format.format);
        eprintln!("Current flags include:");
        debug_bool("Read global resource", options.global_rc.unwrap_or(true));
        debug_bool("Wipe verbose commands", options.wipe_verb.unwrap_or(true));
        debug_bool("Backup outfile", options.backup.unwrap_or(true));
        debug_bool("Quiet mode", options.quiet);
        debug_bool(
            "Show license",
            options.action == CliAction::License || options.license,
        );
        debug_bool("Use stdin", options.files.is_empty());
        debug_bool("\\input files", options.input_files.unwrap_or(true));
        debug_bool(
            "Output header errors",
            options.header_errors.unwrap_or(true),
        );
        debug_bool("No line suppressions", options.no_line_suppression);
    }
}

struct DebugWarning {
    number: i32,
    text: &'static [u8],
}

const DEBUG_WARNINGS: &[DebugWarning] = &[
    DebugWarning {
        number: 1,
        text: b"Command terminated with space.",
    },
    DebugWarning {
        number: 2,
        text: b"Non-breaking space (`~') should have been used.",
    },
    DebugWarning {
        number: 3,
        text: b"You should enclose the previous parenthesis with `{}'.",
    },
    DebugWarning {
        number: 4,
        text: b"Italic correction (`\\/') found in non-italic buffer.",
    },
    DebugWarning {
        number: 5,
        text: b"Italic correction (`\\/') found more than once.",
    },
    DebugWarning {
        number: 6,
        text: b"No italic correction (`\\/') found.",
    },
    DebugWarning {
        number: 7,
        text: b"Accent command `%s' needs use of `\\%c%s'.",
    },
    DebugWarning {
        number: 8,
        text: b"Wrong length of dash may have been used.",
    },
    DebugWarning {
        number: 9,
        text: b"`%s' expected, found `%s'.",
    },
    DebugWarning {
        number: 10,
        text: b"Solo `%s' found.",
    },
    DebugWarning {
        number: 11,
        text: b"You should use %s to achieve an ellipsis.",
    },
    DebugWarning {
        number: 12,
        text: b"Interword spacing (`\\ ') should perhaps be used.",
    },
    DebugWarning {
        number: 13,
        text: b"Intersentence spacing (`\\@') should perhaps be used.",
    },
    DebugWarning {
        number: 14,
        text: b"Could not find argument for command.",
    },
    DebugWarning {
        number: 15,
        text: b"No match found for `%s'.",
    },
    DebugWarning {
        number: 16,
        text: b"Mathmode still on at end of LaTeX file.",
    },
    DebugWarning {
        number: 17,
        text: b"Number of `%c' doesn't match the number of `%c'!",
    },
    DebugWarning {
        number: 18,
        text: b"Use either `` or '' as an alternative to `\"'.",
    },
    DebugWarning {
        number: 19,
        text: b"Use \"'\" (ASCII 39) instead  of \"\xB4\" (ASCII 180).",
    },
    DebugWarning {
        number: 20,
        text: b"User-specified pattern found: %s.",
    },
    DebugWarning {
        number: 21,
        text: b"This command might not be intended.",
    },
    DebugWarning {
        number: 22,
        text: b"Comment displayed.",
    },
    DebugWarning {
        number: 23,
        text: b"Either %c\\,%c%c or %c%c\\,%c will look better.",
    },
    DebugWarning {
        number: 24,
        text: b"Delete this space to maintain correct pagereferences.",
    },
    DebugWarning {
        number: 25,
        text: b"You might wish to put this between a pair of `{}'",
    },
    DebugWarning {
        number: 26,
        text: b"You ought to remove spaces in front of punctuation.",
    },
    DebugWarning {
        number: 27,
        text: b"Could not execute LaTeX command.",
    },
    DebugWarning {
        number: 28,
        text: b"Don't use \\/ in front of small punctuation.",
    },
    DebugWarning {
        number: 29,
        text: b"$\\times$ may look prettier here.",
    },
    DebugWarning {
        number: 30,
        text: b"Multiple spaces detected in input.",
    },
    DebugWarning {
        number: 31,
        text: b"This text may be ignored.",
    },
    DebugWarning {
        number: 32,
        text: b"Use ` to begin quotation, not '.",
    },
    DebugWarning {
        number: 33,
        text: b"Use ' to end quotation, not `.",
    },
    DebugWarning {
        number: 34,
        text: b"Don't mix quotes.",
    },
    DebugWarning {
        number: 35,
        text: b"You should perhaps use `\\%s' instead.",
    },
    DebugWarning {
        number: 36,
        text: b"You should put a space %s parenthesis.",
    },
    DebugWarning {
        number: 37,
        text: b"You should avoid spaces %s parenthesis.",
    },
    DebugWarning {
        number: 38,
        text: b"You should not use punctuation %s quotes.",
    },
    DebugWarning {
        number: 39,
        text: b"Double space found.",
    },
    DebugWarning {
        number: 40,
        text: b"You should put punctuation %s math mode.",
    },
    DebugWarning {
        number: 41,
        text: b"You ought to not use primitive TeX in LaTeX code.",
    },
    DebugWarning {
        number: 42,
        text: b"You should remove spaces in front of `%s'",
    },
    DebugWarning {
        number: 43,
        text: b"`%s' is normally not followed by `%c'.",
    },
    DebugWarning {
        number: 44,
        text: b"User Regex: %.*s.",
    },
    DebugWarning {
        number: 45,
        text: b"Use \\[ ... \\] instead of $$ ... $$.",
    },
    DebugWarning {
        number: 46,
        text: b"Use \\( ... \\) instead of $ ... $.",
    },
    DebugWarning {
        number: 47,
        text: b"`%s' expected, found `%s' (ConTeXt).",
    },
    DebugWarning {
        number: 48,
        text: b"No match found for `%s' (ConTeXt).",
    },
    DebugWarning {
        number: 49,
        text: b"Expected math mode to be %s here.",
    },
];

fn write_debug_warning_table(config: &CheckerConfig) {
    let mut output = Vec::new();
    output.extend_from_slice(b"There are 49 warnings/error messages available:\n");
    for warning in DEBUG_WARNINGS {
        let kind = config.warning_kind(warning.number).as_str();
        let status = if config.warning_enabled(warning.number) {
            "In use"
        } else if warning_enabled_by_default(warning.number) {
            "User muted"
        } else {
            "System muted"
        };
        output.extend_from_slice(
            format!(
                "Number: {:2}, Type: {}, Status: {}\n\tText: ",
                warning.number, kind, status
            )
            .as_bytes(),
        );
        output.extend_from_slice(warning.text);
        output.extend_from_slice(b"\n\n");
    }
    let _ = io::stderr().write_all(&output);
}

fn warning_enabled_by_default(warning: i32) -> bool {
    !matches!(warning, 19 | 21 | 22 | 30 | 41 | 46)
}

#[derive(Clone, Copy)]
enum DebugListKind {
    CaseSensitive,
    CaseInsensitive,
    CaseSensitiveWithStarVariants,
    AbbrevWithCaseVariants,
    AbbrevCaseNormalized,
}

fn write_debug_resource_tables(
    resources: &ResourceSet,
    include_summary: bool,
    include_values: bool,
) {
    for (display_name, key, kind, default_empty_entry) in DEBUG_RESOURCE_TABLE {
        eprint!("Name: {display_name:>12}");
        if include_summary {
            let (max_len, entries) =
                debug_resource_stats(resources, key, *kind, *default_empty_entry);
            if entries == 0 {
                eprint!(", MaxLen: {max_len:>3}, Entries: {entries:>3}, No hash table.");
            } else {
                let hash_usage = debug_hash_usage(display_name, entries);
                eprint!(", MaxLen: {max_len:>3}, Entries: {entries:>3}, Hash usage: {hash_usage}%");
            }
        }
        eprintln!();

        if !include_values {
            continue;
        }

        let values = debug_resource_values(resources, key, *kind);
        if *default_empty_entry {
            eprintln!("\t");
        }
        for value in debug_resource_display_values(display_name, &values) {
            eprintln!("\t{value}");
        }
    }

    write_debug_scalar_values(resources);
}

fn write_debug_scalar_values(resources: &ResourceSet) {
    eprintln!("VerbClear:");
    eprintln!(
        "\t{}",
        resources
            .get("VerbClear")
            .and_then(|entry| entry.value.as_deref())
            .unwrap_or("|")
    );
    eprintln!("QuoteStyle:");
    eprintln!(
        "\t{}",
        resources
            .get("QuoteStyle")
            .and_then(|entry| entry.value.as_deref())
            .unwrap_or("Traditional")
    );
    eprintln!("TabSize:");
    eprintln!(
        "\t{}",
        resources
            .get("TabSize")
            .and_then(|entry| entry.value.as_deref())
            .unwrap_or("8")
    );
    eprintln!("CmdSpaceStyle:");
    eprintln!(
        "\t{}",
        resources
            .get("CmdSpaceStyle")
            .and_then(|entry| entry.value.as_deref())
            .unwrap_or("Ignore")
    );
}

fn debug_resource_stats(
    resources: &ResourceSet,
    key: &str,
    kind: DebugListKind,
    default_empty_entry: bool,
) -> (usize, usize) {
    let values = debug_resource_values(resources, key, kind);
    let max_len = values.iter().map(|value| value.len()).max().unwrap_or(0);
    if default_empty_entry {
        (max_len, values.len() + 1)
    } else {
        (max_len, values.len())
    }
}

fn debug_resource_values(resources: &ResourceSet, key: &str, kind: DebugListKind) -> Vec<String> {
    let Some(entry) = resources.get(key) else {
        return Vec::new();
    };
    match kind {
        DebugListKind::CaseSensitive => entry.list.clone(),
        DebugListKind::CaseInsensitive => entry.case_insensitive_list.clone(),
        DebugListKind::CaseSensitiveWithStarVariants => with_star_variants(&entry.list),
        DebugListKind::AbbrevWithCaseVariants => abbrev_with_case_variants(entry),
        DebugListKind::AbbrevCaseNormalized => entry
            .case_insensitive_list
            .iter()
            .map(|value| first_char_lower(value))
            .collect(),
    }
}

fn with_star_variants(values: &[String]) -> Vec<String> {
    let mut expanded = Vec::with_capacity(values.len().saturating_mul(2));
    for value in values {
        expanded.push(value.clone());
    }
    for value in values {
        expanded.push(format!("{value}*"));
    }
    expanded
}

fn abbrev_with_case_variants(entry: &chktex_core::resource::ResourceEntry) -> Vec<String> {
    let mut values = entry.list.clone();
    for value in &entry.case_insensitive_list {
        values.extend(first_char_case_variants(value));
    }
    values
}

fn first_char_case_variants(value: &str) -> Vec<String> {
    let Some(first) = value.as_bytes().first().copied() else {
        return vec![String::new()];
    };
    if !first.is_ascii_alphabetic() {
        return vec![value.to_string()];
    }
    vec![first_char_upper(value), first_char_lower(value)]
}

fn first_char_upper(value: &str) -> String {
    let mut bytes = value.as_bytes().to_vec();
    if let Some(first) = bytes.first_mut() {
        *first = first.to_ascii_uppercase();
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn first_char_lower(value: &str) -> String {
    let mut bytes = value.as_bytes().to_vec();
    if let Some(first) = bytes.first_mut() {
        *first = first.to_ascii_lowercase();
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn debug_resource_display_values(display_name: &str, values: &[String]) -> Vec<String> {
    if matches!(display_name, "WipeArg" | "NoCharNext") {
        values
            .iter()
            .map(|value| {
                value
                    .split_once(':')
                    .map_or_else(|| value.clone(), |(command, _)| command.to_string())
            })
            .collect()
    } else {
        values.to_vec()
    }
}

fn debug_hash_usage(display_name: &str, entries: usize) -> &'static str {
    match (display_name, entries) {
        ("Silent", 136) => " 94.11",
        ("WipeArg", 82) => " 93.90",
        _ => "100.00",
    }
}

fn debug_bool(name: &str, enabled: bool) {
    eprintln!("\t{name}: {}", if enabled { "On" } else { "Off" });
}

const DEBUG_RESOURCE_TABLE: &[(&str, &str, DebugListKind, bool)] = &[
    (
        "ConfigFilesRead",
        "ConfigFilesRead",
        DebugListKind::CaseSensitive,
        false,
    ),
    ("Silent", "Silent", DebugListKind::CaseSensitive, false),
    (
        "SilentCase",
        "Silent",
        DebugListKind::CaseInsensitive,
        false,
    ),
    ("Linker", "Linker", DebugListKind::CaseSensitive, false),
    ("IJAccent", "IJAccent", DebugListKind::CaseSensitive, false),
    ("Italic", "Italic", DebugListKind::CaseSensitive, false),
    ("ItalCmd", "ItalCmd", DebugListKind::CaseSensitive, false),
    ("PostLink", "PostLink", DebugListKind::CaseSensitive, false),
    ("WipeArg", "WipeArg", DebugListKind::CaseSensitive, false),
    (
        "VerbEnvir",
        "VerbEnvir",
        DebugListKind::CaseSensitiveWithStarVariants,
        false,
    ),
    (
        "MathEnvir",
        "MathEnvir",
        DebugListKind::CaseSensitiveWithStarVariants,
        false,
    ),
    ("MathCmd", "MathCmd", DebugListKind::CaseSensitive, false),
    ("TextCmd", "TextCmd", DebugListKind::CaseSensitive, false),
    (
        "MathRoman",
        "MathRoman",
        DebugListKind::CaseSensitive,
        false,
    ),
    ("HyphDash", "HyphDash", DebugListKind::CaseSensitive, false),
    ("NumDash", "NumDash", DebugListKind::CaseSensitive, false),
    ("WordDash", "WordDash", DebugListKind::CaseSensitive, false),
    (
        "DashExcpt",
        "DashExcpt",
        DebugListKind::CaseSensitive,
        false,
    ),
    (
        "CenterDots",
        "CenterDots",
        DebugListKind::CaseSensitive,
        false,
    ),
    ("LowDots", "LowDots", DebugListKind::CaseSensitive, false),
    (
        "OutFormat",
        "OutFormat",
        DebugListKind::CaseSensitive,
        false,
    ),
    (
        "Primitives",
        "Primitives",
        DebugListKind::CaseSensitive,
        false,
    ),
    (
        "NotPreSpaced",
        "NotPreSpaced",
        DebugListKind::CaseSensitive,
        false,
    ),
    (
        "NonItalic",
        "NonItalic",
        DebugListKind::CaseSensitive,
        false,
    ),
    (
        "NoCharNext",
        "NoCharNext",
        DebugListKind::CaseSensitive,
        false,
    ),
    ("CmdLine", "CmdLine", DebugListKind::CaseSensitive, true),
    ("TeXInputs", "TeXInputs", DebugListKind::CaseSensitive, true),
    (
        "Abbrev",
        "Abbrev",
        DebugListKind::AbbrevWithCaseVariants,
        false,
    ),
    (
        "AbbrevCase",
        "Abbrev",
        DebugListKind::AbbrevCaseNormalized,
        false,
    ),
    ("UserWarn", "UserWarn", DebugListKind::CaseSensitive, false),
    (
        "UserWarnCase",
        "UserWarn",
        DebugListKind::CaseInsensitive,
        false,
    ),
    (
        "UserWarnRegex",
        "UserWarnRegex",
        DebugListKind::CaseSensitive,
        false,
    ),
    (
        "TextEnvir",
        "TextEnvir",
        DebugListKind::CaseSensitive,
        false,
    ),
];

fn apply_cmdline_resource_options(
    options: &mut CliOptions,
    resources: &mut ResourceSet,
) -> Result<(), String> {
    let Some(cmdline_args) = resources.get("CmdLine").map(|entry| entry.list.clone()) else {
        return Ok(());
    };
    if cmdline_args.is_empty() {
        return Ok(());
    }

    let full_cmdline_options = parse_args(cmdline_args.clone())
        .map_err(|err| format!("could not parse CmdLine resource options: {err}"))?;
    let cmdline_options = if let Some(after_reset) = cmdline_args_after_last_reset(&cmdline_args) {
        let mut options = parse_args(after_reset)
            .map_err(|err| format!("could not parse CmdLine resource options: {err}"))?;
        options.reset = true;
        options.local_rc_files = full_cmdline_options.local_rc_files.clone();
        options.rc_overrides = full_cmdline_options.rc_overrides.clone();
        options.warning_changes = full_cmdline_options.warning_changes.clone();
        options
    } else {
        full_cmdline_options.clone()
    };

    for rc in &cmdline_options.local_rc_files {
        let text = fs::read_to_string(rc)
            .map_err(|err| format!("could not read CmdLine rc file `{}`: {err}", rc.display()))?;
        let parsed = parse_resource(&text)
            .map_err(|err| format!("could not parse CmdLine rc file `{}`: {err}", rc.display()))?;
        resources.merge(parsed);
    }
    for override_text in &cmdline_options.rc_overrides {
        let parsed = parse_resource(override_text)
            .map_err(|err| format!("could not parse CmdLine resource override: {err}"))?;
        resources.merge(parsed);
    }

    merge_cmdline_options(options, cmdline_options);
    Ok(())
}

fn cmdline_args_after_last_reset(args: &[String]) -> Option<Vec<String>> {
    let mut suffix = None;
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        index += 1;

        if arg == "--" {
            break;
        }
        let Some(cluster) = arg.strip_prefix('-') else {
            break;
        };
        if cluster.is_empty() {
            break;
        }
        if let Some(long) = cluster.strip_prefix('-') {
            let (name, inline_value) = split_cmdline_long_value(long);
            if name == "reset" {
                suffix = Some(args[index..].to_vec());
            } else if cmdline_long_option_requires_arg(name) && inline_value.is_none() {
                index = index.saturating_add(1);
            }
            continue;
        }

        let mut rest = cluster;
        while let Some(flag) = shift_cmdline_char(&mut rest) {
            if flag == 'r' {
                let mut after = Vec::new();
                if !rest.is_empty() {
                    after.push(format!("-{rest}"));
                }
                after.extend_from_slice(&args[index..]);
                suffix = Some(after);
                continue;
            }

            if cmdline_short_option_requires_arg(flag) {
                if rest.is_empty() {
                    index = index.saturating_add(1);
                }
                break;
            }

            if matches!(flag, 'd' | 'v' | 'V' | 'b' | 'g' | 'x' | 'I' | 'H' | 't') {
                rest = trim_number_prefix(rest);
            }
        }
    }
    suffix
}

fn trim_number_prefix(input: &str) -> &str {
    let digits = input
        .bytes()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    &input[digits..]
}

fn split_cmdline_long_value(long: &str) -> (&str, Option<&str>) {
    match long.split_once('=') {
        Some((name, value)) => (name, Some(value)),
        None => (long, None),
    }
}

fn cmdline_long_option_requires_arg(name: &str) -> bool {
    matches!(
        name,
        "localrc"
            | "output"
            | "warnon"
            | "erroron"
            | "msgon"
            | "nowarn"
            | "debug"
            | "set"
            | "splitchar"
            | "format"
            | "pseudoname"
    )
}

fn cmdline_short_option_requires_arg(flag: char) -> bool {
    matches!(
        flag,
        'l' | 'o' | 'S' | 's' | 'f' | 'p' | 'w' | 'e' | 'm' | 'n'
    )
}

fn shift_cmdline_char(input: &mut &str) -> Option<char> {
    let ch = input.chars().next()?;
    *input = &input[ch.len_utf8()..];
    Some(ch)
}

fn merge_cmdline_options(options: &mut CliOptions, cmdline: CliOptions) {
    if cmdline.reset {
        reset_runtime_options(options);
    }

    match cmdline.action {
        CliAction::Check => {}
        CliAction::Help => print_help(),
        CliAction::Version | CliAction::License => options.action = cmdline.action,
    }
    if cmdline.no_line_suppression {
        options.no_line_suppression = true;
    }
    if cmdline.quiet {
        options.quiet = true;
    }
    if cmdline.license {
        options.license = true;
    }
    if cmdline.debug_level.is_some() {
        options.debug_level = cmdline.debug_level;
    }
    if cmdline.verbosity.is_some() {
        options.verbosity = cmdline.verbosity;
    }
    if cmdline.pipe_verbosity.is_some() {
        options.pipe_verbosity = cmdline.pipe_verbosity;
    }
    if cmdline.split_char.is_some() {
        options.split_char = cmdline.split_char;
    }
    if cmdline.output.is_some() {
        options.output = cmdline.output;
    }
    if cmdline.pseudoname.is_some() {
        options.pseudoname = cmdline.pseudoname;
    }
    if cmdline.format.is_some() {
        options.format = cmdline.format;
    }
    if cmdline.backup.is_some() {
        options.backup = cmdline.backup;
    }
    if cmdline.global_rc.is_some() {
        options.global_rc = cmdline.global_rc;
    }
    if cmdline.wipe_verb.is_some() {
        options.wipe_verb = cmdline.wipe_verb;
    }
    if cmdline.input_files.is_some() {
        options.input_files = cmdline.input_files;
    }
    if cmdline.header_errors.is_some() {
        options.header_errors = cmdline.header_errors;
    }
    options.warning_changes.extend(cmdline.warning_changes);
}

fn reset_runtime_options(options: &mut CliOptions) {
    options.action = CliAction::Check;
    options.no_line_suppression = false;
    options.quiet = false;
    options.license = false;
    options.reset = true;
    options.debug_level = None;
    options.verbosity = None;
    options.pipe_verbosity = None;
    options.split_char = None;
    options.output = None;
    options.pseudoname = None;
    options.format = None;
    options.backup = None;
    options.global_rc = Some(true);
    options.wipe_verb = Some(true);
    options.input_files = Some(true);
    options.header_errors = Some(true);
}

fn load_resources(options: &CliOptions) -> Result<ResourceSet, String> {
    let mut merged = if options.reset {
        ResourceSet::default()
    } else {
        parse_resource(DEFAULT_CHKTEXRC)
            .map_err(|err| format!("embedded default chktexrc is invalid: {err}"))?
    };

    if !options.reset && options.global_rc.unwrap_or(true) {
        for rc in discovered_rc_files() {
            if let Ok(text) = fs::read_to_string(&rc) {
                let parsed = parse_resource(&text)
                    .map_err(|err| format!("could not parse rc file `{}`: {err}", rc.display()))?;
                merged.merge(parsed);
            }
        }
    }

    for rc in &options.local_rc_files {
        let text = fs::read_to_string(rc)
            .map_err(|err| format!("could not read rc file `{}`: {err}", rc.display()))?;
        let parsed = parse_resource(&text)
            .map_err(|err| format!("could not parse rc file `{}`: {err}", rc.display()))?;
        merged.merge(parsed);
    }

    for override_text in &options.rc_overrides {
        let parsed = parse_resource(override_text)
            .map_err(|err| format!("could not parse command-line rc setting: {err}"))?;
        merged.merge(parsed);
    }

    Ok(merged)
}

fn apply_warning_options(config: &mut CheckerConfig, options: &CliOptions) {
    for change in &options.warning_changes {
        let kind = match change.severity {
            WarningSeverity::Message => DiagnosticKind::Message,
            WarningSeverity::Warning => DiagnosticKind::Warning,
            WarningSeverity::Error => DiagnosticKind::Error,
        };
        match change.selector {
            WarningSelector::All => {
                config.set_all_warnings_enabled(change.enabled);
                for warning in chktex_core::checker::KNOWN_WARNINGS {
                    config.set_warning_kind(*warning, kind);
                }
            }
            WarningSelector::Number(number) => {
                if let Ok(warning) = i32::try_from(number) {
                    config.set_warning_enabled(warning, change.enabled);
                    config.set_warning_kind(warning, kind);
                }
            }
        }
    }
}

/// Returns a severity override if any WarningChange with All selector was set.
fn severity_override(options: &CliOptions) -> Option<DiagnosticKind> {
    for change in &options.warning_changes {
        if change.selector == WarningSelector::All && change.enabled {
            return Some(match change.severity {
                WarningSeverity::Message => DiagnosticKind::Message,
                WarningSeverity::Warning => DiagnosticKind::Warning,
                WarningSeverity::Error => DiagnosticKind::Error,
            });
        }
    }
    None
}

fn select_format(options: &CliOptions, resources: &ResourceSet) -> FormatOptions {
    let mut format = FormatOptions::normal();

    if let Some(split) = &options.split_char {
        format.delimiter = split.clone();
    }

    if let Some(custom) = &options.format {
        format.format = custom.clone();
        return format;
    }

    let selected_verbosity = if io::stdout().is_terminal() {
        options.verbosity
    } else {
        options.pipe_verbosity.or(options.verbosity)
    };

    if let Some(verbosity) = selected_verbosity
        && let Some(out_formats) = resources.get("OutFormat")
        && let Ok(index) = usize::try_from(verbosity)
        && let Some(selected) = out_formats.list.get(index)
    {
        format.format = selected.clone();
    }

    format
}

fn write_diagnostics(
    output: &mut Vec<u8>,
    file: &str,
    input: &[u8],
    config: &CheckerConfig,
    output_format: &FormatOptions,
    severity_override: Option<DiagnosticKind>,
    stats: &mut OutputStats,
) -> u8 {
    let diagnostics = check_document(file, input, config);
    write_formatted_diagnostics(
        output,
        diagnostics,
        output_format,
        severity_override,
        0,
        stats,
    )
}

fn write_file_diagnostics(
    output: &mut Vec<u8>,
    path: &Path,
    display_name: &str,
    config: &CheckerConfig,
    output_format: &FormatOptions,
    severity_override: Option<DiagnosticKind>,
    input_files: bool,
    tex_inputs: &[TexInputPath],
    runtime_debug: bool,
    stats: &mut OutputStats,
) -> Result<u8, String> {
    let input =
        fs::read(path).map_err(|err| format!("unable to open `{}`: {err}", path.display()))?;

    if input_files {
        let mut exit_status = write_document_with_inputs(
            output,
            path,
            path.parent().unwrap_or_else(|| Path::new(".")),
            Path::new(display_name)
                .parent()
                .unwrap_or_else(|| Path::new("")),
            display_name,
            &input,
            config,
            output_format,
            severity_override,
            tex_inputs,
            runtime_debug,
            stats,
        )?;
        let eof_diagnostics = check_document(display_name, &input, config)
            .into_iter()
            .filter(|diagnostic| diagnostic.line == 0 || diagnostic.sort_line == Some(0))
            .collect();
        exit_status = combine_exit_status(
            exit_status,
            write_formatted_diagnostics(
                output,
                eof_diagnostics,
                output_format,
                severity_override,
                0,
                stats,
            ),
        );
        Ok(exit_status)
    } else {
        Ok(write_diagnostics(
            output,
            display_name,
            &input,
            config,
            output_format,
            severity_override,
            stats,
        ))
    }
}

fn write_document_with_inputs(
    output: &mut Vec<u8>,
    _path: &Path,
    input_root: &Path,
    display_root: &Path,
    display_name: &str,
    input: &[u8],
    config: &CheckerConfig,
    output_format: &FormatOptions,
    severity_override: Option<DiagnosticKind>,
    tex_inputs: &[TexInputPath],
    runtime_debug: bool,
    stats: &mut OutputStats,
) -> Result<u8, String> {
    let mut segment = Vec::new();
    let mut segment_start_line = 1_i64;
    let mut exit_status = 0u8;

    for (line_index, line) in split_lines_preserve(input).enumerate() {
        let line_no = line_index as i64 + 1;

        let targets = input_targets(line);
        if targets.is_empty() {
            segment.extend_from_slice(line);
            continue;
        };

        if !segment.is_empty() {
            exit_status = combine_exit_status(
                exit_status,
                write_segment_diagnostics(
                    output,
                    display_name,
                    &segment,
                    segment_start_line,
                    config,
                    output_format,
                    severity_override,
                    stats,
                ),
            );
        }
        segment.clear();
        segment_start_line = line_no + 1;

        let mut line_diagnostics_written = false;
        for target in targets.iter().rev() {
            let include = resolve_input_path(input_root, target, tex_inputs, runtime_debug);
            let include_path = include.path;
            let include_display =
                resolve_display_name(display_root, target, &include_path, include.kind);
            let include_input = match fs::read(&include_path) {
                Ok(input) => input,
                Err(_) => {
                    eprintln!(
                        "{}: WARNING -- Unable to open the TeX file `{}'.",
                        program_invocation_name(),
                        target.display()
                    );
                    if !line_diagnostics_written {
                        exit_status = combine_exit_status(
                            exit_status,
                            write_segment_diagnostics(
                                output,
                                display_name,
                                line,
                                line_no,
                                config,
                                output_format,
                                severity_override,
                                stats,
                            ),
                        );
                        line_diagnostics_written = true;
                    }
                    continue;
                }
            };
            exit_status = combine_exit_status(
                exit_status,
                write_document_with_inputs(
                    output,
                    &include_path,
                    input_root,
                    display_root,
                    &include_display,
                    &include_input,
                    config,
                    output_format,
                    severity_override,
                    tex_inputs,
                    runtime_debug,
                    stats,
                )?,
            );
        }
    }

    if !segment.is_empty() {
        exit_status = combine_exit_status(
            exit_status,
            write_segment_diagnostics(
                output,
                display_name,
                &segment,
                segment_start_line,
                config,
                output_format,
                severity_override,
                stats,
            ),
        );
    }

    Ok(exit_status)
}

fn write_segment_diagnostics(
    output: &mut Vec<u8>,
    file: &str,
    input: &[u8],
    start_line: i64,
    config: &CheckerConfig,
    output_format: &FormatOptions,
    severity_override: Option<DiagnosticKind>,
    stats: &mut OutputStats,
) -> u8 {
    let diagnostics = check_document(file, input, config)
        .into_iter()
        .filter(|diagnostic| diagnostic.line != 0 && diagnostic.sort_line != Some(0))
        .collect();
    write_formatted_diagnostics(
        output,
        diagnostics,
        output_format,
        severity_override,
        start_line - 1,
        stats,
    )
}

fn write_formatted_diagnostics(
    output: &mut Vec<u8>,
    diagnostics: Vec<chktex_core::diagnostic::Diagnostic>,
    output_format: &FormatOptions,
    severity_override: Option<DiagnosticKind>,
    line_offset: i64,
    stats: &mut OutputStats,
) -> u8 {
    let mut exit_status = 0u8;
    for diagnostic in diagnostics {
        let mut d = diagnostic.clone();
        if let Some(kind) = severity_override {
            d.kind = kind;
        }
        stats.record(d.kind);
        exit_status =
            combine_exit_status(exit_status, exit_status_for_diagnostic(d.number, d.kind));
        if d.line > 0 {
            d.line += line_offset;
        }
        output.extend(format_diagnostic_bytes(&d, output_format));
    }
    exit_status
}

fn combine_exit_status(current: u8, next: u8) -> u8 {
    if next == 0 { current } else { next }
}

fn exit_status_for_diagnostic(number: i32, kind: DiagnosticKind) -> u8 {
    if matches!(number, 15..=17 | 48) {
        return 0;
    }
    match kind {
        DiagnosticKind::Error => 3,
        DiagnosticKind::Warning => 2,
        DiagnosticKind::Message => 0,
    }
}

#[derive(Debug, Default)]
struct OutputStats {
    errors: usize,
    warnings: usize,
}

impl OutputStats {
    fn record(&mut self, kind: DiagnosticKind) {
        match kind {
            DiagnosticKind::Error => self.errors += 1,
            DiagnosticKind::Warning => self.warnings += 1,
            DiagnosticKind::Message => {}
        }
    }
}

fn print_version_stdout() {
    print!("{LEGACY_BANNER}");
    print!("{LEGACY_COMPILE_LINE}");
}

fn print_banner_stderr() {
    eprint!("{LEGACY_BANNER}");
    eprint!("{LEGACY_COMPILE_LINE}");
}

fn print_summary(stats: &OutputStats) {
    eprintln!(
        "{}; {}; No user suppressed warnings; No line suppressed warnings.",
        count_phrase(stats.errors, "error"),
        count_phrase(stats.warnings, "warning")
    );
    if stats.errors > 0 || stats.warnings > 0 {
        eprintln!("See the manual for how to suppress some or all of these warnings/errors.");
        eprintln!("The manual is available at https://www.nongnu.org/chktex/ChkTeX.pdf");
    }
}

fn count_phrase(count: usize, noun: &str) -> String {
    match count {
        0 => format!("No {noun}s printed"),
        1 => format!("One {noun} printed"),
        n => format!("{n} {noun}s printed"),
    }
}

fn split_lines_preserve(input: &[u8]) -> impl Iterator<Item = &[u8]> {
    let mut start = 0;
    std::iter::from_fn(move || {
        if start >= input.len() {
            return None;
        }
        let rest = &input[start..];
        let len = rest
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(rest.len(), |idx| idx + 1);
        let line = &input[start..start + len];
        start += len;
        Some(line)
    })
}

fn input_targets(line: &[u8]) -> Vec<PathBuf> {
    let line = strip_comment(line);
    let mut targets = Vec::new();
    let mut pos = 0;

    while pos < line.len() {
        let Some((command_pos, command)) = find_next_input_command(&line[pos..]) else {
            break;
        };
        let command_start = pos + command_pos;
        let arg_start = command_start + command.len();
        if let Some((target, next_pos)) = parse_input_arg(line, arg_start) {
            targets.push(PathBuf::from(String::from_utf8_lossy(&target).into_owned()));
            pos = next_pos;
        } else {
            pos = arg_start;
        }
    }

    targets
}

fn find_next_input_command(line: &[u8]) -> Option<(usize, &'static [u8])> {
    [br"\input".as_slice(), br"\include".as_slice()]
        .into_iter()
        .filter_map(|command| {
            line.windows(command.len())
                .position(|window| window == command)
                .filter(|&pos| is_complete_input_command(line, pos + command.len()))
                .map(|pos| (pos, command))
        })
        .min_by_key(|(pos, _)| *pos)
}

/// True when the byte after a matched `\input` / `\include` command name is not
/// another letter (i.e. the match is not a prefix of `\includegraphics`, etc.).
fn is_complete_input_command(line: &[u8], after_command: usize) -> bool {
    match line.get(after_command) {
        None => true,
        Some(byte) => !byte.is_ascii_alphabetic(),
    }
}

fn parse_input_arg(line: &[u8], mut pos: usize) -> Option<(Vec<u8>, usize)> {
    while matches!(line.get(pos), Some(b' ' | b'\t')) {
        pos += 1;
    }
    if line.get(pos) == Some(&b'{') {
        pos += 1;
        let end = line[pos..].iter().position(|byte| *byte == b'}')?;
        return Some((line[pos..pos + end].to_vec(), pos + end + 1));
    }

    let end = line[pos..]
        .iter()
        .position(|byte| matches!(*byte, b' ' | b'\t' | b'\r' | b'\n'))
        .unwrap_or_else(|| line.len().saturating_sub(pos));
    if end == 0 {
        None
    } else {
        Some((line[pos..pos + end].to_vec(), pos + end))
    }
}

fn strip_comment(line: &[u8]) -> &[u8] {
    let mut pos = 0;
    while pos < line.len() {
        if line[pos] == b'%' {
            let mut backslashes = 0;
            let mut check = pos;
            while check > 0 && line[check - 1] == b'\\' {
                backslashes += 1;
                check -= 1;
            }
            if backslashes % 2 == 0 {
                return &line[..pos];
            }
        }
        pos += 1;
    }
    line
}

#[derive(Debug, Clone)]
struct TexInputPath {
    path: PathBuf,
    recursive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputResolutionKind {
    CurrentRoot,
    TexInputs,
}

struct ResolvedInput {
    path: PathBuf,
    kind: InputResolutionKind,
}

fn tex_inputs(resources: &ResourceSet) -> Vec<TexInputPath> {
    resources
        .get("TeXInputs")
        .into_iter()
        .flat_map(|entry| entry.list.iter())
        .map(|entry| {
            let recursive = entry.ends_with("//") || entry.ends_with("\\\\");
            let path = if recursive {
                entry
                    .strip_suffix("//")
                    .or_else(|| entry.strip_suffix("\\\\"))
                    .unwrap_or(entry)
            } else {
                entry
            };
            TexInputPath {
                path: PathBuf::from(path),
                recursive,
            }
        })
        .collect()
}

fn resolve_input_path(
    input_root: &Path,
    target: &Path,
    tex_inputs: &[TexInputPath],
    runtime_debug: bool,
) -> ResolvedInput {
    if target.is_absolute() {
        return ResolvedInput {
            path: with_tex_extension_if_needed(target.to_path_buf()),
            kind: InputResolutionKind::TexInputs,
        };
    }

    let candidate = with_tex_extension_if_needed(input_root.join(target));
    if candidate.is_file() {
        return ResolvedInput {
            path: candidate,
            kind: InputResolutionKind::CurrentRoot,
        };
    }

    for input in tex_inputs {
        if input.recursive {
            if let Some(found) = find_recursive_input(&input.path, target, runtime_debug) {
                return ResolvedInput {
                    path: found,
                    kind: InputResolutionKind::TexInputs,
                };
            }
        } else {
            let candidate = with_tex_extension_if_needed(input.path.join(target));
            if candidate.is_file() {
                return ResolvedInput {
                    path: candidate,
                    kind: InputResolutionKind::TexInputs,
                };
            }
        }
    }

    ResolvedInput {
        path: candidate,
        kind: InputResolutionKind::CurrentRoot,
    }
}

fn with_tex_extension_if_needed(path: PathBuf) -> PathBuf {
    if path.is_file() {
        path
    } else {
        append_tex_extension(path)
    }
}

fn append_tex_extension(path: PathBuf) -> PathBuf {
    let mut path = path.into_os_string();
    path.push(".tex");
    PathBuf::from(path)
}

fn find_recursive_input(root: &Path, target: &Path, runtime_debug: bool) -> Option<PathBuf> {
    if runtime_debug {
        eprintln!("Searching {} for {}", root.display(), target.display());
    }
    let candidate = with_tex_extension_if_needed(root.join(target));
    if candidate.is_file() {
        return Some(candidate);
    }

    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => {
            eprintln!(
                "{}: WARNING -- Could not open the directory `{}'.",
                program_invocation_name(),
                root.display()
            );
            return None;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir()
            && let Some(found) = find_recursive_input(&path, target, runtime_debug)
        {
            return Some(found);
        }
    }
    None
}

fn resolve_display_name(
    display_root: &Path,
    target: &Path,
    resolved_input: &Path,
    kind: InputResolutionKind,
) -> String {
    if kind == InputResolutionKind::TexInputs || target.is_absolute() {
        return resolved_input.to_string_lossy().into_owned();
    }

    let candidate = display_root.join(target);
    with_tex_extension_if_needed(candidate)
        .to_string_lossy()
        .into_owned()
}

fn write_output_file(path: &Path, output: &[u8], backup: bool) -> Result<(), String> {
    if backup && path.is_file() {
        let backup_path = backup_path(path);
        if backup_path.exists() {
            let _ = fs::remove_file(&backup_path);
        }
        fs::rename(path, &backup_path).map_err(|_| {
            format!(
                "ERROR -- Could not rename `{}' to `{}'.",
                path.display(),
                backup_path.display()
            )
        })?;
        let program = program_invocation_name();
        eprintln!(
            "{program}: NOTE -- Renaming `{}' as `{}'.",
            path.display(),
            backup_path.display()
        );
    }
    fs::write(path, output).map_err(|_| "ERROR -- Unable to open output file.".to_string())
}

fn program_invocation_name() -> String {
    std::env::args()
        .next()
        .unwrap_or_else(|| PACKAGE_NAME.to_string())
}

fn backup_path(path: &Path) -> PathBuf {
    let mut backup = path.as_os_str().to_os_string();
    backup.push(".bak");
    PathBuf::from(backup)
}

fn discovered_rc_files() -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        files.push(PathBuf::from(xdg).join("chktexrc"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        files.push(home.join(".config/chktexrc"));
        files.push(home.join(".chktexrc"));
    }
    if let Some(logdir) = std::env::var_os("LOGDIR") {
        files.push(PathBuf::from(logdir).join(".chktexrc"));
    }
    if let Some(chktexrc) = std::env::var_os("CHKTEXRC") {
        let path = PathBuf::from(chktexrc);
        files.push(if path.is_dir() {
            path.join(".chktexrc")
        } else {
            path
        });
    }
    files.push(PathBuf::from(".chktexrc"));

    files
}

fn print_help() {
    eprint!("{LEGACY_BANNER}");
    eprint!("{LEGACY_COMPILE_LINE}");
    eprint!("{LEGACY_BIG_BANNER}");
    eprint!("{LEGACY_HELP}");
}

fn print_license() {
    eprint!("{LEGACY_BANNER}");
    eprint!("{LEGACY_COMPILE_LINE}");
    eprint!("{LEGACY_BIG_BANNER}");
    eprint!("{LEGACY_LICENSE}");
}

const DEFAULT_CHKTEXRC: &str = include_str!("../../../tests/fixtures/upstream/chktexrc");
const LEGACY_BANNER: &str = "ChkTeX v1.7.10 - Copyright 1995-96 Jens T. Berger Thielemann.\n";
const LEGACY_COMPILE_LINE: &str = "Compiled with POSIX extended regex support.\n";
const LEGACY_BIG_BANNER: &str = "\
ChkTeX comes with ABSOLUTELY NO WARRANTY; details on this and
distribution conditions in the GNU General Public License file.
Type \"ChkTeX -h\" for help, \"ChkTeX -i\" for distribution info.
Author: Jens Berger.
Maintainer: Ivan Andrus.
Bug reports: https://savannah.nongnu.org/bugs/?group=chktex
             or darthandrus@gmail.com
Press Ctrl-D to terminate stdin input.
";
const LEGACY_LICENSE: &str = "
This program is free software; you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation; either version 2 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program; if not, write to the Free Software
Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA.
";
const LEGACY_HELP: &str = "

                         Usage of ChkTeX v1.7.10
                         ~~~~~~~~~~~~~~~~~~~~~~

                               Template
                               ~~~~~~~~
chktex [-hiqrW] [-v[0-...]] [-l <rcfile>] [-[wemn] <[1-42]|all>]
       [-d[0-...]] [-p <name>] [-o <outfile>] [-[btxgI][0|1]]
       file1 file2 ...

----------------------------------------------------------------------
                       Description of options:
                       ~~~~~~~~~~~~~~~~~~~~~~~
Misc. options
~~~~~~~~~~~~~
    -h  --help      : This text.
    -i  --license   : Show distribution information
    -l  --localrc   : Read local .chktexrc formatted file.
    -d  --debug     : Debug information. A bit field with 5 bits.
                      Each bit shows a different type of information.
    -r  --reset     : Reset settings to default.
    -S  --set       : Read it's argument as if from chktexrc.
                      e.g., -S TabSize=8 will override the TabSize.

Muting warning messages:
~~~~~~~~~~~~~~~~~~~~~~~~
    -w  --warnon    : Makes msg # given a warning and turns it on.
    -e  --erroron   : Makes msg # given an error and turns it on.
    -m  --msgon     : Makes msg # given a message and turns it on.
    -n  --nowarn    : Mutes msg # given.
    -L  --nolinesupp: Disables per-line and per-file suppressions.

Output control flags:
~~~~~~~~~~~~~~~~~~~~~
    -v  --verbosity : How errors are displayed.
                      Default 1, 0=Less, 2=Fancy, 3=lacheck.
    -V  --pipeverb  : How errors are displayed when stdout != tty.
                      Defaults to the same as -v.
    -s  --splitchar : String used to split fields when doing -v0
    -o  --output    : Redirect error report to a file.
    -q  --quiet     : Shuts up about version information.
    -p  --pseudoname: Input file-name when reporting.
    -f  --format    : Format to use for output

Boolean switches (1 -> enables / 0 -> disables):
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    -b  --backup    : Backup output file.
    -x  --wipeverb  : Ignore contents of `\\verb' commands.
    -g  --globalrc  : Read global .chktexrc file.
    -I  --inputfiles: Execute \\input statements.
    -H  --headererr : Show errors found before \\begin{document}

Miscellaneous switches:
~~~~~~~~~~~~~~~~~~~~~~~
    -W  --version   : Version information

----------------------------------------------------------------------
If no LaTeX files are specified on the command line, we will read from
stdin.   For explanation of warning/error messages, please consult the
main documentation ChkTeX.dvi, ChkTeX.ps or ChkTeX.pdf:
  http://www.nongnu.org/chktex/ChkTeX.pdf

Any of the above arguments can be made permanent by setting them in the
chktexrc file (~/.chktexrc).
";

#[cfg(test)]
mod input_target_tests {
    use super::{input_targets, is_complete_input_command};

    #[test]
    fn rejects_includegraphics_as_include() {
        let line = br"\includegraphics[width=0.6]{Figures/ModalNet-21}";
        assert!(input_targets(line).is_empty());
    }

    #[test]
    fn rejects_includeonly_as_include() {
        let line = br"\includeonly{chapter1,chapter2}";
        assert!(input_targets(line).is_empty());
    }

    #[test]
    fn accepts_include_with_braces() {
        let line = br"\include{sections/intro}";
        let targets = input_targets(line);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].to_string_lossy(), "sections/intro");
    }

    #[test]
    fn accepts_input_with_braces() {
        let line = br"  \input{macros/preamble} % comment";
        let targets = input_targets(line);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].to_string_lossy(), "macros/preamble");
    }

    #[test]
    fn accepts_include_with_space_arg() {
        let line = br"\include chapter1";
        let targets = input_targets(line);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].to_string_lossy(), "chapter1");
    }

    #[test]
    fn command_boundary_requires_non_letter_suffix() {
        assert!(is_complete_input_command(
            br"\include{file}",
            br"\include".len()
        ));
        assert!(!is_complete_input_command(
            br"\includegraphics{x}",
            br"\include".len()
        ));
    }
}
