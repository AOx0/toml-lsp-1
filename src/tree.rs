use aoxo_toml::{span::Span, token};

pub struct Tree<'a> {
    pub kind: Kind,
    pub span: Span,
    pub children: Vec<Child<'a>, &'a bumpalo::Bump>,
}

impl core::fmt::Debug for Tree<'_> {
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

impl<'a> Tree<'a> {
    pub fn new(arena: &'a bumpalo::Bump) -> Self {
        Self {
            kind: Kind::Unkown,
            span: Span::from(0..0),
            children: Vec::new_in(arena),
        }
    }

    pub fn child(&mut self, child: Child<'a>) {
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

pub enum Child<'a> {
    Tree(Tree<'a>),
    Token(token::Token),
}

impl core::fmt::Debug for Child<'_> {
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
    ExtraDelimiter,
    Guard,
    Unkown,
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
                | Self::ExtraDelimiter
                | Self::Guard
                | Self::Unkown
        )
    }

    pub fn is_value(&self) -> bool {
        matches!(
            self,
            Self::String | Self::StringMulti | Self::Integer | Self::Float | Self::Bool
        )
    }
}
