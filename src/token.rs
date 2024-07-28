use crate::span::Span;

// #[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Token {
    pub span: Span,
    pub kind: Kind,
}

impl core::fmt::Debug for Token {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl core::default::Default for Token {
    fn default() -> Self {
        Self {
            span: Span::from(0..0),
            kind: Kind::Eof,
        }
    }
}

impl Token {
    pub fn new(span: Span, kind: Kind) -> Self {
        Self { span, kind }
    }

    pub fn eof() -> Self {
        Self::default()
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
    Key,
    StringOrKey,
    StringMultiline,
    Integer,
    Float,
    Bool,
    Datetime,
    Comma,
    Equal,
    LBracket,
    RBracket,
    LCurly,
    RCurly,
    Newline,
    Space,
    Tab,
    Comment,
    Dot,
    Eof,

    // Errors
    NonClosing,
    Unknown,
    InvalidFloat,
}

impl Kind {
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Unknown | Self::InvalidFloat | Self::NonClosing)
    }
}
