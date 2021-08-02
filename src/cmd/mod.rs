pub mod keybindings;
pub mod translation;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CommandTag {
    Goto,
    Find,
}

impl<'a> From<CommandTag> for &'a str {
    fn from(c: CommandTag) -> Self {
        match c {
            CommandTag::Goto => "Insert line to go to:",
            CommandTag::Find => "Input what to search for:",
        }
    }
}
