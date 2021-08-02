use glfw::Key;
use regex::internal::Input;

use super::translation::InputTranslation;
use std::collections::HashMap;

pub enum BindingRequirement {
    Action(glfw::Action),
    Modifier(glfw::Modifiers),
    ActionModifier(glfw::Action, glfw::Modifiers),
    None,
}

pub struct KeyBindings {
    map: HashMap<glfw::Key, Vec<(InputTranslation, BindingRequirement)>>,
}

impl KeyBindings {
    pub fn new() -> KeyBindings {
        KeyBindings { map: HashMap::new() }
    }

    pub fn default() -> KeyBindings {
        use glfw::Action as A;
        use glfw::Key as K;
        use glfw::Modifiers as M;
        let mut map = KeyBindings::new();
        map.map.insert(K::Escape, vec![(InputTranslation::Cancel, BindingRequirement::None)]);
        map.map
            .insert(K::W, vec![(InputTranslation::CloseActiveView, BindingRequirement::ActionModifier(A::Press, M::Control))]);
        map.map
            .insert(K::F, vec![(InputTranslation::Search, BindingRequirement::ActionModifier(A::Press, M::Control))]);
        map.map
            .insert(K::V, vec![(InputTranslation::Paste, BindingRequirement::ActionModifier(A::Press, M::Control))]);
        map.map
            .insert(K::G, vec![(InputTranslation::Goto, BindingRequirement::ActionModifier(A::Press, M::Control))]);
        map.map
            .insert(K::I, vec![(InputTranslation::OpenFile, BindingRequirement::ActionModifier(A::Press, M::Control | M::Shift))]);
        map.map
            .insert(K::O, vec![(InputTranslation::OpenFile, BindingRequirement::ActionModifier(A::Press, M::Control))]);
            map.map
            .insert(K::Tab, vec![(InputTranslation::OpenFile, BindingRequirement::ActionModifier(A::Press, M::Control | M::Shift))]);
        map
    }
}
