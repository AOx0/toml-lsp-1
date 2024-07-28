use aoxo_toml::token;

#[derive(Debug)]
pub struct Tree<'a> {
    pub kind: Kind,
    pub children: Vec<Child<'a>, &'a bumpalo::Bump>,
}

impl<'a> Tree<'a> {
    pub fn new(arena: &'a bumpalo::Bump) -> Self {
        Self {
            kind: Kind::Unkown,
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
}

#[derive(Debug)]
pub enum Child<'a> {
    Tree(Tree<'a>),
    Token(token::Token),
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
    Unkown,
}
