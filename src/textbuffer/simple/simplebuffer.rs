use std::{
    cmp::min,
    io::{Read, Write},
    path::Path,
};

use super::super::{cursor::BufferCursor, CharBuffer, Movement};
use crate::{
    debugger_catch, only_in_debug,
    textbuffer::{
        metadata::{self, calculate_hash},
        TextKind,
    },
    utils::{copy_slice_to, AsUsize},
};

#[cfg(debug_assertions)]
use crate::DebuggerCatch;

pub enum OperationParameter {
    Char(char),
    Range(String),
}

pub enum Operation {
    Insert(metadata::Index, OperationParameter),
    Delete(metadata::Index, OperationParameter),
}

pub struct SimpleBuffer {
    pub id: u32,
    pub data: Vec<char>,
    edit_cursor: BufferCursor,
    cursor_range_end: Option<metadata::Index>,
    size: usize,
    meta_data: metadata::MetaData,
}

impl std::hash::Hash for SimpleBuffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl SimpleBuffer {
    pub fn new(id: u32, capacity: usize) -> SimpleBuffer {
        SimpleBuffer {
            id: id,
            data: Vec::with_capacity(capacity),
            edit_cursor: BufferCursor::default(),
            cursor_range_end: None,
            size: 0,
            meta_data: metadata::MetaData::new(None),
        }
    }

    pub fn buffer_info(&self) -> (Option<&Path>, BufferCursor) {
        (self.file_name(), self.cursor())
    }

    pub fn cursor(&self) -> BufferCursor {
        self.edit_cursor.clone()
    }

    pub fn get(&self, idx: metadata::Index) -> Option<&char> {
        self.data.get(*idx)
    }

    pub fn get_unchecked(&self, idx: metadata::Index) -> &char {
        unsafe { self.data.get_unchecked(*idx) }
    }

    pub fn get_slice(&self, range: std::ops::Range<usize>) -> &[char] {
        debugger_catch!(
            range.start <= self.len() && range.end <= self.len(),
            DebuggerCatch::Handle(format!("Illegal access of buffer; getting range {:?} from buffer of only {} len", range.clone(), self.len()))
        );
        &self
            .data
            .get(range.clone())
            .expect(&format!("Range out of length: {:?} - buf size: {}", range, self.len()))
    }

    pub fn line_length(&self, line: metadata::Line) -> Option<metadata::Length> {
        use metadata::Length as L;
        self.meta_data.get(line).and_then(|a| {
            self.meta_data
                .get(line.offset(1))
                .map(|b| Some(L(*b - *a)))
                .unwrap_or(Some(L(self.len() - *a)))
        })
    }

    pub fn get_cursor(&self) -> &BufferCursor {
        &self.edit_cursor
    }

