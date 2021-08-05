use crate::{
    cmd::CommandTag,
    textbuffer::{operations::LineOperation, Movement},
    ui::UID,
};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};

use super::input::KeyboardInputContext;

/// InputResponse is communcation that goes from objects that implement InputBehavior trait
/// and as such, might need to communicate back results, to the Application object, such as results from run commands,
/// or returning the value of CopyRange from the buffer etc.
pub enum CommandOutput {
    ClipboardCopy(Option<String>),
    OpenFile(PathBuf),
    SaveFile(Option<PathBuf>),
    Goto(u32),
    Find(String),
    None,
    CommandSelection(CommandTag),
}

pub enum InputElement {
    PopUp,
    InputBox,
    TextView,
}

// Actions that take place inside an InputBox
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InputboxAction {
    Cancel,
    Delete(Movement),
    MovecursorLeft,
    MovecursorRight,
    ScrollSelectionUp,
    ScrollSelectionDown,
    Cut,
    Copy,
    Paste,
    Ok,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ViewAction {
    Cancel,
    SaveFile,
    OpenFile,
    Movement(Movement),
    TextSelect(Movement),
    Find,
    Goto,
    Delete(Movement),
    ChangeValueOfAssignment,
    InsertStr(String),
    Cut,
    Copy,
    Paste,
    Undo,
    Redo,
    LineOperation(LineOperation),
    Debug,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AppAction {
    Cancel,
    OpenFile,
    SaveFile,
    SearchInFiles,
    GotoLineInFile,
    CycleFocus,
    HideFocused,
    ShowAll,
    ShowDebugInterface,
    CloseActiveView(bool),
    Quit,
    OpenNewView,
    ListCommands,
}

impl Display for AppAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppAction::{:?}", *self)
    }
}

impl Display for ViewAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ViewAction::{:?}", *self)
    }
}

impl Display for InputboxAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InputboxAction::{:?}", *self)
    }
}

pub(crate) fn key_press(action: glfw::Action) -> bool {
    action == glfw::Action::Press
}

pub(crate) fn key_press_repeat(action: glfw::Action) -> bool {
    action == glfw::Action::Press || action == glfw::Action::Repeat
}

pub trait InputBehavior {
    fn handle_key(&mut self, key: glfw::Key, action: glfw::Action, modifier: glfw::Modifiers) -> CommandOutput;
    fn handle_char(&mut self, ch: char);
    fn move_cursor(&mut self, movement: Movement);
    fn select_move_cursor(&mut self, movement: Movement);
    fn delete(&mut self, movement: Movement);
    fn copy(&self) -> Option<String>;
    fn cut(&self) -> Option<String>;

    fn context(&self) -> KeyboardInputContext;
    fn get_uid(&self) -> Option<UID>;
}

pub struct InvalidInputElement {}

impl InputBehavior for InvalidInputElement {
    fn handle_key(&mut self, _key: glfw::Key, _action: glfw::Action, _modifier: glfw::Modifiers) -> CommandOutput {
        println!("Default Invalid Input Handler: {:?} {:?} {:?}", _key, _action, _modifier);
        crate::debugger_catch!(false, crate::DebuggerCatch::Handle(format!("InvalidInput State")));
        CommandOutput::None
    }

    fn handle_char(&mut self, _ch: char) {
        println!("Default Invalid Input Handler: {}", _ch);
        crate::debugger_catch!(false, crate::DebuggerCatch::Handle(format!("InvalidInput State")));
    }

    fn get_uid(&self) -> Option<UID> {
        None
    }

    fn move_cursor(&mut self, _movement: Movement) {
        todo!()
    }

    fn context(&self) -> KeyboardInputContext {
        todo!()
    }

    fn select_move_cursor(&mut self, _movement: Movement) {
        todo!()
    }

    fn delete(&mut self, _movement: Movement) {
        todo!()
    }

    fn copy(&self) -> Option<String> {
        todo!()
    }

    fn cut(&self) -> Option<String> {
        todo!()
    }
}
