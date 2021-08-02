use crate::textbuffer::{operations::LineOperation, Movement};
use glfw::{Action, Key, Modifiers};

/// Command enum. This is what user input gets translated to, so that we can have configurability, by reading text files and re-mapping the internal HashMap of KeyInput => Command translations
/// In the examples below, whenever you see two bars around a text item like so: |foo| means the cursor & and it's sibling (meta cursor(s)) cursor has foo selected
/// This way we can textually and visually represent cursor movement and actions
pub enum InputTranslation {
    Cancel,
    Movement(Movement),
    TextSelect(Movement),
    /// let |v| = vec![1, 2] <br>
    /// moves cursor to => "let v = vec![|1, 2|]" => next user input will replace what's between |1, 2|
    ChangeValueOfAssignment,
    StaticInsertStr(&'static str),
    Cut,
    Copy,
    Paste,
    Delete,
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
    CloseActiveView,
    Quit,
    OpenNewView,
    LineOperation(LineOperation),
}

pub enum InputContext {
    ActiveView(InputTranslation),
    Application(InputTranslation),
}

pub fn translate_key_input(_key: Key, _action: Action, _modifier: Modifiers) -> InputTranslation {
    InputTranslation::Cancel
}
