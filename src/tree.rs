use crate::{span::Span, token};

pub struct Tree {
    pub kind: Kind,
    pub span: Span,
    pub children: Vec<Child>,
}

impl core::fmt::Debug for Tree {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if !self.children.is_empty() {
            f.debug_struct("Tree")
                .field("kind", &self.kind)
                .field("span", &self.span)
                .field("children", &self.children)
                .finish()
        } else {
            f.debug_struct("Tree")
                .field("kind", &self.kind)
                .field("span", &self.span)
                .finish()
        }
    }
}

impl Tree {
    pub fn new() -> Self {
        Self {
            kind: Kind::Unknown,
            span: Span::from(0..0),
            children: Vec::new(),
        }
    }

    pub fn child(&mut self, child: Child) {
        self.children.push(child);
    }

    pub fn with_kind(mut self, kind: Kind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    pub fn span(&mut self, span: Span) {
        self.span.start = core::cmp::min(self.span.start, span.start);
        self.span.end = core::cmp::max(self.span.end, span.end);
    }
}

pub enum Child {
    Tree(Tree),
    Token(token::Token),
}

impl core::fmt::Debug for Child {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Child::Tree(tree) => write!(f, "{:#?}", tree),
            Child::Token(token) => write!(f, "{:?}", token),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
    Table,
    TableArray,
    Array,
    InlineTable,
    KeyVal,
    Key,
    Toml,
    String,
    StringMulti,
    Integer,
    Float,
    Bool,

    // Collections
    KeyValList,

    // Errors
    MissingKey,
    MissingValue,
    Extra(token::Kind),
    Guard,
    Unknown,
    Expected(token::Kind),
    ExpectedAny(&'static [token::Kind]),
    UnclosedString,
    InvalidToken,
    Forbidden(token::Kind),
}

impl Kind {
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::MissingKey | Self::MissingValue)
    }

    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Self::MissingKey
                | Self::MissingValue
                | Self::Extra(_)
                | Self::Guard
                | Self::Unknown
                | Self::Expected(_)
                | Self::ExpectedAny(_)
                | Self::UnclosedString
        )
    }

    pub fn is_value(&self) -> bool {
        matches!(
            self,
            Self::String | Self::StringMulti | Self::Integer | Self::Float | Self::Bool
        )
    }
}
