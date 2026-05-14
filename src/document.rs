/// Controls which line variants are editable in the textarea.
///
/// - Plain: all lines (Content + Comment + ConflictMarker) are editable.
/// - Merge: Content and ConflictMarker are editable; Comment is protected.
/// - Squash: only Content is editable; Comment and ConflictMarker are protected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorMode {
    Plain,
    Merge,
    Squash,
}

/// Actions available for a rebase todo entry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RebaseAction {
    Pick,
    Squash,
    Fixup,
    Drop,
    Exec,
}

impl RebaseAction {
    /// Cycle to the next action in the rotation: Pick->Squash->Fixup->Drop->Pick.
    /// Exec does not cycle (returns itself).
    pub fn cycle(&self) -> Self {
        match self {
            RebaseAction::Pick => RebaseAction::Squash,
            RebaseAction::Squash => RebaseAction::Fixup,
            RebaseAction::Fixup => RebaseAction::Drop,
            RebaseAction::Drop => RebaseAction::Pick,
            RebaseAction::Exec => RebaseAction::Exec,
        }
    }

    /// Return the canonical long-form string representation.
    pub fn as_str(&self) -> &str {
        match self {
            RebaseAction::Pick => "pick",
            RebaseAction::Squash => "squash",
            RebaseAction::Fixup => "fixup",
            RebaseAction::Drop => "drop",
            RebaseAction::Exec => "exec",
        }
    }
}

/// Typed representation of a single line in a git-rebase-todo file.
#[derive(Debug, Clone, PartialEq)]
pub enum RebaseLine {
    Action {
        action: RebaseAction,
        hash: String,
        subject: String,
    },
    Comment(String),
}

/// Parse a rebase action keyword (long or short form) into a RebaseAction.
pub fn parse_action(s: &str) -> Option<RebaseAction> {
    match s {
        "pick" | "p" => Some(RebaseAction::Pick),
        "squash" | "s" => Some(RebaseAction::Squash),
        "fixup" | "f" => Some(RebaseAction::Fixup),
        "drop" | "d" => Some(RebaseAction::Drop),
        "exec" | "x" => Some(RebaseAction::Exec),
        _ => None,
    }
}

/// Classify a single raw line from a git-rebase-todo file.
///
/// Lines starting with comment_char are Comments. Lines that start with a
/// recognized action keyword are parsed into Action variants. Exec lines use
/// the remainder of the line as subject with an empty hash. Malformed lines
/// fall back to Comment.
pub fn classify_rebase_line(line: &str, comment_char: char) -> RebaseLine {
    if line.starts_with(comment_char) {
        return RebaseLine::Comment(line.to_string());
    }

    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.is_empty() {
        return RebaseLine::Comment(line.to_string());
    }

    match parse_action(parts[0]) {
        None => RebaseLine::Comment(line.to_string()),
        Some(RebaseAction::Exec) => {
            // exec format: "exec {command}" — no hash, rest of line is subject
            let subject = if parts.len() >= 2 {
                parts[1..].join(" ")
            } else {
                String::new()
            };
            RebaseLine::Action {
                action: RebaseAction::Exec,
                hash: String::new(),
                subject,
            }
        }
        Some(action) => {
            if parts.len() < 2 {
                return RebaseLine::Comment(line.to_string());
            }
            let hash = parts[1].to_string();
            let subject = if parts.len() >= 3 {
                parts[2].to_string()
            } else {
                String::new()
            };
            RebaseLine::Action { action, hash, subject }
        }
    }
}

/// Parse a full git-rebase-todo string into a Vec of RebaseLine.
///
/// Trailing empty token from a trailing newline is skipped, same as Document::parse.
pub fn parse_rebase_todo(raw: &str, comment_char: char) -> Vec<RebaseLine> {
    if raw.is_empty() {
        return Vec::new();
    }

    let raw_lines: Vec<&str> = raw.split('\n').collect();
    let slice = if raw.ends_with('\n') && raw_lines.len() > 1 {
        &raw_lines[..raw_lines.len() - 1]
    } else {
        &raw_lines[..]
    };

    slice.iter().map(|line| classify_rebase_line(line, comment_char)).collect()
}

