use std::path::{PathBuf, Path};

#[derive(Debug)]
pub struct MetaData {
    pub file_name: Option<PathBuf>,
    pub line_begin_indices: Vec<usize>,
}

impl MetaData {
    pub fn new(file_name: Option<&Path>) -> MetaData {
        MetaData {
            file_name: file_name.map(|p| p.to_path_buf()),
            line_begin_indices: vec![0]
        }
    }

    pub fn line_count(&self) -> usize { self.line_begin_indices.len() }

    /// Get absolute buffer index of beginning of line line_number
    pub fn get_line_buffer_index(&self, line_number: usize) -> Option<usize> {
        self.line_begin_indices.get(line_number).map(|v| *v)
    }

    /// Returns the buffer indices of the beginning of line a and b. If *either* line does not exist in buffer, function will return None
    pub fn get_byte_indices_of_lines(&self, line_a: usize, line_b: usize) -> (Option<usize>, Option<usize>) {
        let a = self.get_line_buffer_index(line_a);
        let b = self.get_line_buffer_index(line_b);
        (a, b)
    }

    /// Finds what line in buffer, the absolute cursor position buffer_index is at
    pub fn get_line_number_of_buffer_index(&self, buffer_index: usize) -> Option<usize> {
        self.line_begin_indices.windows(2).enumerate().find(|(_, slice)| {
            // This is safe. Slice will *always* be a Some([a, b]) or a None, at which point this loop exits already
            let (a, b) = (slice[0], slice[1]);
            a <= buffer_index && buffer_index <= b
        }).map(|(index, _)| {
            index
        })
    }

    /// Insert line begin buffer index, of new line created at line_number
    pub fn insert_line_begin(&mut self, buffer_index: usize, line_number: usize) {
        self.line_begin_indices.insert(line_number, buffer_index);
    }

    /// Add line begin buffer index of last line in buffer
    pub fn push_new_line_begin(&mut self, buffer_index: usize) { self.line_begin_indices.push(buffer_index); }

    /// Clears the line index metadata
    pub fn clear_line_index_metadata(&mut self) { self.line_begin_indices.clear(); }

    pub fn update_line_metadata_after_line(&mut self, line: usize, shift_amount: usize) {
        self.line_begin_indices.iter_mut().skip(line+1).for_each(|l| {
            *l += shift_amount;
        });
    }
}