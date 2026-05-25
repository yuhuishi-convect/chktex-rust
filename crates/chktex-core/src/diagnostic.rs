#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    Message,
    Warning,
    Error,
}

impl DiagnosticKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Message => "Message",
            Self::Warning => "Warning",
            Self::Error => "Error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub number: i32,
    pub kind: DiagnosticKind,
    pub file: String,
    pub line: i64,
    pub column: usize,
    pub len: usize,
    pub message: String,
    pub source: Vec<u8>,
}

impl Diagnostic {
    pub fn new(
        number: i32,
        kind: DiagnosticKind,
        file: impl Into<String>,
        line: i64,
        column: usize,
        len: usize,
        message: impl Into<String>,
        source: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            number,
            kind,
            file: file.into(),
            line,
            column,
            len,
            message: message.into(),
            source: source.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatOptions {
    pub format: String,
    pub delimiter: String,
    pub reverse_on: String,
    pub reverse_off: String,
}

impl FormatOptions {
    pub fn normal() -> Self {
        Self {
            format: "%k %n in %f line %l: %m\n%r%s%t\n%u\n".to_string(),
            delimiter: ":".to_string(),
            reverse_on: "\x1b[7m".to_string(),
            reverse_off: "\x1b[0m".to_string(),
        }
    }
}

pub fn format_diagnostic(diagnostic: &Diagnostic, options: &FormatOptions) -> String {
    let mut out = String::new();
    let mut chars = options.format.chars();

    while let Some(ch) = chars.next() {
        if ch != '%' {
            out.push(ch);
            continue;
        }

        let Some(code) = chars.next() else {
            out.push('%');
            break;
        };

        match code {
            'b' => out.push_str(&options.delimiter),
            'c' => out.push_str(&(diagnostic.column + 1).to_string()),
            'd' => out.push_str(&diagnostic.len.to_string()),
            'f' => out.push_str(&diagnostic.file),
            'i' => out.push_str(&options.reverse_on),
            'I' => out.push_str(&options.reverse_off),
            'k' => out.push_str(diagnostic.kind.as_str()),
            'l' => out.push_str(&diagnostic.line.to_string()),
            'm' => out.push_str(&diagnostic.message),
            'n' => out.push_str(&diagnostic.number.to_string()),
            'u' => out.push_str(&underline(diagnostic.column, diagnostic.len)),
            'r' => out.push_str(&lossy_slice(&diagnostic.source, 0, diagnostic.column)),
            's' => out.push_str(&lossy_slice(
                &diagnostic.source,
                diagnostic.column,
                diagnostic.len,
            )),
            't' => out.push_str(&lossy_tail(
                &diagnostic.source,
                diagnostic.column.saturating_add(diagnostic.len),
            )),
            other => out.push(other),
        }
    }

    out
}

fn underline(column: usize, len: usize) -> String {
    let mut out = String::with_capacity(column.saturating_add(len));
    out.extend(std::iter::repeat_n(' ', column));
    out.extend(std::iter::repeat_n('^', len));
    out
}

fn lossy_slice(bytes: &[u8], start: usize, len: usize) -> String {
    if start >= bytes.len() {
        return String::new();
    }
    let end = start.saturating_add(len).min(bytes.len());
    String::from_utf8_lossy(&bytes[start..end]).into_owned()
}

fn lossy_tail(bytes: &[u8], start: usize) -> String {
    if start >= bytes.len() {
        return String::new();
    }
    String::from_utf8_lossy(&bytes[start..]).into_owned()
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, DiagnosticKind, FormatOptions, format_diagnostic};

    #[test]
    fn formats_normal_upstream_style_output() {
        let diagnostic = sample();
        let rendered = format_diagnostic(&diagnostic, &FormatOptions::normal());

        assert_eq!(
            rendered,
            "Warning 1 in Test.tex line 7: Command terminated with space.\n\\foo This\n    ^\n"
        );
    }

    #[test]
    fn formats_split_fields() {
        let diagnostic = sample();
        let options = FormatOptions {
            format: "%f%b%l%b%c%b%n%b%m\n".to_string(),
            delimiter: "\t".to_string(),
            reverse_on: String::new(),
            reverse_off: String::new(),
        };

        assert_eq!(
            format_diagnostic(&diagnostic, &options),
            "Test.tex\t7\t5\t1\tCommand terminated with space.\n"
        );
    }

    #[test]
    fn formats_inverse_source_parts() {
        let diagnostic = sample();
        let options = FormatOptions {
            format: "%r%i%s%I%t".to_string(),
            delimiter: ":".to_string(),
            reverse_on: "<on>".to_string(),
            reverse_off: "<off>".to_string(),
        };

        assert_eq!(
            format_diagnostic(&diagnostic, &options),
            "\\foo<on> <off>This"
        );
    }

    #[test]
    fn unknown_format_code_prints_code_character() {
        let diagnostic = sample();
        let options = FormatOptions {
            format: "%% %z".to_string(),
            delimiter: ":".to_string(),
            reverse_on: String::new(),
            reverse_off: String::new(),
        };

        assert_eq!(format_diagnostic(&diagnostic, &options), "% z");
    }

    fn sample() -> Diagnostic {
        Diagnostic::new(
            1,
            DiagnosticKind::Warning,
            "Test.tex",
            7,
            4,
            1,
            "Command terminated with space.",
            br"\foo This".to_vec(),
        )
    }
}
