use std::{cell::Cell, u8};

use aoxo_toml::lexer::Lexer;

use crate::tree;

#[derive(Debug)]
pub struct Parser<'src, 'a> {
    arena: &'a bumpalo::Bump,
    lexer: Lexer<'src>,
    events: Vec<Event>,
    #[cfg(debug_assertions)]
    fuel: Cell<u8>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Event {
    Close,
    Advance { token: aoxo_toml::token::Token },
    Open { kind: tree::Kind },
}

struct MarkOpen {
    index: usize,
}

impl<'src, 'a> Parser<'src, 'a> {
    pub fn new(source: &'src str, arena: &'a bumpalo::Bump) -> Self {
        Self {
            arena,
            lexer: Lexer::new(source),
            events: Vec::with_capacity(15),
            #[cfg(debug_assertions)]
            fuel: Cell::new(u8::MAX),
        }
    }

    fn open(&mut self) -> MarkOpen {
        self.events.push(Event::Open {
            kind: tree::Kind::Unkown,
        });
        MarkOpen {
            index: self.events.len() - 1,
        }
    }

    fn close(&mut self, mark: MarkOpen, kind: tree::Kind) {
        self.events.push(Event::Close);
        self.events[mark.index] = Event::Open { kind };
    }

    fn advance(&mut self) {
        assert!(!self.eof());
        #[cfg(debug_assertions)]
        self.fuel.set(u8::MAX);
        self.events.push(Event::Advance {
            token: self.lexer.next_token(),
        });
    }

    pub fn peek_kind(&self) -> aoxo_toml::token::Kind {
        #[cfg(debug_assertions)]
        if self.fuel.get() == 0 {
            panic!(
                "parser is stuck at {}",
                self.lexer.peek_span::<0>().location(self.lexer.source())
            )
        }
        self.lexer.peek_kind::<0>()
    }

    fn next_are<const N: usize>(&self, kinds: [aoxo_toml::token::Kind; N]) -> bool {
        #[cfg(debug_assertions)]
        {
            assert!(self.fuel.get() > 0);
            self.fuel
                .set(self.fuel.get().saturating_sub(N.try_into().unwrap()));
        }
        self.lexer.peek_kind_array::<N>() == kinds
    }

    fn next_is(&self, kind: aoxo_toml::token::Kind) -> bool {
        #[cfg(debug_assertions)]
        {
            assert!(self.fuel.get() > 0);
            self.fuel.set(self.fuel.get().saturating_sub(1));
        }
        self.peek_kind() == kind
    }

    fn advance_if(&mut self, kind: aoxo_toml::token::Kind) -> bool {
        self.advance_if_any(&[kind])
    }

    fn expect(&mut self, kind: aoxo_toml::token::Kind) {
        if !self.advance_if(kind) {
            eprintln!(
                "expected {:?} at {}",
                kind,
                self.lexer.peek_span::<0>().location(self.lexer.source())
            );
            self.advance();
        }
    }

    fn advance_if_any(&mut self, kinds: &[aoxo_toml::token::Kind]) -> bool {
        if kinds.contains(&self.peek_kind()) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn eof(&self) -> bool {
        self.lexer.peek_kind::<0>() == aoxo_toml::token::Kind::Eof
    }

    pub fn parse(&mut self) {
        grammar::toml(self)
    }

    pub fn tree(mut self) -> tree::Tree<'a> {
        let mut stack: Vec<tree::Tree<'a>> = Vec::new();

        assert!(matches!(self.events.pop(), Some(Event::Close)));

        for event in self.events.iter().copied() {
            match event {
                Event::Open { kind } => {
                    stack.push(tree::Tree::new(self.arena).with_kind(kind));
                }
                Event::Close => {
                    let tree = stack.pop().unwrap();
                    stack.last_mut().unwrap().child(tree::Child::Tree(tree));
                }
                Event::Advance { token } => {
                    stack.last_mut().unwrap().child(tree::Child::Token(token))
                }
            }
        }

        assert!(stack.len() == 1);
        assert_eq!(self.lexer.peek_kind::<0>(), aoxo_toml::token::Kind::Eof);
        stack.pop().unwrap()
    }
}

mod grammar {
    use super::Parser;
    use crate::tree;
    use aoxo_toml::token::Kind::*;

