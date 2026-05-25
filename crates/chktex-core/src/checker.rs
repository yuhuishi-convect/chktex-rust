use std::collections::BTreeSet;

use crate::{
    diagnostic::{Diagnostic, DiagnosticKind},
    lexer::{Token, TokenKind, lex_line},
    regex_engine::{RegexEngine, parse_pattern_spec},
    resource::ResourceSet,
};

#[cfg(feature = "regex-bytes")]
use crate::regex_engine::bytes::BytesRegexEngine;

pub const WARNING_COMMAND_TERMINATED_WITH_SPACE: i32 = 1;
pub const WARNING_USER_PATTERN: i32 = 20;
pub const WARNING_USER_REGEX: i32 = 44;

#[derive(Debug)]
pub struct CheckerConfig {
    pub silent_commands: BTreeSet<Vec<u8>>,
    pub user_warn: Vec<Vec<u8>>,
    pub user_warn_case_insensitive: Vec<Vec<u8>>,
    #[cfg(feature = "regex-bytes")]
    pub user_warn_regex: Vec<UserRegex>,
    pub warning_1_enabled: bool,
    pub user_warn_enabled: bool,
    pub user_warn_regex_enabled: bool,
}

#[cfg(feature = "regex-bytes")]
#[derive(Debug)]
pub struct UserRegex {
    pub display: Option<String>,
    pub regex: regex::bytes::Regex,
}

impl Default for CheckerConfig {
    fn default() -> Self {
        Self {
            silent_commands: BTreeSet::new(),
            user_warn: Vec::new(),
            user_warn_case_insensitive: Vec::new(),
            #[cfg(feature = "regex-bytes")]
            user_warn_regex: Vec::new(),
            warning_1_enabled: true,
            user_warn_enabled: true,
            user_warn_regex_enabled: true,
        }
    }
}

impl CheckerConfig {
    pub fn from_resources(resources: &ResourceSet) -> Self {
        let mut config = Self::default();
        if let Some(silent) = resources.get("Silent") {
            config.silent_commands = silent
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(user_warn) = resources.get("UserWarn") {
            config.user_warn = user_warn
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
            config.user_warn_case_insensitive = user_warn
                .case_insensitive_list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        #[cfg(feature = "regex-bytes")]
        if let Some(user_warn_regex) = resources.get("UserWarnRegex") {
            config.user_warn_regex = user_warn_regex
                .list
                .iter()
                .filter_map(|raw| {
                    let spec = parse_pattern_spec(raw);
                    let regex = BytesRegexEngine::compile(&spec).ok()??;
                    Some(UserRegex {
                        display: spec.display.map(ToOwned::to_owned),
                        regex,
                    })
                })
                .collect();
        }
        config
    }
}

pub fn check_document(file: &str, input: &[u8], config: &CheckerConfig) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (line_index, line) in split_lines_preserve(input).enumerate() {
        let line_no = i64::try_from(line_index + 1).unwrap_or(i64::MAX);
        let line = normalize_input_line(line);
        diagnostics.extend(check_line(file, line_no, &line, config));
    }

    diagnostics
}

pub fn check_line(
    file: &str,
    line_no: i64,
    line: &[u8],
    config: &CheckerConfig,
) -> Vec<Diagnostic> {
    let tokens = lex_line(line);
    let mut diagnostics = Vec::new();
    let mut math_mode = false;

    if config.user_warn_enabled {
        diagnostics.extend(check_user_warn(file, line_no, line, config));
    }
    #[cfg(feature = "regex-bytes")]
    if config.user_warn_regex_enabled {
        diagnostics.extend(check_user_warn_regex(file, line_no, line, config));
    }

    for (index, token) in tokens.iter().enumerate() {
        if matches!(token.kind, TokenKind::MathShift { .. }) {
            math_mode = !math_mode;
            continue;
        }

        if !config.warning_1_enabled || math_mode {
            continue;
        }

        let TokenKind::Command(command) = &token.kind else {
            continue;
        };

        if command.len() == 2 || config.silent_commands.contains(command) {
            continue;
        }

        if let Some(space) = next_space_token(&tokens, index) {
            diagnostics.push(Diagnostic::new(
                WARNING_COMMAND_TERMINATED_WITH_SPACE,
                DiagnosticKind::Warning,
                file,
                line_no,
                space.span.start,
                1,
                "Command terminated with space.",
                line.to_vec(),
            ));
        }
    }

    diagnostics
}

#[cfg(feature = "regex-bytes")]
fn check_user_warn_regex(
    file: &str,
    line_no: i64,
    line: &[u8],
    config: &CheckerConfig,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for user_regex in &config.user_warn_regex {
        let Ok(matches) = BytesRegexEngine::find_iter(&user_regex.regex, line) else {
            continue;
        };
        for found in matches {
            let message = match &user_regex.display {
                Some(display) if !display.is_empty() => format!("User Regex: {display}."),
                _ => format!(
                    "User Regex: {}.",
                    String::from_utf8_lossy(&line[found.start..found.end])
                ),
            };

            diagnostics.push(Diagnostic::new(
                WARNING_USER_REGEX,
                DiagnosticKind::Warning,
                file,
                line_no,
                found.start,
                found.len(),
                message,
                line.to_vec(),
            ));
        }
    }

    diagnostics
}

fn check_user_warn(
    file: &str,
    line_no: i64,
    line: &[u8],
    config: &CheckerConfig,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for pattern in &config.user_warn {
        diagnostics.extend(find_plain_pattern(line, pattern).map(|column| {
            user_warn_diagnostic(file, line_no, line, column, pattern.len(), pattern)
        }));
    }

    if !config.user_warn_case_insensitive.is_empty() {
        let folded_line = ascii_lowercase(line);
        for pattern in &config.user_warn_case_insensitive {
            let folded_pattern = ascii_lowercase(pattern);
            diagnostics.extend(
                find_plain_pattern(&folded_line, &folded_pattern).map(|column| {
                    user_warn_diagnostic(file, line_no, line, column, pattern.len(), pattern)
                }),
            );
        }
    }

    diagnostics
}

fn user_warn_diagnostic(
    file: &str,
    line_no: i64,
    line: &[u8],
    column: usize,
    len: usize,
    pattern: &[u8],
) -> Diagnostic {
    Diagnostic::new(
        WARNING_USER_PATTERN,
        DiagnosticKind::Warning,
        file,
        line_no,
        column,
        len,
        format!(
            "User-specified pattern found: {}.",
            String::from_utf8_lossy(pattern)
        ),
        line.to_vec(),
    )
}

fn find_plain_pattern<'a>(
    haystack: &'a [u8],
    needle: &'a [u8],
) -> impl Iterator<Item = usize> + 'a {
    let mut offset = 0;
    std::iter::from_fn(move || {
        if needle.is_empty() || offset > haystack.len() {
            return None;
        }
        let found = haystack[offset..]
            .windows(needle.len())
            .position(|window| window == needle)?;
        let start = offset + found;
        offset = start + needle.len();
        Some(start)
    })
}

