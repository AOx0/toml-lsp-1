use crate::tree;
use aoxo_toml::{lexer::Lexer, span::Span, token};
use std::{cell::Cell, u8};

mod grammar;

#[derive(Debug)]
pub struct Error {
    pub span: Span,
    pub kind: tree::Kind,
}

#[derive(Debug)]
pub struct Parser<'src> {
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
    Skip { token: aoxo_toml::token::Token },
    Open { kind: tree::Kind, span: Span },
}

struct MarkOpen {
    index: usize,
}

enum Status {
    Success,
    Failure,
}

impl From<bool> for Status {
    fn from(b: bool) -> Self {
        if b {
            Status::Success
        } else {
            Status::Failure
        }
    }
}

impl Status {
    fn success(&self) -> bool {
        matches!(self, Status::Success)
    }
    fn failed(&self) -> bool {
        matches!(self, Status::Failure)
    }
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
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

    fn add_error(&mut self, kind: tree::Kind) {
        self.errors.push(Error {
            span: self.lexer.peek_span::<0>().reduce_to(1),
            kind,
        })
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
        self.events.push(Event::Skip {
            token: self.lexer.next_token(),
        });
    }

    fn advance_with_error(&mut self, kind: tree::Kind) {
        let mark = self.open();
        self.add_error(kind);
        self.advance();
        self.close(mark, kind);
    }

    pub fn peek_kind(&self) -> aoxo_toml::token::Kind {
        #[cfg(debug_assertions)]
        if self.fuel.get() == 0 {
            panic!(
                "parser is stuck at {} with token {:?}",
                self.lexer
                    .peek_span::<0>()
                    .start_location(self.lexer.source()),
                self.lexer.peek_kind::<0>()
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

    fn advance_if(&mut self, kind: aoxo_toml::token::Kind) -> Status {
        self.advance_if_any(&[kind])
    }

    fn skip_if(&mut self, kind: aoxo_toml::token::Kind) -> Status {
        self.skip_if_any(&[kind])
    }

    fn expect(&mut self, kind: aoxo_toml::token::Kind) {
        if self.advance_if(kind).failed() {
            let m = self.open();
            self.close(m, tree::Kind::Expected(kind));
        }
    }

    fn skip_expect(&mut self, kind: aoxo_toml::token::Kind) {
        if self.skip_if(kind).failed() {
            let m = self.open();
            self.add_error(tree::Kind::Expected(kind));
            self.close(m, tree::Kind::Expected(kind));
        }
    }

    fn advance_if_any(&mut self, kinds: &[aoxo_toml::token::Kind]) -> Status {
        if kinds.contains(&self.peek_kind()) {
            self.advance();
            Status::Success
        } else {
            Status::Failure
        }
    }

    fn skip_if_any(&mut self, kinds: &[aoxo_toml::token::Kind]) -> Status {
        if kinds.contains(&self.peek_kind()) {
            self.skip();
            Status::Success
        } else {
            Status::Failure
        }
    }

    fn eof(&self) -> bool {
        self.lexer.peek_kind::<0>() == aoxo_toml::token::Kind::Eof
    }

    pub fn parse(mut self) -> Self {
        grammar::toml(&mut self);
        self
    }

    pub fn tree(mut self) -> (tree::Tree, Vec<Error>) {
        let mut stack: Vec<tree::Tree> = Vec::new();

        assert!(matches!(self.events.pop(), Some(Event::Close)));

        for event in self.events.iter().copied() {
            match event {
                Event::Open { kind, span } => {
                    stack.push(tree::Tree::new().with_kind(kind).with_span(span));
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
                Event::Skip { token } => {
                    if token.kind != token::Kind::Eof && token.kind != token::Kind::Newline {
                        stack.last_mut().unwrap().span(token.span);
                    }
                }
            }
        }

        assert!(stack.len() == 1, "stack is not empty {:?}", stack);
        assert_eq!(self.lexer.peek_kind::<0>(), aoxo_toml::token::Kind::Eof);

        (stack.pop().unwrap(), self.errors)
    }
}
