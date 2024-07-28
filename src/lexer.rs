use const_str::to_char_array;

use crate::{
    cursor::Cursor,
    span::Span,
    token::{self, Token},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum MultiLine {
    Yes,
    No,
}

#[derive(Debug)]
pub struct Lexer<'src, const LOOK: usize = 3> {
    cursor: Cursor<'src, str>,
    current_kind: [token::Kind; LOOK],
    current_span: [Span; LOOK],
    last_span: Span,
}

impl<'src, const LOOK: usize> Lexer<'src, LOOK> {
    pub fn new(source: &'src str) -> Self {
        let mut res = Self {
            cursor: Cursor::new(source),
            current_kind: [token::Kind::Eof; LOOK],
            current_span: [Span::from(0..0); LOOK],
            last_span: Span { start: 0, end: 0 },
        };

        for i in 0..LOOK {
            let token = res.next_impl();
            res.current_kind[i] = token.kind;
            res.current_span[i] = token.span;
        }

        res
    }

    pub fn source(&'src self) -> &'src str {
        self.cursor.source()
    }

    pub fn peek_kind<const N: usize>(&self) -> token::Kind {
        const {
            assert!(N < LOOK);
        };
        self.current_kind[N]
    }

    pub fn peek_span<const N: usize>(&self) -> Span {
        const {
            assert!(N < LOOK);
        };
        self.current_span[N]
    }

    pub fn peek_kind_array<const N: usize>(&self) -> [token::Kind; N] {
        const {
            assert!(N < LOOK);
        };
        let mut res = [token::Kind::Eof; N];
        res.copy_from_slice(&self.current_kind[0..N]);

        res
    }

    pub fn next_token(&mut self) -> Token {
        let token = Token {
            span: self.current_span[0],
            kind: self.current_kind[0],
        };

        let new = 'a: loop {
            let new = self.next_impl();
            if new.kind != token::Kind::Space && new.kind != token::Kind::Comment {
                break 'a new;
            }
        };

        self.current_kind.rotate_left(1);
        self.current_span.rotate_left(1);
        self.current_kind[const { LOOK - 1 }] = new.kind;
        self.current_span[const { LOOK - 1 }] = new.span;

        token
    }

