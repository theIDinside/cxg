pub mod gap_buffer;
// pub mod text_buffer;

use std::fmt::{Display, Formatter as Fmt, Error as FmtError};


type FileName = String;
type SourceErrorMessage = String;
#[derive(Debug)]
pub enum SaveFileError {
    FileExisted(FileName),
    Other(FileName, SourceErrorMessage)
}

pub type FileResult<T> = std::result::Result<T, SaveFileError>;
impl Display for SaveFileError {
    fn fmt(&self, f: &mut Fmt) -> Result<(), FmtError> {
        let res = match self {
            SaveFileError::FileExisted(fname) => {
                fname.chars().chain(" exists already, writing to file denied.".chars()).collect::<String>()
            },
            SaveFileError::Other(fname, cause) => {
                "Writing to "
                    .chars()
                    .chain(fname.chars())
                    .chain(" failed. Underlying cause was: ".chars())
                    .chain(cause.chars()).collect::<String>()
            }
        };
        write!(f, "{}", res)
    }
}

pub trait Buffer<T> where T: Sized {
    fn insert(&mut self, data: T);
    fn remove(&mut self);
}

pub trait BufferString {
    fn read_string(&self, range: std::ops::Range<usize>) -> String;
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
}