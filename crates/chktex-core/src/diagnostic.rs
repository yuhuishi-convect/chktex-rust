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
    pub sort_line: Option<i64>,
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
            sort_line: None,
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
    String::from_utf8_lossy(&format_diagnostic_bytes(diagnostic, options)).into_owned()
}

pub fn format_diagnostic_bytes(diagnostic: &Diagnostic, options: &FormatOptions) -> Vec<u8> {
    let mut out = Vec::new();
    let mut chars = options.format.chars();

    while let Some(ch) = chars.next() {
        if ch != '%' {
            push_char(&mut out, ch);
            continue;
        }

        let Some(code) = chars.next() else {
            out.push(b'%');
            break;
        };

        match code {
            'b' => out.extend_from_slice(options.delimiter.as_bytes()),
            'c' => out.extend_from_slice((diagnostic.column + 1).to_string().as_bytes()),
            'd' => out.extend_from_slice(diagnostic.len.to_string().as_bytes()),
            'f' => out.extend_from_slice(diagnostic.file.as_bytes()),
            'i' => out.extend_from_slice(options.reverse_on.as_bytes()),
            'I' => out.extend_from_slice(options.reverse_off.as_bytes()),
            'k' => out.extend_from_slice(diagnostic.kind.as_str().as_bytes()),
            'l' => out.extend_from_slice(diagnostic.line.to_string().as_bytes()),
            'm' => out.extend_from_slice(message_bytes(diagnostic).as_slice()),
            'n' => out.extend_from_slice(diagnostic.number.to_string().as_bytes()),
            'u' => out.extend_from_slice(&underline_bytes(diagnostic.column, diagnostic.len)),
            'r' => out.extend_from_slice(raw_slice(&diagnostic.source, 0, diagnostic.column)),
            's' => out.extend_from_slice(raw_slice(
                &diagnostic.source,
                diagnostic.column,
                diagnostic.len,
            )),
            't' => out.extend_from_slice(raw_tail(
                &diagnostic.source,
                diagnostic.column.saturating_add(diagnostic.len),
            )),
            other => push_char(&mut out, other),
        }
    }

    out
}

fn push_char(out: &mut Vec<u8>, ch: char) {
    let mut buf = [0; 4];
    out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
}

fn message_bytes(diagnostic: &Diagnostic) -> Vec<u8> {
    if diagnostic.number == 19 {
        b"Use \"'\" (ASCII 39) instead  of \"\xB4\" (ASCII 180).".to_vec()
    } else {
        diagnostic.message.as_bytes().to_vec()
    }
}

fn underline_bytes(column: usize, len: usize) -> Vec<u8> {
    let mut out = String::with_capacity(column.saturating_add(len));
    out.extend(std::iter::repeat_n(' ', column));
    out.extend(std::iter::repeat_n('^', len));
    out.into_bytes()
}

fn raw_slice(bytes: &[u8], start: usize, len: usize) -> &[u8] {
    if start >= bytes.len() {
        return &[];
    }
    let end = start.saturating_add(len).min(bytes.len());
    &bytes[start..end]
}

fn raw_tail(bytes: &[u8], start: usize) -> &[u8] {
    if start >= bytes.len() {
        return &[];
    }
    &bytes[start..]
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
