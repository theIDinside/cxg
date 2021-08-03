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

    pub fn translate(&self, key: glfw::Key, action: glfw::Action, modifiers: glfw::Modifiers) -> Option<InputTranslation> {
        self.map.get(&key).map(|bindings| {
            for (translation, binding) in bindings.iter() {
                match binding {
                    Action(a) if a == action => {
                        
                    }
                }
            }
        })
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
            .insert(K::C, vec![(InputTranslation::Copy, BindingRequirement::ActionModifier(A::Press, M::Control))]);
        map.map
            .insert(K::X, vec![(InputTranslation::Cut, BindingRequirement::ActionModifier(A::Press, M::Control))]);
        map.map
            .insert(K::G, vec![(InputTranslation::Goto, BindingRequirement::ActionModifier(A::Press, M::Control))]);
        map.map
            .insert(K::I, vec![(InputTranslation::OpenFile, BindingRequirement::ActionModifier(A::Press, M::Control | M::Shift))]);
        map.map
            .insert(K::O, vec![(InputTranslation::OpenFile, BindingRequirement::ActionModifier(A::Press, M::Control))]);
        map.map
            .insert(K::Tab, vec![(InputTranslation::CycleFocus, BindingRequirement::ActionModifier(A::Press, M::Control)),
                                (InputTranslation::LineOperation(LineOperation::ShiftLeft { shift_by: 4 }), BindingRequirement::ActionModifier(A::Press, M::Shift))
                                (InputTranslation::LineOperation(LineOperation::ShiftRight { shift_by: 4 }), BindingRequirement::Action(A::Press))]);

        map.map.insert(K::Q, vec![(InputTranslation::Quit, BindingRequirement::ActionModifier(A::Press, M::Control))]);
        map.map.insert(K::F1, vec![(InputTranslation::Debug, BindingRequirement::Action(A::Press))]);
        map.map.insert(K::D, vec![(InputTranslation::ShowDebugInterface, BindingRequirement::ActionModifier(A::Press, M::Control))]);
        map.map.insert(K::N, vec![(InputTranslation::OpenNewView, BindingRequirement::ActionModifier(A::Press, M::Control))]);
        map.map.insert(K::S, vec![(InputTranslation::SaveFile, BindingRequirement::ActionModifier(A::Press, M::Control))]);

        const left_key: Vec<_> = vec![
            (InputTranslation::Movement(Movement::Backward(TextKind::Char, 1)), BindingRequirement::Action(A::Press | A::Repeat)),
            (InputTranslation::Movement(Movement::Begin(TextKind::Word)), BindingRequirement::ActionModifier(A::Press | A::Repeat, M::Control)),
            (InputTranslation::Movement(Movement::Begin(TextKind::Block)), BindingRequirement::ActionModifier(A::Press | A::Repeat, M::Alt | M::Shift)),
            (InputTranslation::TextSelect(Movement::Begin(TextKind::Word)), BindingRequirement::ActionModifier(A::Press | A::Repeat, M::Control | M::Shift)),
            (InputTranslation::TextSelect(Movement::Backward(TextKind::Char, 1)), BindingRequirement::ActionModifier(A::Press | A::Repeat, M::Shift))];

        const right_key: Vec<_> = vec![
            (InputTranslation::Movement(Movement::Forward(TextKind::Char, 1)), BindingRequirement::Action(A::Press | A::Repeat)),
            (InputTranslation::Movement(Movement::End(TextKind::Word)), BindingRequirement::ActionModifier(A::Press | A::Repeat, M::Control)),
            (InputTranslation::Movement(Movement::End(TextKind::Block)), BindingRequirement::ActionModifier(A::Press | A::Repeat, M::Alt | M::Shift)),
            (InputTranslation::TextSelect(Movement::End(TextKind::Word)), BindingRequirement::ActionModifier(A::Press | A::Repeat, M::Control | M::Shift)),
            (InputTranslation::TextSelect(Movement::Forward(TextKind::Char, 1)), BindingRequirement::ActionModifier(A::Press | A::Repeat, M::Shift))];

        const up_key: Vec<_> = vec![(InputTranslation::Movement(Movement::Backward(TextKind::Line, 1)), BindingRequirement::Action(A::Press | A::Repeat)),
        (InputTranslation::TextSelect(Movement::Backward(TextKind::Line, 1)), BindingRequirement::ActionModifier(A::Press | A::Repeat, M::Shift))];

        const down_key: Vec<_> = vec![(InputTranslation::Movement(Movement::Forward(TextKind::Line, 1)), BindingRequirement::Action(A::Press | A::Repeat)),
        (InputTranslation::TextSelect(Movement::Forward(TextKind::Line, 1)), BindingRequirement::ActionModifier(A::Press | A::Repeat, M::Shift))];

        
        map.map.insert(K::Left,     left_key);
        map.map.insert(K::Right,    right_key);
        map.map.insert(K::Up,       up_key);
        map.map.insert(K::Down,     down_key);


        map.map.insert(K::End, vec![
            (InputTranslation::Movement(Movement::End(TextKind::Line)), BindingRequirement::Action(A::Press)),
            (InputTranslation::Movement(Movement::End(TextKind::File)), BindingRequirement::ActionModifier(A::Press, M::Control)),
            (InputTranslation::TextSelect(Movement::End(TextKind::Line)), BindingRequirement::ActionModifier(A::Press, M::Shift)),
            (InputTranslation::TextSelect(Movement::End(TextKind::File)), BindingRequirement::ActionModifier(A::Press, M::Shift | M::Control)),
        ]);

        map.map.insert(K::Home, vec![
            (InputTranslation::Movement(Movement::Begin(TextKind::Line)), BindingRequirement::Action(A::Press)),
            (InputTranslation::Movement(Movement::Begin(TextKind::File)), BindingRequirement::ActionModifier(A::Press, M::Control)),
            (InputTranslation::TextSelect(Movement::Begin(TextKind::Line)), BindingRequirement::ActionModifier(A::Press, M::Shift)),
            (InputTranslation::TextSelect(Movement::Begin(TextKind::File)), BindingRequirement::ActionModifier(A::Press, M::Shift | M::Control)),
        ]);

        map.map.insert(K::Kp1, vec![
            (InputTranslation::Movement(Movement::End(TextKind::Line)), BindingRequirement::Action(A::Press)),
            (InputTranslation::Movement(Movement::End(TextKind::File)), BindingRequirement::ActionModifier(A::Press, M::Control)),
            (InputTranslation::TextSelect(Movement::End(TextKind::Line)), BindingRequirement::ActionModifier(A::Press, M::Shift)),
            (InputTranslation::TextSelect(Movement::End(TextKind::File)), BindingRequirement::ActionModifier(A::Press, M::Shift | M::Control)),
        ]);

        map.map.insert(K::Kp7, vec![
            (InputTranslation::Movement(Movement::Begin(TextKind::Line)), BindingRequirement::Action(A::Press)),
            (InputTranslation::Movement(Movement::Begin(TextKind::File)), BindingRequirement::ActionModifier(A::Press, M::Control)),
            (InputTranslation::TextSelect(Movement::Begin(TextKind::Line)), BindingRequirement::ActionModifier(A::Press, M::Shift)),
            (InputTranslation::TextSelect(Movement::Begin(TextKind::File)), BindingRequirement::ActionModifier(A::Press, M::Shift | M::Control)),
        ]);

        map.map.insert(K::PageDown, vec![
            (InputTranslation::Movement(Movement::Forward(TextKind::Page, 1)), BindingRequirement::Action(A::Press)),
            (InputTranslation::TextSelect(Movement::Forward(TextKind::Page, 1)), BindingRequirement::ActionModifier(A::Press, M::Shift)),
        ]);
        map.map.insert(K::Kp3, vec![
            (InputTranslation::Movement(Movement::Forward(TextKind::Page, 1)), BindingRequirement::Action(A::Press)),
            (InputTranslation::TextSelect(Movement::Forward(TextKind::Page, 1)), BindingRequirement::ActionModifier(A::Press, M::Shift)),
        ]);

        map.map.insert(K::PageUp, vec![
            (InputTranslation::Movement(Movement::Backward(TextKind::Page, 1)), BindingRequirement::Action(A::Press)),
            (InputTranslation::TextSelect(Movement::Backward(TextKind::Page, 1)), BindingRequirement::ActionModifier(A::Press, M::Shift)),
        ]);
        map.map.insert(K::Kp9, vec![
            (InputTranslation::Movement(Movement::Backward(TextKind::Page, 1)), BindingRequirement::Action(A::Press)),
            (InputTranslation::TextSelect(Movement::Backward(TextKind::Page, 1)), BindingRequirement::ActionModifier(A::Press, M::Shift)),
        ]);

        map.map.insert(K::Delete, vec![
            (InputTranslation::Delete(Movement::Forward(TextKind::Char, 1)), BindingRequirement::Action(A::Press)),
            (InputTranslation::Delete(Movement::Forward(TextKind::Char, 1)), BindingRequirement::ActionModifier(A::Press, M::Control)),
        ]);

        map.map.insert(K::Backspace, vec![
            (InputTranslation::Delete(Movement::Backward(TextKind::Char, 1)), BindingRequirement::Action(A::Press)),
            (InputTranslation::Delete(Movement::Backward(TextKind::Char, 1)), BindingRequirement::ActionModifier(A::Press, M::Control)),
        ]);





        map
    }
}
