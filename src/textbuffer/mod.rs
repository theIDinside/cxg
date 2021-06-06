pub trait TextBuffer {
    fn insert_char(&mut self, ch: char);
    fn insert_range(&mut self, range: Vec<char>);

    fn cursor_index(&self) -> usize;
    fn move_cursor(&mut self, to: usize);
}