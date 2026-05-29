use std::collections::BTreeMap;

use thiserror::Error;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ResourceSet {
    entries: BTreeMap<String, ResourceEntry>,
}

impl ResourceSet {
    pub fn get(&self, key: &str) -> Option<&ResourceEntry> {
        self.entries.get(&normalize_key(key))
    }

    pub fn merge(&mut self, other: ResourceSet) {
        for (key, incoming) in other.entries {
            if key == "cmdline" {
                let entry = self.entries.entry(key).or_default();
                if incoming.value.is_some() {
                    entry.value = incoming.value;
                }
                entry.list.extend(incoming.list);
                entry
                    .case_insensitive_list
                    .extend(incoming.case_insensitive_list);
            } else {
                self.entries.insert(key, incoming);
            }
        }
    }

    fn entry_mut(&mut self, key: &str) -> &mut ResourceEntry {
        self.entries.entry(normalize_key(key)).or_default()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ResourceEntry {
    pub value: Option<String>,
    pub list: Vec<String>,
    pub case_insensitive_list: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ResourceError {
    #[error("line {line}: expected {expected}, found {found}")]
    UnexpectedToken {
        line: usize,
        expected: &'static str,
        found: &'static str,
    },
    #[error("line {line}: unknown escape code !{escape}")]
    UnknownEscape { line: usize, escape: char },
    #[error("line {line}: invalid hex escape")]
    InvalidHexEscape { line: usize },
    #[error("line {line}: invalid octal escape")]
    InvalidOctalEscape { line: usize },
}

pub fn parse_resource(input: &str) -> Result<ResourceSet, ResourceError> {
    Parser::new(input).parse()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    Item(String),
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Equal,
    Eof,
}

impl Token {
    fn kind(&self) -> &'static str {
        match self {
            Token::Word(_) => "word",
            Token::Item(_) => "item",
            Token::OpenBrace => "`{'",
            Token::CloseBrace => "`}'",
            Token::OpenBracket => "`['",
            Token::CloseBracket => "`]'",
            Token::Equal => "`='",
            Token::Eof => "EOF",
        }
    }
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    resources: ResourceSet,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
            resources: ResourceSet::default(),
        }
    }

    fn parse(mut self) -> Result<ResourceSet, ResourceError> {
        loop {
            match self.lexer.next_word()? {
                Token::Word(key) => self.parse_entry(&key)?,
                Token::Eof => return Ok(self.resources),
                found => {
                    return Err(ResourceError::UnexpectedToken {
                        line: self.lexer.line,
                        expected: "word or EOF",
                        found: found.kind(),
                    });
                }
            }
        }
    }

    fn parse_entry(&mut self, key: &str) -> Result<(), ResourceError> {
        loop {
            match self.lexer.next_control()? {
                Token::OpenBrace => self.parse_list(key, ListKind::CaseSensitive)?,
                Token::OpenBracket => self.parse_list(key, ListKind::CaseInsensitive)?,
                Token::Equal => {
                    if !self.parse_assignment(key)? {
                        return Ok(());
                    }
                }
                Token::Word(next_key) => {
                    self.lexer.push_back(Token::Word(next_key));
                    return Ok(());
                }
                Token::Eof => return Ok(()),
                found => {
                    return Err(ResourceError::UnexpectedToken {
                        line: self.lexer.line,
                        expected: "`{', `[', `=', word, or EOF",
                        found: found.kind(),
                    });
                }
            }
        }
    }

    fn parse_assignment(&mut self, key: &str) -> Result<bool, ResourceError> {
        match self.lexer.next_assignment()? {
            Token::Item(value) => {
                self.resources.entry_mut(key).value = Some(value);
                Ok(false)
            }
            Token::OpenBrace => {
                self.resources.entry_mut(key).list.clear();
                self.parse_list(key, ListKind::CaseSensitive)?;
                Ok(true)
            }
            Token::OpenBracket => {
                self.resources.entry_mut(key).case_insensitive_list.clear();
                self.parse_list(key, ListKind::CaseInsensitive)?;
                Ok(true)
            }
            found => Err(ResourceError::UnexpectedToken {
                line: self.lexer.line,
                expected: "item, `{', or `['",
                found: found.kind(),
            }),
        }
    }

