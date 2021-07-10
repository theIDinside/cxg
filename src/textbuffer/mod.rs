pub mod gap_buffer;
pub mod simplebuffer;
pub mod metadata;
// pub mod text_buffer;

pub trait Buffer<T> where T: Sized {
    fn insert(&mut self, data: T);
    fn remove(&mut self);
}

pub trait BufferString {
    fn read_string(&self, range: std::ops::Range<usize>) -> String;
}


pub mod cursor {
    use std::cmp::Ordering;
    #[derive(Default, Debug, Copy, Clone)]
    pub struct BufferCursor {
        /// Absolute index into buffer
        pub pos: usize, 
        pub row: usize,
        pub col: usize
    }

    impl Into<BufferCursor> for (usize, usize, usize) {
        #[inline(always)]
        fn into(self) -> BufferCursor {
            let (pos, row, col) = self;
            BufferCursor { pos, row, col }
        }
    }

    impl PartialEq for BufferCursor {
        fn ne(&self, other: &Self) -> bool {
            self.pos != other.pos
        }

        fn eq(&self, other: &Self) -> bool {
            self.pos == other.pos
        }
    }

    impl Eq for BufferCursor {}

    impl Ord for BufferCursor {
        fn cmp(&self, other: &Self) -> Ordering {
            self.pos.cmp(&other.pos)
        }
    }

    impl PartialOrd for BufferCursor {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    pub enum CursorMovement {
        Valid,
        InvalidColumn
    }

    impl BufferCursor {
        pub fn absolute(&self) -> usize { self.pos }
        pub fn char_at_idx(&self) -> std::ops::Range<usize> { self.pos .. self.pos + 1 }
        pub fn set_pos(&mut self, pos: usize) { self.pos = pos; }
        pub fn set_row(&mut self, row: usize) { self.row = row; }
        pub fn set_col(&mut self, col: usize) { self.col = col; }
    }

}


#[cfg(test)]
mod tests {
    use super::gap_buffer::GapBuffer as GB;
    // use super::Gap::GapBuffer as GB;
    use super::BufferString;


    #[test]
    fn test_iteration() {
        let text_data = include_str!("test_textfile.source");
        let mut gb = GB::new_with_capacity(text_data.len());
        gb.map_to(text_data.chars());
        assert_eq!(gb.len(), text_data.len(), "Gap buffer not matching correct datal length");
    }


    #[test]
    fn test_gapbuffer_insert() {
        let mut gb = GB::new();
        gb.map_to("hello world!".chars());
        assert_eq!(gb.get(0), Some(&'h'));
    }

    #[test]
    fn test_gapbuffer_read_string() {
        let mut gb = GB::new();
        gb.map_to("hello world!".chars());
        assert_eq!(gb.read_string(0..5), "hello");
    }

    #[test]
    fn test_gapbuffer_dump_to_string() {
        let mut gb = GB::new();
        gb.map_to("hello world!".chars());
        assert_eq!(gb.read_string(0..25), "hello world!");
    }

    #[test]
    fn test_insert_move_insert() {
        let mut gb = GB::new();
        gb.map_to("hello world!".chars());
        assert_eq!(gb.read_string(0..25), "hello world!");
        gb.set_gap_position(6);
        gb.map_to("fucking ".chars());
        assert_eq!(gb.read_string(0..25), "hello fucking world!");
    }

    #[test]
    fn test_insert_lines() {

    }

    #[test]
    fn test_remove_char() {
        let mut gb = GB::new();
        gb.map_to("hello world!".chars());
        assert_eq!("hello world", gb.read_string(0..25));
    }

    #[test]
    fn test_insert_newline() {
        let mut gb = GB::new();
        gb.map_to("hello wor".chars());
        gb.set_gap_position(5);
        // gb.delete();
        gb.insert('\n');
        assert_eq!("hello\n wor", gb.read_string(0..25));
    }

    #[test]
    fn test_remove_world_from_hello_world() {
        let mut gb = GB::new();
        gb.map_to("hello world".chars());
        gb.set_gap_position(6);
        for _ in 0..5 {
            gb.delete();
        }
        assert_eq!("hello ", gb.read_string(0..25));
    }

    #[test]
    fn test_replace_world_with_simon() {
        let mut gb = GB::new();
        gb.map_to("hello world".chars());
        gb.set_gap_position(6);
        for _ in 0..5 {
            gb.delete();
        }
        let simon: String = "Simon".into();
        gb.map_to(simon.chars());
        assert_eq!("hello Simon", gb.read_string(0..25));
    }

    #[test]
    fn test_insert_slice() {
        let mut gb = GB::new();
        let foo = String::from("fucker");

        gb.insert('h' as u8);
        gb.insert('e' as u8);
        gb.insert('l' as u8);
        gb.insert('l' as u8);
        gb.insert('o' as u8);
        gb.insert(' ' as u8);
        gb.insert_slice("world! ".as_bytes());
        assert_eq!(gb.read_string(0..gb.len()), "hello world! ");
        gb.insert_slice(foo.as_bytes());
        assert_eq!(gb.read_string(0..gb.len()), "hello world! fucker");
    }
}