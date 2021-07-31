use super::metadata;

pub enum OperationParameter {
    Char(char),
    Range(String),
}

pub enum Operation {
    Insert(metadata::Index, OperationParameter),
    Delete(metadata::Index, OperationParameter),
}