    pub fn insert_slice(&mut self, slice: &[char]) {
        if slice.len() > 128 {
            let mut v = Vec::with_capacity(self.len() + slice.len() * 2);
            unsafe {
                let abs = *self.edit_cursor.absolute() as isize;
                let ptr = v.as_mut_ptr();
                // std::ptr::copy_nonoverlapping(self.data.as_ptr(), v.as_mut_ptr(), *self.cursor.absolute());
                copy_slice_to(ptr, &self.data[..abs as usize]);
                // std::ptr::copy_nonoverlapping(slice.as_ptr(), v.as_mut_ptr().offset(abs), slice.len());
                copy_slice_to(ptr.offset(abs), slice);
                // std::ptr::copy_nonoverlapping(self.data.as_ptr().offset(abs),v.as_mut_ptr().offset(abs + slice.len() as isize), self.len() - abs as usize);
                copy_slice_to(ptr.offset(abs + slice.len() as isize), &self.data[(abs as usize)..]);

                v.set_len(self.len() + slice.len());
                let new_abs_cursor_pos = metadata::Index(abs as usize + slice.len());
                self.size = v.len();
                self.data = v;
                self.rebuild_metadata();
                self.meta_data.set_buffer_size(self.size);
                self.edit_cursor = self.cursor_from_metadata(new_abs_cursor_pos).unwrap();
            }
        } else {
            for c in slice {
                self.insert(*c);
            }
        }
    }
    /// Erases one character at the index of the cursor position
    pub fn remove(&mut self) {
        let idx = *self.edit_cursor.absolute();
        if idx != self.len() && self.len() != 0 {
            self.data.remove(idx);
        }
    }
    /// Returns an iterator iterating over contents in character buffer
    #[inline(always)]
    pub fn iter(&self) -> std::slice::Iter<'_, char> {
        self.data.iter()
    }
    /// Returns an iterator iterating over contents in character buffer
    #[inline(always)]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, char> {
        self.data.iter_mut()
    }
    /// Utility function calling self.iter().skip(count)
    #[inline(always)]
    pub fn iter_skip(&self, skip: usize) -> std::iter::Skip<std::slice::Iter<'_, char>> {
        self.data.iter().skip(skip)
    }
    /// Moves cursor forward, in the fashion specified by TextKind
    pub fn cursor_move_forward(&mut self, kind: TextKind, count: usize) {
        match kind {
            TextKind::Char => self.cursor_step_forward(count),
            TextKind::Word => {
                if count == 1 {
                    if let Some(&c) = self.get(self.edit_cursor.absolute()) {
                        if c.is_alphanumeric() {
                            self.edit_cursor = self.find_next(|c| c.is_whitespace()).unwrap_or(BufferCursor {
                                pos: metadata::Index(self.len()),
                                row: metadata::Line(self.meta_data.line_count() - 1),
                                col: metadata::Column(
                                    self.meta_data
                                        .get_line_start_index(metadata::Line(self.meta_data.line_count() - 1))
                                        .map(|v| self.len() - *v)
                                        .unwrap(),
                                ),
                            });
                        } else if c.is_whitespace() {
                            self.edit_cursor = self.find_next(|c| c.is_alphanumeric()).unwrap_or(BufferCursor {
                                pos: metadata::Index(self.len()),
                                row: metadata::Line(self.meta_data.line_count() - 1),
                                col: metadata::Column(
                                    self.meta_data
                                        .get_line_start_index(metadata::Line(self.meta_data.line_count() - 1))
                                        .map(|v| self.len() - *v)
                                        .unwrap(),
                                ),
                            });
                        }
                    }
                } else {
                    todo!("cursor movement spanning longer than a word not yet done");
                }
            }
            TextKind::Line => {
                for _ in 0..count {
                    self.cursor_move_down();
                }
            }
            TextKind::Block => todo!(),
        }
    }
    /// Moves cursor backward, in the fashion specified by TextKind
    pub fn cursor_move_backward(&mut self, kind: TextKind, count: usize) {
        match kind {
            TextKind::Char => {
                if *self.edit_cursor.absolute() as i64 - count as i64 > 0 {
                    for _ in 0..count {
                        self.edit_cursor.pos -= metadata::Index(1);
                        if let Some('\n') = self.get(self.edit_cursor.absolute()) {
                            self.edit_cursor.row -= metadata::Line(1);
                            self.edit_cursor.col = metadata::Column(
                                *(self.edit_cursor.absolute()
                                    - self
                                        .find_prev_newline_pos_from(self.edit_cursor.absolute())
                                        .unwrap_or(metadata::Index(0))),
                            )
                        } else {
                            self.edit_cursor.col -= metadata::Column(1);
                        }
                    }
                } else {
                    self.edit_cursor = BufferCursor::default();
                }
            }
            TextKind::Word => {
                if count == 1 {
                    if let Some(&c) = self.get(self.edit_cursor.absolute()) {
                        if c.is_alphanumeric() {
                            if let Some(cur) = self.find_prev(|c| c.is_whitespace()) {
                                self.edit_cursor = cur;
                            }
                        } else if c.is_whitespace() {
                            if let Some(cur) = self.find_prev(|c| c.is_alphanumeric()) {
                                self.edit_cursor = cur;
                            }
                        }
                    } else {
                        self.cursor_move_backward(TextKind::Char, 1);
                    }
                } else {
                    todo!("cursor movement spanning longer than a word not yet done");
                }
            }
            TextKind::Line => {
                for _ in 0..count {
                    self.cursor_move_up();
                }
            }
            TextKind::Block => todo!(),
        }
    }
}

