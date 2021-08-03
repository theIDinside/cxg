use super::metadata;
use serde::{Deserialize, Serialize};

pub enum OperationParameter {
    Char(char),
    Range(String),
}

pub enum Operation {
    Insert(metadata::Index, OperationParameter),
    Delete(metadata::Index, OperationParameter),
}

#[derive(Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Deserialize, Serialize)]
pub enum LineOperation {
    ShiftLeft {
        shift_by: usize,
    },
    ShiftRight {
        shift_by: usize,
    },
    InsertElement {
        at_column: metadata::Column,
        insertion: Option<char>,
    },
    InsertString {
        at_column: metadata::Column,
        insertion: Option<String>,
    },
}
