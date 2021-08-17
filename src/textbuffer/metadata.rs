use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::iter::Step;
use std::path::{Path, PathBuf};

/// Macros used in this module
use crate::Assert;
use crate::IndexingType;

use super::CharBuffer;

IndexingType!(/** Wrapper around usize to display that this is an index type */,
    Index, usize);
IndexingType!(/** A wrapper around the usize type, meant to represent line numbers */,
    Line, usize);
IndexingType!( /** Wrapper around usize to signal that this value holds a column position */,
    Column, usize);
IndexingType!( /** Wrapper around a usize to signal that this value holds the length of a range in buffer */,
    Length, usize);

impl Length {
    pub fn as_column(self) -> Column {
        Column(*self)
    }
}

#[derive(Debug)]
pub struct MetaData {
    pub file_name: Option<PathBuf>,
    pub line_begin_indices: Vec<Index>,
    pub buffer_size: usize,
    /// real simple approach to checking file changes
    buf_hash: u64,
    hash_on_open: u64,
}

impl std::fmt::Display for MetaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MetaData {{ File name: {:?}, Lines: {} }}", self.file_name, self.line_begin_indices.len())
    }
}

impl MetaData {
    pub fn new(file_name: Option<&Path>) -> MetaData {
        MetaData {
            file_name: file_name.map(|p| p.to_path_buf()),
            line_begin_indices: vec![Index(0)],
            buffer_size: 0,
            buf_hash: 0,
            hash_on_open: 0,
        }
    }

    /// Guaranteed to always be at least 1, no matter what.
    pub fn line_count(&self) -> usize {
        self.line_begin_indices.len()
    }

    pub fn line_length(&self, line: Line) -> Option<Length> {
        self.get(line).and_then(|a| {
            self.get(line.offset(1))
                .map(|b| Some(Length(*b - *a)))
                .unwrap_or(Some(Length(self.buffer_size - *a)))
        })
    }

    pub fn get_line_length_of(&self, line_index: Line) -> Option<Length> {
        Assert!(*line_index <= self.line_begin_indices.len(), "requested line number is outside of buffer");
        self.line_begin_indices
            .windows(2)
            .skip(*line_index)
            .map(|v| Length(*v[1] - *v[0]))
            .take(1)
            .collect::<Vec<Length>>()
            .get(0)
            .map(|v| v.clone())
    }

    /// Get absolute buffer index of beginning of line line_number
    pub fn get_line_start_index(&self, line_number: Line) -> Option<Index> {
        self.line_begin_indices.get(*line_number).cloned()
    }

    /// This is a safe operation _always_ since there will always be one line in any buffer, regardless
    pub fn get_last_line(&self) -> Index {
        unsafe { *self.line_begin_indices.get_unchecked(self.line_begin_indices.len() - 1) }
    }

    /// Returns the buffer indices of the beginning of line a and b. If *either* line does not exist in buffer, function will return None
    pub fn get_byte_indices_of_lines(&self, line_a: Line, line_b: Line) -> (Option<Index>, Option<Index>) {
        let a = self.get_line_start_index(line_a);
        let b = self.get_line_start_index(line_b);
        (a, b)
    }

    /// Finds what line in buffer, the absolute cursor position buffer_index points into
    /// * `buffer_index` - the buffer index, we want to find the line that it lives on for
    pub fn get_line_number_of_buffer_index(&self, buffer_index: Index) -> Option<usize> {
        self.line_begin_indices
            .windows(2)
            .enumerate()
            .find(|(_, slice)| {
                // This is safe. Slice will *always* be a Some([a, b]) or a None, at which point this loop exits already
                let (a, b) = (slice[0], slice[1]);
                a <= buffer_index && buffer_index < b
            })
            .map(|(index, _)| index)
            .or_else(|| if *buffer_index <= self.buffer_size { Some(self.line_begin_indices.len() - 1) } else { None })
    }

    pub fn get(&self, line: Line) -> Option<Index> {
        self.line_begin_indices.get(*line).cloned()
    }

    pub fn get_line_info(&self, line_number: Line) -> Option<(Index, Length)> {
        self.get(line_number).map(|Index(index)| {
            if let Some(i) = self.get(line_number.offset(1)) {
                (Index(index), Length(*i - index))
            } else {
                (Index(index), Length(self.buffer_size - index))
            }
        })
    }

    pub fn get_lines<T>(&self, lines: T) -> Option<&[Index]>
    where
        T: std::ops::RangeBounds<usize> + std::slice::SliceIndex<[Index], Output = [Index]>,
    {
        self.line_begin_indices.get(lines)
    }

    /// Insert line begin buffer index, of new line created at line_number
    pub fn insert_line_begin(&mut self, buffer_index: Index, line_number: Line) {
        self.line_begin_indices.insert(*line_number, buffer_index);
    }

    /// Add line begin buffer index of last line in buffer
    pub fn push_new_line_begin(&mut self, buffer_index: Index) {
        self.line_begin_indices.push(buffer_index);
    }

    /// Clears the line index metadata
    pub fn clear_line_index_metadata(&mut self) {
        self.line_begin_indices.clear();
        self.line_begin_indices.push(Index(0));
    }

    pub fn update_line_metadata_after_line(&mut self, line: Line, shift_amount: i64) {
        self.line_begin_indices.iter_mut().skip(*line + 1).for_each(|l| {
            *l = l.offset_mut(shift_amount as _);
        });
    }

    pub fn update_line_metadata_from_line(&mut self, line: Line, shift_amount: usize) {
        self.line_begin_indices.iter_mut().skip(*line).for_each(|l| {
            *l = l.offset(shift_amount as _);
        });
    }

    pub fn set_buffer_size(&mut self, size: usize) {
        self.buffer_size = size;
    }

    pub fn set_checksum(&mut self, sum: u64) {
        self.buf_hash = sum;
    }

    pub fn set_pristine_hash(&mut self, sum: u64) {
        self.hash_on_open = sum;
    }

    pub fn get_pristine_hash(&self) -> u64 {
        self.hash_on_open
    }

    pub fn get_current_checksum(&self) -> u64 {
        self.buf_hash
    }
}

pub fn calculate_hash<'a, T: CharBuffer<'a> + Hash + Sized>(buf: &T) -> u64 {
    let mut s = DefaultHasher::new();
    buf.hash(&mut s);
    if let Some(p) = buf.file_name() {
        p.hash(&mut s);
    }
    let l = buf.len();
    l.hash(&mut s);
    s.finish()
}