/// Private interface implementation
impl SimpleBuffer {
    /// Takes a buffer index and tries to build a BufferCursor, using the MetaData member of the SimpleBuffer
    /// After some deliberation, this is the core function that all movement functions of the Buffer will use.
    /// Instead of having each function individually updating the cursor and keeping track of rows and columns
    /// They explicitly only deal with absolute positions/indices, and before returning, calls this function
    /// to return an Option of a well formed BufferCursor

    fn find_index_of_prev_from(&self, start_position: metadata::Index, f: fn(char) -> bool) -> Option<metadata::Index> {
        self.data.get(0..=(*start_position)).and_then(|range| {
            range
                .iter()
                .rev()
                .position(|c| f(*c))
                .map(|len_from_pos| metadata::Index(*start_position - len_from_pos))
        })
    }

    fn find_index_of_next_from(&self, start_position: metadata::Index, f: fn(char) -> bool) -> Option<metadata::Index> {
        self.iter()
            .skip(*start_position)
            .position(|&ch| f(ch))
            .map(|len_from_pos| start_position.offset(len_from_pos as _))
    }

    fn find_next(&self, f: fn(char) -> bool) -> Option<BufferCursor> {
        self.iter()
            .enumerate()
            .skip(*self.cursor_abs() + 1)
            .find(|(_, &ch)| f(ch))
            .and_then(|(i, _)| self.cursor_from_metadata(metadata::Index(i)))
    }

    fn find_prev(&self, f: fn(char) -> bool) -> Option<BufferCursor> {
        let cursor_pos = *self.cursor_abs();
        self.data[..cursor_pos]
            .iter()
            .rev()
            .position(|&c| f(c))
            .and_then(|char_index_predicate_true_for| self.cursor_from_metadata(metadata::Index(cursor_pos - char_index_predicate_true_for - 1)))
    }

    fn find_prev_newline_pos_from(&self, abs_pos: metadata::Index) -> Option<metadata::Index> {
        let abs_pos = *abs_pos;
        if abs_pos >= self.data.len() {
            self.meta_data.line_begin_indices.last().map(|v| *v)
        } else {
            let reversed_abs_position = self.data.len() - abs_pos;
            self.iter()
                .rev()
                .skip(reversed_abs_position)
                .position(|c| *c == '\n')
                .map(|v| metadata::Index(abs_pos - (v)))
        }
    }

    fn cursor_step_forward(&mut self, count: usize) {
        if *self.edit_cursor.absolute().offset(1) <= self.data.len() {
            for _ in 0..count {
                if let Some('\n') = self.get(self.edit_cursor.absolute()) {
                    self.edit_cursor.row = self.edit_cursor.row.offset(1);
                    self.edit_cursor.col = metadata::Column(0);
                } else {
                    self.edit_cursor.col = self.edit_cursor.col.offset(1);
                }
                self.edit_cursor.pos = self.edit_cursor.pos.offset(1);
            }
        } else {
            for _ in *self.edit_cursor.absolute()..self.data.len() {
                if let Some('\n') = self.get(self.edit_cursor.absolute()) {
                    self.edit_cursor.row = self.edit_cursor.row.offset(1);
                    self.edit_cursor.col = metadata::Column(0);
                } else {
                    self.edit_cursor.col = self.edit_cursor.col.offset(1);
                }
                self.edit_cursor.pos = self.edit_cursor.pos.offset(1);
            }
        }
    }

    fn cursor_step_backward(&mut self, count: usize) {
        if *self.edit_cursor.absolute() as i64 - count as i64 > 0 {
            for _ in 0..count {
                self.edit_cursor.pos = self.edit_cursor.pos.offset(-1);
                if let Some('\n') = self.get(self.edit_cursor.absolute()) {
                    self.edit_cursor.row = self.edit_cursor.row.offset(-1);
                    self.edit_cursor.col = metadata::Column(
                        *(self.edit_cursor.absolute()
                            - self
                                .find_prev_newline_pos_from(self.edit_cursor.absolute())
                                .unwrap_or(metadata::Index(0))),
                    )
                } else {
                    self.edit_cursor.col -= metadata::Column(1);
                }
            }
        } else {
            self.edit_cursor = BufferCursor::default();
        }
    }

