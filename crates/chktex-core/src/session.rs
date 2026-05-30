//! Single-buffer checking for programmatic callers (WASM, editors, tests).

use crate::{
    checker::{CheckerConfig, check_document},
    diagnostic::{Diagnostic, DiagnosticKind, FormatOptions, format_diagnostic_bytes},
    resource::{ResourceSet, parse_resource},
};

const DEFAULT_CHKTEXRC: &str = include_str!("../../../tests/fixtures/upstream/chktexrc");

#[derive(Debug, Clone)]
pub struct CheckOptions {
    /// Index into the `OutFormat` list (`-vN`). Default matches ChkTeX `-v2`.
    pub verbosity: i64,
    pub format: Option<String>,
    pub delimiter: Option<String>,
}

impl Default for CheckOptions {
    fn default() -> Self {
        Self {
            verbosity: 2,
            format: None,
            delimiter: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckSummary {
    pub exit_status: u8,
    pub warnings: usize,
    pub errors: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckOutput {
    pub diagnostics: Vec<Diagnostic>,
    pub formatted: Vec<u8>,
    pub summary: CheckSummary,
}

pub fn default_resources() -> ResourceSet {
    parse_resource(DEFAULT_CHKTEXRC).expect("embedded default chktexrc must parse")
}

pub fn check_buffer(
    file: &str,
    input: &[u8],
    resources: &ResourceSet,
    options: &CheckOptions,
) -> CheckOutput {
    let config = CheckerConfig::from_resources(resources);
    let output_format = select_format(resources, options);
    let diagnostics = check_document(file, input, &config);

    let mut formatted = Vec::new();
    let mut warnings = 0usize;
    let mut errors = 0usize;
    let mut exit_status = 0u8;

    for diagnostic in &diagnostics {
        match diagnostic.kind {
            DiagnosticKind::Warning => warnings += 1,
            DiagnosticKind::Error => errors += 1,
            DiagnosticKind::Message => {}
        }
        exit_status = combine_exit_status(
            exit_status,
            exit_status_for_diagnostic(diagnostic.number, diagnostic.kind),
        );
        formatted.extend(format_diagnostic_bytes(diagnostic, &output_format));
    }

    CheckOutput {
        diagnostics,
        formatted,
        summary: CheckSummary {
            exit_status,
            warnings,
            errors,
        },
    }
}

fn select_format(resources: &ResourceSet, options: &CheckOptions) -> FormatOptions {
    let mut format = FormatOptions::normal();

    if let Some(delimiter) = &options.delimiter {
        format.delimiter = delimiter.clone();
    }

    if let Some(custom) = &options.format {
        format.format = custom.clone();
        return format;
    }

    if let Some(out_formats) = resources.get("OutFormat")
        && let Ok(index) = usize::try_from(options.verbosity)
        && let Some(selected) = out_formats.list.get(index)
    {
        format.format = selected.clone();
    }

    format
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::WARNING_1;

    #[test]
    fn check_buffer_reports_warning_1() {
        let resources = default_resources();
        let result = check_buffer(
            "doc.tex",
            br"\foo space\n",
            &resources,
            &CheckOptions::default(),
        );
        assert!(result.diagnostics.iter().any(|d| d.number == WARNING_1));
        assert!(result.summary.warnings >= 1);
        assert!(!result.formatted.is_empty());
    }
}
