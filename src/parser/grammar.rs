use super::{Parser, Status};
use crate::token::{self, Kind::*};
use crate::tree;

const TABLE_FOLLOW: &[token::Kind] = &[StringOrKey, Key, LBracket];

struct Advanced;

impl From<Status> for Advanced {
    fn from(value: Status) -> Self {
        if matches!(value, Status::Advanced) {
            Self
        } else {
            panic!()
        }
    }
}

pub fn ignore_until(p: &mut Parser, matches: &[token::Kind]) {
    while !p.eof() && !matches.contains(&p.peek_kind()) {
        p.ignore();
    }
}

// Toml = Expr*
pub fn toml(p: &mut Parser) {
    let mark = p.open();

    while !p.eof() {
        expr(p);
    }

    p.close(mark, tree::Kind::Toml);
}

fn maybe_key(p: &Parser) -> bool {
    p.next_is(StringOrKey) || p.next_is(Key)
}

// Expr =
//       TableArray
//     | Table
//     | KeyVal
fn expr(p: &mut Parser) -> Advanced {
    if p.next_are([LBracket, LBracket]) {
        table_array(p)
    } else if p.next_is(LBracket) {
        table(p)
    } else if maybe_key(p) {
        key_val(p)
    } else {
        p.ignore().into()
    }
}

// TableArray = '[[' Key ']]' '\n' (KeyVal '\n')*
fn table_array(p: &mut Parser) -> Advanced {
    let mark = p.open();

    p.skip_expect(LBracket);
    p.skip_expect(LBracket);

    if maybe_key(p) {
        key(p);
    } else {
        p.add_error(tree::Kind::MissingKey);
        ignore_until(p, TABLE_FOLLOW); // Recover
        p.close(mark, tree::Kind::TableArray);
        return Advanced; // We did advance at least [LBracket, LBracket]
    }

    if p.next_are([RBracket, RBracket]) {
        p.skip_expect(RBracket);
        p.skip_expect(RBracket);
    } else if p.next_is(RBracket) {
        p.add_error(tree::Kind::Expected(RBracket));
        p.skip();
    } else {
        p.add_error(tree::Kind::Expected(DoubleRBracket));
    }

    p.skip_expect(Newline);

    while maybe_key(p) {
        key_val(p);
        p.skip_expect(Newline);
    }

    p.close(mark, tree::Kind::TableArray);

    Advanced // We did advance at least [LBracket, LBracket]
}

// Table = '[' Key ']' '\n' (KeyVal '\n')*
fn table(p: &mut Parser) -> Advanced {
    let mark = p.open();

    p.skip_expect(LBracket);

    if maybe_key(p) {
        key(p);
    } else {
        p.add_error(tree::Kind::MissingKey);
        ignore_until(p, TABLE_FOLLOW); // Recover
        p.close(mark, tree::Kind::Table);
        return Advanced; // We did advance at least LBracket
    }

    p.skip_expect(RBracket);

    p.skip_expect(Newline);

    while maybe_key(p) {
        key_val(p);
        p.skip_expect(Newline);
    }

    p.close(mark, tree::Kind::Table);

    Advanced // We did advance at least LBracket
}

// KeyVal = Key '=' Value
fn key_val(p: &mut Parser) -> Advanced {
    debug_assert!(maybe_key(p));
    let mark = p.open();

    key(p);
    p.skip_expect(Equal);
    value(p);

    p.close(mark, tree::Kind::KeyVal);

    Advanced // We did advance at least the key
}

// Key = KeyPart ('.' KeyPart)*
fn key(p: &mut Parser) {
    let mark = p.open();

    key_part(p);

    while p.next_is(Dot) {
        p.skip();
        key_part(p);
    }

    p.close(mark, tree::Kind::Key);
}

// KeyPart = 'str_key' | 'key'
fn key_part(p: &mut Parser) {
    if p.next_is(StringOrKey) || p.next_is(Key) {
        p.advance();
    } else {
        p.add_error(tree::Kind::MissingKey);
    }
}

// Value =
//       'string'
//     | 'number'
//     | 'bool'
//     | Array
//     | TableInline
fn value(p: &mut Parser) {
    if p.next_is(StringOrKey) | p.next_is(StringMultiline) {
        p.advance();
    } else if p.next_is(Integer) | p.next_is(Float) {
        p.advance();
    } else if p.next_is(Bool) {
        p.advance();
    } else if p.next_is(LBracket) {
        array(p);
    } else if p.next_is(LCurly) {
        table_inline(p);
    } else {
        p.add_error(tree::Kind::MissingValue);
    }
}

// Array = '[' (Value ( ',' | '\n' ))* ']'
fn array(p: &mut Parser) {
    let mark = p.open();

    p.skip_expect(LBracket);

    while !p.eof() && !p.next_is(RBracket) {
        value(p);

        if p.next_is(Comma) {
            p.skip();
        }

        if p.next_is(Newline) {
            p.skip();
        }

        if p.next_is(RBracket) {
            break;
        }
    }

    p.skip_expect(RBracket);

    p.close(mark, tree::Kind::Array);
}

// TableInline = '{' KeyVal? (',' KeyVal)* '}'
fn table_inline(p: &mut Parser) {
    let mark = p.open();

    p.skip_expect(LCurly);

    if maybe_key(p) {
        key_val(p);
    }

    while p.next_is(Comma) {
        p.skip();
        key_val(p);
    }

    p.skip_expect(RCurly);

    p.close(mark, tree::Kind::InlineTable);
}
