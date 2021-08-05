pub mod keybindings;
#[rustfmt::skip]
pub mod keyimpl;
pub mod translation;

// todo(feature): add SymbolList, for when we want to Go to Symbol
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CommandTag {
    Goto,
    GotoInFile,
    Find,
    OpenFile,
    SaveFile,
}

pub const COMMAND_NAMES: &[(&'static str, &'static CommandTag)] = &[
    ("GOTO", &CommandTag::Goto),
    ("GOTOINFILE", &CommandTag::GotoInFile),
    ("FIND", &CommandTag::Find),
    ("OPENFILE", &CommandTag::OpenFile),
    ("SAVEFILE", &CommandTag::SaveFile),
];

impl CommandTag {
    pub const fn description(tag: CommandTag) -> &'static str {
        match tag {
            CommandTag::Goto => "Insert line to go to:",
            CommandTag::Find => "Input what to search for:",
            CommandTag::GotoInFile => "Insert file:line to go to:",
            CommandTag::OpenFile => "Open file:",
            CommandTag::SaveFile => "Save file:",
        }
    }

    pub const fn name(tag: CommandTag) -> &'static str {
        match tag {
            CommandTag::Goto => "Go to",
            CommandTag::GotoInFile => "Go to in file",
            CommandTag::Find => "Find",
            CommandTag::OpenFile => "Open file",
            CommandTag::SaveFile => "Save file",
        }
    }
}

/// Matches user input against existing commands based on a rank search
pub fn commands_matching(input: &str) -> Option<Vec<&CommandTag>> {
    let mut result = Vec::with_capacity(COMMAND_NAMES.len());

    for (cmd_name, tag) in COMMAND_NAMES {
        if input.len() <= cmd_name.len() {
            let mut current_pos = 0;
            let mut matched = false;
            for c in input.to_uppercase().chars().filter(|c| !c.is_whitespace()) {
                if let Some(p) = cmd_name[current_pos..].find(c) {
                    current_pos = p;
                    matched = true;
                } else {
                    matched = false;
                    break;
                }
            }
            if matched {
                result.push(*tag);
            }
        }
    }
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

pub fn get_command(input: &str) -> Option<&CommandTag> {
    let input: String = input.to_ascii_uppercase().chars().filter(|&c| !c.is_whitespace()).collect();
    COMMAND_NAMES.iter().find(|(tag_str, ..)| *tag_str == input).map(|(.., tag)| *tag)
}

#[cfg(test)]
pub mod tests {
    use crate::cmd::CommandTag;

    use super::commands_matching;

    #[test]
    fn test_matches() {
        let goto_matches = "gt";
        let goto_matches2 = "gtf";
        let file_matches = "ef";

        let fi = "fi";

        let gmatches = commands_matching(goto_matches).unwrap();
        let gmatches2 = commands_matching(goto_matches2).unwrap();
        let fmatches = commands_matching(file_matches).unwrap();
        let fi_matches = commands_matching(fi).unwrap();

        assert_eq!(gmatches.len(), 2, "Length did not match!");
        assert_eq!(gmatches2.len(), 1, "Length did not match!");
        assert_eq!(fmatches.len(), 2, "Length did not match!");
        assert_eq!(fi_matches.len(), 4, "Length did not match!");

        // gt matches against Go To and Go To in file
        assert!(gmatches.contains(&&CommandTag::Goto), "Go to was not found in result");
        assert!(gmatches.contains(&&CommandTag::GotoInFile), "Go to in File was not found in result!");

        // but gtf only matches against Go To in File
        assert!(gmatches2.contains(&&CommandTag::GotoInFile), "Go to in File was not found in result!");

        // ef matches against opEn File and savE File
        assert!(fmatches.contains(&&CommandTag::SaveFile), "Save File was not found in result!");
        assert!(fmatches.contains(&&CommandTag::OpenFile), "Open File was not found in result!");

        // fi matches against open FIle, save FIle, go to in FIle and FInd
        assert!(fi_matches.contains(&&CommandTag::Find), "Save File was not found in result!");
        assert!(fi_matches.contains(&&CommandTag::OpenFile), "Open File was not found in result!");
        assert!(fi_matches.contains(&&CommandTag::SaveFile), "Save File was not found in result!");
        assert!(fi_matches.contains(&&CommandTag::GotoInFile), "Open File was not found in result!");
    }
}
