use std::path::Path;

use crate::{debugger_catch, textbuffer::cursor::BufferCursor};

use self::metadata::{calculate_hash, MetaData};

pub mod buffers;
pub mod cursor;
pub mod gb;
pub mod metadata;
pub mod simple;

#[derive(Debug)]
pub enum TextKind {
    Char,
    Word,
    Line,
    Block,
}

#[derive(Debug)]
pub enum Movement {
    Forward(TextKind, usize),
    Backward(TextKind, usize),
    Begin(TextKind),
    End(TextKind),
}

pub enum SelectMovement {
    /// Communicate to the buffer to invalidate any existing meta cursor and move the edit cursor
    NoSelection(Movement),
    /// Communicate to the buffer to set a new meta cursor, if no meta cursor is set, at current edit_cursor position, before moving the edit_cursor.
    /// If a meta_cursor already exist, leave the meta cursor intact and move the edit_cursor (effectively changing the selection range without changing it's start point)
    ContinueSelection(Movement),
}

pub enum BufferState {
    Empty,
    Pristine,
    NotSaved,
    NotSavedToDisk,
}

pub trait CharBuffer<'a>: std::hash::Hash {
    type ItemIterator: Iterator<Item = &'a char>;
    // todo(feature): Add support for multiple cursors, whether they be implemented as multi-cursors or just as macros pretending to be multiple cursors is utterly irrelevant
    /// Inserts character att current cursor position
    fn insert(&mut self, data: char);
    /// Deletes a TextKind at given Movement direction. Deleting a character forward, requires a parameter of Movement::Forward(TextKind::Char, 1)
    /// Deleting a line is very similar; Movement::Forward(TextKind::Line, 1);
    fn delete(&mut self, dir: Movement);
    /// Copies a slice into the buffer, using memcpy. If there's not enough space, the buffer will have to re-allocate it's data first
    fn insert_slice_fast(&mut self, slice: &[char]);

    /// Moves the cursor in the buffer
    fn move_cursor(&mut self, dir: Movement);

    /// Moves the cursor and sets the meta cursor accordingly.
    fn select_move_cursor(&mut self, movement: Movement);
    /// Capacity of the buffer
    fn capacity(&self) -> usize;
    /// Size of the used space in the buffer
    fn len(&self) -> usize;
    /// Check if the buffer is empty
    fn empty(&self) -> bool {
        self.len() == 0
    }

    /// Hashes the contents of the buffer, and compares it to the last saved state.
    fn pristine(&self) -> bool
    where
        Self: std::hash::Hash + Sized,
    {
        let hash = calculate_hash(self);
        self.meta_data().get_checksum() == hash
    }

    /// Available free space in the buffer
    fn available_space(&self) -> usize {
        self.capacity() - self.len()
    }
    /// Rebuilds the buffer meta data, containing new line indices in the buffer.
    fn rebuild_metadata(&mut self);

    /// Constructs a BufferCursor, from an absolute index position into the buffer, using the metadata
    fn cursor_from_metadata(&self, absolute_position: metadata::Index) -> Option<BufferCursor> {
        use metadata::Column as Col;
        use metadata::Index as Idx;
        use metadata::Line;
        let absolute_position = *absolute_position;
        debugger_catch!(absolute_position <= self.len(), "absolute position is outside of the buffer");
        if absolute_position == self.len() {
            Some(BufferCursor {
                pos: Idx(absolute_position),
                row: Line(self.meta_data().line_count() - 1),
                col: Col(self
                    .meta_data()
                    .line_begin_indices
                    .last()
                    .map(|v| absolute_position - **v as usize)
                    .unwrap()),
            })
        } else {
            self.meta_data()
                .get_line_number_of_buffer_index(Idx(absolute_position))
                .and_then(|line| {
                    self.meta_data()
                        .get_line_start_index(Line(line))
                        .map(|line_begin| (absolute_position, line, absolute_position - *line_begin).into())
                })
        }
    }

    /// Get a reference to the MetaData sturcture
    fn meta_data(&self) -> &MetaData;
    /// Get an iterator to the data of this buffer
    fn iter(&'a self) -> Self::ItemIterator;

    /// Get current cursor line position
    fn cursor_row(&self) -> metadata::Line;
    /// Get current cursor column position
    fn cursor_col(&self) -> metadata::Column;
    /// Get absolute position in buffer
    fn cursor_abs(&self) -> metadata::Index;

    /// Moves cursor to absolute buffer index
    fn cursor_goto(&mut self, buffer_index: metadata::Index) {
        if self.is_valid_index(buffer_index) {
            self.set_cursor(self.cursor_from_metadata(buffer_index).unwrap());
        }
    }

    /// Overwrite cursor. This is an inherently unsafe function, if you overwrite the cursor with bad data (such as an index outside of the buffer) that's on you
    fn set_cursor(&mut self, cursor: BufferCursor);

    /// Checks if the index is a valid buffer index
    fn is_valid_index(&self, index: metadata::Index) -> bool {
        self.len() >= *index
    }

    fn clear(&mut self);

    fn load_file(&mut self, path: &Path);

    fn save_file(&mut self, path: &Path);

    fn file_name(&self) -> Option<&Path>;

    fn copy(&mut self, range: std::ops::Range<usize>) -> String;
}

