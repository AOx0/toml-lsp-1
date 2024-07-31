use crate::tree;
use aoxo_toml::{lexer::Lexer, span::Span};
use std::{cell::Cell, u8};

mod grammar;

#[derive(Debug)]
pub struct Error {
    pub span: Span,
    pub kind: tree::Kind,
}

#[derive(Debug)]
pub struct Parser<'src, 'a> {
    arena: &'a bumpalo::Bump,
    lexer: Lexer<'src>,
    events: Vec<Event>,
    #[cfg(debug_assertions)]
    fuel: Cell<u8>,
    errors: Vec<Error>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Event {
    Close,
    Advance { token: aoxo_toml::token::Token },
    Skip,
    Open { kind: tree::Kind, span: Span },
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
            errors: Vec::new(),
        }
    }

    fn open(&mut self) -> MarkOpen {
        self.events.push(Event::Open {
            kind: tree::Kind::Unkown,
            span: self.lexer.peek_span::<0>(),
        });
        MarkOpen {
            index: self.events.len() - 1,
        }
    }

    fn close(&mut self, mark: MarkOpen, kind: tree::Kind) {
        self.events.push(Event::Close);
        let last = self.events[mark.index];
        if let Event::Open { span, .. } = last {
            self.events[mark.index] = Event::Open { kind, span };
        } else {
            panic!("invalid mark")
        }
    }

    fn advance(&mut self) {
        assert!(!self.eof());
        #[cfg(debug_assertions)]
        self.fuel.set(u8::MAX);
        self.events.push(Event::Advance {
            token: self.lexer.next_token(),
        });
    }

    fn skip(&mut self) {
        assert!(!self.eof());
        #[cfg(debug_assertions)]
        self.fuel.set(u8::MAX);
        self.events.push(Event::Skip);
    }

    fn advance_with_error(&mut self, kind: tree::Kind) {
        let mark = self.open();
        self.errors.push(Error {
            span: self.lexer.peek_span::<0>(),
            kind,
        });
        self.advance();
        self.close(mark, kind);
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

    fn skip_if(&mut self, kind: aoxo_toml::token::Kind) -> bool {
        self.skip_if_any(&[kind])
    }

    fn expect(&mut self, kind: aoxo_toml::token::Kind) {
        if !self.advance_if(kind) {
            let m = self.open();
            self.close(m, tree::Kind::Expected(kind));
        }
    }

    fn skip_expect(&mut self, kind: aoxo_toml::token::Kind) {
        if !self.skip_if(kind) {
            let m = self.open();
            self.close(m, tree::Kind::Expected(kind));
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

    fn skip_if_any(&mut self, kinds: &[aoxo_toml::token::Kind]) -> bool {
        if kinds.contains(&self.peek_kind()) {
            self.skip();
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

    pub fn tree(mut self) -> (tree::Tree<'a>, Vec<Error>) {
        let mut stack: Vec<tree::Tree<'a>> = Vec::new();

        assert!(matches!(self.events.pop(), Some(Event::Close)));

        for event in self.events.iter().copied() {
            match event {
                Event::Open { kind, span } => {
                    stack.push(tree::Tree::new(self.arena).with_kind(kind).with_span(span));
                }
                Event::Close => {
                    let tree = stack.pop().unwrap();
                    stack.last_mut().unwrap().span(tree.span);
                    stack.last_mut().unwrap().child(tree::Child::Tree(tree));
                }
                Event::Advance { token } => {
                    stack.last_mut().unwrap().span(token.span);
                    stack.last_mut().unwrap().child(tree::Child::Token(token));
                }
                Event::Skip => {}
            }
        }

        assert!(stack.len() == 1);
        assert_eq!(self.lexer.peek_kind::<0>(), aoxo_toml::token::Kind::Eof);

        (stack.pop().unwrap(), self.errors)
    }
}
