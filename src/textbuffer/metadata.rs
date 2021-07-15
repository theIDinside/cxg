use std::path::{Path, PathBuf};

/// Macros used in this module
use crate::debugger_catch;
use crate::IndexingType;

IndexingType!(/** Wrapper around usize to display that this is an index type */, 
    Index, usize);
IndexingType!(/** A wrapper around the usize type, meant to represent line numbers */,
    Line, usize);
IndexingType!( /** Wrapper around usize to signal that this value holds a column position */,
    Column, usize);
IndexingType!( /** Wrapper around a usize to signal that this value holds the length of a range in buffer */,
    Length, usize);

#[derive(Debug)]
pub struct MetaData {
    pub file_name: Option<PathBuf>,
    pub line_begin_indices: Vec<Index>,
    pub buffer_size: usize,
}

impl std::fmt::Display for MetaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MetaData {{ File name: {:?}, Lines: {} }}",
            self.file_name,
            self.line_begin_indices.len()
        )
    }
}

impl MetaData {
    pub fn new(file_name: Option<&Path>) -> MetaData {
        MetaData {
            file_name: file_name.map(|p| p.to_path_buf()),
            line_begin_indices: vec![Index(0)],
            buffer_size: 0,
        }
    }

    /// Guaranteed to always be at least 1, no matter what.
    pub fn line_count(&self) -> usize {
        self.line_begin_indices.len()
    }

    pub fn get_line_length_of(&self, line_index: Line) -> Option<Length> {
        debugger_catch!(
            *line_index <= self.line_begin_indices.len(),
            "requested line number is outside of buffer"
        );
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

    /// Returns the buffer indices of the beginning of line a and b. If *either* line does not exist in buffer, function will return None
    pub fn get_byte_indices_of_lines(&self, line_a: Line, line_b: Line) -> (Option<Index>, Option<Index>) {
        let a = self.get_line_start_index(line_a);
        let b = self.get_line_start_index(line_b);
        (a, b)
    }

    /// Finds what line in buffer, the absolute cursor position buffer_index is at
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
            .or_else(|| {
                if *buffer_index <= self.buffer_size {
                    Some(self.line_begin_indices.len() - 1)
                } else {
                    None
                }
            })
    }

    pub fn get(&self, line: Line) -> Option<Index> {
        self.line_begin_indices.get(*line).cloned()
    }

    pub fn get_line_info(&self, line_number: Line) -> Option<(Index, Length)> {
        self.get(line_number).map(|Index(index)| {
            if let Some(i) = self.get(line_number + Line(1)) {
                (Index(index), Length(*i - index))
            } else {
                (Index(index), Length(self.buffer_size - index))
            }
        })
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
    }

    pub fn update_line_metadata_after_line(&mut self, line: Line, shift_amount: i64) {
        self.line_begin_indices.iter_mut().skip(*line + 1).for_each(|l| {
            if shift_amount < 0 {
                *l -= Index((-1 * shift_amount) as usize);
            } else {
                *l += Index(shift_amount as usize);
            }
        });
    }

    pub fn update_line_metadata_from_line(&mut self, line: Line, shift_amount: usize) {
        self.line_begin_indices.iter_mut().skip(*line).for_each(|l| {
            *l += Index(shift_amount);
        });
    }

    pub fn set_buffer_size(&mut self, size: usize) {
        self.buffer_size = size;
    }
}
