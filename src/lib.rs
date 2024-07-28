#![no_std]
#![feature(iter_next_chunk)]
#![feature(let_chains)]

pub mod cursor;
pub mod lexer;
pub mod span;
pub mod token;

pub trait Slice {
    type Item;

    fn length(&self) -> usize;
    fn get_idx(&self, idx: usize) -> Option<Self::Item>;
    fn get_chunk<const N: usize>(&self, idx: usize) -> Option<[Self::Item; N]>;
}

impl<T: Copy> Slice for &[T] {
    type Item = T;

    fn length(&self) -> usize {
        self.len()
    }

    fn get_idx(&self, idx: usize) -> Option<Self::Item> {
        self.get(idx).copied()
    }

    fn get_chunk<const N: usize>(&self, idx: usize) -> Option<[Self::Item; N]> {
        self.get(idx..)
            .map(|s| s.iter().copied().next_chunk::<N>().ok())
            .flatten()
    }
}

impl Slice for &str {
    type Item = char;

    fn length(&self) -> usize {
        self.len()
    }

    fn get_idx(&self, idx: usize) -> Option<Self::Item> {
        self.chars().nth(idx)
    }

    fn get_chunk<const N: usize>(&self, idx: usize) -> Option<[Self::Item; N]> {
        self.get(idx..)
            .map(|s| s.chars().next_chunk::<N>().ok())
            .flatten()
    }
}
