use self::metadata::{MetaData};

pub mod metadata;
pub mod gb;
pub mod simple;
pub mod textbuffer;
pub mod cursor;

// pub mod text_buffer;


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

pub trait CharBuffer<'a> {
    type ItemIterator: Iterator<Item=&'a char>;
    fn insert(&mut self, data: char);
    fn delete(&mut self, dir: Movement);
    fn insert_slice_fast(&mut self, slice: &[char]);
    fn capacity(&self) -> usize;
    fn len(&self) -> usize;
    fn empty(&self) -> bool { self.len() == 0 }
    fn available_space(&self) -> usize { self.capacity() - self.len() }
    fn rebuild_metadata(&mut self);
    fn meta_data(&self) -> &MetaData;
    fn iter(&'a self) -> Self::ItemIterator;
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
    use super::{CharBuffer, SubstringClone};


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
        gb.insert_slice_fast(&mfer[..]);
        println!("buffer contents: {:?}", gb.read_string(0 .. assertion_ok.len()));
        let two_times: Vec<_> = assertion_ok.chars().chain(assertion_ok.chars()).collect();
        gb.set_gap_position(0);
        gb.insert_slice_fast(&assertion_ok.chars().collect::<Vec<_>>()[..]);
        let larger_range = 0 .. two_times.len() * 10;
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
        gb.insert_slice_fast(&mfer[..]);
        println!("buffer contents: {:?}", gb.read_string(0 .. assertion_ok.len()));
        assert_eq!(gb.read_string(0 .. assertion_ok.len()).len(), assertion_ok.len());
        assert_eq!(gb.read_string(0 .. assertion_ok.len()), assertion_ok);
        let two_times: Vec<_> = assertion_ok.chars().chain(assertion_ok.chars()).collect();
        gb.set_gap_position(0);
        gb.insert_slice_fast(&assertion_ok.chars().collect::<Vec<_>>()[..]);
        assert_eq!(gb.read_string(0 .. two_times.len()), two_times.iter().collect::<String>());
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
    fn test_insert_lines() {

    }

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

    #[test]
    fn test_insert_slice() {
        /*
        let mut gb = GB::new();
        let foo = String::from("fucker");

        gb.insert_item('h' as u8);
        gb.insert_item('e' as u8);
        gb.insert_item('l' as u8);
        gb.insert_item('l' as u8);
        gb.insert_item('o' as u8);
        gb.insert_item(' ' as u8);
        gb.insert_slice("world! ".as_bytes());
        assert_eq!(gb.read_string(0..gb.len()), "hello world! ");
        gb.insert_slice(foo.as_bytes());
        assert_eq!(gb.read_string(0..gb.len()), "hello world! fucker");
         */
    }
}