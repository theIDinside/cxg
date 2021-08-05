// For serializing to configuration files (which at first won't be human friendly)
// and deserializing.
use super::keyimpl::{KeyImpl, ModifiersImpl};
use crate::{
    textbuffer::{operations::LineOperation, Movement, TextKind},
    ui::eventhandling::event::{AppAction, InputboxAction, ViewAction},
};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Serialize, Deserialize)]
pub struct TextViewKeyBinding {
    #[serde(default = "Option::<_>::default")]
    pressed: Option<ViewAction>,
    #[serde(default = "Option::<_>::default")]
    repeated: Option<ViewAction>,
    #[serde(default = "Option::<_>::default")]
    released: Option<ViewAction>,
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

#[derive(Serialize, Deserialize)]
pub struct InputboxBinding {
    #[serde(default = "Option::<_>::default")]
    pressed: Option<InputboxAction>,
    #[serde(default = "Option::<_>::default")]
    repeated: Option<InputboxAction>,
    #[serde(default = "Option::<_>::default")]
    released: Option<InputboxAction>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct AppBinding {
    #[serde(default = "Option::<_>::default")]
    pressed: Option<AppAction>,
    #[serde(default = "Option::<_>::default")]
    repeated: Option<AppAction>,
    #[serde(default = "Option::<_>::default")]
    released: Option<AppAction>,
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

//
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingRequirement(KeyImpl, ModifiersImpl);

impl Serialize for BindingRequirement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let BindingRequirement(key, mods) = self;
        let s = mods.to_string();
        let output = if s.is_empty() { format!("{:?}", key) } else { format!("{}+{:?}", s, key) };

        serializer.serialize_str(&output)
    }
}

struct BindingRequirementVisitor;

impl<'de> Visitor<'de> for BindingRequirementVisitor {
    type Value = BindingRequirement;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(
            "Expecting key combinations to be written in the form [modA +.. modN]+Key, for example: 
        'ctrl+shift+O' or 'ctrl+O' or just 'O' for no modifiers",
        )
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(pos) = value.rfind("+") {
            let mods = ModifiersImpl::from_str(&value[0..pos]).unwrap();
            let key = KeyImpl::from_str(&value[pos + 1..]).unwrap();
            Ok(BindingRequirement(key, mods))
        } else {
            let k = KeyImpl::from_str(value).unwrap();
            Ok(BindingRequirement(k, ModifiersImpl::empty()))
        }
    }
}

impl<'de> Deserialize<'de> for BindingRequirement {
    fn deserialize<D>(deserializer: D) -> Result<BindingRequirement, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(BindingRequirementVisitor)
    }
}

#[derive(Serialize, Deserialize)]
pub struct KeyBindings {
    #[serde(
        default = "app_default",
        rename(serialize = "App Actions", deserialize = "App Actions")
    )]
    pub app_actions: HashMap<BindingRequirement, AppBinding>,
    /// Text View key mappings
    #[serde(
        default = "tv_default",
        rename(serialize = "Text View Actions", deserialize = "Text View Actions")
    )]
    pub textview_actions: HashMap<BindingRequirement, TextViewKeyBinding>,
    /// Input box key mappings
    #[serde(
        default = "ib_default",
        rename(serialize = "Input Box Actions", deserialize = "Input Box Actions")
    )]
    pub inputbox_actions: HashMap<BindingRequirement, InputboxBinding>,
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

fn magic(glfw_key: glfw::Key, glfw_modifiers: glfw::Modifiers) -> (KeyImpl, ModifiersImpl) {
    unsafe { (std::mem::transmute(glfw_key), std::mem::transmute(glfw_modifiers)) }
}

/// For serialization purposes we have re-implemented the glfw::Key and glfw::Modifiers
/// Which is why we use our own KeyImpl and ModifiersImpl here. But since they are implemented in an *exact*
/// one-to-one ratio, we can safely transmute between the types and have the compiler verify that we are correct still for doing so.
impl KeyBindings {
    pub fn new() -> KeyBindings {
        KeyBindings { app_actions: HashMap::new(), textview_actions: HashMap::new(), inputbox_actions: HashMap::new() }
    }

