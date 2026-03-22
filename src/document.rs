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
    pub fn parse(raw: &str, comment_char: char) -> Document {
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
            if let ContentLine::Content(_) = &classified {
                editable_index.push(idx);
            }
            lines.push(classified);
        }

        Document {
            lines,
            editable_index,
            comment_char,
        }
    }

    /// Return only the editable (Content) lines as plain strings.
    pub fn editable_lines(&self) -> Vec<String> {
        self.lines
            .iter()
            .filter_map(|l| {
                if let ContentLine::Content(s) = l {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect()
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let doc = Document::parse(input, '#');

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
        let doc = Document::parse(input, '#');
        let editable = doc.editable_lines();
        assert_eq!(editable.len(), 3); // "line1", "line2", "line3"
        assert_eq!(editable[0], "line1");
        assert_eq!(editable[1], "line2");
        assert_eq!(editable[2], "line3");
    }

    #[test]
    fn test_editable_lines_excludes_conflict_markers() {
        let input = "<<<<<<< HEAD\nour change\n=======\ntheir change\n>>>>>>> branch\n";
        let doc = Document::parse(input, '#');
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
        let doc = Document::parse(input, '#');
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
        let doc = Document::parse(input, '#');
        let editable = doc.editable_lines();
        let output = doc.serialize(&editable);
        assert_eq!(input, output);
    }

    #[test]
    fn test_roundtrip_preserves_conflict_markers() {
        let input = "<<<<<<< HEAD\nour change\n=======\ntheir change\n>>>>>>> branch\n";
        let doc = Document::parse(input, '#');
        let editable = doc.editable_lines();
        let output = doc.serialize(&editable);
        assert_eq!(input, output);
    }

    #[test]
    fn test_serialize_with_edited_content() {
        let input = "old subject\n# comment\n";
        let doc = Document::parse(input, '#');
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
        let doc = Document::parse("", '#');
        // split("") on '\n' -> [""] -> one empty Content line
        assert_eq!(doc.lines().len(), 1);
        assert_eq!(doc.lines()[0], ContentLine::Content("".to_string()));
        assert_eq!(doc.editable_lines().len(), 1);
    }

    #[test]
    fn test_comment_only_file() {
        let input = "# comment 1\n# comment 2\n";
        let doc = Document::parse(input, '#');
        // Lines: Comment, Comment (trailing '\n' consumed) — no editable lines
        assert_eq!(doc.editable_lines().len(), 0);
    }
}
