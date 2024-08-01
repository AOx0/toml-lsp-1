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

#[derive(PartialEq, Eq, Clone, Copy)]
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

    DoubleLBracket,
    DoubleRBracket,

    // Errors
    NonClosingString,
    NonClosingMultilineString,
    Unknown,
    InvalidFloat,
}

impl std::fmt::Debug for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::Key => "Key",
            Self::StringOrKey => "StringOrKey",
            Self::StringMultiline => "StringMultiline",
            Self::Integer => "Integer",
            Self::Float => "Float",
            Self::Bool => "`true` or `false`",
            Self::Datetime => "Datetime",
            Self::Comma => "`,`",
            Self::Equal => "`=`",
            Self::LBracket => "`[`",
            Self::RBracket => "`]`",
            Self::DoubleLBracket => "`[[`",
            Self::DoubleRBracket => "`]]`",
            Self::LCurly => "`{`",
            Self::RCurly => "`}`",
            Self::Newline => "Newline",
            Self::Space => "Space",
            Self::Tab => "Tab",
            Self::Comment => "Comment",
            Self::Dot => "`.`",
            Self::Eof => "Eof",
            Self::NonClosingString => "NonClosingString",
            Self::NonClosingMultilineString => "NonClosingMultilineString",
            Self::Unknown => "Unknown",
            Self::InvalidFloat => "InvalidFloat",
        };
        write!(f, "{}", s)
    }
}

impl Kind {
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Self::Unknown
                | Self::InvalidFloat
                | Self::NonClosingString
                | Self::NonClosingMultilineString
        )
    }
}
