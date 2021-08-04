// For serializing to configuration files (which at first won't be human friendly)
// and deserializing.
use super::keyimpl::{KeyImpl, ModifiersImpl};
use crate::{
    textbuffer::{operations::LineOperation, Movement, TextKind},
    ui::eventhandling::event::{AppAction, InputboxAction, ViewAction},
};
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, fmt::Display};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TextViewKeyBinding {
    pressed: Option<ViewAction>,
    repeated: Option<ViewAction>,
    released: Option<ViewAction>,
}

impl Display for TextViewKeyBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"{{ "pressed": "{}", "repeated": "{}", "released": "{}" }}"#,
            self.pressed.as_ref().map(|d| d.to_string()).unwrap_or("None".into()),
            self.repeated.as_ref().map(|d| d.to_string()).unwrap_or("None".into()),
            self.released.as_ref().map(|d| d.to_string()).unwrap_or("None".into()),
        )
    }
}

impl TextViewKeyBinding {
    pub fn press(act: ViewAction) -> TextViewKeyBinding {
        TextViewKeyBinding { pressed: Some(act), repeated: None, released: None }
    }

    pub fn release(act: ViewAction) -> TextViewKeyBinding {
        TextViewKeyBinding { pressed: None, released: Some(act), repeated: None }
    }

    pub fn held(act: ViewAction) -> TextViewKeyBinding {
        TextViewKeyBinding { pressed: Some(act.clone()), repeated: Some(act), released: None }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InputboxBinding {
    pressed: Option<InputboxAction>,
    repeated: Option<InputboxAction>,
    released: Option<InputboxAction>,
}

impl Display for InputboxBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"{{ "pressed": "{}", "repeated": "{}", "released": "{}" }}"#,
            self.pressed.as_ref().map(|d| d.to_string()).unwrap_or("None".into()),
            self.repeated.as_ref().map(|d| d.to_string()).unwrap_or("None".into()),
            self.released.as_ref().map(|d| d.to_string()).unwrap_or("None".into()),
        )
    }
}

impl InputboxBinding {
    pub fn press(act: InputboxAction) -> InputboxBinding {
        InputboxBinding { pressed: Some(act), repeated: None, released: None }
    }

    pub fn release(act: InputboxAction) -> InputboxBinding {
        InputboxBinding { pressed: None, released: Some(act), repeated: None }
    }

    pub fn held(act: InputboxAction) -> InputboxBinding {
        InputboxBinding { pressed: Some(act.clone()), repeated: Some(act), released: None }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AppBinding {
    pressed: Option<AppAction>,
    repeated: Option<AppAction>,
    released: Option<AppAction>,
}

impl Display for AppBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"{{ "pressed": "{}", "repeated": "{}", "released": "{}" }}"#,
            self.pressed.as_ref().map(|d| d.to_string()).unwrap_or("None".into()),
            self.repeated.as_ref().map(|d| d.to_string()).unwrap_or("None".into()),
            self.released.as_ref().map(|d| d.to_string()).unwrap_or("None".into()),
        )
    }
}

impl AppBinding {
    pub fn press(act: AppAction) -> AppBinding {
        AppBinding { pressed: Some(act), repeated: None, released: None }
    }

    pub fn release(act: AppAction) -> AppBinding {
        AppBinding { pressed: None, released: Some(act), repeated: None }
    }

    pub fn held(act: AppAction) -> AppBinding {
        AppBinding { pressed: Some(act.clone()), repeated: Some(act), released: None }
    }
}

// type BindingRequirement = (KeyImpl, ModifiersImpl);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BindingRequirement(#[serde(with = "serde_with::rust::display_fromstr")] KeyImpl, #[serde(with = "serde_with::rust::display_fromstr")] ModifiersImpl);

