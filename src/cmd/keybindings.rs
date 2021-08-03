// For serializing to configuration files (which at first won't be human friendly)
// and deserializing.
use serde::{Deserialize, Serialize};

use crate::textbuffer::operations::LineOperation;
use crate::textbuffer::{Movement, TextKind};

use super::keyimpl::{KeyImpl, ModifiersImpl};

use super::translation::InputTranslation;
use std::collections::HashMap;

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
}

impl KeyBindings {
    pub fn new() -> KeyBindings {
        KeyBindings { key_map: HashMap::new() }
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
    
    pub fn default() -> KeyBindings {
        use KeyImpl as K;
        use ModifiersImpl as M;
        let mut map = KeyBindings::new();
        let kb = &mut map.key_map;
        use BindingRequirement as BR;
        use InputTranslation as IT;
        use KeyBinding as KB;

        let mut no_mod = ModifiersImpl::CONTROL;
        no_mod.toggle(ModifiersImpl::CONTROL);

        kb.insert(K::Enter, BR::any(KB::holding(IT::Enter)));

        kb.insert(K::Escape, BR::any(KB::press(IT::Cancel)));
        kb.insert(K::CapsLock, BR::any(KB::press(IT::Cancel)));
        kb.insert(
            K::W,
            BR::mods(vec![
                (M::CONTROL, KB::press(IT::CloseActiveView(false))),
                (M::CONTROL | M::SHIFT, KB::press(IT::CloseActiveView(true))),
            ]),
        );
        kb.insert(K::F, BR::ctrl(KB::press(IT::Search)));
        kb.insert(K::V, BR::ctrl(KB::press(IT::Paste)));
        kb.insert(K::C, BR::ctrl(KB::press(IT::Copy)));
        kb.insert(K::X, BR::ctrl(KB::press(IT::Cut)));
        kb.insert(K::G, BR::ctrl(KB::press(IT::Goto)));
        kb.insert(K::I, BR::ctrl_shift(KB::press(IT::OpenFile)));
        kb.insert(K::O, BR::ctrl(KB::press(IT::OpenFile)));
        let tab_translations = vec![
            (no_mod, KB::press(IT::LineOperation(LineOperation::ShiftRight { shift_by: 4 }))),
            (M::SHIFT, KB::press(IT::LineOperation(LineOperation::ShiftLeft { shift_by: 4 }))),
            (M::CONTROL, KB::press(IT::CycleFocus)),
        ];
        kb.insert(K::Tab, BR::mods(tab_translations));
        kb.insert(K::Q, BR::ctrl(KB::press(IT::Quit)));
        kb.insert(K::F1, BR::ctrl(KB::press(IT::Debug)));
        kb.insert(K::D, BR::ctrl(KB::press(IT::ShowDebugInterface)));
        kb.insert(K::N, BR::ctrl(KB::press(IT::OpenNewView)));
        kb.insert(K::S, BR::ctrl(KB::press(IT::SaveFile)));

        kb.insert(
            K::Left,
            BR::mods(vec![
                (no_mod, KB::holding(IT::Movement(Movement::Backward(TextKind::Char, 1)))),
                (M::CONTROL, KB::holding(IT::Movement(Movement::Begin(TextKind::Word)))),
                (M::SHIFT | M::ALT, KB::holding(IT::Movement(Movement::Begin(TextKind::Block)))),
                (M::CONTROL | M::SHIFT, KB::holding(IT::TextSelect(Movement::Begin(TextKind::Word)))),
                (M::SHIFT, KB::holding(IT::TextSelect(Movement::Backward(TextKind::Char, 1)))),
            ]),
        );

        kb.insert(
            K::Right,
            BR::mods(vec![
                (no_mod, KB::holding(IT::Movement(Movement::Forward(TextKind::Char, 1)))),
                (M::CONTROL, KB::holding(IT::Movement(Movement::End(TextKind::Word)))),
                (M::SHIFT | M::ALT, KB::holding(IT::Movement(Movement::End(TextKind::Block)))),
                (M::CONTROL | M::SHIFT, KB::holding(IT::TextSelect(Movement::End(TextKind::Word)))),
                (M::SHIFT, KB::holding(IT::TextSelect(Movement::Forward(TextKind::Char, 1)))),
            ]),
        );

        kb.insert(
            K::Up,
            BR::mods(vec![
                (no_mod, KB::holding(IT::Movement(Movement::Backward(TextKind::Line, 1)))),
                (M::SHIFT, KB::holding(IT::TextSelect(Movement::Backward(TextKind::Line, 1)))),
            ]),
        );

        kb.insert(
            K::Down,
            BR::mods(vec![
                (no_mod, KB::holding(IT::Movement(Movement::Forward(TextKind::Line, 1)))),
                (M::SHIFT, KB::holding(IT::TextSelect(Movement::Forward(TextKind::Line, 1)))),
            ]),
        );

        kb.insert(
            K::End,
            BR::mods(vec![
                (no_mod, KB::press(IT::Movement(Movement::End(TextKind::Line)))),
                (M::CONTROL, KB::press(IT::Movement(Movement::End(TextKind::File)))),
                (M::SHIFT, KB::press(IT::TextSelect(Movement::End(TextKind::Line)))),
                (M::SHIFT | M::CONTROL, KB::press(IT::TextSelect(Movement::End(TextKind::File)))),
            ]),
        );

        kb.insert(
            K::Home,
            BR::mods(vec![
                (no_mod, KB::press(IT::Movement(Movement::Begin(TextKind::Line)))),
                (M::CONTROL, KB::press(IT::Movement(Movement::Begin(TextKind::File)))),
                (M::SHIFT, KB::press(IT::TextSelect(Movement::Begin(TextKind::Line)))),
                (M::SHIFT | M::CONTROL, KB::press(IT::TextSelect(Movement::Begin(TextKind::File)))),
            ]),
        );

        kb.insert(
            K::Kp1,
            BR::mods(vec![
                (no_mod, KB::press(IT::Movement(Movement::End(TextKind::Line)))),
                (M::CONTROL, KB::press(IT::Movement(Movement::End(TextKind::File)))),
                (M::SHIFT, KB::press(IT::TextSelect(Movement::End(TextKind::Line)))),
                (M::SHIFT | M::CONTROL, KB::press(IT::TextSelect(Movement::End(TextKind::File)))),
            ]),
        );

        kb.insert(
            K::Kp7,
            BR::mods(vec![
                (no_mod, KB::press(IT::Movement(Movement::Begin(TextKind::Line)))),
                (M::CONTROL, KB::press(IT::Movement(Movement::Begin(TextKind::File)))),
                (M::SHIFT, KB::press(IT::TextSelect(Movement::Begin(TextKind::Line)))),
                (M::SHIFT | M::CONTROL, KB::press(IT::TextSelect(Movement::Begin(TextKind::File)))),
            ]),
        );

        kb.insert(
            K::PageDown,
            BR::mods(vec![
                (no_mod, KB::holding(IT::Movement(Movement::Forward(TextKind::Page, 1)))),
                (M::SHIFT, KB::holding(IT::TextSelect(Movement::Forward(TextKind::Page, 1)))),
            ]),
        );

        kb.insert(
            K::PageUp,
            BR::mods(vec![
                (no_mod, KB::holding(IT::Movement(Movement::Backward(TextKind::Page, 1)))),
                (M::SHIFT, KB::holding(IT::TextSelect(Movement::Backward(TextKind::Page, 1)))),
            ]),
        );

        kb.insert(
            K::Kp3,
            BR::mods(vec![
                (no_mod, KB::holding(IT::Movement(Movement::Forward(TextKind::Page, 1)))),
                (M::SHIFT, KB::holding(IT::TextSelect(Movement::Forward(TextKind::Page, 1)))),
            ]),
        );

        kb.insert(
            K::Kp9,
            BR::mods(vec![
                (no_mod, KB::holding(IT::Movement(Movement::Backward(TextKind::Page, 1)))),
                (M::SHIFT, KB::holding(IT::TextSelect(Movement::Backward(TextKind::Page, 1)))),
            ]),
        );

        kb.insert(
            K::Delete,
            BR::mods(vec![
                (no_mod, KB::holding(IT::Delete(Movement::Forward(TextKind::Char, 1)))),
                (M::CONTROL, KB::holding(IT::Delete(Movement::Forward(TextKind::Word, 1)))),
            ]),
        );

        kb.insert(
            K::Backspace,
            BR::mods(vec![
                (no_mod, KB::holding(IT::Delete(Movement::Backward(TextKind::Char, 1)))),
                (M::CONTROL, KB::holding(IT::Delete(Movement::Backward(TextKind::Word, 1)))),
            ]),
        );

        map
    }
}
