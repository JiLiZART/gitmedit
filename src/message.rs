/// A git message file split into the part the user writes and the comment trailer git owns.
#[derive(Debug, Clone, PartialEq)]
pub struct MessageFile {
    /// Lines shown in the editor.
    pub message: Vec<String>,
    /// Comment block, and anything below a scissors line, written back verbatim.
    pub trailer: Vec<String>,
    final_newline: bool,
}

/// `<cc> ------------------------ >8 ------------------------`, as `git commit -v` writes.
pub fn is_scissors(line: &str, cc: char) -> bool {
    line.strip_prefix(cc)
        .is_some_and(|rest| rest.trim() == "------------------------ >8 ------------------------")
}

pub fn split(raw: &str, cc: char) -> MessageFile {
    let final_newline = raw.ends_with('\n');
    let body = raw.strip_suffix('\n').unwrap_or(raw);
    let lines: Vec<&str> = if raw.is_empty() {
        Vec::new()
    } else {
        body.split('\n').collect()
    };

    let scissors = lines
        .iter()
        .position(|l| is_scissors(l, cc))
        .unwrap_or(lines.len());
    let first_comment = lines[..scissors]
        .iter()
        .position(|l| l.starts_with(cc))
        .unwrap_or(scissors);

    let mut message: Vec<String> = lines[..first_comment]
        .iter()
        .map(|l| l.to_string())
        .collect();
    trim_trailing_blank(&mut message);

    let mut trailer = Vec::new();
    let mut prev_moved: Option<usize> = None;
    for (i, line) in lines.iter().enumerate().take(scissors).skip(first_comment) {
        if line.starts_with(cc) || line.trim().is_empty() {
            trailer.push(line.to_string());
            continue;
        }
        // Message text interleaved with comments (squash files) moves up, with one blank line
        // between runs that were separated in the file.
        if !message.is_empty() && prev_moved != Some(i - 1) {
            message.push(String::new());
        }
        message.push(line.to_string());
        prev_moved = Some(i);
    }
    trailer.extend(lines[scissors..].iter().map(|l| l.to_string()));

    MessageFile {
        message,
        trailer,
        final_newline,
    }
}

impl MessageFile {
    /// Message without trailing blank lines, one blank line, then the trailer as read.
    pub fn assemble(&self, message: &[String]) -> String {
        let mut out: Vec<String> = message.to_vec();
        trim_trailing_blank(&mut out);
        if !self.trailer.is_empty() {
            out.push(String::new());
            out.extend(self.trailer.iter().cloned());
        }
        let mut s = out.join("\n");
        if self.final_newline {
            s.push('\n');
        }
        s
    }
}

fn trim_trailing_blank(lines: &mut Vec<String>) {
    while lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AMEND2: &str = include_str!("../fixtures/amend2/COMMIT_EDITMSG");
    const AMEND1: &str = include_str!("../fixtures/amend1/COMMIT_EDITMSG");
    const MERGE: &str = include_str!("../fixtures/merge1/MERGE_MSG");
    const MERGE2: &str = include_str!("../fixtures/merge2/MERGE_MSG");
    const REBASE_MSG: &str = include_str!("../fixtures/rebase2/COMMIT_EDITMSG");
    const PULL_REBASE: &str = include_str!("../fixtures/rebase3/MERGE_MSG");

    fn owned(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn commit_message_is_split_from_comments() {
        let f = split(AMEND1, '#');
        assert_eq!(
            f.message,
            owned(&["fix: hello here is a demo of gitmedit edit features for all git situations"])
        );
        assert!(f.trailer.iter().all(|l| l.starts_with('#') || l.is_empty()));
    }

    #[test]
    fn merge_message_keeps_blank_lines_between_comment_groups_in_trailer() {
        let f = split(MERGE, '#');
        assert_eq!(
            f.message,
            owned(&["Merge remote-tracking branch 'origin/BRANCH-NAME2' into 'BRANCH_NAME'"])
        );
        assert!(f.trailer.iter().any(|l| l.is_empty()));
    }

    #[test]
    fn untouched_fixtures_round_trip_byte_for_byte() {
        for raw in [AMEND2, AMEND1, MERGE, MERGE2, REBASE_MSG, PULL_REBASE] {
            let f = split(raw, '#');
            assert_eq!(f.assemble(&f.message), raw);
        }
    }

    #[test]
    fn fresh_commit_has_empty_message_and_round_trips() {
        let raw = "\n# Please enter the commit message\n#\n";
        let f = split(raw, '#');
        assert!(f.message.is_empty());
        assert_eq!(f.assemble(&f.message), raw);
    }

    #[test]
    fn edited_message_keeps_trailer() {
        let f = split(AMEND1, '#');
        let out = f.assemble(&owned(&["feat: new subject", "", "body"]));
        assert!(out.starts_with("feat: new subject\n\nbody\n\n# Please enter the commit message"));
        assert!(out.ends_with("#\tmodified:   packages/@project/components/contract-form/elements/EscrowDetails.tsx\n#\n"));
    }

    #[test]
    fn trailing_blank_lines_in_edited_message_collapse_to_one_separator() {
        let f = split("subject\n\n# c\n", '#');
        assert_eq!(
            f.assemble(&owned(&["subject", "", "", ""])),
            "subject\n\n# c\n"
        );
    }

    #[test]
    fn interleaved_squash_messages_move_into_message() {
        let raw = "# This is a combination of 2 commits.\n# This is the 1st commit message:\n\nfeat: one\n\nbody one\n\n# This is the commit message #2:\n\nfix: two\n";
        let f = split(raw, '#');
        assert_eq!(
            f.message,
            owned(&["feat: one", "", "body one", "", "fix: two"])
        );
        assert!(f.trailer.iter().all(|l| l.starts_with('#') || l.is_empty()));
        let out = f.assemble(&f.message);
        let kept: Vec<&str> = out.lines().filter(|l| !l.starts_with('#')).collect();
        assert_eq!(kept.join("\n").trim(), "feat: one\n\nbody one\n\nfix: two");
    }

    #[test]
    fn scissors_and_diff_stay_in_trailer() {
        let raw = "subject\n\n# Please enter\n# ------------------------ >8 ------------------------\n# Do not modify or remove the line above.\ndiff --git a/x b/x\n+added\n";
        let f = split(raw, '#');
        assert_eq!(f.message, owned(&["subject"]));
        assert!(f.trailer.contains(&"diff --git a/x b/x".to_string()));
        assert_eq!(f.assemble(&f.message), raw);
    }

    #[test]
    fn file_without_comments_has_no_trailer() {
        let f = split("just a message\n", '#');
        assert!(f.trailer.is_empty());
        assert_eq!(f.assemble(&f.message), "just a message\n");
    }

    #[test]
    fn missing_final_newline_is_preserved() {
        let f = split("subject\n\n# c", '#');
        assert_eq!(f.assemble(&f.message), "subject\n\n# c");
    }

    #[test]
    fn custom_comment_char() {
        let f = split("subject\n\n; comment\n", ';');
        assert_eq!(f.message, owned(&["subject"]));
        assert_eq!(f.trailer, owned(&["; comment"]));
    }

    #[test]
    fn scissors_detection() {
        assert!(is_scissors(
            "# ------------------------ >8 ------------------------",
            '#'
        ));
        assert!(!is_scissors("# ------------------------", '#'));
    }
}