/// Serialize a Vec of RebaseLine back to a git-rebase-todo string.
///
/// Action lines are always emitted with long-form action names.
/// Each line is followed by '\n'.
pub fn serialize_rebase_todo(lines: &[RebaseLine]) -> String {
    let mut output = String::new();
    for line in lines {
        match line {
            RebaseLine::Action { action, hash, subject } => {
                if *action == RebaseAction::Exec {
                    output.push_str("exec ");
                    output.push_str(subject);
                } else {
                    output.push_str(action.as_str());
                    output.push(' ');
                    output.push_str(hash);
                    output.push(' ');
                    output.push_str(subject);
                }
            }
            RebaseLine::Comment(text) => {
                output.push_str(text);
            }
        }
        output.push('\n');
    }
    output
}

/// Detect the boundary between the git-generated squash header and the editable message.
///
/// A SQUASH_MSG file generated by git starts with `"# This is a combination of N commits."`.
/// The header is the contiguous block of comment lines at the top of the file starting with
/// that marker. This function returns `Some(header_end_index)` where `header_end_index` is
/// the line index (0-based) of the first non-comment line after the squash header block.
///
/// If the entire file consists of the header (no non-comment lines follow), returns
/// `Some(total_lines)`.
///
/// If no squash marker is found, returns `None`.
///
/// The function uses `comment_char` rather than a hardcoded `'#'` because git respects
/// `core.commentChar` when generating the squash header.
pub fn detect_squash_header(raw: &str, comment_char: char) -> Option<usize> {
    let lines: Vec<&str> = raw.split('\n').collect();
    // Discard the phantom empty token that split() produces after a trailing '\n'.
    let lines = if raw.ends_with('\n') && lines.len() > 1 {
        &lines[..lines.len() - 1]
    } else {
        &lines[..]
    };

    let marker = format!("{} This is a combination of", comment_char);
    let mut in_header = false;

    for (i, line) in lines.iter().enumerate() {
        if !in_header && line.starts_with(&marker) {
            in_header = true;
            continue;
        }
        if in_header && !line.starts_with(comment_char) {
            return Some(i);
        }
    }

    // Entered header but never found a non-comment line: whole file is header.
    if in_header {
        return Some(lines.len());
    }

    None
}

/// Typed representation of a single line in the file being edited.
#[derive(Debug, Clone, PartialEq)]
pub enum ContentLine {
    Content(String),
    Comment(String),
    ConflictMarker(String),
}

/// Read the comment character from `git config core.commentchar`.
///
/// Returns `'#'` as the fallback in all error cases, including:
/// - git not in PATH
/// - no config value set
/// - value is "auto" (v1 limitation, auto-detection not implemented)
/// - output is empty
pub fn read_comment_char() -> char {
    let result = std::process::Command::new("git")
        .args(["config", "--get", "core.commentchar"])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let trimmed = stdout.trim();
            if trimmed == "auto" || trimmed.is_empty() {
                '#'
            } else {
                trimmed.chars().next().unwrap_or('#')
            }
        }
        _ => '#',
    }
}

/// Classify a single raw line into its ContentLine variant.
pub fn classify_line(line: &str, comment_char: char) -> ContentLine {
    if line.starts_with(comment_char) {
        ContentLine::Comment(line.to_string())
    } else if line.starts_with("<<<<<<")
        || line.starts_with("======")
        || line.starts_with(">>>>>>")
    {
        ContentLine::ConflictMarker(line.to_string())
    } else {
        ContentLine::Content(line.to_string())
    }
}