    // Toml = Expr*
    pub fn toml(p: &mut Parser) {
        let mark = p.open();
        while !p.eof() {
            expr(p);
        }

        p.close(mark, tree::Kind::Toml);
    }

    // Expr =  TableArray | Table | KeyValueDecl
    fn expr(p: &mut Parser) {
        if p.next_are([LBracket, LBracket]) {
            let mark = p.open();
            table_array(p);
            p.close(mark, tree::Kind::TableArray);
        } else if p.next_is(LBracket) {
            let mark = p.open();
            table(p);
            p.close(mark, tree::Kind::Table);
        } else {
            keyval(p);
            p.expect(Newline);
            while p.advance_if(Newline) {}
        }
    }

    fn new_lines(p: &mut Parser) {
        if p.next_is(Eof) {
            return;
        }
        while p.advance_if(Newline) {}
    }

    // Table = "[" Keys "]" '\n'+ (KeyVal '\n'+)*
    fn table(p: &mut Parser) {
        p.expect(LBracket);
        key(p);
        p.expect(RBracket);
        new_lines(p);

        let mark = p.open();
        while !p.next_is(LBracket) && !p.next_is(Eof) {
            keyval(p);
            new_lines(p);
        }

        p.close(mark, tree::Kind::KeyValList);
    }

    // TableArray = "[[" Keys "]] '\n'+ KeyVal*
    fn table_array(p: &mut Parser) {
        p.expect(LBracket);
        p.expect(LBracket);
        key(p);
        p.expect(RBracket);
        p.expect(RBracket);

        p.expect(Newline);
        while p.advance_if(Newline) {}

        let mark = p.open();
        while !p.next_is(LBracket) && !p.next_is(Eof) {
            keyval(p);
            new_lines(p);
        }

        p.close(mark, tree::Kind::KeyValList);
    }

    fn key(p: &mut Parser) {
        let mark = p.open();
        let error = !p.advance_if_any(&[StringOrKey, Key]);
        while p.advance_if(Dot) {
            p.advance_if_any(&[StringOrKey, Key]);
        }

        if error {
            p.close(mark, tree::Kind::MissingKey);
        } else {
            p.close(mark, tree::Kind::Key);
        }
    }

    // KeyVal = Keys '=' Value
    fn keyval(p: &mut Parser) {
        let mark = p.open();
        key(p);
        p.expect(Equal);
        value(p);
        p.close(mark, tree::Kind::KeyVal);
    }

    // Value = String | Number | Bool | Array | TableInline
    fn value(p: &mut Parser) {
        let mark = p.open();
        if p.next_is(StringOrKey) {
            p.advance();
            p.close(mark, tree::Kind::String);
        } else if p.next_is(StringMultiline) {
            p.advance();
            p.close(mark, tree::Kind::StringMulti);
        } else if p.next_is(Float) {
            p.advance();
            p.close(mark, tree::Kind::Float);
        } else if p.next_is(Integer) {
            p.advance();
            p.close(mark, tree::Kind::Integer);
        } else if p.next_is(Bool) {
            p.advance();
            p.close(mark, tree::Kind::Bool);
        } else if p.next_is(LBracket) {
            array(p);
            p.close(mark, tree::Kind::Array);
        } else if p.next_is(LCurly) {
            table_inline(p);
            p.close(mark, tree::Kind::InlineTable);
        } else {
            p.close(mark, tree::Kind::MissingValue);
        }
    }

    // Array = "[" (Value(,|\n)+)* "]"
    fn array(p: &mut Parser) {
        p.expect(LBracket);
        if p.next_is(Newline) {
            new_lines(p);
        }
        while !p.next_is(RBracket) && !p.next_is(Eof) {
            value(p);
            while p.next_is(Comma) || p.next_is(Newline) {
                p.advance();
            }
        }
        p.expect(RBracket);
    }

    // TableInline = "{" KeyVal,* "}"
    fn table_inline(p: &mut Parser) {
        p.expect(LCurly);
        while !p.next_is(RCurly) && !p.next_is(Eof) {
            keyval(p);
            while p.advance_if(Comma) {}
        }
        p.expect(RCurly);
    }
}
