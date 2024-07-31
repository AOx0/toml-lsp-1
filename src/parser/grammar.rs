use super::{Error, Parser};
use crate::tree;
use aoxo_toml::token::{self, Kind::*};

const TABLE_FOLLOW: [token::Kind; 3] = [StringOrKey, Key, LBracket];

// Toml = Expr*
pub fn toml(p: &mut Parser) {
    let mark = p.open();
    while !p.eof() {
        expr(p);
    }

    p.close(mark, tree::Kind::Toml);
}

// Expr = TableArray | Table | KeyValueDecl
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
        p.skip_expect(Newline);
    }
}

// Table = '[' Key ']' '\n' (KeyVal '\n')*
fn table(p: &mut Parser) {
    p.skip_expect(LBracket);
    key(p);
    if guard(p, &[Newline], &TABLE_FOLLOW) {
        return;
    }
    p.skip_expect(RBracket);
    p.skip_expect(Newline);

    let mark = p.open();
    while !p.next_is(LBracket) && !p.next_is(Eof) {
        keyval(p);
        p.skip_expect(Newline);
    }

    p.close(mark, tree::Kind::KeyValList);
}

fn guard(
    p: &mut Parser,
    forbid: &[aoxo_toml::token::Kind],
    advance_until: &[aoxo_toml::token::Kind],
) -> bool {
    if forbid.contains(&p.peek_kind()) {
        let mark = p.open();

        while !advance_until.contains(&p.peek_kind()) {
            p.advance();
        }

        p.close(mark, tree::Kind::Guard);
        true
    } else {
        false
    }
}

// TableArray = "[[" Keys "]] '\n'+ KeyVal*
fn table_array(p: &mut Parser) {
    p.skip_expect(LBracket);
    p.skip_expect(LBracket);
    while p.next_is(LBracket) {
        p.advance_with_error(tree::Kind::Extra(LBracket));
    }

    key(p);
    if guard(p, &[Newline], &[StringOrKey, Key, LBracket]) {
        return;
    }
    p.skip_expect(RBracket);
    p.skip_expect(RBracket);
    while p.next_is(RBracket) {
        p.advance_with_error(tree::Kind::Extra(RBracket));
    }

    p.skip_expect(Newline);

    let mark = p.open();
    while !p.next_is(LBracket) && !p.next_is(Eof) {
        keyval(p);
        p.skip_expect(Newline);
    }

    p.close(mark, tree::Kind::KeyValList);
}

// Key = (StringOrKey | Key) ('.' (StringOrKey | Key))*
fn key(p: &mut Parser) {
    let mark = p.open();
    if p.skip_expect_any(&[StringOrKey, Key]).failed() {
        p.close(mark, tree::Kind::MissingKey);
        return;
    }

    while p.advance_if(Dot).success() {
        p.advance_if_any(&[StringOrKey, Key]);
    }

    p.close(mark, tree::Kind::Key);
}

// KeyVal = Key '=' Value
fn keyval(p: &mut Parser) {
    let mark = p.open();
    key(p);
    p.skip_expect(Equal);
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
        p.errors.push(Error {
            kind: tree::Kind::MissingValue,
            span: p.lexer.peek_span::<0>(),
        });
        p.close(mark, tree::Kind::MissingValue);
    }
}

// Array = "[" (Value(,|\n)+)* "]"
fn array(p: &mut Parser) {
    p.skip_expect(LBracket);
    p.skip_if(Newline);

    while !p.next_is(RBracket) && !p.next_is(Eof) {
        value(p);
        if p.peek_kind() == Comma {
            p.advance();
        } else if p.peek_kind() == Newline {
            p.advance();
        }

        while matches!(p.peek_kind(), Newline | Comma) {
            p.errors.push(Error {
                kind: tree::Kind::Extra(Newline),
                span: p.lexer.peek_span::<0>(),
            });
            p.advance();
        }
    }
    p.expect(RBracket);
}

// TableInline = "{" KeyVal,* "}"
fn table_inline(p: &mut Parser) {
    p.skip_expect(LCurly);
    while !p.next_is(RCurly) && !p.next_is(Eof) {
        keyval(p);
        while p.advance_if(Comma).success() {}
    }
    p.skip_expect(RCurly);
}
