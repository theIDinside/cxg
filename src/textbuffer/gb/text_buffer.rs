/// Text data type that uses a GapBuffer as backing store
use super::gap_buffer::{GapBuffer, GapBufferIterator};
use crate::textbuffer::{cursor::BufferCursor, metadata, metadata::MetaData, CharBuffer};

type TextGapBuffer = GapBuffer<char>;
type TextBufferIterator<'a> = GapBufferIterator<'a, char>;
#[allow(unused)]
pub struct TextBuffer {
    data: TextGapBuffer,
    meta_data: MetaData,
    cursor: BufferCursor,
    size: usize,
}

impl std::hash::Hash for TextBuffer {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        todo!()
    }
}

impl TextBuffer {}

#[allow(unused)]
impl<'a> CharBuffer<'a> for TextBuffer {
    type ItemIterator = TextBufferIterator<'a>;

    fn insert(&mut self, data: char) {
        self.data.insert_item(data);
    }

    fn delete(&mut self, dir: crate::textbuffer::Movement) {
        todo!()
    }

    fn insert_slice_fast(&mut self, slice: &[char]) {
        self.data.insert_slice(slice);
    }

    fn move_cursor(&mut self, dir: crate::textbuffer::Movement) {
        todo!()
    }

    fn capacity(&self) -> usize {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn rebuild_metadata(&mut self) {
        todo!()
    }

    fn meta_data(&self) -> &MetaData {
        todo!()
    }

    fn iter(&'a self) -> Self::ItemIterator {
        todo!()
    }

    fn cursor_row(&self) -> metadata::Line {
        todo!()
    }

    fn cursor_col(&self) -> metadata::Column {
        todo!()
    }

    fn cursor_abs(&self) -> metadata::Index {
        todo!()
    }

    fn set_cursor(&mut self, cursor: BufferCursor) {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }

    fn load_file(&mut self, path: &std::path::Path) {
        todo!()
    }

    fn save_file(&mut self, path: &std::path::Path) {
        todo!()
    }

    fn file_name(&self) -> Option<&std::path::Path> {
        todo!()
    }

    fn copy(&mut self, range: std::ops::Range<usize>) -> String {
        todo!()
    }

    fn select_move_cursor_absolute(&mut self, movement: crate::textbuffer::Movement) {
        todo!()
    }

    fn goto_line(&mut self, line: usize) {
        todo!()
    }

    fn line_operation<RangeType>(&mut self, lines: RangeType, op: crate::textbuffer::LineOperation)
    where
        RangeType: std::ops::RangeBounds<usize> + std::slice::SliceIndex<[metadata::Index], Output = [metadata::Index]> + Clone + std::ops::RangeBounds<usize>,
    {
        todo!()
    }
}
