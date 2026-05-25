use std::{
    fs,
    io::{self, Read},
    process::ExitCode,
};

use chktex_core::{
    PACKAGE_NAME, PACKAGE_VERSION,
    checker::{
        CheckerConfig, WARNING_COMMAND_TERMINATED_WITH_SPACE, WARNING_USER_PATTERN,
        WARNING_USER_REGEX, check_document,
    },
    cli::{CliAction, CliOptions, WarningSelector, parse_args},
    diagnostic::{FormatOptions, format_diagnostic},
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
            println!("{PACKAGE_NAME} v{PACKAGE_VERSION} - Rust rewrite prototype.");
            ExitCode::SUCCESS
        }
        CliAction::Help => {
            print_help();
            ExitCode::SUCCESS
        }
        CliAction::Check => run_check(&options),
    }
}

fn run_check(options: &CliOptions) -> ExitCode {
    let resources = match load_resources(options) {
        Ok(resources) => resources,
        Err(err) => {
            eprintln!("{PACKAGE_NAME}: {err}");
            return ExitCode::from(1);
        }
    };

    let mut config = CheckerConfig::from_resources(&resources);
    apply_warning_options(&mut config, options);

    let output_format = select_format(options, &resources);
    let mut diagnostics_found = false;

    if options.files.is_empty() {
        let mut input = Vec::new();
        if let Err(err) = io::stdin().read_to_end(&mut input) {
            eprintln!("{PACKAGE_NAME}: failed to read stdin: {err}");
            return ExitCode::from(1);
        }
        diagnostics_found |= print_diagnostics("stdin", &input, &config, &output_format);
    } else {
        for file in &options.files {
            let input = match fs::read(file) {
                Ok(input) => input,
                Err(err) => {
                    eprintln!("{PACKAGE_NAME}: unable to open `{}`: {err}", file.display());
                    return ExitCode::from(1);
                }
            };
            diagnostics_found |=
                print_diagnostics(&file.to_string_lossy(), &input, &config, &output_format);
        }
    }

    if diagnostics_found {
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    }
}

fn load_resources(options: &CliOptions) -> Result<ResourceSet, String> {
    let mut merged = parse_resource(DEFAULT_CHKTEXRC)
        .map_err(|err| format!("embedded default chktexrc is invalid: {err}"))?;

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
        if warning_selector_matches(&change.selector, WARNING_COMMAND_TERMINATED_WITH_SPACE) {
            config.warning_1_enabled = change.enabled;
        }
        if warning_selector_matches(&change.selector, WARNING_USER_PATTERN) {
            config.user_warn_enabled = change.enabled;
        }
        if warning_selector_matches(&change.selector, WARNING_USER_REGEX) {
            config.user_warn_regex_enabled = change.enabled;
        }
    }
}

fn warning_selector_matches(selector: &WarningSelector, warning: i32) -> bool {
    matches!(selector, WarningSelector::All)
        || matches!(selector, WarningSelector::Number(number) if *number == i64::from(warning))
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

    if let Some(verbosity) = options.verbosity
        && let Some(out_formats) = resources.get("OutFormat")
        && let Ok(index) = usize::try_from(verbosity)
        && let Some(selected) = out_formats.list.get(index)
    {
        format.format = selected.clone();
    }

    format
}

fn print_diagnostics(
    file: &str,
    input: &[u8],
    config: &CheckerConfig,
    output_format: &FormatOptions,
) -> bool {
    let diagnostics = check_document(file, input, config);
    for diagnostic in &diagnostics {
        print!("{}", format_diagnostic(diagnostic, output_format));
    }
    !diagnostics.is_empty()
}

fn print_help() {
    println!(
        "\
Usage: chktex [OPTIONS] [FILE]...

This Rust rewrite is currently a compatibility-focused prototype.
"
    );
}

const DEFAULT_CHKTEXRC: &str = include_str!("../../../tests/fixtures/upstream/chktexrc");
