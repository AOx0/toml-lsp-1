#![feature(allocator_api)]

mod parser;
mod tree;

fn main() {
    let file = std::env::args().nth(1).unwrap();
    let source = std::fs::read_to_string(&file).unwrap();

    let arena = bumpalo::Bump::new();

    let mut parser = parser::Parser::new(&source, &arena);
    parser.parse();
    let tree = parser.tree();

    println!("{:#?}", tree);
}