    fn parse_list(&mut self, key: &str, kind: ListKind) -> Result<(), ResourceError> {
        loop {
            let token = match kind {
                ListKind::CaseSensitive => self.lexer.next_item()?,
                ListKind::CaseInsensitive => self.lexer.next_case_item()?,
            };

            match token {
                Token::Item(item) => match kind {
                    ListKind::CaseSensitive => self.resources.entry_mut(key).list.push(item),
                    ListKind::CaseInsensitive => self
                        .resources
                        .entry_mut(key)
                        .case_insensitive_list
                        .push(item),
                },
                Token::CloseBrace if kind == ListKind::CaseSensitive => return Ok(()),
                Token::CloseBracket if kind == ListKind::CaseInsensitive => return Ok(()),
                found => {
                    return Err(ResourceError::UnexpectedToken {
                        line: self.lexer.line,
                        expected: "item or list close",
                        found: found.kind(),
                    });
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListKind {
    CaseSensitive,
    CaseInsensitive,
}

struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
    pushed: Option<Token>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
            pushed: None,
        }
    }

    fn push_back(&mut self, token: Token) {
        assert!(self.pushed.is_none());
        self.pushed = Some(token);
    }

    fn next_word(&mut self) -> Result<Token, ResourceError> {
        self.next_token(Mode::Word)
    }

    fn next_control(&mut self) -> Result<Token, ResourceError> {
        self.next_token(Mode::Control)
    }

    fn next_assignment(&mut self) -> Result<Token, ResourceError> {
        self.next_token(Mode::Assignment)
    }

    fn next_item(&mut self) -> Result<Token, ResourceError> {
        self.next_token(Mode::BraceItem)
    }

    fn next_case_item(&mut self) -> Result<Token, ResourceError> {
        self.next_token(Mode::BracketItem)
    }

    fn next_token(&mut self, mode: Mode) -> Result<Token, ResourceError> {
        if let Some(token) = self.pushed.take() {
            return Ok(token);
        }

        self.skip_ws_and_comments();

        let Some(byte) = self.peek() else {
            return Ok(Token::Eof);
        };

        match byte {
            b'{' if mode.accepts_control() => {
                self.bump();
                Ok(Token::OpenBrace)
            }
            b'}' if mode.accepts_control() || mode == Mode::BraceItem => {
                self.bump();
                Ok(Token::CloseBrace)
            }
            b'[' if mode.accepts_control() => {
                self.bump();
                Ok(Token::OpenBracket)
            }
            b']' if mode.accepts_control() || mode == Mode::BracketItem => {
                self.bump();
                Ok(Token::CloseBracket)
            }
            b'=' if mode.accepts_control() => {
                self.bump();
                Ok(Token::Equal)
            }
            b'"' => self.quoted_item(),
            byte if mode.accepts_word() && is_ascii_alpha(byte) => Ok(Token::Word(self.word())),
            _ => self.unquoted_item(),
        }
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
                self.bump();
            }

            if self.peek() == Some(b'#') {
                while let Some(byte) = self.bump() {
                    if byte == b'\n' {
                        break;
                    }
                }
                continue;
            }

            break;
        }
    }

    fn word(&mut self) -> String {
        let start = self.pos;
        while matches!(self.peek(), Some(b'a'..=b'z' | b'A'..=b'Z')) {
            self.bump();
        }
        self.input[start..self.pos].to_string()
    }

    fn quoted_item(&mut self) -> Result<Token, ResourceError> {
        self.bump();
        let mut item = String::new();

        while let Some(byte) = self.peek() {
            match byte {
                b'"' => {
                    self.bump();
                    break;
                }
                b'!' => {
                    self.bump();
                    item.push(self.map_escape()?);
                }
                _ => {
                    self.bump();
                    item.push(byte as char);
                }
            }
        }

        Ok(Token::Item(item))
    }

    fn unquoted_item(&mut self) -> Result<Token, ResourceError> {
        let mut item = String::new();

        while let Some(byte) = self.peek() {
            match byte {
                b'#' | b' ' | b'\t' | b'\r' | b'\n' => break,
                b'!' => {
                    self.bump();
                    item.push(self.map_escape()?);
                }
                _ => {
                    self.bump();
                    item.push(byte as char);
                }
            }
        }

        Ok(Token::Item(item))
    }

    fn map_escape(&mut self) -> Result<char, ResourceError> {
        let Some(byte) = self.bump() else {
            return Ok('\0');
        };

        let mapped = match byte {
            b'"' => '"',
            b'!' => '!',
            b'#' => '#',
            b'n' | b'N' => '\n',
            b'r' | b'R' => '\r',
            b'b' | b'B' => '\u{0008}',
            b't' | b'T' => '\t',
            b'f' | b'F' => '\u{000c}',
            b'{' => '{',
            b'}' => '}',
            b'[' => '[',
            b']' => ']',
            b'=' => '=',
            b' ' => ' ',
            b'x' | b'X' => return self.hex_escape(),
            b'0'..=b'7' => return self.octal_escape(byte),
            other => {
                return Err(ResourceError::UnknownEscape {
                    line: self.line,
                    escape: other as char,
                });
            }
        };

        Ok(mapped)
    }

    fn hex_escape(&mut self) -> Result<char, ResourceError> {
        let mut value = 0u32;
        for _ in 0..2 {
            let Some(byte) = self.bump() else {
                return Err(ResourceError::InvalidHexEscape { line: self.line });
            };
            let Some(digit) = (byte as char).to_digit(16) else {
                return Err(ResourceError::InvalidHexEscape { line: self.line });
            };
            value = (value << 4) + digit;
        }
        char::from_u32(value).ok_or(ResourceError::InvalidHexEscape { line: self.line })
    }

