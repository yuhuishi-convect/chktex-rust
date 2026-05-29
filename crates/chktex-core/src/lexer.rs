#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Command(Vec<u8>),
    Text(Vec<u8>),
    Space,
    Comment(Vec<u8>),
    BeginGroup,
    EndGroup,
    BeginOptional,
    EndOptional,
    MathShift { display: bool },
    Punctuation(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn len(self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}

pub fn lex_line(input: &[u8]) -> Vec<Token> {
    Lexer::new(input).lex()
}

struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            pos: 0,
            tokens: Vec::new(),
        }
    }

    fn lex(mut self) -> Vec<Token> {
        while let Some(byte) = self.peek() {
            match byte {
                b'\\' => self.command(),
                b'%' => self.comment(),
                b' ' | b'\t' | b'\r' | b'\n' => self.space(),
                b'{' => self.single(TokenKind::BeginGroup),
                b'}' => self.single(TokenKind::EndGroup),
                b'[' => self.single(TokenKind::BeginOptional),
                b']' => self.single(TokenKind::EndOptional),
                b'$' => self.math_shift(),
                b'.' | b',' | b';' | b':' | b'!' | b'?' | b'`' | b'\'' | b'(' | b')' | b'-'
                | b'~' | b'^' | b'_' | b'#' | b'&' => self.single(TokenKind::Punctuation(byte)),
                _ => self.text(),
            }
        }

        self.tokens
    }

    fn command(&mut self) {
        let start = self.pos;
        self.bump();

        if matches!(self.peek(), Some(byte) if is_tex_letter(byte)) {
            while matches!(self.peek(), Some(byte) if is_tex_letter(byte)) {
                self.bump();
            }
        } else if self.peek().is_some() {
            self.bump();
        }

        self.push(
            start,
            TokenKind::Command(self.input[start..self.pos].to_vec()),
        );
    }

    fn comment(&mut self) {
        let start = self.pos;
        while let Some(byte) = self.peek() {
            self.bump();
            if byte == b'\n' {
                break;
            }
        }
        self.push(
            start,
            TokenKind::Comment(self.input[start..self.pos].to_vec()),
        );
    }

    fn space(&mut self) {
        let start = self.pos;
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.bump();
        }
        self.push(start, TokenKind::Space);
    }

    fn math_shift(&mut self) {
        let start = self.pos;
        self.bump();
        let display = if self.peek() == Some(b'$') {
            self.bump();
            true
        } else {
            false
        };
        self.push(start, TokenKind::MathShift { display });
    }

    fn text(&mut self) {
        let start = self.pos;
        while let Some(byte) = self.peek() {
            if is_special(byte) {
                break;
            }
            self.bump();
        }
        self.push(start, TokenKind::Text(self.input[start..self.pos].to_vec()));
    }

    fn single(&mut self, kind: TokenKind) {
        let start = self.pos;
        self.bump();
        self.push(start, kind);
    }

    fn push(&mut self, start: usize, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            span: Span {
                start,
                end: self.pos,
            },
        });
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.pos += 1;
        Some(byte)
    }
}

fn is_tex_letter(byte: u8) -> bool {
    byte.is_ascii_alphabetic()
}

fn is_special(byte: u8) -> bool {
    matches!(
        byte,
        b'\\'
            | b'%'
            | b' '
            | b'\t'
            | b'\r'
            | b'\n'
            | b'{'
            | b'}'
            | b'['
            | b']'
            | b'$'
            | b'.'
            | b','
            | b';'
            | b':'
            | b'!'
            | b'?'
            | b'`'
            | b'\''
            | b'('
            | b')'
            | b'-'
            | b'~'
            | b'^'
            | b'_'
            | b'#'
            | b'&'
    )
}

#[cfg(test)]
mod tests {
    use super::{Span, Token, TokenKind, lex_line};

    #[test]
    fn lexes_commands_spaces_and_text_with_byte_spans() {
        let tokens = lex_line(br"\foo This");

        assert_eq!(
            tokens,
            vec![
                token(TokenKind::Command(br"\foo".to_vec()), 0, 4),
                token(TokenKind::Space, 4, 5),
                token(TokenKind::Text(b"This".to_vec()), 5, 9),
            ]
        );
    }

    #[test]
    fn lexes_single_character_control_sequence() {
        let tokens = lex_line(br"\{x");

        assert_eq!(
            tokens,
            vec![
                token(TokenKind::Command(br"\{".to_vec()), 0, 2),
                token(TokenKind::Text(b"x".to_vec()), 2, 3),
            ]
        );
    }

    #[test]
    fn lexes_groups_optional_math_and_punctuation() {
        let tokens = lex_line(br"$x$ $$y$$ [a], {b}");

        assert_eq!(
            tokens,
            vec![
                token(TokenKind::MathShift { display: false }, 0, 1),
                token(TokenKind::Text(b"x".to_vec()), 1, 2),
                token(TokenKind::MathShift { display: false }, 2, 3),
                token(TokenKind::Space, 3, 4),
                token(TokenKind::MathShift { display: true }, 4, 6),
                token(TokenKind::Text(b"y".to_vec()), 6, 7),
                token(TokenKind::MathShift { display: true }, 7, 9),
                token(TokenKind::Space, 9, 10),
                token(TokenKind::BeginOptional, 10, 11),
                token(TokenKind::Text(b"a".to_vec()), 11, 12),
                token(TokenKind::EndOptional, 12, 13),
                token(TokenKind::Punctuation(b','), 13, 14),
                token(TokenKind::Space, 14, 15),
                token(TokenKind::BeginGroup, 15, 16),
                token(TokenKind::Text(b"b".to_vec()), 16, 17),
                token(TokenKind::EndGroup, 17, 18),
            ]
        );
    }

    #[test]
    fn comment_consumes_to_end_of_line() {
        let tokens = lex_line(b"text % comment\nnext");

        assert_eq!(
            tokens,
            vec![
                token(TokenKind::Text(b"text".to_vec()), 0, 4),
                token(TokenKind::Space, 4, 5),
                token(TokenKind::Comment(b"% comment\n".to_vec()), 5, 15),
                token(TokenKind::Text(b"next".to_vec()), 15, 19),
            ]
        );
    }

    #[test]
    fn non_ascii_bytes_are_text_not_letters() {
        let tokens = lex_line(b"caf\xc3\xa9 \\\xc3\xa9");

        assert_eq!(
            tokens,
            vec![
                token(TokenKind::Text(b"caf\xc3\xa9".to_vec()), 0, 5),
                token(TokenKind::Space, 5, 6),
                token(TokenKind::Command(b"\\\xc3".to_vec()), 6, 8),
                token(TokenKind::Text(b"\xa9".to_vec()), 8, 9),
            ]
        );
    }

    fn token(kind: TokenKind, start: usize, end: usize) -> Token {
        Token {
            kind,
            span: Span { start, end },
        }
    }
}
