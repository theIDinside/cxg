use std::{cmp::min, io::Read, path::Path};

#[cfg(debug_assertions)]
use crate::DebuggerCatch;
use super::{cursor::BufferCursor, metadata::{MetaData, Line, Index}};

pub struct SimpleBuffer {
    _id: u32,
    pub data: Vec<char>,
    cursor: BufferCursor,
    size: usize,
    meta_data: MetaData
}

#[derive(Debug)]
pub enum TextKind {
    Char,
    Word,
    Line,
    Block
}

#[derive(Debug)]
pub enum Movement {
    Forward(TextKind, usize),
    Backward(TextKind, usize),
    Begin(TextKind),
    End(TextKind)
}

pub trait CountDigits {
    fn digits(&self) -> usize;
}

impl CountDigits for usize {
    fn digits(&self) -> usize {
        let mut value = *self;
        let mut digits = 1;
        if value < 10 {
            digits 
        } else {
            while value >= 10 {
                value /= 10;
                digits += 1;
            }
            digits
        }    
    }
}

impl SimpleBuffer {
    pub fn new(id: u32, capacity: usize) -> SimpleBuffer {
        SimpleBuffer {
            _id: id,
            data: Vec::with_capacity(capacity),
            cursor: BufferCursor::default(),
            size: 0,
            meta_data: MetaData::new(None)
        }
    }

    pub fn get(&self, idx: usize) -> Option<&char> {
        self.data.get(idx)
    }

    pub fn get_slice(&self, range: std::ops::Range<usize>) -> &[char] {
        debugger_catch!(range.start <= self.len() && range.end <= self.len(), DebuggerCatch::Handle(format!("Illegal access of buffer; getting range {:?} from buffer of only {} len", range.clone(), self.len())));
        &self.data.get(range.clone()).expect(&format!("Range out of length: {:?} - buf size: {}", range, self.len()))
    }

    pub fn from_file(id: u32, path: &Path) -> std::io::Result<SimpleBuffer> {
        let mut file = std::fs::OpenOptions::new().open(path)?;
        let file_length = file.metadata()?.len();
        let mut s = String::with_capacity(file_length as usize);
        file.read(unsafe { s.as_bytes_mut() })?;
        
        Ok(
            SimpleBuffer {
                _id: id,
                data: s.chars().collect(),
                cursor: BufferCursor::default(),
                size: file_length as usize,
                meta_data: MetaData::new(Some(path))
            }
        )
    }

    pub fn line_length(&self, line: super::metadata::Index) -> Option<usize> {
        let super::metadata::Index(index) = line;
        self.meta_data.get(index).and_then(|a| {
            self.meta_data.get(index+1).map(|b| {
                Some(*b - *a)
            }).unwrap_or(Some(self.len() - *a))
        })
    }

    #[cfg(debug_assertions)]
    pub fn debug_metadata(&self) {
        println!("#Line index:Buffer Index Pos - [line contents, newlines represented as /]");
        println!("-----------------------------");
        let digits = self.meta_data.line_begin_indices.len().digits();
        let idx_digits = self.len().digits();

        let mut strs: Vec<String> = self.meta_data.line_begin_indices.windows(2).enumerate().filter(|(index, _)| {
            *index < 2 || *index > (self.meta_data.line_begin_indices.len() - 3)
        }).map(|(index, slice)| {
            let (a, b) = (slice[0], slice[1]);
            format!("#{:0line_pad$}:{:0idx_pad$} - '{}'", index, a, &self.data[a..b].iter().map(|c| {
                if *c == '\n' {
                    &'/'
                } else {
                    c
                }
            }).collect::<String>(), line_pad = digits, idx_pad = idx_digits)
        }).collect();
        strs.insert(2, "........".into());
        for s in strs {
            println!("{}", s);
        }
        let &a = self.meta_data.line_begin_indices.get(self.meta_data.line_begin_indices.len()-1).unwrap_or(&0);
        println!("#{:0line_pad$}:{:0idx_pad$} - '{}'", self.meta_data.line_count() - 1, a, &self.data[a..self.len()].iter().map(|c| c).collect::<String>(), line_pad = digits, idx_pad = idx_digits);
        println!("{}", self.meta_data);
        self.debug_cursor();
    }

