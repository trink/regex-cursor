pub trait IntoCursor {
    type Cursor: Cursor;
    fn into_cursor(self) -> Self::Cursor;
}

impl<C: Cursor> IntoCursor for C {
    type Cursor = Self;

    fn into_cursor(self) -> Self {
        self
    }
}

impl<C: Cursor> Cursor for &mut C {
    fn chunk(&self) -> &[u8] {
        C::chunk(self)
    }

    fn utf8_aware(&self) -> bool {
        C::utf8_aware(self)
    }

    fn advance(&mut self) -> bool {
        C::advance(self)
    }

    fn backtrack(&mut self) -> bool {
        C::backtrack(self)
    }
}

impl<'h> IntoCursor for ropey::iter::Chunks<'h> {
    type Cursor = RopeyCursor<'h>;

    fn into_cursor(self) -> Self::Cursor {
        RopeyCursor::new(self)
    }
}

pub trait Cursor {
    fn chunk(&self) -> &[u8];
    /// Whether this cursor can be used for unicode/utf8 mode matching That
    /// means specifically that it promises that unicode codepoints are never
    /// split across chunk boundaries
    fn utf8_aware(&self) -> bool;
    fn advance(&mut self) -> bool;
    fn backtrack(&mut self) -> bool;
}

impl Cursor for &[u8] {
    fn chunk(&self) -> &[u8] {
        self
    }

    // true since there are no chunk bounderies
    fn utf8_aware(&self) -> bool {
        true
    }

    fn advance(&mut self) -> bool {
        false
    }

    fn backtrack(&mut self) -> bool {
        false
    }
}

impl Cursor for &str {
    fn chunk(&self) -> &[u8] {
        self.as_bytes()
    }

    // true since there are no chunk bounderies
    fn utf8_aware(&self) -> bool {
        true
    }

    fn advance(&mut self) -> bool {
        false
    }

    fn backtrack(&mut self) -> bool {
        false
    }
}

pub struct Bytes<'a, I> {
    iter: I,
    current: &'a [u8],
}

impl<'a, I: Iterator<Item = &'a [u8]>> Cursor for Bytes<'a, I> {
    fn chunk(&self) -> &[u8] {
        self.current
    }

    fn advance(&mut self) -> bool {
        for next in self.iter.by_ref() {
            if next.is_empty() {
                continue;
            }
            self.current = next;
            return true;
        }
        false
    }

    fn backtrack(&mut self) -> bool {
        false
    }

    fn utf8_aware(&self) -> bool {
        false
    }
}

pub struct Utf8Bytes<'a, I> {
    iter: I,
    current: &'a [u8],
}

impl<'a, I: Iterator<Item = &'a str>> Cursor for Utf8Bytes<'a, I> {
    fn chunk(&self) -> &[u8] {
        self.current
    }

    fn advance(&mut self) -> bool {
        for next in self.iter.by_ref() {
            if next.is_empty() {
                continue;
            }
            self.current = next.as_bytes();
            return true;
        }
        false
    }

    fn backtrack(&mut self) -> bool {
        false
    }

    fn utf8_aware(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy)]
enum Pos {
    ChunkStart,
    ChunkEnd,
}

#[derive(Clone)]
pub struct RopeyCursor<'a> {
    iter: ropey::iter::Chunks<'a>,
    current: &'a [u8],
    pos: Pos,
}

impl<'a> RopeyCursor<'a> {
    pub fn new(mut iter: ropey::iter::Chunks<'a>) -> Self {
        Self { current: iter.next().unwrap_or_default().as_bytes(), iter, pos: Pos::ChunkEnd }
    }
}

impl Cursor for RopeyCursor<'_> {
    fn chunk(&self) -> &[u8] {
        self.current
    }

    fn advance(&mut self) -> bool {
        match self.pos {
            Pos::ChunkStart => {
                self.iter.next();
                self.pos = Pos::ChunkEnd;
            }
            Pos::ChunkEnd => (),
        }
        for next in self.iter.by_ref() {
            if next.is_empty() {
                continue;
            }
            self.current = next.as_bytes();
            return true;
        }
        false
    }

    fn backtrack(&mut self) -> bool {
        match self.pos {
            Pos::ChunkStart => {}
            Pos::ChunkEnd => {
                self.iter.prev();
                self.pos = Pos::ChunkStart;
            }
        }
        while let Some(prev) = self.iter.prev() {
            if prev.is_empty() {
                continue;
            }
            self.current = prev.as_bytes();
            return true;
        }
        false
    }

    fn utf8_aware(&self) -> bool {
        true
    }
}
