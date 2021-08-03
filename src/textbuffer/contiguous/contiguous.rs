use std::{
    cmp::min,
    io::{Read, Write},
    iter::FromIterator,
    ops::Bound,
    path::Path,
};

use super::super::{cursor::BufferCursor, CharBuffer, Movement};
use crate::{
    debugger_catch, only_in_debug,
    textbuffer::{
        cursor::MetaCursor,
        metadata::{self, calculate_hash},
        LineOperation, TextKind,
    },
    utils::{copy_slice_to, AsUsize},
};

#[cfg(debug_assertions)]
use crate::DebuggerCatch;

pub struct ContiguousBuffer {
    pub id: u32,
    pub data: Vec<char>,
    edit_cursor: BufferCursor,
    pub meta_cursor: Option<MetaCursor>,
    size: usize,
    meta_data: metadata::MetaData,
}

impl std::hash::Hash for ContiguousBuffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl ContiguousBuffer {
    pub fn new(id: u32, capacity: usize) -> ContiguousBuffer {
        ContiguousBuffer {
            id: id,
            data: Vec::with_capacity(capacity),
            edit_cursor: BufferCursor::default(),
            meta_cursor: None,
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

    pub fn set_absolute_meta_cursor(&mut self, pos: metadata::Index) {
        self.meta_cursor = Some(MetaCursor::Absolute(pos));
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
        unsafe { &self.data.get_unchecked(range.clone()) }
    }

    pub fn get_lines_as_slices(&self, first: metadata::Line, last: metadata::Line) -> Vec<&[char]> {
        debug_assert!(first < last, "Last line must come after first line");
        let mut res = Vec::with_capacity(*(last - first));
        for l in first..=last {
            let line_begin = self.meta_data.get_line_start_index(l).map(|i| *i).unwrap();
            let line_end = self.meta_data.get_line_start_index(l.offset(1)).map_or(self.len(), |i| *i);
            res.push(self.get_slice(line_begin..line_end));
        }
        res
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
        if let Some(mc) = &self.meta_cursor {
            match *mc {
                MetaCursor::Absolute(marker) => {
                    let (erase_from, erase_to) = if marker < self.cursor_abs() {
                        (*marker, *self.edit_cursor.pos)
                    } else {
                        (*self.edit_cursor.pos, *marker)
                    };
                    self.data.drain(erase_from..=erase_to);
                    self.meta_cursor = None;
                    self.size = self.data.len();
                    self.rebuild_metadata();
                    self.cursor_goto(metadata::Index(erase_from));
                }
                #[allow(unused)]
                MetaCursor::LineRange { column, begin, end } => todo!(),
            }
        }
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
            TextKind::Block => {
                for _ in 0..count {
                    self.move_cursor(Movement::End(TextKind::Block));
                }
            }
            TextKind::Page => { todo!("TextKind::Page not yet implemented") },
            TextKind::File => { todo!("TextKind::File not yet implemented") }
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
            TextKind::Block => {
                for _ in 0..count {
                    self.move_cursor(Movement::Begin(TextKind::Block));
                }
            }
            TextKind::Page => { todo!("TextKind::Page not yet implemented") },
            TextKind::File => { todo!("TextKind::File not yet implemented") }
        }
    }

    /// Copies the selected text (if any text is selected) otherwise copies the contents of the line
    pub fn copy_range_or_line(&self) -> Option<String> {
        if let Some(meta_cursor) = &self.meta_cursor {
            match *meta_cursor {
                MetaCursor::Absolute(meta_cursor) => {
                    if *self.cursor_abs() >= self.len() || *meta_cursor >= self.len() {
                        None
                    } else {
                        if meta_cursor < self.edit_cursor.pos {
                            Some(String::from_iter(self.get_slice(*meta_cursor..*self.edit_cursor.pos.offset(1))))
                        } else {
                            Some(String::from_iter(self.get_slice(*self.edit_cursor.pos..*meta_cursor.offset(1))))
                        }
                    }
                }
                #[allow(unused)]
                MetaCursor::LineRange { column, begin, end } => todo!(),
            }
        } else {
            let row = self.edit_cursor.row;
            self.meta_data
                .get_line_start_index(row)
                .zip(
                    self.meta_data
                        .get_line_start_index(row.offset(1))
                        .or_else(|| Some(metadata::Index(self.len()))),
                )
                .map(|(begin, end)| String::from_iter(self.get_slice(*begin..*end)))
        }
    }

    /// Returns the (possibly) selected range. This always makes sure to return begin .. end, since the meta cursor can be both behind and in front
    /// of the edit_cursor
    pub fn get_selection(&self) -> Option<(metadata::Index, metadata::Index)> {
        if let Some(meta_cursor) = &self.meta_cursor {
            match *meta_cursor {
                MetaCursor::Absolute(meta_cursor) => {
                    if meta_cursor < self.edit_cursor.pos {
                        Some((meta_cursor, self.edit_cursor.pos))
                    } else {
                        Some((self.edit_cursor.pos, meta_cursor))
                    }
                }
                #[allow(unused)]
                MetaCursor::LineRange { column, begin, end } => {
                    let md = self.meta_data();
                    md.get(begin).zip(md.get(end.offset(1))).map(|(b, e)| (b, e.offset(-1)))
                }
            }
        } else {
            None
        }
    }
}

/// Private interface implementation
impl ContiguousBuffer {
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
            self.cursor_goto(metadata::Index(0));
        } else {
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

    pub fn search_next(&mut self, find: &str) {
        let v: Vec<char> = find.chars().collect();
        let mut idx = *self.edit_cursor.pos + 1;
        while idx < self.len() {
            if self.data[idx] == v[0] {
                if let Some(sub_ref_slice) = &self.data.get(idx..idx + v.len()) {
                    if sub_ref_slice[v.len() - 1] == v[v.len() - 1] {
                        if sub_ref_slice[..] == v[..] {
                            println!("Found {} at {} ({:?})", find, idx, &self.data[idx..(idx + v.len())]);
                            self.cursor_goto(metadata::Index(idx));
                            return;
                        } else {
                            idx += v.len();
                        }
                    } else {
                        idx += v.len();
                    }
                } else {
                    println!("could not find __{}__", find);
                    return;
                }
            } else {
                idx += 1;
            }
        }
        println!("could not find {}", find);
    }
}

/// Trait implementation definitions for SimpleBuffer

impl std::ops::Index<usize> for ContiguousBuffer {
    type Output = char;
    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.data.get_unchecked(index) }
    }
}

