// For serializing to configuration files (which at first won't be human friendly)
// and deserializing.
use serde::{Deserialize, Serialize};

use crate::textbuffer::operations::LineOperation;
use crate::textbuffer::{Movement, TextKind};

use super::keyimpl::{KeyImpl, ModifiersImpl};

use super::translation::InputTranslation;
use std::collections::HashMap;

use crate::ui::eventhandling::events::{ViewAction, InputboxAction};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TextViewKeyBinding {
    pressed: Option<ViewAction>,
    repeated: Option<ViewAction>,
    released: Option<ViewAction>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InputboxBinding {
    pressed: Option<InputboxAction>,
    repeated: Option<InputboxAction>,
    released: Option<InputboxAction>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum BindingRequirement {
    Any {
        binding: KeyBinding,
    },
    None {
        binding: KeyBinding,
    },
    WithModifiers {
        bindings: Vec<(ModifiersImpl, KeyBinding)>,
    },
}

impl BindingRequirement {
    pub fn any(kb: KeyBinding) -> BindingRequirement {
        BindingRequirement::Any { binding: kb }
    }

    pub fn none(kb: KeyBinding) -> BindingRequirement {
        BindingRequirement::None { binding: kb }
    }

    pub fn mods(bindings: Vec<(ModifiersImpl, KeyBinding)>) -> BindingRequirement {
        BindingRequirement::WithModifiers { bindings }
    }

    pub fn ctrl(binding: KeyBinding) -> BindingRequirement {
        BindingRequirement::WithModifiers { bindings: vec![(ModifiersImpl::CONTROL, binding)] }
    }

    pub fn ctrl_shift(binding: KeyBinding) -> BindingRequirement {
        BindingRequirement::WithModifiers { bindings: vec![(ModifiersImpl::CONTROL | ModifiersImpl::SHIFT, binding)] }
    }

    pub fn alt_shift(binding: KeyBinding) -> BindingRequirement {
        BindingRequirement::WithModifiers { bindings: vec![(ModifiersImpl::SHIFT | ModifiersImpl::ALT, binding)] }
    }

    pub fn ctrl_alt_shift(binding: KeyBinding) -> BindingRequirement {
        BindingRequirement::WithModifiers { bindings: vec![(ModifiersImpl::CONTROL | ModifiersImpl::ALT | ModifiersImpl::SHIFT, binding)] }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct KeyBinding {
    pressed: Option<InputTranslation>,
    repeated: Option<InputTranslation>,
    released: Option<InputTranslation>,
}

type Translation = Option<InputTranslation>;

impl KeyBinding {
    pub fn kb(pressed: Translation, repeated: Translation, released: Translation) -> KeyBinding {
        KeyBinding { pressed, repeated, released }
    }

    pub fn press(pressed: InputTranslation) -> KeyBinding {
        KeyBinding { pressed: Some(pressed), repeated: None, released: None }
    }

    pub fn repeated(repeated: InputTranslation) -> KeyBinding {
        KeyBinding { pressed: None, repeated: Some(repeated), released: None }
    }

    pub fn released(repeated: InputTranslation) -> KeyBinding {
        KeyBinding { pressed: None, repeated: Some(repeated), released: None }
    }

    pub fn holding(translation: InputTranslation) -> KeyBinding {
        KeyBinding { pressed: Some(translation.clone()), repeated: Some(translation), released: None }
    }
}

#[derive(Serialize, Deserialize)]
pub struct KeyBindings {
    key_map: HashMap<KeyImpl, BindingRequirement>,
    /// Text View key mappings
    tv_key_map: HashMap<(KeyImpl, ModifiersImpl), TextViewKeyBinding>,
    /// Input box key mappings
    ib_key_map: HashMap<(KeyImpl, ModifiersImpl), InputboxBinding>,
}

fn magic(glfw_key: &glfw::Key, glfw_modifiers: &glfw::Modifiers) -> (&KeyImpl, &ModifiersImpl) {
    unsafe { (std::mem::transmute(key), std::mem::transmute(modifiers)) }    
}

/// For serialization purposes we have re-implemented the glfw::Key and glfw::Modifiers
/// Which is why we use our own KeyImpl and ModifiersImpl here. But since they are implemented in an *exact*
/// one-to-one ratio, we can safely transmute between the types and have the compiler verify that we are correct still for doing so.
impl KeyBindings {
    pub fn new() -> KeyBindings {
        KeyBindings { key_map: HashMap::new(), tv_key_map: HashMap::new(), ib_key_map: HashMap::new() }
    }

    pub fn translate_textview_input(&self, key: glfw::Key, action: glfw::Action, modifiers: glfw::Modifiers) -> Option<ViewAction> {
        let key = unsafe { std::mem::transmute(key) };
        let modifiers = unsafe { std::mem::transmute(modifiers) };
    }

    pub fn translate_command_input(&self, key: glfw::Key, action: glfw::Action, modifiers: glfw::Modifiers) -> Option<InputboxAction> {
        let key = unsafe { std::mem::transmute(key) };
        let modifiers = unsafe { std::mem::transmute(modifiers) };
    }

    pub fn translate(&self, key: glfw::Key, action: glfw::Action, modifiers: glfw::Modifiers) -> Option<InputTranslation> {
        let key = unsafe { std::mem::transmute(key) };
        let modifiers = unsafe { std::mem::transmute(modifiers) };

        self.key_map.get(&key).and_then(|br| match br {
            BindingRequirement::Any { binding } => match action {
                glfw::Action::Release => binding.released.clone(),
                glfw::Action::Press => binding.pressed.clone(),
                glfw::Action::Repeat => binding.repeated.clone(),
            },
            BindingRequirement::None { binding } => match action {
                glfw::Action::Release => binding.released.clone(),
                glfw::Action::Press => binding.pressed.clone(),
                glfw::Action::Repeat => binding.repeated.clone(),
            },
            BindingRequirement::WithModifiers { bindings } => match action {
                glfw::Action::Release => bindings.iter().find(|(m, ..)| *m == modifiers).and_then(|(_, b)| b.released.clone()),
                glfw::Action::Press => bindings.iter().find(|(m, ..)| *m == modifiers).and_then(|(_, b)| b.pressed.clone()),
                glfw::Action::Repeat => bindings.iter().find(|(m, ..)| *m == modifiers).and_then(|(_, b)| b.repeated.clone()),
            },
        })
    }
}