    pub fn get_cursor(&self) -> &BufferCursor {
        &self.cursor
    }

    pub fn debug_cursor(&self) {
        println!("Buffer cursor: {:?} - current element: {}", self.cursor, self.data.get(self.cursor.absolute()).map(|&c| {
            if c == '\n' {
                "NEWLINE".into()
            } else {
                let mut s = String::new();
                s.push(c);
                s
            }
            }).unwrap_or("EOF".into()));
    }

    pub fn str_view(&self, range: &std::ops::Range<usize>) -> &[char] {
        &self.data[range.clone()]
    }

    pub fn len(&self) -> usize { self.data.len() }

    pub fn insert_char(&mut self, ch: char) {
        debug_assert!(self.cursor.absolute() <= self.len(), "You can't insert something outside of the range of [0..len()]");
        if ch == '\n' {
            self.data.insert(self.cursor.absolute(), ch);
            self.cursor.pos += 1;
            self.cursor.col = 0;
            self.cursor.row += 1;
            self.meta_data.insert_line_begin(self.cursor.absolute(), self.cursor.row);
            self.meta_data.update_line_metadata_after_line(self.cursor.row, 1);
        } else {
            self.data.insert(self.cursor.absolute(), ch);
            self.cursor.pos += 1;
            self.cursor.col += 1;
            self.meta_data.update_line_metadata_after_line(self.cursor.row, 1);
        }
        self.size += 1;
        self.meta_data.set_buffer_size(self.size);
    }

    // todo(optimization): don't do the expensive rebuild of meta data after each delete. It's a pretty costly operation.
    pub fn delete(&mut self, dir: Movement) {
        if self.len() != 0 {
            match dir {
                Movement::Forward(kind, count) => {
                    match kind {
                        TextKind::Char => {
                            // clamp the count of characters removed, so we don't try to remove "outside" of our buffer
                            let count = if self.cursor.absolute() + count <= self.data.len() { count } else { self.data.len() - self.cursor.absolute() };
                            for _ in 0 .. count {
                                self.data.remove(self.cursor.absolute());
                            }
                        },
                        TextKind::Word => {
                            if let Some(c) = self.get(self.cursor_abs()) {
                                if c.is_whitespace() {
                                    if let Some(p) = self.find_next(|c| !c.is_whitespace()).map(|c| c.pos) {
                                        self.data.drain(self.cursor_abs() .. p);
                                    }
                                } else if c.is_alphanumeric() {
                                    if let Some(p) = self.find_next(|c| !c.is_alphanumeric()).map(|c| c.pos) {
                                        self.data.drain(self.cursor_abs() .. p);
                                    }
                                } else { 
                                    // If we are standing on, say +-/_* (non-alphanumerics) just delete one character at a time
                                    self.data.remove(self.cursor_abs());
                                }
                            }
                        },
                        TextKind::Line => todo!(),
                        TextKind::Block => todo!(),
                    }
                },
                Movement::Backward(kind, count) if self.cursor.absolute() != 0 => {
                    match kind {
                        TextKind::Char =>     {
                            let count = 
                            if self.cursor.absolute() as i64 - count as i64 >= 0 {
                                count
                            } else {
                                self.cursor.absolute()
                            };
                            self.cursor_move_backward(TextKind::Char, count);
                            for _ in 0 .. count {
                                self.remove();
                            }

                        },
                        TextKind::Word => {
                            if self.cursor_abs() == 0 { return; }
                            if let Some(true) = self.get(self.cursor_abs().wrapping_sub(1)).map(|c| c.is_whitespace()) {
                                if let Some(pos) = self.find_prev(|c| c.is_alphanumeric()).map(|c| c.absolute()) {
                                    self.data.drain(pos + 1 .. self.cursor_abs());
                                    self.cursor = self.cursor_from_metadata(pos+1).unwrap();
                                }
                            } else if let Some(true) = self.get(self.cursor_abs().wrapping_sub(1)).map(|c| c.is_alphanumeric()) {
                                if let Some(pos) = self.find_prev(|c| c.is_whitespace()).map(|c| c.absolute()) {
                                    self.data.drain(pos .. self.cursor_abs());
                                    self.cursor = self.cursor_from_metadata(pos).unwrap();
                                }
                            }
                        },
                        TextKind::Line => todo!(),
                        TextKind::Block => todo!(),
                    }
                },
                _ => {}
            }
            self.size = self.data.len();
        }
        self.rebuild_metadata();
    }


