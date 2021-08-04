use crate::{
    textbuffer::{operations::LineOperation, Movement},
    ui::UID,
};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf, str::FromStr};

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
}

pub enum InputElement {
    PopUp,
    InputBox,
    TextView,
}

// Actions that take place inside an InputBox
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone)]
pub enum InputboxAction {
    Cancel,
    Delete,
    MovecursorLeft,
    MovecursorRight,
    ScrollSelectionUp,
    ScrollSelectionDown,
    Cut,
    Copy,
    Paste,
    Ok,
}

impl FromStr for InputboxAction {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "InputboxAction::Cancel" => Ok(InputboxAction::Cancel),
            "InputboxAction::MovecursorLeft" => Ok(InputboxAction::MovecursorLeft),
            "InputboxAction::MovecursorRight" => Ok(InputboxAction::MovecursorRight),
            "InputboxAction::ScrollSelectionUp" => Ok(InputboxAction::ScrollSelectionUp),
            "InputboxAction::ScrollSelectionDown" => Ok(InputboxAction::ScrollSelectionDown),
            "InputboxAction::Cut" => Ok(InputboxAction::Cut),
            "InputboxAction::Copy" => Ok(InputboxAction::Copy),
            "InputboxAction::Paste" => Ok(InputboxAction::Paste),
            "InputboxAction::Ok" => Ok(InputboxAction::Ok),
            _ => Err("Wrong string input for InputboxAction conversion"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone)]
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

impl FromStr for ViewAction {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const TAG: &'static str = "ViewAction::";

        match s {
            "ViewAction::Cancel" => Ok(ViewAction::Cancel),
            "ViewAction::SaveFile" => Ok(ViewAction::SaveFile),
            "ViewAction::OpenFile" => Ok(ViewAction::OpenFile),
            "ViewAction::Find" => Ok(ViewAction::Find),
            "ViewAction::Goto" => Ok(ViewAction::Goto),
            "ViewAction::ChangeValueOfAssignment" => Ok(ViewAction::ChangeValueOfAssignment),
            "ViewAction::Cut" => Ok(ViewAction::Cut),
            "ViewAction::Copy" => Ok(ViewAction::Copy),
            "ViewAction::Paste" => Ok(ViewAction::Paste),
            "ViewAction::Undo" => Ok(ViewAction::Undo),
            "ViewAction::Redo" => Ok(ViewAction::Redo),
            "ViewAction::Debug" => Ok(ViewAction::Debug),
            _ => match &s[0..TAG.len()] {
                "ViewAction::" => {
                    if &s[TAG.len()..TAG.len() + "InsertStr(".len()] == "InsertStr(" {
                        const I_TAG: &'static str = "InsertStr(";
                        if let Some(pos) = &s[TAG.len() + I_TAG.len()..].find(r#"")"#) {
                            let s = &s[TAG.len() + "InsertStr(".len()..pos + r#"")"#.len() + 1];
                            Ok(ViewAction::InsertStr(s.to_owned()))
                        } else {
                            Err("Could not find string contents in relation to InsertStr view command")
                        }
                    } else if &s[TAG.len()..TAG.len() + "Delete(".len()] == "Delete(" {
                        let start = TAG.len() + "Delete(".len();
                        let m = Movement::from_str(&s[start..s.len() - 1]);
                        m.map_or_else(|m| Err("Could not create ViewAction from string value"), |m| Ok(ViewAction::Delete(m)))
                    } else if &s[TAG.len()..TAG.len() + "LineOperation(".len()] == "LineOperation(" {
                        let start = TAG.len() + "LineOperation(".len();
                        let lo = LineOperation::from_str(&s[start..s.len() - 1]);
                        lo.map_or_else(|m| Err("Could not create ViewAction from string value"), |lo| Ok(ViewAction::LineOperation(lo)))
                    } else if &s[TAG.len()..TAG.len() + "Movement(".len()] == "Movement(" {
                        let start = TAG.len() + "Movement(".len();
                        let m = Movement::from_str(&s[start..s.len() - 1]);
                        m.map_or_else(|m| Err("Could not create ViewAction from string value"), |m| Ok(ViewAction::Movement(m)))
                    } else if &s[TAG.len()..TAG.len() + "TextSelect(".len()] == "TextSelect(" {
                        let start = TAG.len() + "TextSelect(".len();
                        let m = Movement::from_str(&s[start..s.len() - 1]);
                        m.map_or_else(|m| Err("Could not create ViewAction from string value"), |m| Ok(ViewAction::TextSelect(m)))
                    } else {
                        Err("Could not create ViewAction from string value")
                    }
                }
                _ => Err("Could not create ViewAction from string value"),
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone)]
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
