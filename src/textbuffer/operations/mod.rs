use super::metadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperationParameter {
    Char(char),
    Range(String),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operation {
    Insert(metadata::Index, OperationParameter),
    Delete(metadata::Index, OperationParameter),
}

impl Operation {
    pub fn index(&self) -> metadata::Index {
        match self {
            Operation::Insert(i, ..) => *i,
            Operation::Delete(i, ..) => *i,
        }
    }
}

#[derive(Debug)]
pub struct History {
    history_stack: Vec<Operation>,
    /// the undo stack are just for operation which we want to redo
    /// so if we undo an operation, it gets put here. Every time the user types something, it
    /// invalidates the undo stack, since the user has created a new time line (which still exists in the history stack, but the undone operations are now purged)
    undo_stack: Vec<Operation>,
}

impl History {
    pub fn new() -> History {
        History { history_stack: Vec::with_capacity(1024), undo_stack: vec![] }
    }

    #[inline(always)]
    fn invalidate_undo_stack(&mut self) {
        self.undo_stack.clear();
        todo!();
    }

    /// Pushes a new operation on to the history stack. This invalidates the undo stack.
    /// * `index` - the buffer position where the insert operation was made
    /// * `op_param` - The operation parameter, whether the user typed a range of characters or just 1 character
    /// * `coalesce` - if this is true, it will coalesce the last few operations until it finds an operation with a operation parameter ending and/or containing a white space
    pub fn push_insert(&mut self, index: metadata::Index, op_param: OperationParameter, coalesce: bool) {
        self.history_stack.push(Operation::Insert(index, op_param));
        if coalesce {
            for item in self.history_stack.iter().rev() {
                match item {
                    Operation::Insert(_, _) => todo!(),
                    Operation::Delete(_, _) => todo!(),
                }
            }
        }
        self.invalidate_undo_stack();
        todo!();
    }

    pub fn push_insert_char(&mut self, index: metadata::Index, ch: char, coalesce: bool) {
        self.history_stack.push(Operation::Insert(index, OperationParameter::Char(ch)));
        self.undo_stack.clear();
        if coalesce {
            let p = self
                .history_stack
                .iter()
                .rev()
                .skip(1)
                .position(|item| match item {
                    Operation::Insert(_, c) => match c {
                        OperationParameter::Char(c) => c.is_whitespace(),
                        OperationParameter::Range(c) => c.chars().any(|c| c.is_whitespace()),
                    },
                    Operation::Delete(_, _) => true,
                })
                .map(|v| (self.history_stack.len() - 1) - (v + 1) + 1)
                .unwrap_or(0);

            let elems_to_coalesce: Vec<Operation> = self.history_stack.drain(p..).collect();
            if let Some(buffer_index) = elems_to_coalesce.get(0).map(Operation::index) {
                let mut chars = Vec::with_capacity(elems_to_coalesce.len() * 4);
                for elem in elems_to_coalesce {
                    match elem {
                        Operation::Insert(.., op) => match op {
                            OperationParameter::Char(c) => chars.push(c),
                            OperationParameter::Range(range) => chars.extend(range.chars()),
                        },
                        Operation::Delete(_, _) => panic!("this must not happen"),
                    }
                }
                self.history_stack
                    .push(Operation::Insert(buffer_index, OperationParameter::Range(chars.iter().collect())));
            } else {
                // do nothing and abort coalesce
            }
        }
    }

    /// When a user deletes something, we push a delete operation
    pub fn push_delete(&mut self, index: metadata::Index, op_param: OperationParameter) {
        self.history_stack.push(Operation::Delete(index, op_param));
        todo!();
    }

    fn pop(&mut self) -> Option<Operation> {
        self.history_stack.pop()
    }

    /// Pops the latest operation from the history stack and pushes it onto the undo stack.
    /// It takes the operation and inverses it. So if when you hit "undo", it will take whatever's top of the history stack
    /// inverse it (from a delete->insert and vice versa) and push that onto the undo stack. This is how one can achieve undo / redo
    pub fn undo(&mut self) -> Option<&Operation> {
        let popped = self.pop();
        if let Some(op) = popped {
            self.undo_stack.push(op);
        }
        self.undo_stack.last()
    }
}

#[derive(Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Deserialize, Serialize)]
pub enum LineOperation {
    ShiftLeft { shift_by: usize },
    ShiftRight { shift_by: usize },
    PasteAt { insertion: char },
}

#[cfg(test)]
pub mod tests {
    use crate::textbuffer::{metadata, operations::OperationParameter};

    use super::{History, Operation};

    #[test]
    fn test_coalescing() {
        let mut history = History::new();
        let mut offset = 0;
        let start = metadata::Index(0);

        history.push_insert_char(start.offset(offset), 'c', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), 'a', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), 'l', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), 'l', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), ' ', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), '9', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), '1', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), '1', true);
        let last = history.history_stack.last().unwrap();
        assert_eq!(*last, Operation::Insert(metadata::Index(5), OperationParameter::Range("911".into())));
        offset += 1;
        history.push_insert_char(start.offset(offset), '!', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), '!', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), '!', true);
        let last = history.history_stack.last().unwrap();
        assert_eq!(*last, Operation::Insert(metadata::Index(5), OperationParameter::Range("911!!!".into())));
        let undo_911___ = history.undo();
        assert_eq!(Some(&Operation::Insert(metadata::Index(5), OperationParameter::Range("911!!!".into()))), undo_911___);
        // here, history will look like this:
        // History Stack: ['c', 'a', 'l', 'l', ' '] |---| Undo Stack: ["911!!!"]
        history.push_insert_char(start.offset(offset), 'n', true);
        assert_eq!(0, history.undo_stack.len());
    }

    #[test]
    fn test_invalidate_undo_stack_after_insert() {
        let mut history = History::new();
        let mut offset = 0;
        let start = metadata::Index(0);

        history.push_insert_char(start.offset(offset), 'c', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), 'a', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), 'l', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), 'l', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), ' ', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), '9', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), '1', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), '1', true);
        let last = history.history_stack.last().unwrap();
        assert_eq!(*last, Operation::Insert(metadata::Index(5), OperationParameter::Range("911".into())));
        let _ = history.undo();
        history.push_insert_char(start.offset(offset), 'n', false);
        let last = history.history_stack.last();
        assert_eq!(history.undo_stack.len(), 0);
        assert_eq!(last, Some(&Operation::Insert(metadata::Index(offset as _), OperationParameter::Char('n'))));
    }

    #[test]
    fn test_invalidate_undo_stack_after_insert_then_coalesce() {
        let mut history = History::new();
        let mut offset = 0;
        let start = metadata::Index(0);

        history.push_insert_char(start.offset(offset), 'c', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), 'a', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), 'l', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), 'l', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), ' ', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), '9', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), '1', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), '1', true);
        let last = history.history_stack.last().unwrap();
        assert_eq!(*last, Operation::Insert(metadata::Index(5), OperationParameter::Range("911".into())));
        let _ = history.undo();
        let now_begin = offset;
        history.push_insert_char(start.offset(offset), 'n', false);
        offset += 1;
        assert_eq!(history.undo_stack.len(), 0);
        history.push_insert_char(start.offset(offset), 'o', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), 'w', true);
        assert_eq!(Some(&Operation::Insert(metadata::Index(now_begin as _), OperationParameter::Range("now".into()))), history.history_stack.last());
    }
}