    /// Erases one character at the index of the cursor position
    pub fn remove(&mut self) {
        let idx = self.cursor.absolute();
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

    pub fn cursor_move_forward(&mut self, kind: TextKind, count: usize) {
        match kind {
            TextKind::Char => {
                if self.cursor.absolute() + count <= self.data.len() {
                    for _ in 0 .. count {
                        if let Some('\n') = self.data.get(self.cursor.absolute()) {
                            self.cursor.row += 1;
                            self.cursor.col = 0;
                        } else {
                            self.cursor.col += 1;
                        }
                        self.cursor.pos += 1;
                    }
                } else {
                    for _ in self.cursor.absolute() .. self.data.len() {
                        if let Some('\n') = self.data.get(self.cursor.absolute()) {
                            self.cursor.row += 1;
                            self.cursor.col = 0;
                        } else {
                            self.cursor.col += 1;
                        }
                        self.cursor.pos += 1;
                    }
                }
            },
            TextKind::Word => {
                if count == 1 {
                    if let Some(&c) = self.data.get(self.cursor.absolute()) {
                        if c.is_alphanumeric() {
                            self.cursor = self.find_next(|c| c.is_whitespace()).unwrap_or(BufferCursor { pos: self.len(), row: self.meta_data.line_count()-1, col: self.meta_data.get_line_buffer_index(self.meta_data.line_count()-1).map(|v| self.len() - v).unwrap() } );
                        } else if c.is_whitespace() {
                            self.cursor = self.find_next(|c| c.is_alphanumeric()).unwrap_or(BufferCursor { pos: self.len(), row: self.meta_data.line_count()-1, col: self.meta_data.get_line_buffer_index(self.meta_data.line_count()-1).map(|v| self.len() - v).unwrap() } );
                        }
                    }
                } else {
                    todo!("cursor movement spanning longer than a word not yet done");
                }
            },
            TextKind::Line => for _ in 0 .. count { self.cursor_move_down(); },
            TextKind::Block => todo!(),
        }

    #[cfg(debug_assertions)]
    {
        let (super::metadata::Index(_), super::metadata::Length(l)) = self.meta_data.get_line_info(self.cursor_row()).expect("fucking row all fucked up again");
        debugger_catch!(self.cursor_col() < l, "Col is outside of max position on this line!");
    }
    }

    pub fn cursor_move_backward(&mut self, kind: TextKind, count: usize) {
        match kind {
            TextKind::Char => {
                if self.cursor.absolute() as i64 - count as i64 > 0 {
                    for _ in 0 .. count {
                        self.cursor.pos -= 1;
                        if let Some('\n') = self.data.get(self.cursor.absolute()) {
                            self.cursor.row -= 1;
                            self.cursor.col = self.cursor.absolute() - self.find_prev_newline_pos_from(self.cursor.absolute()).unwrap_or(0);
                        } else {
                            self.cursor.col -= 1;
                        }
                        
                    }
                } else {
                    self.cursor = BufferCursor::default();
                }
            },
            TextKind::Word => {
                if count == 1 {
                    if let Some(&c) = self.data.get(self.cursor.absolute()) {
                        if c.is_alphanumeric() {
                            if let Some(cur) = self.find_prev(|c| c.is_whitespace()) {
                                self.cursor = cur;
                            }
                        } else if c.is_whitespace() {
                            if let Some(cur) = self.find_prev(|c| c.is_alphanumeric()) {
                                self.cursor = cur;
                            }
                        }
                    } else {
                        self.cursor_move_backward(TextKind::Char, 1);
                    }
                } else {
                    todo!("cursor movement spanning longer than a word not yet done");
                }
            },
            TextKind::Line => for _ in 0 .. count { self.cursor_move_up(); },
            TextKind::Block => todo!(),
        }
    }
    pub fn cursor_goto(&mut self, buffer_index: Index) {
        if self.is_valid_index(buffer_index) {
            self.cursor = self.cursor_from_metadata(*buffer_index).unwrap();
        }
    }

    pub fn rebuild_metadata(&mut self) {
        self.meta_data.clear_line_index_metadata();
        self.meta_data.push_new_line_begin(0);
        for (i, ch) in self.data.iter().enumerate() {
            if *ch == '\n' {
                self.meta_data.push_new_line_begin(i+1);
            }
        }
    }

    #[inline(always)]
    pub fn cursor_row(&self) -> usize {
        self.cursor.row
    }

    #[inline(always)]
    pub fn cursor_col(&self) -> usize {
        self.cursor.col
    }

    #[inline(always)]
    pub fn cursor_abs(&self) -> usize {
        self.cursor.pos
    }

    #[inline(always)]
    pub fn meta_data(&self) -> &MetaData {
        &self.meta_data
    }
}

/// Private interface implementation
impl SimpleBuffer {