/// Traits that defines behavior for cloning a sub string of the buffer.
/// This is particularly useful for gap buffers, as we can not safely assume to be able to return ranges of references to the underlying data
pub trait SubstringClone {
    fn read_string(&self, range: std::ops::Range<usize>) -> String;
}

#[cfg(test)]
mod tests {
    use super::gb::gap_buffer::GapBuffer as GB;
    // use super::Gap::GapBuffer as GB;
    use super::SubstringClone;

    #[test]
    fn test_iteration() {
        let text_data = include_str!("test_textfile.source");
        let mut gb = GB::new_with_capacity(text_data.len());
        gb.map_into(text_data.chars());
        assert_eq!(gb.len(), text_data.len(), "Gap buffer not matching correct datal length");
    }

    #[test]
    fn test_get_entire_content_but_provide_larger_range_parameter() {
        let mut gb = GB::new_with_capacity(13);
        let mfer: Vec<char> = " go fuck yourself biatch!".chars().collect();
        let assertion_ok = "hello go fuck yourself biatch! world";
        gb.map_into("hello world".chars());
        gb.set_gap_position(5);
        gb.insert_slice(&mfer[..]);
        println!("buffer contents: {:?}", gb.read_string(0..assertion_ok.len()));
        let two_times: Vec<_> = assertion_ok.chars().chain(assertion_ok.chars()).collect();
        gb.set_gap_position(0);
        gb.insert_slice(&assertion_ok.chars().collect::<Vec<_>>()[..]);
        let larger_range = 0..two_times.len() * 10;
        gb.debug();
        assert_eq!(gb.read_string(larger_range), two_times.iter().collect::<String>());
    }

    #[test]
    fn test_slice_insertion() {
        let mut gb = GB::new_with_capacity(13);
        let mfer: Vec<char> = " go fuck yourself biatch!".chars().collect();
        let assertion_ok = "hello go fuck yourself biatch! world";
        gb.map_into("hello world".chars());
        gb.set_gap_position(5);
        gb.insert_slice(&mfer[..]);
        println!("buffer contents: {:?}", gb.read_string(0..assertion_ok.len()));
        assert_eq!(gb.read_string(0..assertion_ok.len()).len(), assertion_ok.len());
        assert_eq!(gb.read_string(0..assertion_ok.len()), assertion_ok);
        let two_times: Vec<_> = assertion_ok.chars().chain(assertion_ok.chars()).collect();
        gb.set_gap_position(0);
        gb.insert_slice(&assertion_ok.chars().collect::<Vec<_>>()[..]);
        assert_eq!(gb.read_string(0..two_times.len()), two_times.iter().collect::<String>());
    }

    #[test]
    fn test_subslices() {
        let hw = "hello world! Good morning vietnam";
        let mut gb = GB::new_with_capacity(hw.len());
        gb.map_into(hw.chars());
        gb.set_gap_position(5);
        {
            let (sub_a, sub_b) = gb.data_slices();
            // sub_a === "hello", sub_b === " world! Good morning vietnam"
            for (gb_char, subs_char) in hw.chars().zip(sub_a.iter().chain(sub_b.iter())) {
                assert_eq!(gb_char, *subs_char, "characters were not equal");
            }
        }
    }

    #[test]
    fn test_gapbuffer_insert() {
        let mut gb = GB::new();
        gb.map_into("hello world!".chars());
        assert_eq!(gb.get(0), Some(&'h'));
    }

    #[test]
    fn test_gapbuffer_read_string() {
        let mut gb = GB::new();
        gb.map_into("hello world!".chars());
        assert_eq!(gb.read_string(0..5), "hello");
    }

    #[test]
    fn test_gapbuffer_dump_to_string() {
        let mut gb = GB::new();
        gb.map_into("hello world!".chars());
        assert_eq!(gb.read_string(0..25), "hello world!");
    }

    #[test]
    fn test_insert_move_insert() {
        let mut gb = GB::new();
        gb.map_into("hello world!".chars());
        assert_eq!(gb.read_string(0..25), "hello world!");
        gb.set_gap_position(6);
        gb.map_into("fucking ".chars());
        assert_eq!(gb.read_string(0..25), "hello fucking world!");
    }

    #[test]
    fn test_insert_lines() {}

    #[test]
    fn test_remove_char() {
        let mut gb = GB::new();
        gb.map_into("hello world!".chars());
        gb.remove();
        assert_eq!("hello world", gb.read_string(0..25));
    }

    #[test]
    fn test_insert_newline() {
        let mut gb = GB::new();
        gb.map_into("hello world".chars());
        gb.set_gap_position(5);
        // gb.delete();
        gb.insert_item('\n');
        assert_eq!("hello\n world", gb.read_string(0..25));
    }

    #[test]
    fn test_remove_world_from_hello_world() {
        let mut gb = GB::new();
        gb.map_into("hello world".chars());
        gb.set_gap_position(6);
        for _ in 0..5 {
            gb.delete();
        }
        assert_eq!("hello ", gb.read_string(0..25));
    }

    #[test]
    fn test_replace_world_with_simon() {
        let mut gb = GB::new();
        gb.map_into("hello world".chars());
        gb.set_gap_position(6);
        for _ in 0..5 {
            gb.delete();
        }
        let simon: String = "Simon".into();
        gb.map_into(simon.chars());
        assert_eq!("hello Simon", gb.read_string(0..25));
    }
}
