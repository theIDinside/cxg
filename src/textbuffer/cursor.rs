use super::metadata::{Column, Index, Line};
use std::cmp::Ordering;

#[derive(Default, Debug, Copy, Clone)]
pub struct BufferCursor {
    /// Absolute index into buffer
    pub pos: Index,
    pub row: Line,
    pub col: Column,
}

impl Into<BufferCursor> for (usize, usize, usize) {
    #[inline(always)]
    fn into(self) -> BufferCursor {
        let (pos, row, col) = self;
        BufferCursor { pos: Index(pos), row: Line(row), col: Column(col) }
    }
}

impl PartialEq for BufferCursor {
    fn ne(&self, other: &Self) -> bool {
        self.pos != other.pos
    }

    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl Eq for BufferCursor {}

impl Ord for BufferCursor {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pos.cmp(&other.pos)
    }
}

impl PartialOrd for BufferCursor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub enum CursorMovement {
    Valid,
    InvalidColumn,
}

impl BufferCursor {
    pub fn absolute(&self) -> Index {
        self.pos
    }
    pub fn set_pos(&mut self, pos: Index) {
        self.pos = pos;
    }
    pub fn set_row(&mut self, row: Line) {
        self.row = row;
    }
    pub fn set_col(&mut self, col: Column) {
        self.col = col;
    }
}