/// The parsed representation of a file.
///
/// `lines` holds every line in original order, typed by role.
/// `editable_index` maps TextArea row index -> index into `lines`.
/// Only `ContentLine::Content` variants appear in the editable index.
pub struct Document {
    lines: Vec<ContentLine>,
    editable_index: Vec<usize>,
    comment_char: char,
    mode: EditorMode,
}

impl Document {
    /// Parse a raw file string into a Document.
    ///
    /// Lines are split on `'\n'`. A trailing newline is tracked separately so
    /// that `serialize()` can reconstruct the original byte-for-byte. The
    /// phantom empty string that `split('\n')` produces after a trailing `'\n'`
    /// is discarded from the `lines` Vec; `serialize()` always appends `'\n'`
    /// after every stored line, so a file that originally ended with `'\n'`
    /// will round-trip correctly.
    ///
    /// An empty file ("") produces a single empty `ContentLine::Content("")`.
    pub fn parse(raw: &str, comment_char: char, mode: EditorMode) -> Document {
        let mut lines: Vec<ContentLine> = Vec::new();
        let mut editable_index: Vec<usize> = Vec::new();

        // If the string ends with '\n', skip the final empty token that
        // split() would produce, because serialize() re-emits '\n' after
        // every line anyway.
        let raw_lines: Vec<&str> = raw.split('\n').collect();
        let slice = if raw.ends_with('\n') && raw_lines.len() > 1 {
            &raw_lines[..raw_lines.len() - 1]
        } else {
            &raw_lines[..]
        };

        for line in slice {
            let classified = classify_line(line, comment_char);
            let idx = lines.len();
            let include = match &classified {
                ContentLine::Content(_) => true,
                ContentLine::Comment(_) => matches!(mode, EditorMode::Plain),
                ContentLine::ConflictMarker(_) => matches!(mode, EditorMode::Plain | EditorMode::Merge),
            };
            if include {
                editable_index.push(idx);
            }
            lines.push(classified);
        }

        Document {
            lines,
            editable_index,
            comment_char,
            mode,
        }
    }

    /// Return only the editable lines as plain strings, respecting the EditorMode
    /// set at parse time. Drives off `editable_index` which was built mode-awarerly.
    pub fn editable_lines(&self) -> Vec<String> {
        self.editable_index
            .iter()
            .map(|&i| match &self.lines[i] {
                ContentLine::Content(s)
                | ContentLine::Comment(s)
                | ContentLine::ConflictMarker(s) => s.clone(),
            })
            .collect()
    }

    /// Return the EditorMode this document was parsed with.
    pub fn mode(&self) -> EditorMode {
        self.mode
    }

    /// Map a TextArea row index to the full `lines` Vec index.
    pub fn full_row_for_editable(&self, editable_row: usize) -> usize {
        self.editable_index[editable_row]
    }

    /// Number of editable (Content) lines.
    pub fn editable_count(&self) -> usize {
        self.editable_index.len()
    }

    /// All lines in original order.
    pub fn lines(&self) -> &Vec<ContentLine> {
        &self.lines
    }

    /// The comment character used during parsing.
    pub fn comment_char(&self) -> char {
        self.comment_char
    }

    /// Serialize back to a raw string, replacing editable lines with the
    /// content from `textarea_lines` while preserving all Comment and
    /// ConflictMarker lines byte-for-byte in original order.
    ///
    /// CRITICAL: preserves comment bytes verbatim (CTX-06).
    pub fn serialize(&self, textarea_lines: &[String]) -> String {
        let mut output = String::new();
        let mut content_idx: usize = 0;

        for line in &self.lines {
            match line {
                ContentLine::Content(_) => {
                    output.push_str(&textarea_lines[content_idx]);
                    content_idx += 1;
                }
                ContentLine::Comment(s) | ContentLine::ConflictMarker(s) => {
                    output.push_str(s);
                }
            }
            output.push('\n');
        }

        output
    }

