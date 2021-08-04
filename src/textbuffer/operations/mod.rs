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
    ShiftLeft { shift_by: usize },
    ShiftRight { shift_by: usize },
    PasteAt { insertion: char },
}

impl std::str::FromStr for LineOperation {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const SL: &str = "ShiftLeft";
        const SR: &str = "ShiftRight";
        const PA: &str = "PasteAt";

        if &s[0..SL.len()] == SL {
            if let (Some(_start), Some(end)) = (s.find("{"), s.find("}")) {
                if let Some(colon) = s.find(":") {
                    let n = s[colon + 1..end].chars().filter(|c| c.is_whitespace()).collect::<String>();
                    if let Ok(n) = n.parse::<usize>() {
                        Ok(LineOperation::ShiftLeft { shift_by: n })
                    } else {
                        Err("Couldn't parse LineOperation from str")
                    }
                } else {
                    Err("Couldn't parse LineOperation from str")
                }
            } else {
                Err("Couldn't parse LineOperation from str")
            }
        } else if &s[0..SR.len()] == SR {
            if let (Some(_start), Some(end)) = (s.find("{"), s.find("}")) {
                if let Some(colon) = s.find(":") {
                    let n = s[colon + 1..end].chars().filter(|c| c.is_whitespace()).collect::<String>();
                    if let Ok(n) = n.parse::<usize>() {
                        Ok(LineOperation::ShiftRight { shift_by: n })
                    } else {
                        Err("Couldn't parse LineOperation from str")
                    }
                } else {
                    Err("Couldn't parse LineOperation from str")
                }
            } else {
                Err("Couldn't parse LineOperation from str")
            }
        } else if &s[0..PA.len()] == PA {
            if let (Some(_start), Some(end)) = (s.find("{"), s.find("}")) {
                if let Some(colon) = s.find(":") {
                    let n = s[colon + 1..end]
                        .chars()
                        .filter(|c| c.is_whitespace() || *c == '\'')
                        .collect::<String>();
                    Ok(LineOperation::PasteAt { insertion: n.chars().take(1).collect::<Vec<char>>()[0] })
                } else {
                    Err("Couldn't parse LineOperation from str")
                }
            } else {
                Err("Couldn't parse LineOperation from str")
            }
        } else {
            Err("Couldn't parse LineOperation from str")
        }
    }
}