    fn cursor_move_up(&mut self) {
        if self.cursor_row() == metadata::Line(0) {
            return;
        }
        let prior_line = self.cursor_row().offset(-1);
        self.edit_cursor = self
            .meta_data
            .get_line_start_index(prior_line)
            .and_then(|index| {
                self.meta_data
                    .get_line_length_of(prior_line)
                    .map(|prior_line_len| {
                        let pos = index.offset(min(prior_line_len.offset(-1).as_usize() as _, self.cursor_col().as_usize() as _));
                        self.cursor_from_metadata(pos)
                    })
                    .unwrap_or(self.cursor_from_metadata(index))
            })
            .unwrap_or(BufferCursor::default())
    }

    fn cursor_move_down(&mut self) {
        // This is all the lines up until the 3rd to last - normal behavior, 2nd to last means we are moving into the last, other behavior applies in the else branch
        let next_line_index = self.cursor_row().offset(1);

        #[cfg(debug_assertions)]
        {
            let a = self.line_length(next_line_index);
            let b = self.meta_data.line_length(next_line_index);
            debugger_catch!(a == b, DebuggerCatch::Handle(format!("Line length operation failed")));
        }
        let new_cursor = self
            .line_length(next_line_index)
            .map(|l| l.as_column())
            .and_then(|next_line_length| {
                if let Some(line_begin) = self.meta_data.get(self.edit_cursor.row.offset(1)) {
                    let new_buffer_index = line_begin.offset(if self.cursor_col() <= next_line_length.offset(-1) {
                        *self.cursor_col() as _
                    } else {
                        *(next_line_length.offset(-1)) as _
                    });
                    self.cursor_from_metadata(new_buffer_index)
                } else {
                    None
                }
            });
        self.set_cursor(new_cursor.unwrap_or(self.edit_cursor));
    }
}

/// Trait implementation definitions for SimpleBuffer

impl std::ops::Index<usize> for SimpleBuffer {
    type Output = char;
    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.data.get_unchecked(index) }
    }
}

impl std::ops::IndexMut<usize> for SimpleBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { self.data.get_unchecked_mut(index) }
    }
}

impl std::ops::Index<std::ops::Range<usize>> for SimpleBuffer {
    type Output = [char];
    #[inline(always)]
    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        unsafe { self.data.get_unchecked(index) }
    }
}

impl std::ops::IndexMut<std::ops::Range<usize>> for SimpleBuffer {
    fn index_mut(&mut self, index: std::ops::Range<usize>) -> &mut Self::Output {
        unsafe { self.data.get_unchecked_mut(index) }
    }
}

impl<'a> CharBuffer<'a> for SimpleBuffer {
    type ItemIterator = std::slice::Iter<'a, char>;

    fn file_name(&self) -> Option<&Path> {
        self.meta_data.file_name.as_ref().map(|pb| pb.as_path())
    }

    fn clear(&mut self) {
        self.data.clear();
        self.edit_cursor = BufferCursor::default();
        self.meta_data.clear_line_index_metadata();
    }

    #[inline(always)]
    fn cursor_row(&self) -> metadata::Line {
        self.edit_cursor.row
    }

    #[inline(always)]
    fn cursor_col(&self) -> metadata::Column {
        self.edit_cursor.col
    }

    #[inline(always)]
    fn cursor_abs(&self) -> metadata::Index {
        self.edit_cursor.pos
    }

    fn insert(&mut self, ch: char) {
        use metadata::{Column as Col, Index};
        debug_assert!(self.edit_cursor.absolute() <= Index(self.len()), "You can't insert something outside of the range of [0..len()]");
        if ch == '\n' {
            self.data.insert(*self.edit_cursor.absolute(), ch);
            self.edit_cursor.pos = self.edit_cursor.pos.offset(1);
            self.edit_cursor.col = Col(0);
            self.edit_cursor.row = self.edit_cursor.row.offset(1);
            self.meta_data.insert_line_begin(self.edit_cursor.absolute(), self.edit_cursor.row);
            self.meta_data.update_line_metadata_after_line(self.edit_cursor.row, 1);
        } else {
            self.data.insert(*self.edit_cursor.absolute(), ch);
            self.edit_cursor.pos = self.edit_cursor.pos.offset(1);
            self.edit_cursor.col = self.edit_cursor.col.offset(1);
            self.meta_data.update_line_metadata_after_line(self.edit_cursor.row, 1);
        }
        self.size += 1;
        self.meta_data.set_buffer_size(self.size);
    }

