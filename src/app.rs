use glfw::{Window, Context, Key, Action};
use std::sync::mpsc::Receiver;

pub struct TextBuffer {
    buf: Vec<char>
}

enum ActiveInput {
    TextFile,
    Application
}

pub struct Application {
    title_bar: String,
    width: u32,
    height: u32,
    buf: TextBuffer,
    active_input: ActiveInput
}

impl Application {
    pub fn create() -> Application {
        Application {
            title_bar: "cxgledit".into(),
            width: 1024,
            height: 768,
            buf: TextBuffer { buf: Vec::new() },
            active_input: ActiveInput::TextFile
        }
    }

    pub fn char_insert(&mut self, ch: char) {
        match self.active_input {
            ActiveInput::TextFile => {
                self.buf.buf.push(ch);
            },
            _ => {}
        }
    }


    fn handle_keyboard_input(&mut self) {
        
    }

    // NOTE: not the same version as in common.rs!
    pub fn process_events(&mut self, window: &mut Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
        for (_, event) in glfw::flush_messages(events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    // make sure the viewport matches the new window dimensions; note that width and
                    // height will be significantly larger than specified on retina displays.
                    println!("App window {}x{} ===> {}x{}", self.width, self.height, width, height);
                    self.width = width as u32;
                    self.height = height as u32;
                    unsafe { 
                        gl::Viewport(0, 0, width, height) 
                    }
                },
                glfw::WindowEvent::Char(ch) => {
                    self.char_insert(ch);
                    println!("char input: {}", ch)
                },
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    println!("Dumping buf contents: {:?}", self.buf.buf);
                    window.set_should_close(true);
                },
                glfw::WindowEvent::Key(Key::Enter, _, Action::Press, _) => {
                    match self.active_input {
                        ActiveInput::Application => println!("Handle execution of commands etc"),
                        ActiveInput::TextFile => self.char_insert('\n'),
                    }
                }
                glfw::WindowEvent::Key(k, _, Action::Press, _) => {
                    // println!("Key input handler - Key: {}  Scancode: {}", k.get_name().unwrap(), k.get_scancode().unwrap());
                },
                _ => {}
            }
        }
    }
}