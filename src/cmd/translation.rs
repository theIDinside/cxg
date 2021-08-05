use crate::textbuffer::{operations::LineOperation, Movement};
use glfw::{Action, Key, Modifiers};
use serde::{Deserialize, Serialize};

/// Command enum. This is what user input gets translated to, so that we can have configurability, by reading text files and re-mapping the internal HashMap of KeyInput => Command translations
/// In the examples below, whenever you see two bars around a text item like so: |foo| means the cursor & and it's sibling (meta cursor(s)) cursor has foo selected
/// This way we can textually and visually represent cursor movement and actions
#[derive(Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Deserialize, Serialize)]
pub enum InputTranslation {
    Cancel,
    Enter,
    Movement(Movement),
    TextSelect(Movement),
    Delete(Movement),
    /// let |v| = vec![1, 2] <br>
    /// moves cursor to => "let v = vec![|1, 2|]" => next user input will replace what's between |1, 2|
    ChangeValueOfAssignment,
    InsertStr(String),
    Cut,
    Copy,
    Paste,
    Undo,
    Redo,
    OpenFile,
    SaveFile,
    Search,
    Goto,
    CycleFocus,
    HideFocused,
    ShowAll,
    ShowDebugInterface,
    CloseActiveView(bool),
    Quit,
    OpenNewView,
    LineOperation(LineOperation),
    Debug,
}

pub enum ViewUserInput {
    Enter,
    Movement(Movement),
    TextSelect(Movement),
    Delete(Movement),
    ChangeValueOfAssignment,
    InsertStr(String),
    Search,
    Goto,
    LineOperation(LineOperation),
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
}

pub enum CommandUserInput {
    Cancel,
    MovecursorLeft,
    MovecursorRight,
    SelectTextLeft,
    SelectTextRight,
    ScrollSelectionUp,
    ScrollSelectionDown,
    Cut,
    Copy,
    Paste,
    Ok
}

pub enum InputContext {
    ActiveView(ViewUserInput),
    InputBox(CommandUserInput),
    Application(InputTranslation),
}

pub fn translate_key_input(_key: Key, _action: Action, _modifier: Modifiers) -> InputTranslation {
    InputTranslation::Cancel
}
