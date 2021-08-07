use super::metadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum OperationParameter {
    Char(char),
    Range(String),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
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

type Operations = Vec<Operation>;

#[derive(Debug)]
pub struct History {
    history_stack: Operations,
    /// the undo stack are just for operation which we want to redo
    /// so if we undo an operation, it gets put here. Every time the user types something, it
    /// invalidates the undo stack, since the user has created a new time line (which still exists in the history stack, but the undone operations are now purged)
    undo_stack: Operations,
}

impl History {
    pub fn new() -> History {
        History { history_stack: Vec::with_capacity(1024), undo_stack: vec![] }
    }

    #[inline(always)]
    fn invalidate_undo_stack(&mut self) {
        self.undo_stack.clear();
    }

    pub fn push_insert_range(&mut self, index: metadata::Index, op_param: String) {
        self.history_stack
            .push(Operation::Insert(index, OperationParameter::Range(op_param)));
        self.invalidate_undo_stack();
    }

    fn coalesce_history_stack_top(&mut self) {
        if !self.history_stack.len() > 1 {
            // to coalesce the stack, it makes sense we need more than 1 element
            let last = self.history_stack.pop().unwrap();
            let start_i = Operation::index(&last);
            match last {
                Operation::Insert(i, t) => {
                    let p = self
                        .history_stack
                        .iter()
                        .rev()
                        .scan(start_i, |state, i| {
                            let copy = *state;
                            *state = Operation::index(i);
                            Some((copy, i))
                        })
                        .position(|(above_index, item)| match item {
                            Operation::Insert(i, c) => match c {
                                OperationParameter::Char(c) => c.is_whitespace() || i.offset(1) != above_index,
                                OperationParameter::Range(c) => c.chars().any(|c| c.is_whitespace()) || i.offset(c.len() as isize) != above_index,
                            },
                            Operation::Delete(_, _) => true,
                        })
                        .map(|v| (self.history_stack.len() - 1) - v + 1)
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
                        match t {
                            OperationParameter::Char(c) => {
                                chars.push(c);
                            }
                            OperationParameter::Range(r) => {
                                chars.extend(r.chars());
                            }
                        }
                        self.history_stack
                            .push(Operation::Insert(buffer_index, OperationParameter::Range(chars.iter().collect())));
                    }
                }
                Operation::Delete(i, t) => {
                    let p = self
                        .history_stack
                        .iter()
                        .rev()
                        .scan(start_i, |state, i| {
                            let copy = *state;
                            *state = Operation::index(i);
                            Some((copy, i))
                        })
                        .position(|(above_index, item)| match item {
                            Operation::Insert(..) => true,
                            Operation::Delete(i, o) => match o {
                                OperationParameter::Char(c) => c.is_whitespace() || i.offset(-1) != above_index,
                                OperationParameter::Range(c) => c.chars().any(|c| c.is_whitespace()) || i.offset(-(c.len() as isize)) != above_index,
                            },
                        })
                        .map(|v| (self.history_stack.len() - 1) - v + 1)
                        .unwrap_or(0);
                    let elems_to_coalesce: Vec<Operation> = self.history_stack.drain(p..).collect();
                    let mut chars = Vec::with_capacity(elems_to_coalesce.len() * 4);
                    for elem in elems_to_coalesce {
                        match elem {
                            Operation::Insert(..) => {
                                panic!("This must not happen!")
                            }
                            Operation::Delete(.., op) => match op {
                                OperationParameter::Char(c) => chars.push(c),
                                OperationParameter::Range(range) => chars.extend(range.chars()),
                            },
                        }
                    }
                    match t {
                        OperationParameter::Char(c) => {
                            chars.push(c);
                        }
                        OperationParameter::Range(r) => {
                            chars.extend(r.chars());
                        }
                    }
                    if chars.len() == 1 {
                        self.history_stack.push(Operation::Delete(i, OperationParameter::Char(chars[0])));
                    } else {
                        self.history_stack
                            .push(Operation::Delete(i, OperationParameter::Range(chars.iter().rev().collect())));
                    }
                }
            }
        }
    }

    #[allow(unused)]
    fn coalesce_undo_stack_top(&mut self) {
        todo!("coalesce_undo_stack_top() not implemented yet");
    }

    pub fn push_insert_char(&mut self, index: metadata::Index, ch: char, coalesce: bool) {
        self.history_stack.push(Operation::Insert(index, OperationParameter::Char(ch)));
        self.invalidate_undo_stack();
        if coalesce {
            self.coalesce_history_stack_top();
        }
    }

    /// When a user deletes something, we push a delete operation
    pub fn push_delete_char(&mut self, index: metadata::Index, ch: char, coalesce: bool) {
        self.history_stack.push(Operation::Delete(index, OperationParameter::Char(ch)));
        self.invalidate_undo_stack();
        if coalesce {
            self.coalesce_history_stack_top();
        }
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
    fn test_insert_coalescing() {
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
        assert_eq!(*last, Operation::Insert(metadata::Index(5), OperationParameter::Range("911".into())), "coalesce failed");
        offset += 1;
        history.push_insert_char(start.offset(offset), '!', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), '!', false);
        offset += 1;
        history.push_insert_char(start.offset(offset), '!', true);
        let last = history.history_stack.last().unwrap();
        assert_eq!(*last, Operation::Insert(metadata::Index(5), OperationParameter::Range("911!!!".into())), "2nd coalesce failed");
        let undo_911___ = history.undo();
        assert_eq!(Some(&Operation::Insert(metadata::Index(5), OperationParameter::Range("911!!!".into()))), undo_911___, "Undo operation failed");
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
    fn test_invalidate_undo_stack_after_insert_then_coalesce_inserts() {
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
        history.push_insert_char(start.offset(offset), 'w', false);
        history.coalesce_history_stack_top();
        assert_eq!(Some(&Operation::Insert(metadata::Index(now_begin as _), OperationParameter::Range("now".into()))), history.history_stack.last());
    }

    #[test]
    fn test_delete_coalescing() {
        let mut history = History::new();
        let mut offset = 20;
        let start = metadata::Index(0);
        // !!!! A
        history.push_delete_char(start.offset(offset), 'A', false);
        offset -= 1;

        history.push_delete_char(start.offset(offset), ' ', false);
        offset -= 1;

        history.push_delete_char(start.offset(offset), '!', false);
        offset -= 1;
        history.push_delete_char(start.offset(offset), '!', false);
        offset -= 1;
        history.push_delete_char(start.offset(offset), '!', false);
        let last = history.history_stack.last().unwrap();
        assert_eq!(*last, Operation::Delete(metadata::Index(offset as _), OperationParameter::Char('!')));
        offset -= 1isize;
        history.push_delete_char(start.offset(offset), '!', true);
        let last = history.history_stack.last().unwrap();
        assert_eq!(*last, Operation::Delete(start.offset(offset), OperationParameter::Range("!!!!".into())));

        offset -= 1isize;
        history.push_delete_char(start.offset(offset), ' ', false);
        offset -= 1isize;
        history.push_delete_char(start.offset(offset), 'r', false);
        offset -= 1isize;
        history.push_delete_char(start.offset(offset), 'a', false);
        offset -= 1isize;
        history.push_delete_char(start.offset(offset), 'b', false);
        offset -= 1isize;
        history.push_delete_char(start.offset(offset), 'o', false);
        offset -= 1isize;
        history.push_delete_char(start.offset(offset), 'o', false);
        offset -= 1isize;
        history.push_delete_char(start.offset(offset), 'f', true);
        let last = history.history_stack.last().unwrap().clone();
        assert_eq!(last, Operation::Delete(start.offset(offset), OperationParameter::Range("foobar".into())));
        let undo = history.undo().clone();
        assert_eq!(last, *undo.unwrap());
        offset -= 1isize;
        history.push_delete_char(start.offset(offset), 'f', true);
        let undo = history.undo().unwrap().clone();
        assert_ne!(last, undo);
        assert_eq!(history.undo_stack.len(), 1);
        assert_eq!(undo, Operation::Delete(start.offset(offset), OperationParameter::Char('f')));
    }
}
