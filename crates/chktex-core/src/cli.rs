use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliOptions {
    pub action: CliAction,
    pub files: Vec<PathBuf>,
    pub local_rc_files: Vec<PathBuf>,
    pub rc_overrides: Vec<String>,
    pub warning_changes: Vec<WarningChange>,
    pub no_line_suppression: bool,
    pub quiet: bool,
    pub license: bool,
    pub reset: bool,
    pub debug_level: Option<i64>,
    pub verbosity: Option<i64>,
    pub pipe_verbosity: Option<i64>,
    pub split_char: Option<String>,
    pub output: Option<PathBuf>,
    pub pseudoname: Option<String>,
    pub format: Option<String>,
    pub backup: Option<bool>,
    pub global_rc: Option<bool>,
    pub wipe_verb: Option<bool>,
    pub input_files: Option<bool>,
    pub header_errors: Option<bool>,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            action: CliAction::Check,
            files: Vec::new(),
            local_rc_files: Vec::new(),
            rc_overrides: Vec::new(),
            warning_changes: Vec::new(),
            no_line_suppression: false,
            quiet: false,
            license: false,
            reset: false,
            debug_level: None,
            verbosity: None,
            pipe_verbosity: None,
            split_char: None,
            output: None,
            pseudoname: None,
            format: None,
            backup: None,
            global_rc: None,
            wipe_verb: None,
            input_files: None,
            header_errors: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliAction {
    Check,
    Help,
    License,
    Version,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarningChange {
    pub selector: WarningSelector,
    pub severity: WarningSeverity,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WarningSelector {
    All,
    Number(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningSeverity {
    Message,
    Warning,
    Error,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CliError {
    #[error("unknown option: {0}")]
    UnknownOption(String),
    #[error("missing argument for option: {0}")]
    MissingArgument(String),
    #[error("invalid warning selector: {0}")]
    InvalidWarningSelector(String),
    #[error("output file specified more than once")]
    OutputSpecifiedTwice,
    #[error("invalid long option syntax: {0}")]
    InvalidLongOption(String),
}

pub fn parse_args<I, S>(args: I) -> Result<CliOptions, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut parser = Parser::new(args.into_iter().map(Into::into).collect());
    parser.parse()
}

struct Parser {
    args: Vec<String>,
    index: usize,
    options: CliOptions,
}

impl Parser {
    fn new(args: Vec<String>) -> Self {
        Self {
            args,
            index: 0,
            options: CliOptions::default(),
        }
    }

    fn parse(&mut self) -> Result<CliOptions, CliError> {
        while self.index < self.args.len() {
            let arg = self.args[self.index].clone();
            self.index += 1;

            if arg == "?" {
                self.options.action = CliAction::Help;
                break;
            }

            if arg == "--" {
                self.options
                    .files
                    .extend(self.args[self.index..].iter().map(PathBuf::from));
                break;
            }

            if let Some(long) = arg.strip_prefix("--") {
                self.parse_long(long)?;
            } else if arg.starts_with('-') && arg.len() > 1 {
                self.parse_short_cluster(&arg[1..])?;
            } else {
                self.options.files.push(PathBuf::from(arg));
                self.options
                    .files
                    .extend(self.args[self.index..].iter().map(PathBuf::from));
                break;
            }
        }

        Ok(std::mem::take(&mut self.options))
    }

    fn parse_long(&mut self, long: &str) -> Result<(), CliError> {
        let (name, inline_value) = split_long_value(long);
        match name {
            "help" => self.options.action = CliAction::Help,
            "version" => self.options.action = CliAction::Version,
            "license" => self.options.action = CliAction::License,
            "nolinesupp" => self.options.no_line_suppression = true,
            "quiet" => self.options.quiet = true,
            "reset" => self.reset_runtime_options(),
            "localrc" => {
                let value = self.required_long_arg(name, inline_value)?;
                self.options.local_rc_files.push(PathBuf::from(value));
            }
            "output" => {
                let value = self.required_long_arg(name, inline_value)?;
                self.set_output(value)?;
            }
            "warnon" => {
                let value = self.required_long_arg(name, inline_value)?;
                self.push_warning(value, WarningSeverity::Warning, true)?;
            }
            "erroron" => {
                let value = self.required_long_arg(name, inline_value)?;
                self.push_warning(value, WarningSeverity::Error, true)?;
            }
            "msgon" => {
                let value = self.required_long_arg(name, inline_value)?;
                self.push_warning(value, WarningSeverity::Message, true)?;
            }
            "nowarn" => {
                let value = self.required_long_arg(name, inline_value)?;
                self.push_warning(value, WarningSeverity::Message, false)?;
            }
            "verbosity" => {
                self.options.verbosity = Some(parse_optional_number(inline_value, 2));
            }
            "pipeverb" => {
                self.options.pipe_verbosity = Some(parse_optional_number(inline_value, 1));
            }
            "debug" => {
                let value = self.required_long_arg(name, inline_value)?;
                self.options.debug_level = Some(parse_number_prefix(&value).0.unwrap_or(0));
            }
            "set" => {
                let value = self.required_long_arg(name, inline_value)?;
                self.options.rc_overrides.push(value);
            }
            "splitchar" => {
                self.options.split_char = Some(self.required_long_arg(name, inline_value)?);
            }
            "format" => {
                self.options.format = Some(self.required_long_arg(name, inline_value)?);
            }
            "pseudoname" => {
                self.options.pseudoname = Some(self.required_long_arg(name, inline_value)?);
            }
            "inputfiles" => {
                self.options.input_files = Some(parse_optional_bool(inline_value, false));
            }
            "backup" => {
                self.options.backup = Some(parse_optional_bool(inline_value, false));
            }
            "globalrc" => {
                self.options.global_rc = Some(parse_optional_bool(inline_value, false));
            }
            "wipeverb" => {
                self.options.wipe_verb = Some(parse_optional_bool(inline_value, false));
            }
            "tictoc" => {}
            "headererr" => {
                self.options.header_errors = Some(parse_optional_bool(inline_value, false));
            }
            _ => return Err(CliError::UnknownOption(format!("--{name}"))),
        }
        Ok(())
    }

    fn parse_short_cluster(&mut self, cluster: &str) -> Result<(), CliError> {
        let mut rest = cluster;
        while let Some(flag) = shift_char(&mut rest) {
            match flag {
                'h' => self.options.action = CliAction::Help,
                'W' => self.options.action = CliAction::Version,
                'i' => self.options.action = CliAction::License,
                'L' => self.options.no_line_suppression = true,
                'q' => self.options.quiet = true,
                'r' => self.reset_runtime_options(),
                'l' => {
                    let value = self.required_short_arg('l', rest)?;
                    rest = "";
                    self.options.local_rc_files.push(PathBuf::from(value));
                }
                'o' => {
                    let value = self.required_short_arg('o', rest)?;
                    rest = "";
                    self.set_output(value)?;
                }
                'S' => {
                    let value = self.required_short_arg('S', rest)?;
                    rest = "";
                    self.options.rc_overrides.push(value);
                }
                's' => {
                    let value = self.required_short_arg('s', rest)?;
                    rest = "";
                    self.options.split_char = Some(value);
                }
                'f' => {
                    let value = self.required_short_arg('f', rest)?;
                    rest = "";
                    self.options.format = Some(value);
                }
                'p' => {
                    let value = self.required_short_arg('p', rest)?;
                    rest = "";
                    self.options.pseudoname = Some(value);
                }
                'w' => {
                    let value = self.required_short_arg('w', rest)?;
                    self.push_warning(value.clone(), WarningSeverity::Warning, true)?;
                    rest = consume_numeric_or_all(rest);
                }
                'e' => {
                    let value = self.required_short_arg('e', rest)?;
                    self.push_warning(value.clone(), WarningSeverity::Error, true)?;
                    rest = consume_numeric_or_all(rest);
                }
                'm' => {
                    let value = self.required_short_arg('m', rest)?;
                    self.push_warning(value.clone(), WarningSeverity::Message, true)?;
                    rest = consume_numeric_or_all(rest);
                }
                'n' => {
                    let value = self.required_short_arg('n', rest)?;
                    self.push_warning(value.clone(), WarningSeverity::Message, false)?;
                    rest = consume_numeric_or_all(rest);
                }
                'd' => {
                    let (value, next) = self.optional_short_number(rest);
                    self.options.debug_level = Some(value.unwrap_or(0xffff));
                    rest = next;
                }
                'v' => {
                    let (value, next) = parse_number_prefix(rest);
                    self.options.verbosity = Some(value.unwrap_or(2));
                    rest = next;
                }
                'V' => {
                    let (value, next) = parse_number_prefix(rest);
                    self.options.pipe_verbosity = Some(value.unwrap_or(1));
                    rest = next;
                }
                'b' => {
                    let (value, next) = self.optional_short_bool(rest, false);
                    self.options.backup = Some(value);
                    rest = next;
                }
                'g' => {
                    let (value, next) = self.optional_short_bool(rest, false);
                    self.options.global_rc = Some(value);
                    rest = next;
                }
                'x' => {
                    let (value, next) = self.optional_short_bool(rest, false);
                    self.options.wipe_verb = Some(value);
                    rest = next;
                }
                'I' => {
                    let (value, next) = self.optional_short_bool(rest, false);
                    self.options.input_files = Some(value);
                    rest = next;
                }
                'H' => {
                    let (value, next) = self.optional_short_bool(rest, false);
                    self.options.header_errors = Some(value);
                    rest = next;
                }
                't' => {
                    let (_value, next) = self.optional_short_bool(rest, false);
                    rest = next;
                }
                _ => return Err(CliError::UnknownOption(format!("-{flag}"))),
            }
        }
        Ok(())
    }

    fn required_short_arg(&mut self, flag: char, rest: &str) -> Result<String, CliError> {
        if !rest.is_empty() {
            return Ok(rest.to_string());
        }
        self.args
            .get(self.index)
            .cloned()
            .inspect(|_| self.index += 1)
            .ok_or_else(|| CliError::MissingArgument(format!("-{flag}")))
    }

    fn optional_short_number<'a>(&mut self, rest: &'a str) -> (Option<i64>, &'a str) {
        let (value, next) = parse_number_prefix(rest);
        if value.is_some() || !next.is_empty() {
            return (value, next);
        }
        if let Some(arg) = self.args.get(self.index)
            && !arg.is_empty()
            && arg.bytes().all(|byte| byte.is_ascii_digit())
        {
            self.index += 1;
            return (arg.parse().ok(), "");
        }
        (None, rest)
    }

    fn optional_short_bool<'a>(&mut self, rest: &'a str, current: bool) -> (bool, &'a str) {
        let (value, next) = parse_bool_prefix(rest, current);
        if !next.is_empty() || parse_number_prefix(rest).0.is_some() {
            return (value, next);
        }
        if let Some(arg) = self.args.get(self.index)
            && matches!(arg.as_str(), "0" | "1")
        {
            self.index += 1;
            return (arg != "0", "");
        }
        (value, next)
    }

    fn required_long_arg(
        &mut self,
        name: &str,
        inline_value: Option<&str>,
    ) -> Result<String, CliError> {
        if let Some(value) = inline_value {
            return Ok(value.to_string());
        }
        self.args
            .get(self.index)
            .cloned()
            .inspect(|_| self.index += 1)
            .ok_or_else(|| CliError::MissingArgument(format!("--{name}")))
    }

    fn push_warning(
        &mut self,
        value: String,
        severity: WarningSeverity,
        enabled: bool,
    ) -> Result<(), CliError> {
        let selector = parse_warning_selector(&value)?;
        self.options.warning_changes.push(WarningChange {
            selector,
            severity,
            enabled,
        });
        Ok(())
    }

    fn set_output(&mut self, value: String) -> Result<(), CliError> {
        if self.options.output.is_some() {
            return Err(CliError::OutputSpecifiedTwice);
        }
        self.options.output = Some(PathBuf::from(value));
        Ok(())
    }

    fn reset_runtime_options(&mut self) {
        self.options.action = CliAction::Check;
        self.options.no_line_suppression = false;
        self.options.quiet = false;
        self.options.license = false;
        self.options.reset = true;
        self.options.debug_level = None;
        self.options.verbosity = None;
        self.options.pipe_verbosity = None;
        self.options.split_char = None;
        self.options.output = None;
        self.options.pseudoname = None;
        self.options.format = None;
        self.options.backup = Some(true);
        self.options.global_rc = Some(true);
        self.options.wipe_verb = Some(true);
        self.options.input_files = Some(true);
        self.options.header_errors = Some(true);
    }
}

fn split_long_value(long: &str) -> (&str, Option<&str>) {
    match long.split_once('=') {
        Some((name, value)) => (name, Some(value)),
        None => (long, None),
    }
}

fn shift_char(input: &mut &str) -> Option<char> {
    let ch = input.chars().next()?;
    *input = &input[ch.len_utf8()..];
    Some(ch)
}

fn parse_number_prefix(input: &str) -> (Option<i64>, &str) {
    let digits = input
        .bytes()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digits == 0 {
        return (None, input);
    }
    let (num, rest) = input.split_at(digits);
    (num.parse().ok(), rest)
}

fn parse_bool_prefix(input: &str, current: bool) -> (bool, &str) {
    let (number, rest) = parse_number_prefix(input);
    match number {
        Some(value) => (value != 0, rest),
        None => (!current, input),
    }
}

fn parse_optional_number(value: Option<&str>, default: i64) -> i64 {
    value
        .and_then(|value| parse_number_prefix(value).0)
        .unwrap_or(default)
}

fn parse_optional_bool(value: Option<&str>, current: bool) -> bool {
    value
        .and_then(|value| parse_number_prefix(value).0)
        .map(|value| value != 0)
        .unwrap_or(!current)
}

fn consume_numeric_or_all(input: &str) -> &str {
    if input.eq_ignore_ascii_case("all") {
        ""
    } else {
        parse_number_prefix(input).1
    }
}

fn parse_warning_selector(value: &str) -> Result<WarningSelector, CliError> {
    if value.eq_ignore_ascii_case("all") {
        return Ok(WarningSelector::All);
    }

    let (number, _) = parse_number_prefix(value);
    number
        .map(WarningSelector::Number)
        .ok_or_else(|| CliError::InvalidWarningSelector(value.to_string()))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{CliAction, CliError, WarningChange, WarningSelector, WarningSeverity, parse_args};

    #[test]
    fn parses_basic_files_after_options() {
        let parsed = parse_args(["-q", "one.tex", "two.tex"]).unwrap();

        assert!(parsed.quiet);
        assert_eq!(
            parsed.files,
            [PathBuf::from("one.tex"), PathBuf::from("two.tex")]
        );
    }

    #[test]
    fn parses_upstream_compact_test_options() {
        let parsed =
            parse_args(["-mall", "-r", "-g0", "-lchktexrc", "-v5", "-q", "Test.tex"]).unwrap();

        assert_eq!(
            parsed.warning_changes,
            [WarningChange {
                selector: WarningSelector::All,
                severity: WarningSeverity::Message,
                enabled: true,
            }]
        );
        assert!(parsed.reset);
        assert_eq!(parsed.global_rc, Some(false));
        assert_eq!(parsed.local_rc_files, [PathBuf::from("chktexrc")]);
        assert_eq!(parsed.verbosity, Some(5));
        assert!(parsed.quiet);
        assert_eq!(parsed.files, [PathBuf::from("Test.tex")]);
    }

    #[test]
    fn numeric_options_continue_parsing_cluster() {
        let parsed = parse_args(["-v2q"]).unwrap();

        assert_eq!(parsed.verbosity, Some(2));
        assert!(parsed.quiet);
    }

    #[test]
    fn optional_bool_options_continue_parsing_cluster() {
        let parsed = parse_args(["-g0qI"]).unwrap();

        assert_eq!(parsed.global_rc, Some(false));
        assert!(parsed.quiet);
        assert_eq!(parsed.input_files, Some(true));
    }

    #[test]
    fn parses_separated_short_bool_values() {
        let parsed = parse_args(["-g", "0", "-b", "1", "-x", "0", "file.tex"]).unwrap();

        assert_eq!(parsed.global_rc, Some(false));
        assert_eq!(parsed.backup, Some(true));
        assert_eq!(parsed.wipe_verb, Some(false));
        assert_eq!(parsed.files, [PathBuf::from("file.tex")]);
    }

    #[test]
    fn parses_license_action() {
        let parsed = parse_args(["-i"]).unwrap();

        assert_eq!(parsed.action, CliAction::License);
    }

    #[test]
    fn parses_required_argument_from_next_argv() {
        let parsed = parse_args(["-o", "out.txt", "--localrc", "custom.rc"]).unwrap();

        assert_eq!(parsed.output, Some(PathBuf::from("out.txt")));
        assert_eq!(parsed.local_rc_files, [PathBuf::from("custom.rc")]);
    }

    #[test]
    fn parses_debug_number_from_next_argv() {
        let parsed = parse_args(["-d", "4", "-STabSize=7"]).unwrap();

        assert_eq!(parsed.debug_level, Some(4));
        assert_eq!(parsed.rc_overrides, ["TabSize=7"]);
        assert!(parsed.files.is_empty());
    }

    #[test]
    fn rejects_duplicate_output() {
        let err = parse_args(["-oone", "--output=two"]).unwrap_err();

        assert_eq!(err, CliError::OutputSpecifiedTwice);
    }

    #[test]
    fn reset_allows_later_output_option() {
        let parsed = parse_args(["-oone", "-r", "--output=two"]).unwrap();

        assert!(parsed.reset);
        assert_eq!(parsed.output, Some(PathBuf::from("two")));
    }

    #[test]
    fn parses_long_optional_arguments() {
        let parsed = parse_args([
            "--verbosity=3",
            "--pipeverb",
            "--backup=0",
            "--inputfiles",
            "--headererr=1",
        ])
        .unwrap();

        assert_eq!(parsed.verbosity, Some(3));
        assert_eq!(parsed.pipe_verbosity, Some(1));
        assert_eq!(parsed.backup, Some(false));
        assert_eq!(parsed.input_files, Some(true));
        assert_eq!(parsed.header_errors, Some(true));
    }

    #[test]
    fn parses_question_as_help() {
        let parsed = parse_args(["?"]).unwrap();

        assert_eq!(parsed.action, CliAction::Help);
    }
}
