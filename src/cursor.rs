use crate::Slice;

#[derive(Debug)]
pub struct Cursor<'src, Items: ?Sized + 'src> {
    slice: &'src Items,
    cursor: usize,
}

impl<Item, Items: ?Sized> Cursor<'_, Items>
where
    Item: PartialEq,
    for<'a> &'a Items: Slice<Item = Item>,
{
    pub fn source(&self) -> &Items {
        self.slice
    }

    pub fn matches(&self, other: Item) -> bool {
        self.peek().is_some_and(|c| c == other)
    }

    pub fn peek(&self) -> Option<Item> {
        self.slice.get_idx(self.cursor)
    }

    pub fn peek_ahead(&self, n: usize) -> Option<Item> {
        self.slice.get_idx(self.cursor + n)
    }

    pub fn peek_chunk<const SIZE: usize>(&self) -> Option<[Item; SIZE]> {
        self.slice.get_chunk::<SIZE>(self.cursor)
    }

    pub fn bump(&mut self) {
        self.cursor += 1;
    }

    pub fn bump_n(&mut self, n: usize) {
        self.cursor += n;
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }
}

impl<'src, T: ?Sized> Cursor<'src, T> {
    pub fn new(slice: &'src T) -> Self {
        Self { slice, cursor: 0 }
    }
}