fn ascii_lowercase(bytes: &[u8]) -> Vec<u8> {
    bytes.iter().map(|byte| byte.to_ascii_lowercase()).collect()
}

fn next_space_token(tokens: &[Token], index: usize) -> Option<&Token> {
    tokens
        .get(index + 1)
        .filter(|token| matches!(token.kind, TokenKind::Space))
}

fn split_lines_preserve(input: &[u8]) -> impl Iterator<Item = &[u8]> {
    input.split_inclusive(|byte| *byte == b'\n')
}

fn normalize_input_line(line: &[u8]) -> Vec<u8> {
    let mut normalized = line
        .iter()
        .map(|byte| match byte {
            b'\n' | b'\r' => b' ',
            other => *other,
        })
        .collect::<Vec<_>>();

    if !matches!(normalized.last(), Some(b' ')) {
        normalized.push(b' ');
    }

    normalized
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        checker::{
            CheckerConfig, WARNING_COMMAND_TERMINATED_WITH_SPACE, check_document, check_line,
        },
        diagnostic::{FormatOptions, format_diagnostic},
        resource::parse_resource,
    };

    #[test]
    fn warning_1_reports_command_terminated_by_space() {
        let diagnostics = check_line(
            "stdin",
            1,
            br"\foo This is an error.  ",
            &CheckerConfig::default(),
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].number, WARNING_COMMAND_TERMINATED_WITH_SPACE);
        assert_eq!(diagnostics[0].column, 4);
        assert_eq!(diagnostics[0].len, 1);
        assert_eq!(
            format_diagnostic(&diagnostics[0], &FormatOptions::normal()),
            "Warning 1 in stdin line 1: Command terminated with space.\n\\foo This is an error.  \n    ^\n"
        );
    }

    #[test]
    fn warning_1_ignores_single_character_control_sequences() {
        let diagnostics = check_line("stdin", 1, br"\{ text", &CheckerConfig::default());

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn warning_1_ignores_silent_commands_from_resources() {
        let resources = parse_resource(r"Silent { \foo }").unwrap();
        let config = CheckerConfig::from_resources(&resources);
        let diagnostics = check_line("stdin", 1, br"\foo text", &config);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn warning_1_ignores_commands_in_math_mode() {
        let diagnostics = check_line("stdin", 1, br"$\foo x$ \bar y", &CheckerConfig::default());

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].column, 13);
    }

    #[test]
    fn check_document_tracks_line_numbers() {
        let diagnostics = check_document("file.tex", b"ok\n\\foo bad\n", &CheckerConfig::default());

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, 2);
        assert_eq!(diagnostics[0].file, "file.tex");
        assert_eq!(diagnostics[0].source, br"\foo bad ".to_vec());
    }

    #[test]
    fn check_document_appends_space_to_final_line_without_newline() {
        let diagnostics = check_document("file.tex", br"\foo bad", &CheckerConfig::default());

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].source, br"\foo bad ".to_vec());
    }

    #[test]
    fn warning_1_can_be_disabled() {
        let config = CheckerConfig {
            warning_1_enabled: false,
            user_warn_enabled: true,
            user_warn_regex_enabled: true,
            silent_commands: BTreeSet::new(),
            user_warn: Vec::new(),
            user_warn_case_insensitive: Vec::new(),
            #[cfg(feature = "regex-bytes")]
            user_warn_regex: Vec::new(),
        };

        assert!(check_line("stdin", 1, br"\foo text", &config).is_empty());
    }

    #[test]
    fn user_warn_reports_case_sensitive_patterns() {
        let resources = parse_resource(r"UserWarn { TODO }").unwrap();
        let config = CheckerConfig::from_resources(&resources);
        let diagnostics = check_line("stdin", 1, b"TODO and TODO ", &config);

        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].number, super::WARNING_USER_PATTERN);
        assert_eq!(diagnostics[0].column, 0);
        assert_eq!(diagnostics[1].column, 9);
        assert_eq!(
            format_diagnostic(&diagnostics[0], &FormatOptions::normal()),
            "Warning 20 in stdin line 1: User-specified pattern found: TODO.\nTODO and TODO \n^^^^\n"
        );
    }

    #[test]
    fn user_warn_reports_case_insensitive_patterns() {
        let resources = parse_resource(r"UserWarn [ chktex ]").unwrap();
        let config = CheckerConfig::from_resources(&resources);
        let diagnostics = check_line("stdin", 1, b"ChkTeX ", &config);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].column, 0);
        assert_eq!(diagnostics[0].len, 6);
        assert_eq!(
            diagnostics[0].message,
            "User-specified pattern found: chktex."
        );
    }

    #[test]
    fn user_warn_can_be_disabled() {
        let resources = parse_resource(r"UserWarn { TODO }").unwrap();
        let mut config = CheckerConfig::from_resources(&resources);
        config.user_warn_enabled = false;

        assert!(check_line("stdin", 1, b"TODO ", &config).is_empty());
    }

    #[cfg(feature = "regex-bytes")]
    #[test]
    fn user_warn_regex_reports_default_match_text() {
        let resources = parse_resource(r"UserWarnRegex { intro }").unwrap();
        let config = CheckerConfig::from_resources(&resources);
        let diagnostics = check_line("stdin", 1, b"a good intro ", &config);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].number, super::WARNING_USER_REGEX);
        assert_eq!(diagnostics[0].column, 7);
        assert_eq!(diagnostics[0].message, "User Regex: intro.");
    }

    #[cfg(feature = "regex-bytes")]
    #[test]
    fn user_warn_regex_reports_display_comment() {
        let resources =
            parse_resource(r"UserWarnRegex { (?!#Always! use! \nmid)\\not! *(\\mid|\|) }").unwrap();
        let config = CheckerConfig::from_resources(&resources);
        let diagnostics = check_line("stdin", 1, br"$p\not|n$ ", &config);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].number, super::WARNING_USER_REGEX);
        assert_eq!(diagnostics[0].message, r"User Regex: Always use \nmid.");
    }

    #[cfg(feature = "regex-bytes")]
    #[test]
    fn user_warn_regex_skips_pcre_prefixed_patterns_in_default_engine() {
        let resources = parse_resource(r"UserWarnRegex { PCRE:\[(?!bad) }").unwrap();
        let config = CheckerConfig::from_resources(&resources);

        assert!(config.user_warn_regex.is_empty());
    }

    #[cfg(feature = "regex-bytes")]
    #[test]
    fn user_warn_regex_can_be_disabled() {
        let resources = parse_resource(r"UserWarnRegex { intro }").unwrap();
        let mut config = CheckerConfig::from_resources(&resources);
        config.user_warn_regex_enabled = false;

        assert!(check_line("stdin", 1, b"intro ", &config).is_empty());
    }
}
