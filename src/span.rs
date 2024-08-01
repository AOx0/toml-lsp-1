use tower_lsp::lsp_types::Position;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl core::fmt::Debug for Span {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{start}..{end}", start = self.start, end = self.end)
    }
}

impl From<core::ops::Range<usize>> for Span {
    fn from(value: core::ops::Range<usize>) -> Self {
        Span {
            start: value.start,
            end: value.end,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

impl core::fmt::Display for Location {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{line}:{col}", line = self.line, col = self.col)
    }
}

impl Into<Position> for Location {
    fn into(self) -> Position {
        Position {
            line: (self.line - 1) as u32,
            character: (self.col - 1) as u32,
        }
    }
}

impl Span {
    pub fn reduce_to(&self, len: usize) -> Span {
        Span {
            start: self.start,
            end: self.end.min(self.start + len),
        }
    }

    pub fn end_location(&self, source: &str) -> Location {
        let mut line = 1;
        let mut col = 1;
        for (i, c) in source.chars().enumerate() {
            if i == self.end {
                return Location { line, col };
            }
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        Location { line, col }
    }

    pub fn start_location(&self, source: &str) -> Location {
        let mut line = 1;
        let mut col = 1;
        for (i, c) in source.chars().enumerate() {
            if i == self.start {
                return Location { line, col };
            }
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        Location { line, col }
    }
}