    // todo(optimization): don't do the expensive rebuild of meta data after each delete. It's a pretty costly operation.
    fn delete(&mut self, dir: Movement) {
        use metadata::Index;
        if self.empty() {
            return;
        }
        match dir {
            Movement::Forward(kind, count) => match kind {
                TextKind::Char => {
                    // clamp the count of characters removed, so we don't try to remove "outside" of our buffer
                    let count = if self.edit_cursor.absolute().offset(count as isize) <= Index(self.data.len()) {
                        count
                    } else {
                        self.data.len() - *self.edit_cursor.absolute()
                    };
                    for _ in 0..count {
                        self.data.remove(*self.edit_cursor.absolute());
                    }
                }
                TextKind::Word => {
                    if let Some(c) = self.get(self.cursor_abs()) {
                        if c.is_whitespace() {
                            if let Some(Index(p)) = self.find_next(|c| !c.is_whitespace()).map(|c| c.pos) {
                                self.data.drain(*self.cursor_abs()..p);
                            }
                        } else if c.is_alphanumeric() {
                            if let Some(Index(p)) = self.find_next(|c| !c.is_alphanumeric()).map(|c| c.pos) {
                                self.data.drain(*self.cursor_abs()..p);
                            }
                        } else {
                            // If we are standing on, say +-/_* (non-alphanumerics) just delete one character at a time
                            self.data.remove(*self.cursor_abs());
                        }
                    }
                }
                TextKind::Line => todo!(),
                TextKind::Block => todo!(),
            },

            Movement::Backward(kind, count) if self.edit_cursor.absolute() != Index(0) => match kind {
                TextKind::Char => {
                    let count = if *self.edit_cursor.absolute() as i64 - count as i64 >= 0 {
                        count
                    } else {
                        *self.edit_cursor.absolute()
                    };
                    self.cursor_move_backward(TextKind::Char, count);
                    for _ in 0..count {
                        self.remove();
                    }
                }
                TextKind::Word => {
                    let idx_pos = self.edit_cursor.pos;
                    self.move_cursor(Movement::Begin(TextKind::Word));
                    let len = *(idx_pos - self.edit_cursor.pos);
                    for _ in 0..len {
                        self.remove();
                    }
                }
                TextKind::Line => todo!(),
                TextKind::Block => todo!(),
            },
            _ => {}
        }
        self.size = self.data.len();
        self.rebuild_metadata();
    }

    fn insert_slice_fast(&mut self, slice: &[char]) {
        self.insert_slice(slice);
        self.meta_data.set_buffer_size(self.size);
    }