    /// Takes a buffer index and tries to build a BufferCursor, using the MetaData member of the SimpleBuffer
    /// After some deliberation, this is the core function that all movement functions of the Buffer will use.
    /// Instead of having each function individually updating the cursor and keeping track of rows and columns
    /// They explicitly only deal with absolute positions/indices, and before returning, calls this function
    /// to return an Option of a well formed BufferCursor
    fn cursor_from_metadata(&self, absolute_position: usize) -> Option<BufferCursor> {
        debugger_catch!(absolute_position <= self.len(), "absolute position is outside of the buffer");
        if absolute_position == self.len() {
            Some((absolute_position, self.meta_data.line_count() - 1, *self.meta_data.line_begin_indices.last().unwrap()).into())
        } else {
            self.meta_data.get_line_number_of_buffer_index(absolute_position).and_then(|line| {
                self.meta_data.get_line_buffer_index(line).map(|line_begin| {
                    (absolute_position, line, absolute_position - line_begin).into()
                })
            })
        }
    }

    fn find_next(&self, f: fn(char) -> bool) -> Option<BufferCursor> {
        self.iter()
        .enumerate()
        .skip(self.cursor_abs()+1)
        .find(|(_, &ch)| {
            f(ch)
        }).and_then(|(i, _)| {
            self.cursor_from_metadata(i)
        })
    }

    fn find_prev(&self, f: fn(char) -> bool) -> Option<BufferCursor> {
        let cursor_pos = self.cursor_abs();
        self.data[..cursor_pos].iter().rev().position(|&c| f(c)).and_then(|char_index_predicate_true_for| {
            self.cursor_from_metadata(cursor_pos - char_index_predicate_true_for - 1)
        })
    }

    fn find_prev_newline_pos_from(&self, abs_pos: usize) -> Option<usize> {
        if abs_pos >= self.data.len() {
            self.meta_data.line_begin_indices.last().map(|v| *v)
        } else {
            let reversed_abs_position = self.data.len() - abs_pos;
            let res = self.iter().rev().skip(reversed_abs_position).position(|c| *c == '\n').and_then(|v| Some(abs_pos - (v)));
            res
        }
    }

    fn cursor_move_up(&mut self) {
        if self.cursor_row() == 0 { return; }
        let prior_line = self.cursor_row()-1;
        self.cursor = self.meta_data.get_line_buffer_index(prior_line).and_then(|index| {
            self.meta_data.get_line_length_of(prior_line).map(|super::metadata::Length(len)| {
                let pos = index + min(len-1, self.cursor_col());
                self.cursor_from_metadata(pos)
            }).unwrap_or(self.cursor_from_metadata(index))
        }).unwrap_or(BufferCursor::default())
    }

    fn cursor_move_down(&mut self) {
        // This is all the lines up until the 3rd to last - normal behavior, 2nd to last means we are moving into the last, other behavior applies in the else branch
        
        let next_line_index = self.cursor_row() + 1;
        let new_cursor = self.line_length(super::metadata::Index(next_line_index))
            .and_then(|next_line_length| {
                let Line(line_begin) = self.meta_data.get(self.cursor.row + 1).unwrap();
                let new_buffer_index = 
                line_begin + if self.cursor_col() <= next_line_length -1 {
                    self.cursor_col()
                } else {
                    next_line_length-1
                };
                Some(self.cursor_from_metadata(new_buffer_index).unwrap())
            });
        self.cursor = new_cursor.unwrap_or(self.cursor);
    }

    fn is_valid_index(&self, index: Index) -> bool {
        self.len() >= *index
    }
}