    pub fn translate_textview_input(&self, key: glfw::Key, action: glfw::Action, modifiers: glfw::Modifiers) -> Option<ViewAction> {
        let (key, modifier) = magic(key, modifiers);
        self.textview_actions
            .get(&BindingRequirement(key, modifier))
            .and_then(|binding| match action {
                glfw::Action::Release => binding.released.clone(),
                glfw::Action::Press => binding.pressed.clone(),
                glfw::Action::Repeat => binding.repeated.clone(),
            })
    }

    pub fn translate_command_input(&self, key: glfw::Key, action: glfw::Action, modifiers: glfw::Modifiers) -> Option<InputboxAction> {
        let (key, modifier) = magic(key, modifiers);
        self.inputbox_actions
            .get(&BindingRequirement(key, modifier))
            .and_then(|binding| match action {
                glfw::Action::Release => binding.released.clone(),
                glfw::Action::Press => binding.pressed.clone(),
                glfw::Action::Repeat => binding.repeated.clone(),
            })
    }

    pub fn translate_app_input(&self, key: glfw::Key, action: glfw::Action, modifiers: glfw::Modifiers) -> Option<AppAction> {
        let (key, modifier) = magic(key, modifiers);
        self.app_actions
            .get(&BindingRequirement(key, modifier))
            .and_then(|binding| match action {
                glfw::Action::Release => binding.released.clone(),
                glfw::Action::Press => binding.pressed.clone(),
                glfw::Action::Repeat => binding.repeated.clone(),
            })
    }

    pub fn default() -> KeyBindings {
        let app_actions = app_default();
        let textview_actions = tv_default();
        let inputbox_actions = ib_default();
        KeyBindings { app_actions, textview_actions, inputbox_actions }
    }

    pub fn total_keybindings(&self) -> usize {
        self.app_actions.len() + self.textview_actions.len() + self.inputbox_actions.len()
    }
}

pub fn tv_default() -> HashMap<BindingRequirement, TextViewKeyBinding> {
    use KeyImpl as K;
    use ModifiersImpl as M;
    use TextViewKeyBinding as B;
    use ViewAction as A;

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

    m.insert(BindingRequirement(K::Escape, M::empty()), B::press(A::Cancel));
    m.insert(BindingRequirement(K::CapsLock, M::empty()), B::press(A::Cancel));
    m.insert(BindingRequirement(K::S, M::CONTROL), B::press(A::SaveFile));
    m.insert(BindingRequirement(K::O, M::CONTROL), B::press(A::OpenFile));
    m.insert(BindingRequirement(K::Left, M::empty()), B::held(A::Movement(Movement::Backward(TextKind::Char, 1))));
    m.insert(BindingRequirement(K::Left, M::SHIFT), B::held(A::TextSelect(Movement::Backward(TextKind::Char, 1))));
    m.insert(BindingRequirement(K::Left, M::CONTROL), B::held(A::Movement(Movement::Begin(TextKind::Word))));
    m.insert(BindingRequirement(K::Left, M::CONTROL | M::SHIFT), B::held(A::TextSelect(Movement::Begin(TextKind::Word))));
    m.insert(BindingRequirement(K::Right, M::empty()), B::held(A::Movement(Movement::Forward(TextKind::Char, 1))));
    m.insert(BindingRequirement(K::Right, M::SHIFT), B::held(A::TextSelect(Movement::Forward(TextKind::Char, 1))));
    m.insert(BindingRequirement(K::Right, M::CONTROL), B::held(A::Movement(Movement::End(TextKind::Word))));
    m.insert(BindingRequirement(K::Right, M::CONTROL | M::SHIFT), B::held(A::TextSelect(Movement::End(TextKind::Word))));
    m.insert(BindingRequirement(K::Up, M::empty()), B::held(A::Movement(Movement::Backward(TextKind::Line, 1))));
    m.insert(BindingRequirement(K::Up, M::SHIFT), B::held(A::TextSelect(Movement::Backward(TextKind::Line, 1))));
    m.insert(BindingRequirement(K::Down, M::empty()), B::held(A::Movement(Movement::Forward(TextKind::Line, 1))));
    m.insert(BindingRequirement(K::Down, M::SHIFT), B::held(A::TextSelect(Movement::Forward(TextKind::Line, 1))));

    m.insert(BindingRequirement(K::Home, M::empty()), B::held(A::Movement(Movement::Begin(TextKind::Line))));
    m.insert(BindingRequirement(K::Home, M::CONTROL), B::held(A::Movement(Movement::Begin(TextKind::File))));
    m.insert(BindingRequirement(K::Home, M::SHIFT), B::held(A::TextSelect(Movement::Begin(TextKind::Line))));
    m.insert(BindingRequirement(K::Home, M::CONTROL | M::SHIFT), B::held(A::TextSelect(Movement::Begin(TextKind::File))));

    m.insert(BindingRequirement(K::End, M::empty()), B::held(A::Movement(Movement::End(TextKind::Line))));
    m.insert(BindingRequirement(K::End, M::SHIFT), B::held(A::TextSelect(Movement::End(TextKind::Line))));

    m.insert(BindingRequirement(K::End, M::CONTROL), B::held(A::Movement(Movement::End(TextKind::File))));
    m.insert(BindingRequirement(K::End, M::SHIFT | M::CONTROL), B::held(A::TextSelect(Movement::End(TextKind::File))));

    m.insert(BindingRequirement(K::F, M::CONTROL), B::press(A::Find));
    m.insert(BindingRequirement(K::G, M::CONTROL), B::press(A::Goto));
    m.insert(BindingRequirement(K::Delete, M::empty()), B::held(A::Delete(Movement::Forward(TextKind::Char, 1))));
    m.insert(BindingRequirement(K::Delete, M::CONTROL), B::held(A::Delete(Movement::Forward(TextKind::Word, 1))));
    m.insert(BindingRequirement(K::Backspace, M::empty()), B::held(A::Delete(Movement::Backward(TextKind::Char, 1))));
    m.insert(BindingRequirement(K::Backspace, M::CONTROL), B::held(A::Delete(Movement::Backward(TextKind::Word, 1))));
    m.insert(BindingRequirement(K::C, M::CONTROL), B::press(A::Copy));
    m.insert(BindingRequirement(K::X, M::CONTROL), B::press(A::Cut));
    m.insert(BindingRequirement(K::V, M::CONTROL), B::press(A::Paste));
    m.insert(BindingRequirement(K::Tab, M::empty()), B::press(A::LineOperation(LineOperation::ShiftRight { shift_by: 4 })));
    m.insert(BindingRequirement(K::Tab, M::SHIFT), B::press(A::LineOperation(LineOperation::ShiftLeft { shift_by: 4 })));
    m
}

