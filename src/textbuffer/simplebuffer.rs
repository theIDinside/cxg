use std::{io::Read, path::Path};

use crate::DebuggerCatch;

use super::{cursor::BufferCursor, metadata::MetaData};



pub struct SimpleBuffer {
    _id: u32,
    pub data: Vec<char>,
    cursor: BufferCursor,
    size: usize,
    meta_data: MetaData
}

pub enum TextKind {
    Char,
    Word,
    Line,
    Block
}

pub enum Movement {
    Forward(TextKind, usize),
    Backward(TextKind, usize)
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
        &self.data.get(range).expect("Range out of length")
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


    pub fn debug_metadata(&self) {
        println!("#Line index:Buffer Index Pos - [line contents, newlines represented as /]");
        println!("-----------------------------");
        let digits = self.meta_data.line_begin_indices.len().digits();
        let idx_digits = self.len().digits();
        for (index, slice) in self.meta_data.line_begin_indices.windows(2).enumerate() {
                let (a, b) = (slice[0], slice[1]);
                println!("#{:0line_pad$}:{:0idx_pad$} - '{}'", index, a, &self.data[a..b].iter().map(|c| {
                    if *c == '\n' {
                        &'/'
                    } else {
                        c
                    }
                }).collect::<String>(), line_pad = digits, idx_pad = idx_digits);
        }
        let &a = self.meta_data.line_begin_indices.get(self.meta_data.line_begin_indices.len()-1).unwrap_or(&0);
        println!("#{:0line_pad$}:{:0idx_pad$} - '{}'", self.meta_data.line_count() - 1, a, &self.data[a..self.len()].iter().map(|c| c).collect::<String>(), line_pad = digits, idx_pad = idx_digits);
        println!("{:?}, total lines: {}", self.meta_data, self.meta_data.line_begin_indices.len());
        self.debug_cursor();
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
    }

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
                            
                        },
                        TextKind::Line => todo!(),
                        TextKind::Block => todo!(),
                    }
                },
                Movement::Backward(kind, count) if self.cursor.absolute() != 0 => {
                    match kind {
                        TextKind::Char => {
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
                        TextKind::Word => todo!(),
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
                            if let Some(cur) = self.find_next(|c| c.is_whitespace()) {
                                self.cursor = cur;
                            }
                        } else {
                            if let Some(cur) = self.find_next(|c| c.is_alphanumeric()) {
                                self.cursor = cur;
                            }
                        }
                    }
                } else {
                    todo!("cursor movement spanning longer than a word not yet done");
                }
            },
            TextKind::Line => {
                for _ in 0..count {
                    if let Some(c) = self.find_next_newline_cursor() {
                        self.cursor = c;
                    } else {
                        break;
                    }
                }
            },
            TextKind::Block => todo!(),
        }

        self.debug_cursor();
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
                println!("Moving backwards 1 word...");
                if count == 1 {
                    if let Some(&c) = self.data.get(self.cursor.absolute()) {
                        if c.is_alphanumeric() {
                            if let Some(cur) = self.find_prev(|c| c.is_whitespace()) {
                                self.cursor = cur;
                            }
                        } else {
                            if let Some(cur) = self.find_prev(|c| c.is_alphanumeric()) {
                                self.cursor = cur;
                            }
                        }
                    }
                } else {
                    todo!("cursor movement spanning longer than a word not yet done");
                }
            },
            TextKind::Line => todo!(),
            TextKind::Block => todo!(),
        }
        self.debug_cursor();
    }

    pub fn cursor_move(&mut self, dir: Movement) {
        match dir {
            Movement::Forward(kind, count) => self.cursor_move_forward(kind, count),
            Movement::Backward(kind, count) => {
                match kind {
                    TextKind::Char => todo!(),
                    TextKind::Word => todo!(),
                    TextKind::Line => {
                        for _ in 0..count {
                            if let Some(c) = self.find_prev_newline_cursor() {
                                self.cursor = c;
                            } else {
                                break;
                            }
                        }
                    },
                    TextKind::Block => todo!(),
                }
            }
        }
    }

    pub fn rebuild_metadata(&mut self) {
        self.meta_data.clear_line_index_metadata();
        let mut line_begin_index = 0;
        self.meta_data.push_new_line_begin(line_begin_index);
        for &c in self.data.iter() {
            line_begin_index += 1;
            if c == '\n' {
                self.meta_data.push_new_line_begin(line_begin_index);
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
    fn find_next(&self, f: fn(char) -> bool) -> Option<BufferCursor> {
        let mut cur = self.cursor.clone();
        for &c in self.iter().skip(self.cursor.pos+1) {
            cur.forward(c);
            if f(c) {
                return Some(cur)
            }
        }
        None
    }

    fn find_prev(&self, f: fn(char) -> bool) -> Option<BufferCursor> {
        let mut cur = self.cursor.clone();
        let start_skip = self.len() - self.cursor.pos;
        if start_skip == 0 {
            cur.pos -= 1;
        }
        for &c in self.iter().rev().skip(start_skip) {
            print!("{}", c);
            let r = cur.backward(c);
            match r {
                super::cursor::CursorMovement::Valid => {},
                super::cursor::CursorMovement::InvalidColumn => {
                    let line_number = self.meta_data.get_line_number_of_buffer_index(cur.absolute()).expect("failed to get line number by buffer index");
                    assert_eq!(line_number, cur.row);
                    let line_begin_index = self.meta_data.get_line_buffer_index(line_number).expect("failed to get line begin index");
                    cur.col = cur.pos - line_begin_index;
                }
            }
            if f(c) {
                println!("Found cursor pos: {:?}", cur);
                return Some(cur)
            }
        }
        None
    }

    fn find_prev_newline_pos_from(&self, abs_pos: usize) -> Option<usize> {
        if abs_pos >= self.data.len() {
            None
        } else {
            let reversed_abs_position = self.data.len() - abs_pos;
            let res = self.iter().rev().skip(reversed_abs_position).position(|c| *c == '\n').and_then(|v| Some(abs_pos - (v)));
            res
        }
    }

    fn find_next_newline_cursor(&self) -> Option<BufferCursor> {
        if self.cursor.pos == self.data.len() {
            return None;
        }
        let mut copy = self.cursor.clone();
        if let Some('\n') = self.data.get(self.cursor.pos) {
            copy.pos += 1;
            copy.row += 1;
            copy.col = 0;
        }
        self.iter().skip(copy.pos).position(|c| *c == '\n').and_then(|pos| {
            copy.col += pos;
            copy.pos += pos;
            Some(copy)
        })
    }

    fn find_prev_newline_cursor(&self) -> Option<BufferCursor> {
        if self.cursor.pos == 0 || self.cursor.row == 0 {
            return None;
        }
        let mut c = self.cursor.clone();
        debug_assert!(c.pos - (c.col + 1) == {
            let mut c2 = c.clone();
            c2.pos -= c2.col + 1;
            c2.pos
        }, "debug checking the compiler?");

        c.pos -= c.col + 1;
        c.row -= 1;
        let skip_to = self.data.len() - c.pos;
        c.col = self.iter().rev().skip(skip_to).position(|e| *e == '\n').and_then(|v| Some(v-1)).expect("calculating column position failed");

        Some(c)
    }
}