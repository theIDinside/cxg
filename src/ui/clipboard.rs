pub struct ClipBoard {
    data: String,
}

impl ClipBoard {
    pub fn new() -> ClipBoard {
        ClipBoard { data: String::new() }
    }

    pub fn copy(&mut self, data: &str) {
        self.data = data.to_owned();
    }

    pub fn take(&mut self, data: String) {
        self.data = data;
    }

    pub fn give(&self) -> Option<&String> {
        if self.data.is_empty() {
            None
        } else {
            Some(&self.data)
        }
    }

    pub fn release(&mut self) -> Option<String> {
        if self.data.is_empty() {
            None
        } else {
            let mut res = String::with_capacity(self.data.len());
            std::mem::swap(&mut res, &mut self.data);
            Some(res)
        }
    }
}
