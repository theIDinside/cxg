use std::path::PathBuf;

use crate::{debugger_catch, ui::UID};

pub enum InputResponse {
    Command,
    File(PathBuf),
    None,
}

pub trait InputBehavior {
    fn handle_key(&mut self, key: glfw::Key, action: glfw::Action, modifier: glfw::Modifiers) -> InputResponse;
    fn handle_char(&mut self, ch: char);
    fn get_uid(&self) -> Option<UID>;
}

pub struct InvalidInputElement {}

impl InputBehavior for InvalidInputElement {
    fn handle_key(&mut self, _key: glfw::Key, _action: glfw::Action, _modifier: glfw::Modifiers) -> InputResponse {
        println!("Default Invalid Input Handler: {:?} {:?} {:?}", _key, _action, _modifier);
        debugger_catch!(false, crate::DebuggerCatch::Handle(format!("InvalidInput State")));
        InputResponse::None
    }

    fn handle_char(&mut self, _ch: char) {
        println!("Default Invalid Input Handler: {}", _ch);
        debugger_catch!(false, crate::DebuggerCatch::Handle(format!("InvalidInput State")));
    }

    fn get_uid(&self) -> Option<UID> {
        None
    }
}
