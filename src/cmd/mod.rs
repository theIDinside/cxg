pub mod keybindings;
#[rustfmt::skip]
pub mod keyimpl;
pub mod translation;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CommandTag {
    Goto,
    GotoInFile,
    Find,
}

impl<'a> From<CommandTag> for &'a str {
    fn from(c: CommandTag) -> Self {
        match c {
            CommandTag::Goto => "Insert line to go to:",
            CommandTag::Find => "Input what to search for:",
            CommandTag::GotoInFile => "Insert file:line to go to",
        }
    }
}
