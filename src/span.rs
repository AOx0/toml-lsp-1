#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
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

impl Span {
    pub fn location(&self, source: &str) -> Location {
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