    fn next_impl(&mut self) -> Token {
        let start = self.cursor.cursor();
        let Some(peek) = self.cursor.peek() else {
            return Token::eof().with_span(self.last_span);
        };
        self.cursor.bump();

        let kind = match peek {
            ' ' | '\t' | '\r' => token::Kind::Space,
            // '\t' => token::Kind::Tab,
            '\n' => token::Kind::Newline,
            '-' | '+' => self.consume_number_or_key(),
            '0'..='9' => self.consume_number_or_key(),
            '\'' if self.matches(to_char_array!("''")) => {
                self.cursor.bump_n(3);
                self.consume_delimited(MultiLine::Yes, to_char_array!("'''"))
                    .unwrap_or(token::Kind::NonClosing)
            }
            '\'' => self
                .consume_delimited(MultiLine::No, to_char_array!("'"))
                .unwrap_or(token::Kind::NonClosing),
            '"' if self.matches(to_char_array!(r#""""#)) => {
                self.cursor.bump_n(3);
                self.consume_delimited(MultiLine::Yes, to_char_array!(r#"""""#))
                    .unwrap_or(token::Kind::NonClosing)
            }
            '"' => self
                .consume_delimited(MultiLine::No, to_char_array!("\""))
                .unwrap_or(token::Kind::NonClosing),
            '[' => token::Kind::LBracket,
            ']' => token::Kind::RBracket,
            '{' => token::Kind::LCurly,
            '}' => token::Kind::RCurly,
            ',' => token::Kind::Comma,
            '=' => token::Kind::Equal,
            '#' => self.consume_comment(),
            '.' => token::Kind::Dot,
            'a'..='z' | 'A'..='Z' | '_' => self.consume_key(start),
            _ => token::Kind::Unknown,
        };

        let span = Span::from(start..self.cursor.cursor());
        self.last_span = span;
        Token { span, kind }
    }

    fn consume_comment(&mut self) -> token::Kind {
        while let Some(peek) = self.cursor.peek() {
            match peek {
                '\n' => break,
                _ => {
                    self.cursor.bump();
                }
            }
        }

        token::Kind::Comment
    }

    fn consume_key(&mut self, start: usize) -> token::Kind {
        while let Some(peek) = self.cursor.peek() {
            match peek {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => {
                    self.cursor.bump();
                }
                _ => {
                    break;
                }
            }
        }

        match &self.cursor.source()[start..self.cursor.cursor()] {
            "true" | "false" => return token::Kind::Bool,
            "nan" | "inf" => return token::Kind::Float,
            _ => token::Kind::Key,
        }
    }

    fn consume_number_or_key(&mut self) -> token::Kind {
        if let Some(chunk) = self.cursor.peek_chunk::<3>()
            && (chunk == to_char_array!("nan") || chunk == to_char_array!("inf"))
        {
            self.cursor.bump_n(3);
            return token::Kind::Float;
        }

        let mut dots: usize = 0;
        let mut seen_chars: bool = false;
        while let Some(peek) = self.cursor.peek() {
            match peek {
                '0'..='9' | '_' => {
                    self.cursor.bump();
                }
                'a'..='z' | 'A'..='Z' | '-' => {
                    seen_chars = true;
                    self.cursor.bump();
                }
                '.' if !seen_chars
                    && self
                        .cursor
                        .peek_ahead(1)
                        .is_some_and(|c| matches!(c, '0'..='9' | '_')) =>
                {
                    dots += 1;
                    self.cursor.bump();
                }
                _ => {
                    break;
                }
            }
        }

        match (dots, seen_chars) {
            (2.., false) => token::Kind::InvalidFloat,
            (0, false) => token::Kind::Integer,
            (1, false) => token::Kind::Float,
            (0, true) => token::Kind::Key,
            (1.., true) => panic!("When a charater is found we dont match dots"),
        }
    }

    fn matches<const N: usize>(&self, chars: [char; N]) -> bool {
        self.cursor.peek_chunk::<N>().map_or(false, |s| s == chars)
    }

    fn consume_delimited<const N: usize>(
        &mut self,
        multiline: MultiLine,
        delimiter: [char; N],
    ) -> Option<token::Kind> {
        while let Some(char) = self.cursor.peek() {
            match char {
                '\n' if multiline == MultiLine::No => return None,
                c if delimiter.starts_with(&[c]) && self.matches(delimiter) => {
                    self.cursor.bump_n(delimiter.len());
                    return Some(if multiline == MultiLine::Yes {
                        token::Kind::StringMultiline
                    } else {
                        token::Kind::StringOrKey
                    });
                }
                _ => {
                    self.cursor.bump();
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use crate::lexer::Lexer;

    #[test]
    fn delimited_string() {
        let source = "['hello']";
        let mut lexer: Lexer<1> = super::Lexer::new(source);

        assert_eq!(lexer.next_token().kind, super::token::Kind::LBracket);
        assert_eq!(lexer.next_token().kind, super::token::Kind::StringOrKey);
        assert_eq!(lexer.next_token().kind, super::token::Kind::RBracket);
        assert!(lexer.cursor.peek().is_none())
    }

    #[test]
    fn undelimted_string() {
        let source = "['hello]";
        let mut lexer: Lexer<3> = super::Lexer::new(source);

        assert_eq!(lexer.next_token().kind, super::token::Kind::LBracket);
        assert_eq!(lexer.next_token().kind, super::token::Kind::NonClosing);
        assert!(lexer.cursor.peek().is_none())
    }

    #[test]
    fn recover_unterminated_string() {
        let source = "['hello\\nworld\n[";
        let mut lexer: Lexer<3> = super::Lexer::new(source);

        assert_eq!(lexer.next_token().kind, super::token::Kind::LBracket);
        assert_eq!(lexer.next_token().kind, super::token::Kind::NonClosing);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Newline);
        assert_eq!(lexer.next_token().kind, super::token::Kind::LBracket);
        assert!(lexer.cursor.peek().is_none())
    }

    #[test]
    fn delimiter_multiline_string() {
        let source = "['''hello\nworld''']";
        let mut lexer: Lexer<3> = super::Lexer::new(source);

        assert_eq!(lexer.next_token().kind, super::token::Kind::LBracket);
        assert_eq!(lexer.next_token().kind, super::token::Kind::StringMultiline);
        assert_eq!(lexer.next_token().kind, super::token::Kind::RBracket);
        assert!(lexer.cursor.peek().is_none());

        let source = r#"["""hello\nworld"""]"#;
        let mut lexer: Lexer<3> = super::Lexer::new(source);

        assert_eq!(lexer.next_token().kind, super::token::Kind::LBracket);
        assert_eq!(lexer.next_token().kind, super::token::Kind::StringMultiline);
        assert_eq!(lexer.next_token().kind, super::token::Kind::RBracket);
        assert!(lexer.cursor.peek().is_none());
    }

    #[test]
    fn number() {
        let source = "[123 456]";
        let mut lexer: Lexer<3> = super::Lexer::new(source);

        assert_eq!(lexer.next_token().kind, super::token::Kind::LBracket);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Integer);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Space);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Integer);
        assert_eq!(lexer.next_token().kind, super::token::Kind::RBracket);
        assert!(lexer.cursor.peek().is_none());
    }

    #[test]
    fn float() {
        let source = "[123_12.456 456.12_123 -nan nan inf]";
        let mut lexer: Lexer<3> = super::Lexer::new(source);

        assert_eq!(lexer.next_token().kind, super::token::Kind::LBracket);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Float);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Space);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Float);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Space);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Float);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Space);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Float);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Space);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Float);
        assert_eq!(lexer.next_token().kind, super::token::Kind::RBracket);
        assert!(lexer.cursor.peek().is_none());
    }

    #[test]
    fn key_starting_with_number() {
        let source = "[123key]";
        let mut lexer: Lexer<3> = super::Lexer::new(source);

        assert_eq!(lexer.next_token().kind, super::token::Kind::LBracket);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Key);
        assert_eq!(lexer.next_token().kind, super::token::Kind::RBracket);
        assert!(lexer.cursor.peek().is_none());

        let source = "[1.a23key.01_a-no.ther.123]";
        let mut lexer: Lexer<3> = super::Lexer::new(source);

        assert_eq!(lexer.next_token().kind, super::token::Kind::LBracket);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Integer);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Dot);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Key);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Dot);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Key);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Dot);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Key);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Dot);
        assert_eq!(lexer.next_token().kind, super::token::Kind::Integer);
        assert_eq!(lexer.next_token().kind, super::token::Kind::RBracket);
        assert!(lexer.cursor.peek().is_none());
    }
}