pub fn ib_default() -> HashMap<BindingRequirement, InputboxBinding> {
    use InputboxAction as A;
    use InputboxBinding as B;
    use KeyImpl as K;
    use ModifiersImpl as M;
    let mut ib_key_map = HashMap::new();
    ib_key_map.insert(BindingRequirement(K::Escape, M::empty()), B::press(A::Cancel));
    ib_key_map.insert(BindingRequirement(K::CapsLock, M::empty()), B::press(A::Cancel));
    ib_key_map.insert(BindingRequirement(K::Enter, M::empty()), B::press(A::Ok));
    ib_key_map.insert(BindingRequirement(K::Left, M::empty()), B::held(A::MovecursorLeft));
    ib_key_map.insert(BindingRequirement(K::Right, M::empty()), B::held(A::MovecursorRight));
    ib_key_map.insert(BindingRequirement(K::Up, M::empty()), B::held(A::ScrollSelectionUp));
    ib_key_map.insert(BindingRequirement(K::Down, M::empty()), B::held(A::ScrollSelectionDown));
    ib_key_map.insert(BindingRequirement(K::X, M::CONTROL), B::press(A::Cut));
    ib_key_map.insert(BindingRequirement(K::C, M::CONTROL), B::press(A::Copy));
    ib_key_map.insert(BindingRequirement(K::V, M::CONTROL), B::press(A::Paste));

    ib_key_map.insert(BindingRequirement(K::Backspace, M::CONTROL), B::held(A::Delete(Movement::Backward(TextKind::Word, 1))));
    ib_key_map.insert(BindingRequirement(K::Backspace, M::empty()), B::held(A::Delete(Movement::Backward(TextKind::Char, 1))));

    ib_key_map.insert(BindingRequirement(K::Delete, M::CONTROL), B::held(A::Delete(Movement::Forward(TextKind::Word, 1))));
    ib_key_map.insert(BindingRequirement(K::Delete, M::empty()), B::held(A::Delete(Movement::Forward(TextKind::Char, 1))));

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
    map.insert(BindingRequirement(K::P, M::CONTROL | M::SHIFT), B::press(A::ListCommands));
    map
}