    fn capacity(&self) -> usize {
        self.data.capacity()
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn rebuild_metadata(&mut self) {
        self.meta_data.clear_line_index_metadata();
        for (i, ch) in self.data.iter().enumerate() {
            if *ch == '\n' {
                self.meta_data.push_new_line_begin(metadata::Index(i + 1));
            }
        }
        let cs = calculate_hash(self);
        self.meta_data.set_checksum(cs);
    }

    #[inline(always)]
    fn meta_data(&self) -> &metadata::MetaData {
        &self.meta_data
    }

    fn iter(&'a self) -> Self::ItemIterator {
        self.data.iter()
    }

    #[allow(non_snake_case)]
    fn move_cursor(&mut self, dir: Movement) {
        use super::super::metadata::Index;
        match dir {
            Movement::Forward(kind, count) => {
                self.cursor_move_forward(kind, count);
            }
            Movement::Backward(kind, count) => {
                self.cursor_move_backward(kind, count);
            }
            Movement::Begin(kind) => match kind {
                TextKind::Char => self.cursor_step_backward(1),
                TextKind::Word => {
                    if let Some(c) = self.get(self.edit_cursor.pos.offset(-1)) {
                        let predicate = predicate_generate(c);
                        let start_position = self.edit_cursor.pos.offset(-2);
                        let i = self
                            .find_index_of_prev_from(start_position, predicate)
                            .unwrap_or(Index::default())
                            .offset(1);
                        let len = *(self.edit_cursor.pos - i);
                        self.cursor_step_backward(len);
                    }
                }
                TextKind::Line => {
                    if let Some(start) = self.meta_data.get(self.cursor_row()) {
                        self.cursor_goto(start);
                    }
                }
                TextKind::Block => {
                    if let Some(block_begin) = self.find_index_of_prev_from(self.edit_cursor.pos.offset(-1), |f| f == '{') {
                        self.cursor_goto(block_begin);
                    }
                }
            },
            Movement::End(kind) => match kind {
                TextKind::Char => self.cursor_step_forward(1),
                TextKind::Word => {
                    if let Some(c) = self.get(self.edit_cursor.pos) {
                        let start = self.edit_cursor.pos.offset(1);
                        let predicate = predicate_generate(c);
                        let new_pos = self.find_index_of_next_from(start, predicate).unwrap_or(Index(self.len())); // .and_then(|i| self.cursor_from_metadata(i));
                        let step_length = *(new_pos - self.edit_cursor.pos);
                        self.cursor_step_forward(step_length);

                        // self.cursor = new_pos.unwrap_or(self.cursor_from_metadata(Index(self.len())).unwrap());
                    }
                }
                TextKind::Line => {
                    if let Some(end) = self.meta_data.get(self.cursor_row().offset(1)).map(|Index(start)| Index(start - 1)) {
                        self.cursor_goto(end);
                    }
                }
                TextKind::Block => {
                    if let Some(block_begin) = self.find_index_of_next_from(self.edit_cursor.pos.offset(1), |f| f == '}') {
                        self.cursor_goto(block_begin);
                    }
                }
            },
        }
    }

    fn set_cursor(&mut self, cursor: BufferCursor) {
        self.edit_cursor = cursor;
    }

    fn load_file(&mut self, path: &Path) {
        let file_options = std::fs::OpenOptions::new().read(true).open(path);
        let mut strbuf = String::with_capacity(10000);

        match file_options {
            Ok(mut file) => match file.read_to_string(&mut strbuf) {
                Ok(_) => {
                    for (i, ch) in strbuf.chars().enumerate() {
                        self.data.insert(i, ch);
                    }
                    self.rebuild_metadata();
                    self.edit_cursor = self
                        .cursor_from_metadata(metadata::Index(self.len()))
                        .unwrap_or(BufferCursor::default());
                    self.size = self.data.len();
                    self.meta_data.set_buffer_size(self.size);
                    self.meta_data.file_name = Some(path.to_path_buf());
                    let cs = calculate_hash(self);
                    self.meta_data.set_checksum(cs);
                }
                Err(e) => println!("failed to read data: {}", e),
            },
            Err(e) => {
                println!("failed to OPEN file: {}", e);
            }
        }
    }

    fn save_file(&mut self, path: &Path) {
        let checksum = calculate_hash(self);
        if checksum != self.meta_data.get_checksum() {
            match std::fs::OpenOptions::new().write(true).create(true).open(path) {
                Ok(mut file) => match file.write(self.data.iter().map(|c| *c).collect::<String>().as_bytes()) {
                    Ok(_bytes_written) => {
                        only_in_debug!(println!("wrote {} bytes to {}", _bytes_written, path.display()));
                        let checksum = calculate_hash(self);
                        self.meta_data.set_checksum(checksum);
                        self.meta_data.file_name = Some(path.to_path_buf());
                    }
                    Err(_err) => {}
                },
                Err(_err) => {}
            }
        } else {
            println!("File is already pristine!");
        }
    }

    fn copy(&mut self, range: std::ops::Range<usize>) -> &[char] {
        &self.data[range]
    }
}

#[inline(always)]
pub fn predicate_generate(c: &char) -> fn(char) -> bool {
    if c.is_whitespace() {
        |ch: char| !ch.is_whitespace()
    } else if c.is_alphanumeric() {
        |ch: char| !ch.is_alphanumeric()
    } else {
        |ch: char| !ch.is_ascii_punctuation()
    }
}