    /// Replace the content of an editable line in-place.
    pub fn update_content_line(&mut self, editable_row: usize, new_content: String) {
        let full_idx = self.editable_index[editable_row];
        self.lines[full_idx] = ContentLine::Content(new_content);
    }

    /// Return the first editable (Content) line as a String.
    /// Returns an empty string if no editable lines exist.
    pub fn first_line(&self) -> String {
        self.editable_lines().first().cloned().unwrap_or_default()
    }

    /// Detect whether a blank line exists between the subject line and the body.
    ///
    /// Returns `true` if:
    /// - fewer than 2 editable lines exist (single-line messages don't require a separator), or
    /// - the second editable line is blank (trims to empty string).
    ///
    /// Returns `false` when two or more editable lines exist and the second
    /// line has visible content (i.e. no blank separator).
    pub fn has_blank_line_after_subject(&self) -> bool {
        let editable = self.editable_lines();
        if editable.len() < 2 {
            return true;
        }
        editable[1].trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // RebaseAction and RebaseLine tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_action_long_forms() {
        assert_eq!(parse_action("pick"), Some(RebaseAction::Pick));
        assert_eq!(parse_action("squash"), Some(RebaseAction::Squash));
        assert_eq!(parse_action("fixup"), Some(RebaseAction::Fixup));
        assert_eq!(parse_action("drop"), Some(RebaseAction::Drop));
        assert_eq!(parse_action("exec"), Some(RebaseAction::Exec));
    }

    #[test]
    fn test_parse_action_short_forms() {
        assert_eq!(parse_action("p"), Some(RebaseAction::Pick));
        assert_eq!(parse_action("s"), Some(RebaseAction::Squash));
        assert_eq!(parse_action("f"), Some(RebaseAction::Fixup));
        assert_eq!(parse_action("d"), Some(RebaseAction::Drop));
        assert_eq!(parse_action("x"), Some(RebaseAction::Exec));
    }

    #[test]
    fn test_parse_action_invalid() {
        assert_eq!(parse_action("invalid"), None);
        assert_eq!(parse_action(""), None);
    }

    #[test]
    fn test_classify_rebase_content_line() {
        let result = classify_rebase_line("pick abc1234 Fix bug", '#');
        assert_eq!(
            result,
            RebaseLine::Action {
                action: RebaseAction::Pick,
                hash: "abc1234".to_string(),
                subject: "Fix bug".to_string(),
            }
        );
    }

    #[test]
    fn test_classify_rebase_comment() {
        let result = classify_rebase_line("# Rebase abc..def onto ghi", '#');
        assert_eq!(
            result,
            RebaseLine::Comment("# Rebase abc..def onto ghi".to_string())
        );
    }

    #[test]
    fn test_classify_rebase_abbreviated() {
        let result = classify_rebase_line("p abc1234 Fix bug", '#');
        assert_eq!(
            result,
            RebaseLine::Action {
                action: RebaseAction::Pick,
                hash: "abc1234".to_string(),
                subject: "Fix bug".to_string(),
            }
        );
    }

    #[test]
    fn test_classify_rebase_malformed() {
        let result = classify_rebase_line("garbage line", '#');
        assert_eq!(result, RebaseLine::Comment("garbage line".to_string()));
    }

    #[test]
    fn test_classify_rebase_subject_with_spaces() {
        let result = classify_rebase_line("pick abc1234 Fix the authentication bug", '#');
        assert_eq!(
            result,
            RebaseLine::Action {
                action: RebaseAction::Pick,
                hash: "abc1234".to_string(),
                subject: "Fix the authentication bug".to_string(),
            }
        );
    }

    #[test]
    fn test_action_cycle_pick() {
        assert_eq!(RebaseAction::Pick.cycle(), RebaseAction::Squash);
    }

    #[test]
    fn test_action_cycle_squash() {
        assert_eq!(RebaseAction::Squash.cycle(), RebaseAction::Fixup);
    }

