use super::simple::simplebuffer::SimpleBuffer;

pub struct Buffers {
    buffers: Vec<Box<SimpleBuffer>>,
    /// Keeps track of how many buffers we've opened so far. This has to be tracked, as it's not
    /// necessarily as many that are in buffers, so not buffers.len(), since a View might request a buffer
    /// and the view will take ownership and store the Box inside itself, then hand it back, if it wants to switch to editing another buffer for instace
    live_buffer_ids: Vec<u32>,
}

impl Buffers {
    pub fn new() -> Self {
        Buffers { buffers: vec![], live_buffer_ids: vec![] }
    }

    /// Creates an un-managed text buffer. Useful for text views that do not have multiple buffers, or have some buffer managing logic of it's own
    pub fn free_buffer() -> Box<SimpleBuffer> {
        Box::new(SimpleBuffer::new(0, 1024))
    }

    pub fn request_new_buffer(&mut self) -> Box<SimpleBuffer> {
        let buf = SimpleBuffer::new(self.live_buffer_ids.len() as _, 1024);
        self.live_buffer_ids.push(self.live_buffer_ids.len() as _);
        Box::new(buf)
    }

    pub fn take_buffer(&mut self, id: u32) -> Option<Box<SimpleBuffer>> {
        if let Some(index) = self.buffers.iter().position(|b| b.id == id) {
            Some(self.buffers.remove(index))
        } else {
            None
        }
    }

    pub fn give_back_buffer(&mut self, buffer: Box<SimpleBuffer>) {
        self.buffers.push(buffer);
    }

    pub fn destroy_buffer(&mut self, buffer: Box<SimpleBuffer>) {
        debug_assert!(self.live_buffer_ids.iter().any(|&i| buffer.id == i), "No buffer managed by that ID!");
        self.live_buffer_ids.retain(|&i| i != buffer.id);
        drop(buffer);
    }
}
