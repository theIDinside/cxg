use super::metadata;
use serde::{Deserialize, Serialize};

// Todo: Implement serialization of the History data, to be used in the file caching/backup scheme

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

    pub fn push_insert(&mut self, index: metadata::Index, ch: char) {
        self.invalidate_undo_stack();
        let mut coalesced = false;
        if !ch.is_whitespace() {
            if let Some(Operation::Insert(i, o)) = self.history_stack.last_mut() {
                match o {
                    OperationParameter::Char(c) if !c.is_whitespace() && i.offset(1) == index => {
                        let mut s = String::with_capacity(2);
                        s.push(*c);
                        s.push(ch);
                        *o = OperationParameter::Range(s);
                        coalesced = true;
                    }
                    OperationParameter::Range(d) if i.offset(d.len() as _) == index => {
                        d.push(ch);
                        coalesced = true;
                    }
                    _ => {}
                }
            }
        }
        if !coalesced {
            self.history_stack.push(Operation::Insert(index, OperationParameter::Char(ch)));
        }
    }

    pub fn push_delete(&mut self, index: metadata::Index, ch: char) {
        self.invalidate_undo_stack();
        let mut coalesced = false;
        if !ch.is_whitespace() {
            if let Some(Operation::Delete(i, o)) = self.history_stack.last_mut() {
                coalesced = match o {
                    OperationParameter::Char(c) if !c.is_whitespace() => {
                        if index.offset(1) == *i {
                            // we've deleted backwards (backspace)
                            // thus, ch is going to be the first character in this range (from left to right)
                            let mut s = String::with_capacity(2);
                            s.push(ch);
                            s.push(*c);
                            *o = OperationParameter::Range(s);
                            *i = index;
                            true
                        } else if *i == index {
                            // we've deleted forwards (delete key). here we reverse the pushing of c and ch, since the first character
                            // in the range delete, was c
                            let mut s = String::with_capacity(2);
                            s.push(*c);
                            s.push(ch);
                            *o = OperationParameter::Range(s);
                            *i = index;
                            true
                        } else {
                            false
                        }
                    }
                    OperationParameter::Range(d) => {
                        if *i == index {
                            // we've deleted forwards
                            d.push(ch);
                            true
                        } else if index.offset(1) == *i {
                            //we've deleted backwards
                            d.insert(0, ch);
                            *i = index;
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                };
            }
        }
        if !coalesced {
            self.history_stack.push(Operation::Delete(index, OperationParameter::Char(ch)));
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
            self.undo_stack.last()
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<&Operation> {
        let popped = self.undo_stack.pop();
        if let Some(op) = popped {
            self.history_stack.push(op);
            self.history_stack.last()
        } else {
            None
        }
    }
}

#[derive(Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Deserialize, Serialize)]
pub enum LineOperation {
    ShiftLeft { shift_by: usize },
    ShiftRight { shift_by: usize },
    PasteAt { insertion: char },
}

#[cfg(test)]
pub mod history_tests {
    use crate::textbuffer::{contiguous::contiguous::ContiguousBuffer, metadata, operations::OperationParameter, CharBuffer, Movement, TextKind};

    use super::{History, Operation};

    #[test]
    fn test_invalidate_undo_stack_after_insert() {
        let mut history = History::new();
        let mut offset = 0;
        let start = metadata::Index(0);

        history.push_insert(start.offset(offset), 'c');
        offset += 1;
        history.push_insert(start.offset(offset), 'a');
        offset += 1;
        history.push_insert(start.offset(offset), 'l');
        offset += 1;
        history.push_insert(start.offset(offset), 'l');
        offset += 1;
        history.push_insert(start.offset(offset), ' ');
        offset += 1;
        history.push_insert(start.offset(offset), '9');
        offset += 1;
        history.push_insert(start.offset(offset), '1');
        offset += 1;
        history.push_insert(start.offset(offset), '1');
        let last = history.history_stack.last().unwrap();
        assert_eq!(*last, Operation::Insert(metadata::Index(5), OperationParameter::Range("911".into())));
        let _ = history.undo();
        history.push_insert(start.offset(offset), 'n');
        let last = history.history_stack.last();
        assert_eq!(history.undo_stack.len(), 0);
        assert_eq!(last, Some(&Operation::Insert(metadata::Index(offset as _), OperationParameter::Char('n'))));
    }

    #[test]
    fn test_invalidate_undo_stack_after_insert_then_coalesce_inserts() {
        let mut history = History::new();
        let mut offset = 0;
        let start = metadata::Index(0);

        history.push_insert(start.offset(offset), 'c');
        offset += 1;
        history.push_insert(start.offset(offset), 'a');
        offset += 1;
        history.push_insert(start.offset(offset), 'l');
        offset += 1;
        history.push_insert(start.offset(offset), 'l');
        offset += 1;
        history.push_insert(start.offset(offset), ' ');
        offset += 1;
        history.push_insert(start.offset(offset), '9');
        offset += 1;
        history.push_insert(start.offset(offset), '1');
        offset += 1;
        history.push_insert(start.offset(offset), '1');
        let last = history.history_stack.last().unwrap();
        assert_eq!(*last, Operation::Insert(metadata::Index(5), OperationParameter::Range("911".into())));
        let _ = history.undo();
        let now_begin = offset;
        history.push_insert(start.offset(offset), 'n');
        offset += 1;
        assert_eq!(history.undo_stack.len(), 0);
        history.push_insert(start.offset(offset), 'o');
        offset += 1;
        history.push_insert(start.offset(offset), 'w');
        assert_eq!(Some(&Operation::Insert(metadata::Index(now_begin as _), OperationParameter::Range("now".into()))), history.history_stack.last());
    }

    #[test]
    fn test_delete_bwd_coalescing() {
        let mut history = History::new();
        let mut offset = 30;
        let start = metadata::Index(0);
        // !!!! A
        history.push_delete(start.offset(offset), 'A');
        offset -= 1;

        history.push_delete(start.offset(offset), ' ');
        offset -= 1;

        history.push_delete(start.offset(offset), '!');
        offset -= 1;
        history.push_delete(start.offset(offset), '!');
        offset -= 1;
        history.push_delete(start.offset(offset), '!');
        offset -= 1isize;
        history.push_delete(start.offset(offset), '!');
        let last = history.history_stack.last().unwrap();
        assert_eq!(*last, Operation::Delete(start.offset(offset), OperationParameter::Range("!!!!".into())));

        offset -= 1isize;
        history.push_delete(start.offset(offset), ' ');
        offset -= 1isize;
        history.push_delete(start.offset(offset), 'r');
        offset -= 1isize;
        history.push_delete(start.offset(offset), 'a');
        offset -= 1isize;
        history.push_delete(start.offset(offset), 'b');
        offset -= 1isize;
        history.push_delete(start.offset(offset), 'o');
        offset -= 1isize;
        history.push_delete(start.offset(offset), 'o');
        offset -= 1isize;
        history.push_delete(start.offset(offset), 'f');
        let last = history.history_stack.last().unwrap().clone();
        assert_eq!(last, Operation::Delete(start.offset(offset), OperationParameter::Range("foobar".into())));
        let undo = history.undo().clone();
        assert_eq!(last, *undo.unwrap());
        offset -= 1isize;
        history.push_delete(start.offset(100), 'f');
        let undo = history.undo().unwrap().clone();
        assert_ne!(last, undo);
        assert_eq!(history.undo_stack.len(), 1);
        assert_eq!(undo, Operation::Delete(start.offset(100), OperationParameter::Char('f')));
    }

    #[test]
    fn test_delete_fwd_coalescing() {
        let mut history = History::new();
        let start = metadata::Index(30);
        // delete "foobar", starting at f and deleting forwards (i.e. simulating the user hitting the delete key)
        history.push_delete(start, 'F');
        assert_eq!(history.history_stack.last(), Some(&Operation::Delete(start, OperationParameter::Char('F'))));
        history.push_delete(start, 'o');
        history.push_delete(start, 'o');
        assert_eq!(history.history_stack.last(), Some(&Operation::Delete(start, OperationParameter::Range(String::from("Foo")))));
        history.push_delete(start, 'b');
        history.push_delete(start, 'a');
        history.push_delete(start, 'r');
        assert_eq!(history.history_stack.last(), Some(&Operation::Delete(start, OperationParameter::Range("Foobar".into()))));
    }

    #[test]
    fn test_delete_fwd_coalesce_then_move_cursor_and_delete() {
        let mut history = History::new();
        let start = metadata::Index(30);
        // delete "foobar", starting at f and deleting forwards (i.e. simulating the user hitting the delete key)
        history.push_delete(start, 'F');
        history.push_delete(start, 'o');
        history.push_delete(start, 'o');
        history.push_delete(start, 'b');
        history.push_delete(start, 'a');
        history.push_delete(start, 'r');
        // Moving cursor
        let offset = 30;
        let new_idx = start.offset(offset);
        history.push_delete(new_idx, 'H');
        assert_ne!(history.history_stack.last(), Some(&Operation::Delete(start, OperationParameter::Range("FoobarH".into()))));
        assert_eq!(history.history_stack.last(), Some(&Operation::Delete(new_idx, OperationParameter::Char('H'))));
    }

    #[test]
    fn test_always_coalesce_insert() {
        let mut history = History::new();
        let mut offset = 0;
        let start = metadata::Index(0);
        history.push_insert(start.offset(offset), 'c');
        offset += 1;
        history.push_insert(start.offset(offset), 'a');
        offset += 1;
        history.push_insert(start.offset(offset), 'l');
        offset += 1;
        history.push_insert(start.offset(offset), 'l');
        offset += 1;
        history.push_insert(start.offset(offset), ' ');
        offset += 1;
        history.push_insert(start.offset(offset), '9');
        offset += 1;
        history.push_insert(start.offset(offset), '1');
        offset += 1;
        history.push_insert(start.offset(offset), '1');

        let last = history.history_stack.last().unwrap();
        assert_eq!(*last, Operation::Insert(metadata::Index(5), OperationParameter::Range("911".into())), "coalesce failed");
        offset += 1;
        history.push_insert(start.offset(offset), '!');
        offset += 1;
        history.push_insert(start.offset(offset), '!');
        offset += 1;
        history.push_insert(start.offset(offset), '!');
        let last = history.history_stack.last().unwrap();
        assert_eq!(*last, Operation::Insert(metadata::Index(5), OperationParameter::Range("911!!!".into())), "2nd coalesce failed");
        let undo_911___ = history.undo();
        assert_eq!(Some(&Operation::Insert(metadata::Index(5), OperationParameter::Range("911!!!".into()))), undo_911___, "Undo operation failed");
        // here, history will look like this:
        // History Stack: ['c', 'a', 'l', 'l', ' '] |---| Undo Stack: ["911!!!"]
    }

    #[allow(unused)]
    #[test]
    fn test_use_with_buffer() {
        let add_char = |ch: char, buf: &mut ContiguousBuffer, history: &mut History| {
            let i = buf.cursor().absolute();
            buf.insert(ch, true);
            history.push_insert(i, ch);
        };

        let delete_char = |pos: metadata::Index, buf: &mut ContiguousBuffer, history: &mut History| {
            let i = buf.cursor().absolute().offset(-1);
            if let Some((from, to)) = buf.get_buffer_movement_result(Movement::Backward(TextKind::Char, 1)) {
                if let Some(ch) = buf.get(to).cloned() {
                    buf.delete(Movement::Backward(TextKind::Char, 1));
                    history.push_delete(i, ch);
                }
            }
        };

        let mut sb = ContiguousBuffer::new(0, 1024);
        let mut history = History::new();
        let s = "Hello world";
        for c in s.chars() {
            add_char(c, &mut sb, &mut history);
        }

        if let Some(undo) = history.undo() {
            match undo {
                Operation::Insert(i, op) => match op {
                    OperationParameter::Char(c) => sb.delete_at(*i),
                    OperationParameter::Range(d) => sb.delete_range(*i, i.offset(d.len() as _)),
                },
                Operation::Delete(i, op) => {}
            }
        }

        println!("{:#?}", history);
        println!("{:?}. Cursor: p{:?}", sb.data, sb.cursor());
    }
}
