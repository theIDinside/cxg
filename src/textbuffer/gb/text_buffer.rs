#![allow(unused)]
/// Text data type that uses a GapBuffer as backing store

use std::ops::Index;
use std::ops::Range;
use std::ptr::copy as copyrange;
use crate::textbuffer::{metadata, metadata::MetaData, CharBuffer, cursor::BufferCursor};
use crate::utils::copy_slice_to;

use super::gap_buffer::{GapBuffer, GapBufferIterator};



type TextGapBuffer = GapBuffer<char>;
type TextBufferIterator<'a> = GapBufferIterator<'a, char>;
pub struct TextBuffer {
    data: TextGapBuffer,
    meta_data: MetaData,
    cursor: BufferCursor,
    size: usize,
}

impl TextBuffer {

}

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

    fn str_view(&'a self, range: std::ops::Range<usize>) -> std::iter::Take<std::iter::Skip<Self::ItemIterator>> {
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
}


