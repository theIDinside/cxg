use super::metadata;

pub enum OperationParameter {
    Char(char),
    Range(String),
}

pub enum Operation {
    Insert(metadata::Index, OperationParameter),
    Delete(metadata::Index, OperationParameter),
}

pub enum LineOperation<'a> {
    ShiftLeft {
        shift_by: usize,
    },
    ShiftRight {
        shift_by: usize,
    },
    InsertElement {
        at_column: metadata::Column,
        insertion: char,
    },
    InsertString {
        at_column: metadata::Column,
        insertion: &'a str,
    },
}