impl Display for BindingRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let BindingRequirement(k, m) = self;
        write!(f, "{}+{:?}", *m, *k)
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct AppActions {
    appActions: HashMap<BindingRequirement, AppBinding>,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct TextViewActions {
    pub textViewActions: HashMap<BindingRequirement, TextViewKeyBinding>,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct InputBoxActions {
    pub inputBoxActions: HashMap<BindingRequirement, InputboxBinding>,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct KeyBindings {
    //    #[serde(with = "serde_with::rust::display_fromstr")]
    pub appActions: AppActions,
    /// Text View key mappings
    //#[serde(with = "serde_with::rust::display_fromstr")]
    pub textViewActions: TextViewActions,
    /// Input box key mappings
    // #[serde(with = "serde_with::rust::display_fromstr")]
    pub inputBoxActions: InputBoxActions,
}

/*
    App Actions: {
        "ctrl+O": {
            "pressed": "AppAction::OpenFile",
            "repeated": "None",
            "released": "None",
        },
    }
*/

impl Display for AppActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let map_data = self
            .appActions
            .iter()
            .map(|(br, b)| format!(r#""{}": {}"#, br, b))
            .collect::<Vec<String>>()
            .join(",");

        write!(f, r#""App Actions": {{ {} }}"#, map_data)
    }
}

impl Display for TextViewActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let map_data = self
            .textViewActions
            .iter()
            .map(|(br, b)| format!(r#""{}": {}"#, br, b))
            .collect::<Vec<String>>()
            .join(",");

        write!(f, r#""Text View Actions": {{ {} }}"#, map_data)
    }
}

impl Display for InputBoxActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let map_data = self
            .inputBoxActions
            .iter()
            .map(|(br, b)| format!(r#""{}": {}"#, br, b))
            .collect::<Vec<String>>()
            .join(",");

        write!(f, r#""Input Box Actions": {{ {} }}"#, map_data)
    }
}

fn magic(glfw_key: glfw::Key, glfw_modifiers: glfw::Modifiers) -> (KeyImpl, ModifiersImpl) {
    unsafe { (std::mem::transmute(glfw_key), std::mem::transmute(glfw_modifiers)) }
}

/// For serialization purposes we have re-implemented the glfw::Key and glfw::Modifiers
/// Which is why we use our own KeyImpl and ModifiersImpl here. But since they are implemented in an *exact*
/// one-to-one ratio, we can safely transmute between the types and have the compiler verify that we are correct still for doing so.
impl KeyBindings {
    pub fn new() -> KeyBindings {
        KeyBindings {
            appActions: AppActions { appActions: HashMap::new() },
            textViewActions: TextViewActions { textViewActions: HashMap::new() },
            inputBoxActions: InputBoxActions { inputBoxActions: HashMap::new() },
        }
    }

    pub fn translate_textview_input(&self, key: glfw::Key, action: glfw::Action, modifiers: glfw::Modifiers) -> Option<ViewAction> {
        let (key, modifier) = magic(key, modifiers);
        self.textViewActions
            .textViewActions
            .get(&BindingRequirement(key, modifier))
            .and_then(|binding| match action {
                glfw::Action::Release => binding.released.clone(),
                glfw::Action::Press => binding.pressed.clone(),
                glfw::Action::Repeat => binding.repeated.clone(),
            })
    }

    pub fn translate_command_input(&self, key: glfw::Key, action: glfw::Action, modifiers: glfw::Modifiers) -> Option<InputboxAction> {
        let (key, modifier) = magic(key, modifiers);
        self.inputBoxActions
            .inputBoxActions
            .get(&BindingRequirement(key, modifier))
            .and_then(|binding| match action {
                glfw::Action::Release => binding.released.clone(),
                glfw::Action::Press => binding.pressed.clone(),
                glfw::Action::Repeat => binding.repeated.clone(),
            })
    }

    pub fn translate_app_input(&self, key: glfw::Key, action: glfw::Action, modifiers: glfw::Modifiers) -> Option<AppAction> {
        let (key, modifier) = magic(key, modifiers);
        self.appActions
            .appActions
            .get(&BindingRequirement(key, modifier))
            .and_then(|binding| match action {
                glfw::Action::Release => binding.released.clone(),
                glfw::Action::Press => binding.pressed.clone(),
                glfw::Action::Repeat => binding.repeated.clone(),
            })
    }

    pub fn default() -> KeyBindings {
        let ib_key_map = ib_default();
        let key_map = app_default();
        let tv_key_map = tv_default();
        KeyBindings {
            appActions: AppActions { appActions: key_map },
            textViewActions: TextViewActions { textViewActions: tv_key_map },
            inputBoxActions: InputBoxActions { inputBoxActions: ib_key_map },
        }
    }

    pub fn total_keybindings(&self) -> usize {
        self.appActions.appActions.len() + self.textViewActions.textViewActions.len() + self.inputBoxActions.inputBoxActions.len()
    }
}

pub fn tv_default() -> HashMap<BindingRequirement, TextViewKeyBinding> {
    use KeyImpl as K;
    use ModifiersImpl as M;
    use TextViewKeyBinding as TVB;
    use ViewAction as V;

    let mut m = HashMap::new();
    /*
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
    */

    m.insert(BindingRequirement(K::Escape, M::empty()), TVB::press(V::Cancel));
    m.insert(BindingRequirement(K::S, M::CONTROL), TVB::press(V::SaveFile));
    m.insert(BindingRequirement(K::O, M::CONTROL), TVB::press(V::OpenFile));
    m.insert(BindingRequirement(K::Left, M::empty()), TVB::held(V::Movement(Movement::Backward(TextKind::Char, 1))));
    m.insert(BindingRequirement(K::Left, M::SHIFT), TVB::held(V::TextSelect(Movement::Backward(TextKind::Char, 1))));
    m.insert(BindingRequirement(K::Left, M::CONTROL), TVB::held(V::Movement(Movement::Begin(TextKind::Word))));
    m.insert(BindingRequirement(K::Left, M::CONTROL | M::SHIFT), TVB::held(V::TextSelect(Movement::Begin(TextKind::Word))));
    m.insert(BindingRequirement(K::Right, M::empty()), TVB::held(V::Movement(Movement::Forward(TextKind::Char, 1))));
    m.insert(BindingRequirement(K::Right, M::SHIFT), TVB::held(V::TextSelect(Movement::Forward(TextKind::Char, 1))));
    m.insert(BindingRequirement(K::Right, M::CONTROL), TVB::held(V::Movement(Movement::End(TextKind::Word))));
    m.insert(BindingRequirement(K::Right, M::CONTROL | M::SHIFT), TVB::held(V::TextSelect(Movement::End(TextKind::Word))));
    m.insert(BindingRequirement(K::Up, M::empty()), TVB::held(V::Movement(Movement::Backward(TextKind::Line, 1))));
    m.insert(BindingRequirement(K::Up, M::SHIFT), TVB::held(V::TextSelect(Movement::Backward(TextKind::Line, 1))));
    m.insert(BindingRequirement(K::Down, M::empty()), TVB::held(V::Movement(Movement::Forward(TextKind::Line, 1))));
    m.insert(BindingRequirement(K::Down, M::SHIFT), TVB::held(V::TextSelect(Movement::Forward(TextKind::Line, 1))));
    m.insert(BindingRequirement(K::Home, M::empty()), TVB::held(V::Movement(Movement::Begin(TextKind::Line))));
    m.insert(BindingRequirement(K::Home, M::SHIFT), TVB::held(V::TextSelect(Movement::Begin(TextKind::Line))));
    m.insert(BindingRequirement(K::End, M::empty()), TVB::held(V::Movement(Movement::End(TextKind::Line))));
    m.insert(BindingRequirement(K::End, M::SHIFT), TVB::held(V::TextSelect(Movement::End(TextKind::Line))));
    m.insert(BindingRequirement(K::F, M::CONTROL), TVB::press(V::Find));
    m.insert(BindingRequirement(K::G, M::CONTROL), TVB::press(V::Goto));
    m.insert(BindingRequirement(K::Delete, M::empty()), TVB::held(V::Delete(Movement::Forward(TextKind::Char, 1))));
    m.insert(BindingRequirement(K::Delete, M::CONTROL), TVB::held(V::Delete(Movement::Forward(TextKind::Word, 1))));
    m.insert(BindingRequirement(K::Backspace, M::empty()), TVB::held(V::Delete(Movement::Backward(TextKind::Char, 1))));
    m.insert(BindingRequirement(K::Backspace, M::CONTROL), TVB::held(V::Delete(Movement::Backward(TextKind::Word, 1))));
    m.insert(BindingRequirement(K::C, M::CONTROL), TVB::press(V::Copy));
    m.insert(BindingRequirement(K::X, M::CONTROL), TVB::press(V::Cut));
    m.insert(BindingRequirement(K::V, M::CONTROL), TVB::press(V::Paste));
    m.insert(BindingRequirement(K::Tab, M::empty()), TVB::press(V::LineOperation(LineOperation::ShiftRight { shift_by: 4 })));
    m.insert(BindingRequirement(K::Tab, M::SHIFT), TVB::press(V::LineOperation(LineOperation::ShiftLeft { shift_by: 4 })));
    m
}

pub fn ib_default() -> HashMap<BindingRequirement, InputboxBinding> {
    use InputboxAction as I;
    use KeyImpl as K;
    use ModifiersImpl as M;
    let mut ib_key_map = HashMap::new();
    ib_key_map.insert(BindingRequirement(K::Escape, M::empty()), InputboxBinding::press(I::Cancel));
    ib_key_map.insert(BindingRequirement(K::Enter, M::empty()), InputboxBinding::press(I::Ok));
    ib_key_map.insert(BindingRequirement(K::Left, M::empty()), InputboxBinding::press(I::MovecursorLeft));
    ib_key_map.insert(BindingRequirement(K::Right, M::empty()), InputboxBinding::press(I::MovecursorRight));
    ib_key_map.insert(BindingRequirement(K::Up, M::empty()), InputboxBinding::press(I::ScrollSelectionUp));
    ib_key_map.insert(BindingRequirement(K::Down, M::empty()), InputboxBinding::press(I::ScrollSelectionDown));
    ib_key_map.insert(BindingRequirement(K::X, M::CONTROL), InputboxBinding::press(I::Cut));
    ib_key_map.insert(BindingRequirement(K::C, M::CONTROL), InputboxBinding::press(I::Copy));
    ib_key_map.insert(BindingRequirement(K::V, M::CONTROL), InputboxBinding::press(I::Paste));

    ib_key_map
}

pub fn app_default() -> HashMap<BindingRequirement, AppBinding> {
    use AppAction as A;
    use AppBinding as B;
    use KeyImpl as K;
    use ModifiersImpl as M;
    let mut map = HashMap::new();
    /*
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
    */
    map.insert(BindingRequirement(K::Escape, M::empty()), B::press(A::Cancel));
    map.insert(BindingRequirement(K::O, M::CONTROL), B::press(A::OpenFile));
    map.insert(BindingRequirement(K::I, M::CONTROL | M::SHIFT), B::press(A::OpenFile));
    map.insert(BindingRequirement(K::S, M::CONTROL), B::press(A::SaveFile));
    map.insert(BindingRequirement(K::F, M::CONTROL | M::SHIFT), B::press(A::SearchInFiles));
    map.insert(BindingRequirement(K::G, M::CONTROL | M::SHIFT), B::press(A::GotoLineInFile));
    map.insert(BindingRequirement(K::Tab, M::CONTROL), B::press(A::CycleFocus));
    map.insert(BindingRequirement(K::D, M::CONTROL), B::press(A::ShowDebugInterface));
    map.insert(BindingRequirement(K::W, M::CONTROL), B::press(A::CloseActiveView(false)));
    map.insert(BindingRequirement(K::W, M::CONTROL | M::SHIFT), B::press(A::CloseActiveView(true)));
    map.insert(BindingRequirement(K::Q, M::CONTROL), B::press(A::Quit));
    map.insert(BindingRequirement(K::N, M::CONTROL), B::press(A::OpenNewView));
    map
}
