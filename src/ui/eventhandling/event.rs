use std::path::PathBuf;

use crate::ui::UID;

/// InputResponse is communcation that goes from objects that implement InputBehavior trait
/// and as such, might need to communicate back results, to the Application object, such as results from run commands,
/// or returning the value of CopyRange from the buffer etc.
pub enum InputResponse {
    ClipboardCopy(Option<String>),
    OpenFile(PathBuf),
    SaveFile(Option<PathBuf>),
    Goto(u32),
    Find(String),
    None,
}

pub enum InputElement {
    PopUp,
    InputBox,
    TextView,
}

pub enum ModifierCombo {
    Alt,
    Control,
    Shift,
    AltControl,
    AltShift,
    AltControlShift,
    ControlShift,
}

pub(crate) fn key_press(action: glfw::Action) -> bool {
    action == glfw::Action::Press
}

pub(crate) fn key_press_repeat(action: glfw::Action) -> bool {
    action == glfw::Action::Press || action == glfw::Action::Repeat
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
        crate::debugger_catch!(false, crate::DebuggerCatch::Handle(format!("InvalidInput State")));
        InputResponse::None
    }

    fn handle_char(&mut self, _ch: char) {
        println!("Default Invalid Input Handler: {}", _ch);
        crate::debugger_catch!(false, crate::DebuggerCatch::Handle(format!("InvalidInput State")));
    }

    fn get_uid(&self) -> Option<UID> {
        None
    }
}