    #[test]
    fn test_action_cycle_fixup() {
        assert_eq!(RebaseAction::Fixup.cycle(), RebaseAction::Drop);
    }

    #[test]
    fn test_action_cycle_drop() {
        assert_eq!(RebaseAction::Drop.cycle(), RebaseAction::Pick);
    }

    #[test]
    fn test_action_cycle_exec() {
        assert_eq!(RebaseAction::Exec.cycle(), RebaseAction::Exec);
    }

    #[test]
    fn test_action_as_str() {
        assert_eq!(RebaseAction::Pick.as_str(), "pick");
        assert_eq!(RebaseAction::Squash.as_str(), "squash");
        assert_eq!(RebaseAction::Fixup.as_str(), "fixup");
        assert_eq!(RebaseAction::Drop.as_str(), "drop");
        assert_eq!(RebaseAction::Exec.as_str(), "exec");
    }

    #[test]
    fn test_parse_rebase_todo() {
        let input = "pick abc1234 Fix bug\nsquash def5678 Add test\n# comment\n";
        let lines = parse_rebase_todo(input, '#');
        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines[0],
            RebaseLine::Action {
                action: RebaseAction::Pick,
                hash: "abc1234".to_string(),
                subject: "Fix bug".to_string(),
            }
        );
        assert_eq!(
            lines[1],
            RebaseLine::Action {
                action: RebaseAction::Squash,
                hash: "def5678".to_string(),
                subject: "Add test".to_string(),
            }
        );
        assert_eq!(lines[2], RebaseLine::Comment("# comment".to_string()));
    }

    #[test]
    fn test_serialize_rebase_todo() {
        let lines = vec![
            RebaseLine::Action {
                action: RebaseAction::Pick,
                hash: "abc1234".to_string(),
                subject: "Fix bug".to_string(),
            },
            RebaseLine::Action {
                action: RebaseAction::Squash,
                hash: "def5678".to_string(),
                subject: "Add test".to_string(),
            },
            RebaseLine::Comment("# comment".to_string()),
        ];
        let result = serialize_rebase_todo(&lines);
        assert_eq!(result, "pick abc1234 Fix bug\nsquash def5678 Add test\n# comment\n");
    }

    #[test]
    fn test_roundtrip_rebase_todo() {
        let input = "pick abc1234 Fix bug\nsquash def5678 Add test\n# comment\n";
        let lines = parse_rebase_todo(input, '#');
        let output = serialize_rebase_todo(&lines);
        assert_eq!(input, output);
    }

    #[test]
    fn test_roundtrip_with_abbreviated_actions() {
        let input = "p abc1234 Fix bug\n";
        let lines = parse_rebase_todo(input, '#');
        let output = serialize_rebase_todo(&lines);
        assert_eq!(output, "pick abc1234 Fix bug\n");
    }

    #[test]
    fn test_empty_rebase_todo() {
        let lines = parse_rebase_todo("", '#');
        assert!(lines.is_empty());
    }

    #[test]
    fn test_exec_line_parsing() {
        let result = classify_rebase_line("exec make test", '#');
        assert_eq!(
            result,
            RebaseLine::Action {
                action: RebaseAction::Exec,
                hash: "".to_string(),
                subject: "make test".to_string(),
            }
        );
    }

    // -------------------------------------------------------------------------
    // read_comment_char
    // -------------------------------------------------------------------------

    #[test]
    fn test_read_comment_char_default() {
        // We cannot mock `git config` in unit tests, so we verify that the
        // function does not panic and returns a char.  The default '#' is
        // expected in most CI environments where core.commentchar is unset.
        let ch = read_comment_char();
        // Must be a valid char; common outcomes are '#' or whatever the
        // developer has configured.  We at least know it won't panic.
        let _ = ch; // non-panicking return is sufficient
    }

    // -------------------------------------------------------------------------
    // classify_line
    // -------------------------------------------------------------------------

    #[test]
    fn test_classify_comment_line() {
        assert_eq!(
            classify_line("# Please enter the commit message", '#'),
            ContentLine::Comment("# Please enter the commit message".to_string())
        );
    }

    #[test]
    fn test_classify_content_line() {
        assert_eq!(
            classify_line("Fix the authentication bug", '#'),
            ContentLine::Content("Fix the authentication bug".to_string())
        );
    }

    #[test]
    fn test_classify_conflict_marker_left() {
        assert_eq!(
            classify_line("<<<<<<", '#'),
            ContentLine::ConflictMarker("<<<<<<".to_string())
        );
    }

    #[test]
    fn test_classify_conflict_marker_separator() {
        assert_eq!(
            classify_line("======", '#'),
            ContentLine::ConflictMarker("======".to_string())
        );
    }

    #[test]
    fn test_classify_conflict_marker_right() {
        assert_eq!(
            classify_line(">>>>>>", '#'),
            ContentLine::ConflictMarker(">>>>>>".to_string())
        );
    }

    #[test]
    fn test_classify_seven_char_markers() {
        assert_eq!(
            classify_line("<<<<<<< HEAD", '#'),
            ContentLine::ConflictMarker("<<<<<<< HEAD".to_string())
        );
        assert_eq!(
            classify_line("======= separator", '#'),
            ContentLine::ConflictMarker("======= separator".to_string())
        );
        assert_eq!(
            classify_line(">>>>>>> branch-name", '#'),
            ContentLine::ConflictMarker(">>>>>>> branch-name".to_string())
        );
    }

    // -------------------------------------------------------------------------
    // Document::parse
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_mixed_content() {
        let input = "Fix the bug\n# Please enter the commit message\n# Lines starting with '#' will be ignored\nSigned-off-by: Dev\n";
        let doc = Document::parse(input, '#', EditorMode::Squash);

        // Trailing '\n' is consumed — 4 lines stored, not 5.
        // Lines: Content, Comment, Comment, Content
        assert_eq!(doc.lines().len(), 4);
        assert_eq!(doc.lines()[0], ContentLine::Content("Fix the bug".to_string()));
        assert_eq!(
            doc.lines()[1],
            ContentLine::Comment("# Please enter the commit message".to_string())
        );
        assert_eq!(
            doc.lines()[2],
            ContentLine::Comment("# Lines starting with '#' will be ignored".to_string())
        );
        assert_eq!(
            doc.lines()[3],
            ContentLine::Content("Signed-off-by: Dev".to_string())
        );

        // editable_lines() returns 2: "Fix the bug", "Signed-off-by: Dev"
        assert_eq!(doc.editable_lines().len(), 2);
    }

    // -------------------------------------------------------------------------
    // editable_lines
    // -------------------------------------------------------------------------

    #[test]
    fn test_editable_lines_excludes_comments() {
        // 3 content lines, 2 comment lines; trailing '\n' is consumed
        let input = "line1\n# comment1\nline2\n# comment2\nline3\n";
        let doc = Document::parse(input, '#', EditorMode::Squash);
        let editable = doc.editable_lines();
        assert_eq!(editable.len(), 3); // "line1", "line2", "line3"
        assert_eq!(editable[0], "line1");
        assert_eq!(editable[1], "line2");
        assert_eq!(editable[2], "line3");
    }

    #[test]
    fn test_editable_lines_excludes_conflict_markers() {
        let input = "<<<<<<< HEAD\nour change\n=======\ntheir change\n>>>>>>> branch\n";
        let doc = Document::parse(input, '#', EditorMode::Squash);
        let editable = doc.editable_lines();
        // "our change", "their change" (trailing '\n' consumed)
        assert_eq!(editable.len(), 2);
        assert_eq!(editable[0], "our change");
        assert_eq!(editable[1], "their change");
    }

    // -------------------------------------------------------------------------
    // editable_index / full_row_for_editable
    // -------------------------------------------------------------------------

    #[test]
    fn test_editable_index_maps_correctly() {
        // Lines: Comment(idx=0), Content(idx=1), Comment(idx=2), Content(idx=3)
        let input = "# comment\nfirst content\n# another comment\nsecond content\n";
        let doc = Document::parse(input, '#', EditorMode::Squash);
        // editable row 0 -> lines index 1
        assert_eq!(doc.full_row_for_editable(0), 1);
        // editable row 1 -> lines index 3
        assert_eq!(doc.full_row_for_editable(1), 3);
    }

    // -------------------------------------------------------------------------
    // Roundtrip serialization
    // -------------------------------------------------------------------------

    #[test]
    fn test_roundtrip_preserves_comments() {
        let input = "subject line\n\n# comment line\nbody text\n";
        let doc = Document::parse(input, '#', EditorMode::Squash);
        let editable = doc.editable_lines();
        let output = doc.serialize(&editable);
        assert_eq!(input, output);
    }

    #[test]
    fn test_roundtrip_preserves_conflict_markers() {
        let input = "<<<<<<< HEAD\nour change\n=======\ntheir change\n>>>>>>> branch\n";
        let doc = Document::parse(input, '#', EditorMode::Squash);
        let editable = doc.editable_lines();
        let output = doc.serialize(&editable);
        assert_eq!(input, output);
    }

    #[test]
    fn test_serialize_with_edited_content() {
        let input = "old subject\n# comment\n";
        let doc = Document::parse(input, '#', EditorMode::Squash);
        let mut edited = doc.editable_lines();
        edited[0] = "new subject".to_string();
        let output = doc.serialize(&edited);
        assert_eq!(output, "new subject\n# comment\n");
    }

    // -------------------------------------------------------------------------
    // Edge cases
    // -------------------------------------------------------------------------

    #[test]
    fn test_empty_file() {
        let doc = Document::parse("", '#', EditorMode::Squash);
        // split("") on '\n' -> [""] -> one empty Content line
        assert_eq!(doc.lines().len(), 1);
        assert_eq!(doc.lines()[0], ContentLine::Content("".to_string()));
        assert_eq!(doc.editable_lines().len(), 1);
    }

    #[test]
    fn test_comment_only_file() {
        let input = "# comment 1\n# comment 2\n";
        let doc = Document::parse(input, '#', EditorMode::Squash);
        // Lines: Comment, Comment (trailing '\n' consumed) — no editable lines
        assert_eq!(doc.editable_lines().len(), 0);
    }

    // -------------------------------------------------------------------------
    // first_line
    // -------------------------------------------------------------------------

    #[test]
    fn test_first_line_returns_subject() {
        let doc = Document::parse("Subject\n\nBody\n", '#', EditorMode::Squash);
        assert_eq!(doc.first_line(), "Subject");
    }

    #[test]
    fn test_first_line_empty_when_no_editable() {
        let doc = Document::parse("# only comment\n", '#', EditorMode::Squash);
        assert_eq!(doc.first_line(), "");
    }

    // -------------------------------------------------------------------------
    // has_blank_line_after_subject
    // -------------------------------------------------------------------------

    #[test]
    fn test_has_blank_line_true_when_blank_present() {
        let doc = Document::parse("Subject\n\nBody\n", '#', EditorMode::Squash);
        assert!(doc.has_blank_line_after_subject());
    }

    #[test]
    fn test_has_blank_line_false_when_missing() {
        let doc = Document::parse("Subject\nBody\n", '#', EditorMode::Squash);
        assert!(!doc.has_blank_line_after_subject());
    }

    #[test]
    fn test_has_blank_line_true_single_line() {
        let doc = Document::parse("Subject\n", '#', EditorMode::Squash);
        assert!(doc.has_blank_line_after_subject());
    }

    // -------------------------------------------------------------------------
    // detect_squash_header
    // -------------------------------------------------------------------------

    #[test]
    fn test_detect_squash_header_standard() {
        let input = "# This is a combination of 2 commits.\n# This is the 1st commit message:\n# First commit\n#\n# This is the commit message #2:\n# Second commit\n\nCombined message\n";
        // The blank line before "Combined message" is the first non-comment line after the squash header
        let result = detect_squash_header(input, '#');
        assert!(result.is_some());
        let header_end = result.unwrap();
        // header_end points to the blank line (first non-comment line after header)
        let lines: Vec<&str> = input.split('\n').collect();
        // The line at header_end should be the empty line before "Combined message"
        assert!(lines[header_end].is_empty() || !lines[header_end].starts_with('#'));
    }

    #[test]
    fn test_detect_squash_header_none() {
        let input = "Just a normal commit\n";
        let result = detect_squash_header(input, '#');
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_squash_header_single_commit() {
        let input = "# This is a combination of 1 commits.\n# This is the 1st commit message:\n# Original\nEditable\n";
        let result = detect_squash_header(input, '#');
        assert!(result.is_some());
        let header_end = result.unwrap();
        // header_end should point to "Editable"
        let lines: Vec<&str> = input.split('\n').collect();
        assert_eq!(lines[header_end], "Editable");
    }

    #[test]
    fn test_detect_squash_header_all_comments() {
        let input = "# This is a combination of 2 commits.\n# All comments\n";
        let result = detect_squash_header(input, '#');
        assert!(result.is_some());
        let header_end = result.unwrap();
        // Entire file is header: header_end == number of lines
        let lines: Vec<&str> = input.split('\n').collect();
        // Trailing newline causes an extra empty token; strip it
        let effective_len = if input.ends_with('\n') && lines.len() > 1 { lines.len() - 1 } else { lines.len() };
        assert_eq!(header_end, effective_len);
    }

    #[test]
    fn test_detect_squash_header_custom_comment_char() {
        let input = "; This is a combination of 2 commits.\n; First commit\n\nEditable\n";
        let result = detect_squash_header(input, ';');
        assert!(result.is_some());
        let header_end = result.unwrap();
        let lines: Vec<&str> = input.split('\n').collect();
        assert!(lines[header_end].is_empty());
    }

    // -------------------------------------------------------------------------
    // EditorMode tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_editor_mode_plain_makes_comments_editable() {
        let input = "subject\n# comment\nbody\n";
        let doc = Document::parse(input, '#', EditorMode::Plain);
        assert_eq!(doc.editable_lines(), vec!["subject", "# comment", "body"]);
    }

    #[test]
    fn test_editor_mode_plain_makes_conflict_markers_editable() {
        let input = "a\n<<<<<<< HEAD\nb\n=======\nc\n>>>>>>> br\nd\n";
        let doc = Document::parse(input, '#', EditorMode::Plain);
        let editable = doc.editable_lines();
        assert_eq!(editable.len(), 7);
        assert!(editable.contains(&"<<<<<<< HEAD".to_string()));
        assert!(editable.contains(&"=======".to_string()));
        assert!(editable.contains(&">>>>>>> br".to_string()));
    }

    #[test]
    fn test_editor_mode_merge_protects_comments_but_promotes_conflict_markers() {
        let input = "subject\n# comment\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> br\n";
        let doc = Document::parse(input, '#', EditorMode::Merge);
        assert_eq!(
            doc.editable_lines(),
            vec!["subject", "<<<<<<< HEAD", "ours", "=======", "theirs", ">>>>>>> br"]
        );
    }

    #[test]
    fn test_editor_mode_squash_unchanged() {
        let input = "subject\n# comment\n<<<<<<<\nbody\n";
        let doc = Document::parse(input, '#', EditorMode::Squash);
        assert_eq!(doc.editable_lines(), vec!["subject", "body"]);
    }
}