    fn octal_escape(&mut self, first: u8) -> Result<char, ResourceError> {
        let mut value = u32::from(first - b'0');
        for _ in 0..2 {
            let Some(byte) = self.peek() else {
                break;
            };
            if !(b'0'..=b'7').contains(&byte) {
                break;
            }
            self.bump();
            value = (value << 3) + u32::from(byte - b'0');
        }
        char::from_u32(value).ok_or(ResourceError::InvalidOctalEscape { line: self.line })
    }

    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.pos += 1;
        if byte == b'\n' {
            self.line += 1;
        }
        Some(byte)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Word,
    Control,
    Assignment,
    BraceItem,
    BracketItem,
}

impl Mode {
    fn accepts_control(self) -> bool {
        matches!(self, Mode::Control | Mode::Assignment)
    }

    fn accepts_word(self) -> bool {
        matches!(self, Mode::Word | Mode::Control)
    }
}

fn normalize_key(key: &str) -> String {
    key.to_ascii_lowercase()
}

fn is_ascii_alpha(byte: u8) -> bool {
    byte.is_ascii_alphabetic()
}

#[cfg(test)]
mod tests {
    use super::{ResourceError, parse_resource};

    #[test]
    fn parses_sensitive_and_case_insensitive_lists() {
        let resources = parse_resource(
            r#"
Silent
{
    \rm \em
}
[
    \\start.* \\stop.*
]
"#,
        )
        .unwrap();

        let silent = resources.get("silent").unwrap();
        assert_eq!(silent.list, [r"\rm", r"\em"]);
        assert_eq!(silent.case_insensitive_list, [r"\\start.*", r"\\stop.*"]);
    }

    #[test]
    fn parses_scalar_assignment() {
        let resources = parse_resource("TabSize = 8\nQuoteStyle = Traditional\n").unwrap();

        assert_eq!(
            resources.get("TabSize").unwrap().value.as_deref(),
            Some("8")
        );
        assert_eq!(
            resources.get("quotestyle").unwrap().value.as_deref(),
            Some("Traditional")
        );
    }

    #[test]
    fn assignment_to_list_replaces_existing_list() {
        let resources = parse_resource(
            r#"
Silent { \rm \em }
Silent = { \bf }
"#,
        )
        .unwrap();

        assert_eq!(resources.get("Silent").unwrap().list, [r"\bf"]);
    }

    #[test]
    fn merge_appends_cmdline_entries() {
        let mut resources = parse_resource("CmdLine { -n36 }\n").unwrap();
        resources.merge(parse_resource("CmdLine { -r -q -v0 }\n").unwrap());

        assert_eq!(
            resources.get("CmdLine").unwrap().list,
            ["-n36", "-r", "-q", "-v0"]
        );
    }

    #[test]
    fn parses_quoted_items_and_escapes() {
        let resources = parse_resource(
            r#"
OutFormat
{
    "%k %n in %f line %l: %m!n%r%s%t!n%u!n"
    "literal !# and !! and !{braces!}"
}
"#,
        )
        .unwrap();

        let out_format = &resources.get("OutFormat").unwrap().list;
        assert_eq!(out_format[0], "%k %n in %f line %l: %m\n%r%s%t\n%u\n");
        assert_eq!(out_format[1], "literal # and ! and {braces}");
    }

    #[test]
    fn parses_hex_and_octal_escapes() {
        let resources = parse_resource(r#"Key { !x41 !101 }"#).unwrap();

        assert_eq!(resources.get("Key").unwrap().list, ["A", "A"]);
    }

    #[test]
    fn comments_are_ignored_outside_quotes() {
        let resources = parse_resource(
            r#"
Key { value # ignored
      "kept # inside quote" }
"#,
        )
        .unwrap();

        assert_eq!(
            resources.get("Key").unwrap().list,
            ["value", "kept # inside quote"]
        );
    }

    #[test]
    fn reports_unknown_escape() {
        let err = parse_resource(r#"Key { !q }"#).unwrap_err();

        assert_eq!(
            err,
            ResourceError::UnknownEscape {
                line: 1,
                escape: 'q'
            }
        );
    }

    #[test]
    fn parses_upstream_chktexrc_fixture() {
        let fixture = include_str!("../../../tests/fixtures/upstream/chktexrc");
        let resources = parse_resource(fixture).unwrap();

        assert_eq!(
            resources.get("TabSize").unwrap().value.as_deref(),
            Some("8")
        );
        assert!(!resources.get("OutFormat").unwrap().list.is_empty());
        assert!(!resources.get("Silent").unwrap().list.is_empty());
        assert!(!resources.get("UserWarnRegex").unwrap().list.is_empty());
    }
}
