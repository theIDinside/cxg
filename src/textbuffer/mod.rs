use crate::textbuffer::cursor::BufferCursor;
use crate::Assert;
use serde::{Deserialize, Serialize};
use std::path::Path;

use self::{
    metadata::{calculate_hash, MetaData},
    operations::LineOperation,
};

/// Buffer manager module
pub mod buffers;
/// ContiguousBuffer module - a buffer that keeps a simple String-like buffer, no extra bookkeeping tricks like for instance GapBuffer
pub mod contiguous;
/// Cursor module - definitions of BufferCursor and MetaCursor objects
pub mod cursor;
/// GapBuffer module
pub mod gb;
/// Buffer metadata module
pub mod metadata;
// Definitions of abstractions of operations on buffers
pub mod operations;

#[derive(Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Deserialize, Serialize)]
pub enum TextKind {
    Char,
    Word,
    Line,
    Block,
    Page,
    File,
}

impl std::str::FromStr for TextKind {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Char" => Ok(TextKind::Char),
            "Word" => Ok(TextKind::Word),
            "Line" => Ok(TextKind::Line),
            "Block" => Ok(TextKind::Block),
            "Page" => Ok(TextKind::Page),
            "File" => Ok(TextKind::File),
            _ => Err("Unknown Text Kind type"),
        }
    }
}
#[derive(Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Deserialize, Serialize)]
pub enum Movement {
    Forward(TextKind, usize),
    Backward(TextKind, usize),
    Begin(TextKind),
    End(TextKind),
}

impl std::str::FromStr for Movement {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if &s[0.."Forward(".len()] == "Forward(" {
            let items: Vec<&str> = s["Forward(".len()..s.len() - 1].split_ascii_whitespace().collect();
            if let (Some(kind), Some(count)) = (items.get(0), items.get(1)) {
                let t_kind = TextKind::from_str(&kind[..kind.len() - 1]);
                let count = count.parse::<usize>();
                t_kind
                    .ok()
                    .zip(count.ok())
                    .map_or(Err("Could not parse movement"), |(t, c)| Ok(Movement::Forward(t, c)))
            } else {
                Err("could not create Movement from str")
            }
        } else if &s[0.."Backward(".len()] == "Backward(" {
            let items: Vec<&str> = s["Backward(".len()..s.len() - 1].split_ascii_whitespace().collect();
            if let (Some(kind), Some(count)) = (items.get(0), items.get(1)) {
                let t_kind = TextKind::from_str(&kind[..kind.len() - 1]);
                let count = count.parse::<usize>();
                t_kind
                    .ok()
                    .zip(count.ok())
                    .map_or(Err("Could not parse movement"), |(t, c)| Ok(Movement::Forward(t, c)))
            } else {
                Err("could not create Movement from str")
            }
        } else if &s[0.."Begin(".len()] == "Begin(" {
            let kind = TextKind::from_str(&s["Begin(".len()..s.len() - 1]);
            if let Ok(kind) = kind {
                Ok(Movement::Begin(kind))
            } else {
                Err("could not create Movement from str")
            }
        } else if &s[0.."End(".len()] == "End(" {
            let kind = TextKind::from_str(&s["End(".len()..s.len() - 1]);
            if let Ok(kind) = kind {
                Ok(Movement::Begin(kind))
            } else {
                Err("could not create Movement from str")
            }
        } else {
            Err("could not create Movement from str")
        }
    }
}

impl Movement {
    pub fn transform_page_param(self, view_page_size: usize) -> Movement {
        match self {
            Movement::Forward(a, c) => match a {
                TextKind::Page => Movement::Forward(TextKind::Line, c * view_page_size),
                _ => self,
            },
            Movement::Backward(a, c) => match a {
                TextKind::Page => Movement::Backward(TextKind::Line, c * view_page_size),
                _ => self,
            },
            Movement::Begin(a) => match a {
                TextKind::Page => {
                    todo!("Movement to Begin TextKind::Page does not make sense")
                }
                _ => self,
            },
            Movement::End(a) => match a {
                TextKind::Page => {
                    todo!("Movement to End TextKind::Page does not make sense")
                }
                _ => self,
            },
        }
    }
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
    /// * `data` - the element to be inserted into the buffer
    /// * `register_history` - if this operation should be registered in the history stack
    fn insert(&mut self, data: char, register_history: bool);

    /// Deletes a TextKind at given Movement direction. Deleting a character forward, requires a parameter of Movement::Forward(TextKind::Char, 1)
    /// Deleting a line is very similar; Movement::Forward(TextKind::Line, 1);
    fn delete(&mut self, dir: Movement);

    /// Deletes element at buffer position. Index must be a valid one, otherwise a panic will be triggered
    /// * `index` - the index of the element to be removed
    fn delete_at(&mut self, index: metadata::Index);

    /// Deletes elements at buffer range. Indices must be valid, otherwise a panic will be triggered
    /// * `begin` - the start of the range to be removed
    /// * `end` - the end of the range to be removed, this index excluded
    fn delete_range(&mut self, begin: metadata::Index, end: metadata::Index);

    /// deletes data from the buffer, if there exists a selection
    fn delete_if_selection(&mut self) -> bool;

    /// Simulates a cursor movement and returns the positional data for the cursor, along with a reference to the data that the cursor spanned
    fn get_buffer_movement_result(&mut self, dir: Movement) -> Option<(metadata::Index, metadata::Index)>;

    fn undo(&mut self);

    fn redo(&mut self);

    /// Copies a slice into the buffer, using memcpy. If there's not enough space, the buffer will have to re-allocate it's data first
    /// * `slice` - the slice to copy into the buffer
    fn insert_slice_fast(&mut self, slice: &[char]);

    /// Moves the cursor in the buffer
    fn move_cursor(&mut self, dir: Movement);

    /// Moves the cursor and sets the meta cursor accordingly.
    fn select_move_cursor_absolute(&mut self, movement: Movement);

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

        self.meta_data().get_pristine_hash() == hash
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
        Assert!(absolute_position <= self.len(), "absolute position is outside of the buffer");
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

    /// Goes to a line in buffer if it exists
    /// * `line` - Line to go to
    fn goto_line(&mut self, line: usize);

    /// This operation only succeeds, if lines is a valid range of lines in the buffer.
    /// If the end is beyond the actual amount of lines in the buffer, no operation will be performed.
    /// It is therefore up to the call site to make sure that the line range is contained inside the buffer.
    /// This makes it possible to wrap a "safe" or "always succeed" API around it.
    /// * `lines` -
    fn line_operation<RangeType>(&mut self, lines: RangeType, op: &LineOperation)
    where
        RangeType: std::ops::RangeBounds<usize> + std::slice::SliceIndex<[metadata::Index], Output = [metadata::Index]> + Clone + std::ops::RangeBounds<usize>;
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