impl std::ops::IndexMut<usize> for ContiguousBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { self.data.get_unchecked_mut(index) }
    }
}

impl std::ops::Index<std::ops::Range<usize>> for ContiguousBuffer {
    type Output = [char];
    #[inline(always)]
    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        unsafe { self.data.get_unchecked(index) }
    }
}

impl std::ops::IndexMut<std::ops::Range<usize>> for ContiguousBuffer {
    fn index_mut(&mut self, index: std::ops::Range<usize>) -> &mut Self::Output {
        unsafe { self.data.get_unchecked_mut(index) }
    }
}

impl<'a> CharBuffer<'a> for ContiguousBuffer {
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
        if let Some(marker) = &self.meta_cursor {
            match *marker {
                MetaCursor::Absolute(marker) => {
                    let (erase_from, erase_to) = if marker < self.cursor_abs() {
                        (*marker, *self.edit_cursor.pos)
                    } else {
                        (*self.edit_cursor.pos, *marker)
                    };
                    self.data.drain(erase_from..=erase_to);
                    self.meta_cursor = None;
                    self.size = self.data.len();
                    self.rebuild_metadata();
                    self.cursor_goto(Index(erase_from));
                }
                #[allow(unused)]
                MetaCursor::LineRange { column, begin, end } => todo!(),
            }
        }
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
        if let Some(marker) = &self.meta_cursor {
            match *marker {
                MetaCursor::Absolute(marker) => {
                    let (erase_from, erase_to) = if marker < self.cursor_abs() {
                        (*marker, std::cmp::min(*self.edit_cursor.pos, self.len() - 1))
                    } else {
                        (*self.edit_cursor.pos, std::cmp::min(*marker, self.len() - 1))
                    };

                    self.data.drain(erase_from..=erase_to);
                    self.meta_cursor = None;
                    self.size = self.data.len();
                    self.rebuild_metadata();
                    self.cursor_goto(Index(erase_from));
                    return;
                }
                #[allow(unused)]
                MetaCursor::LineRange { column, begin, end } => {
                    let md = self.meta_data();
                    if let Some((begin, end)) = md.get(begin).zip(md.get(end.offset(1))).map(|(b, e)| (b, e.offset(-1))) {
                        self.data.drain(*begin..=*end);
                        self.meta_cursor = None;
                        self.size = self.data.len();
                        self.rebuild_metadata();
                        self.cursor_goto(begin);
                    }
                }
            }
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
                TextKind::Page => { todo!("TextKind::Page not yet implemented") },
                TextKind::File => { todo!("TextKind::File not yet implemented") }
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
                TextKind::Page => { todo!("TextKind::Page not yet implemented") },
                TextKind::File => { todo!("TextKind::File not yet implemented") }
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

    fn select_move_cursor_absolute(&mut self, movement: Movement) {
        match self.meta_cursor {
            Some(MetaCursor::Absolute(i)) => {
                self.move_cursor(movement);
                self.set_absolute_meta_cursor(i);
            }
            #[allow(unused)]
            Some(MetaCursor::LineRange { column, begin, end }) => {
                todo!();
            }
            None => {
                let mc_idx = self.edit_cursor.pos;
                self.move_cursor(movement);
                self.set_absolute_meta_cursor(mc_idx);
            }
        }
    }

    /// Clears the meta cursor when moving, so if the desired action is to set a range of selected data
    /// the start position of the meta cursor has to be set _after_ calling this method
    fn move_cursor(&mut self, dir: Movement) {
        use super::super::metadata::Index;
        self.meta_cursor = None;
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
                },
                TextKind::Page => { todo!("TextKind::Page not yet implemented") },
                TextKind::File => { todo!("TextKind::File not yet implemented") }
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
                    }
                }
                TextKind::Line => {
                    let end = self
                        .meta_data
                        .get(self.cursor_row().offset(1))
                        .map_or(Index(self.len()), |Index(start)| Index(start - 1));
                    self.cursor_goto(end);
                }
                TextKind::Block => {
                    if let Some(block_begin) = self.find_index_of_next_from(self.edit_cursor.pos.offset(1), |f| f == '}') {
                        self.cursor_goto(block_begin);
                    }
                }
                TextKind::Page => { todo!("TextKind::Page not yet implemented") },
                TextKind::File => { todo!("TextKind::File not yet implemented") }
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
                    self.meta_data.set_pristine_hash(cs);
                }
                // todo: remove debug println, and instead create a UI representation of this error message
                Err(e) => println!("failed to read data: {}", e),
            },
            Err(e) => {
                // todo: remove debug println, and instead create a UI representation of this error message
                println!("failed to OPEN file: {}", e);
            }
        }
    }

    fn save_file(&mut self, path: &Path) {
        let checksum = calculate_hash(self);
        if checksum != self.meta_data.get_pristine_hash() {
            match std::fs::OpenOptions::new().write(true).create(true).open(path) {
                Ok(mut file) => match file.write(self.data.iter().map(|c| *c).collect::<String>().as_bytes()) {
                    Ok(_bytes_written) => {
                        only_in_debug!(println!("wrote {} bytes to {}", _bytes_written, path.display()));
                        let checksum = calculate_hash(self);
                        self.meta_data.set_checksum(checksum);
                        self.meta_data.set_pristine_hash(checksum);
                        self.meta_data.file_name = Some(path.to_path_buf());
                    }
                    Err(_err) => {}
                },
                Err(_err) => {}
            }
        } else {
            // todo: remove debug println, and instead create a UI representation of this error message
            println!("File is already pristine!");
        }
    }

    fn copy(&mut self, range: std::ops::Range<usize>) -> String {
        String::from_iter(&self.data[range])
    }

    fn goto_line(&mut self, line: usize) {
        self.cursor_goto(
            self.meta_data
                .get_line_start_index(metadata::Line(line))
                .unwrap_or(self.cursor_abs()),
        );
    }

    #[allow(unused)]
    fn line_operation<T>(&mut self, lines: T, op: LineOperation)
    where
        T: std::ops::RangeBounds<usize> + std::slice::SliceIndex<[metadata::Index], Output = [metadata::Index]> + Clone + std::ops::RangeBounds<usize>,
    {
        let a = match lines.start_bound() {
            Bound::Included(a) => *a,
            Bound::Excluded(a) => *a,
            Bound::Unbounded => self.len(),
        };

        let mut shift_tracking = 0;
        match op {
            LineOperation::ShiftLeft { shift_by } => {
                if let Some(lines) = self.meta_data.get_lines(lines.clone()).or(self.meta_data.get_lines(a..)) {
                    for (cnt, &lb) in lines.iter().enumerate() {
                        if let Some(next_line_begin) = self.meta_data.get(metadata::Line(a + cnt + 1)) {
                            let line_len = *next_line_begin - *lb;
                            let lb = *lb.offset(shift_tracking as isize);
                            let shiftable = self.data[lb..lb + std::cmp::min(shift_by, line_len)]
                                .iter()
                                .take_while(|c| c.is_ascii_whitespace() && **c != '\n')
                                .take(shift_by)
                                .count();
                            if shiftable > 0 {
                                let drain = lb..lb + shiftable;
                                let cnt = drain.len();
                                assert_eq!(cnt, shiftable);
                                self.data.drain(drain);
                                shift_tracking -= cnt as i32;
                            }
                        } else {
                            let lb = *lb.offset(shift_tracking as isize);
                            let shiftable = self.data[lb..]
                                .iter()
                                .take_while(|c| c.is_ascii_whitespace() && **c != '\n')
                                .take(shift_by)
                                .count();
                            if shiftable > 0 {
                                let drain = lb..lb + shiftable;
                                let cnt = drain.len();
                                assert_eq!(cnt, shiftable);
                                self.data.drain(drain);
                                shift_tracking -= cnt as i32;
                            }
                        }
                    }
                }
            }
            LineOperation::ShiftRight { shift_by } => {
                if let Some(lines) = self.meta_data.get_lines(lines) {
                    let data: Vec<_> = (0..shift_by).map(|_| ' ').collect();
                    for &lb in lines.iter() {
                        let lb = lb.offset(shift_tracking as _);
                        self.data.splice(*lb..*lb, data.iter().copied());
                        shift_tracking += shift_by as i32;
                    }
                }
            }
            LineOperation::InsertElement { at_column, insertion } => todo!(),
            LineOperation::InsertString { at_column, insertion } => todo!(),
        }

        self.rebuild_metadata();
        match self.meta_cursor {
            Some(MetaCursor::Absolute(ref mut i)) => {
                if *i < self.edit_cursor.pos {
                    self.cursor_goto(self.edit_cursor.pos.offset(shift_tracking as _));
                } else {
                    *i = i.offset(shift_tracking as _);
                }
            }
            Some(MetaCursor::LineRange { column, begin, end }) => todo!(),
            None => todo!(),
        }
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

#[rustfmt::skip]
#[allow(unused)]
#[cfg(test)]
mod tests {
    // For using benchmarking
    extern crate test;

    use super::ContiguousBuffer;
    use crate::textbuffer::{metadata as md, CharBuffer, LineOperation, Movement, TextKind};

    #[test]
    fn cursor_move_in_empty() {
        let mut b = Box::new(ContiguousBuffer::new(0, 1024));
        b.move_cursor(Movement::Forward(TextKind::Char, 1));
        assert_eq!(b.edit_cursor.pos, md::Index(0));
        b.move_cursor(Movement::Backward(TextKind::Char, 1));
        assert_eq!(b.edit_cursor.pos, md::Index(0));

        b.move_cursor(Movement::Forward(TextKind::Block, 1));
        assert_eq!(b.edit_cursor.pos, md::Index(0));
        b.move_cursor(Movement::Backward(TextKind::Block, 1));
        assert_eq!(b.edit_cursor.pos, md::Index(0));

        b.move_cursor(Movement::Forward(TextKind::Line, 1));
        assert_eq!(b.edit_cursor.pos, md::Index(0));
        b.move_cursor(Movement::Backward(TextKind::Line, 1));
        assert_eq!(b.edit_cursor.pos, md::Index(0));
    }

    #[test]
    fn length_checks() {
        let v: Vec<char> = "Hello test world".chars().collect();

        let slice = &v[..];
        let mut b = Box::new(ContiguousBuffer::new(0, 1024));
        b.insert_slice(slice);
        let mut multiple = 1;
        assert_eq!(b.len(), v.len() * multiple);
        b.insert_slice(slice);
        multiple += 1;
        assert_eq!(b.len(), v.len() * multiple);
    }

    #[test]
    fn test_copy_only_line() {
        let v: Vec<char> = "Hello test world".chars().collect();
        let slice = &v[..];
        let mut b = Box::new(ContiguousBuffer::new(0, 1024));
        b.insert_slice(slice);
        let mut multiple = 1;
        assert_eq!(b.len() * multiple, v.len());
        b.insert_slice(slice);
        let copy = b.copy_range_or_line();
        assert_eq!(copy, Some(v.iter().chain(v.iter()).collect::<String>()));
    }

    #[test]
    fn copy_paste_hello() {
        let v: Vec<char> = "Hello test world".chars().collect();
        let mut b = Box::new(ContiguousBuffer::new(0, 1024));
        for c in v {
            b.insert(c);
        }
        b.move_cursor(Movement::Backward(TextKind::Line, 1));
        assert_eq!(b.edit_cursor.pos, md::Index(0));
        b.select_move_cursor_absolute(Movement::Forward(TextKind::Char, 4));
        let copy = b.copy_range_or_line();
        assert_eq!(Some("Hello".into()), copy);
        b.move_cursor(Movement::End(TextKind::Word));
        b.move_cursor(Movement::End(TextKind::Word));
        for c in copy.unwrap().chars() {
            b.insert(c);
        }
        let new_copy = b.copy_range_or_line();
        assert_eq!(Some("Hello Hellotest world".into()), new_copy);
        for c in new_copy.unwrap().chars() {
            b.insert(c);
        }
        let last_copy = b.copy_range_or_line();
        assert_eq!(Some("Hello HelloHello Hellotest worldtest world".into()), last_copy);
        b.move_cursor(Movement::End(TextKind::Line));
        assert_eq!(b.edit_cursor.pos, md::Index(b.len()));
    }

    #[test]
    fn test_4_shift_left_of_lines() {
        // this tests shifting by four, it also tests shifting lines with
        // length less than 4, and it also tests shifting lines with less than 4 whitespaces in front
        let d = format!(
            "    // this is going to test shifting
fn main() {{
    println!('hello world')
   if let Some(foo) = test {{
        println!('test');
   }}
  //


}}
    //");
        let assert_str = format!(
            "// this is going to test shifting
fn main() {{
println!('hello world')
if let Some(foo) = test {{
    println!('test');
}}
//


}}
//");
        let mut sb = Box::new(ContiguousBuffer::new(0, 1024));
        for c in d.chars() {
            sb.insert(c);
        }
        let validate_first: String = sb.data.iter().map(|v| *v).collect();
        assert_eq!(d, validate_first);

        sb.cursor_goto(md::Index(0));
        sb.line_operation(0..11, LineOperation::ShiftLeft { shift_by: 4 });
        let res: String = sb.data.iter().map(|v| *v).collect();
        assert_eq!(assert_str, res);
    }

    #[test]
    fn test_4_shift_right_of_lines() {
        // this tests shifting by four, it also tests shifting lines with
        // length less than 4, and it also tests shifting lines with less than 4 whitespaces in front
        let d = format!(
"// this is going to test shifting
fn main() {{
    println!('hello world')
   if let Some(foo) = test {{
        println!('test');
   }}
}}");
        let assert_str = format!(
"    // this is going to test shifting
    fn main() {{
        println!('hello world')
       if let Some(foo) = test {{
            println!('test');
       }}
    }}");

        let mut sb = Box::new(ContiguousBuffer::new(0, 1024));
        for c in d.chars() {
            sb.insert(c);
        }
        let validate_first: String = sb.data.iter().map(|v| *v).collect();
        assert_eq!(d, validate_first);

        sb.cursor_goto(md::Index(0));
        sb.line_operation(0..7, LineOperation::ShiftRight { shift_by: 4 });
        let res: String = sb.data.iter().map(|v| *v).collect();
        assert_eq!(assert_str, res);
    }

    #[test]
    fn test_shift_should_not_alter() {
        // this tests shifting by four, it also tests shifting lines with
        // length less than 4, and it also tests shifting lines with less than 4 whitespaces in front
        let assert_str = format!(
            "// this is going to test shifting
fn main() {{
    println!('hello world')
   if let Some(foo) = test {{
        println!('test');
   }}
}}"
        );

        let mut sb = Box::new(ContiguousBuffer::new(0, 1024));
        for c in assert_str.chars() {
            sb.insert(c);
        }
        let validate_first: String = sb.data.iter().map(|v| *v).collect();
        assert_eq!(assert_str, validate_first);

        sb.cursor_goto(md::Index(0));
        // lines range (the end) are out of bounds. No operation should be done
        sb.line_operation(0..10, LineOperation::ShiftRight { shift_by: 4 });
        let res: String = sb.data.iter().map(|v| *v).collect();
        assert_eq!(assert_str, res);
    }

    #[bench]
    fn copy_paste_per_char(b: &mut test::Bencher) {
        let text_data = include_str!("contiguous.rs");
        let mut b = Box::new(ContiguousBuffer::new(0, 1024));
        if b.edit_cursor.pos == md::Index(0) {}
    }

    #[bench]
    fn copy_paste_as_slice(b: &mut test::Bencher) {
        let text_data = include_str!("contiguous.rs");
        let mut b = Box::new(ContiguousBuffer::new(0, 1024));
    }
}
