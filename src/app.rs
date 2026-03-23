use crate::context::GitContext;
use crate::document::{Document, RebaseLine, detect_squash_header, parse_rebase_todo, read_comment_char, serialize_rebase_todo};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use ratatui_textarea::TextArea;

/// Actions that can be applied to the App state machine.
#[derive(Debug)]
pub enum Action {
    Save,
    Cancel,
    Noop,
    Help,               // triggered by Ctrl+H
    DismissHelp,        // triggered by Esc while help visible
    CycleRebaseAction,  // Tab key in rebase mode
    MoveRebaseDown,     // Down arrow in rebase mode
    MoveRebaseUp,       // Up arrow in rebase mode
}

/// Outcomes returned by `App::apply()`.
#[derive(Debug, PartialEq)]
pub enum Outcome {
    Save,
    Cancel,
    Continue,
}

/// Application state: holds parsed Document, TextArea for editing, and detected git context.
pub struct App {
    document: Document,
    textarea: TextArea<'static>,
    context: GitContext,
    show_help: bool,                  // tracks if help overlay is visible
    rebase_lines: Vec<RebaseLine>,    // populated only in Rebase context
    selected_rebase_idx: usize,       // index into selectable_indices
    selectable_indices: Vec<usize>,   // indices of Action (non-Comment) lines in rebase_lines
    squash_log: Vec<String>,          // raw comment lines forming the squash header (empty if not squash mode)
}

impl App {
    pub fn new(raw_content: &str, context: GitContext) -> Self {
        let comment_char = read_comment_char();

        // For Squash mode, split raw_content into the read-only squash log header
        // and the editable message portion. The squash log is stored separately and
        // rendered read-only; only the message portion is parsed by Document.
        let (squash_log, content_for_document) = if matches!(context, GitContext::Squash) {
            if let Some(header_end) = detect_squash_header(raw_content, comment_char) {
                let raw_lines: Vec<&str> = raw_content.split('\n').collect();
                let effective_len = if raw_content.ends_with('\n') && raw_lines.len() > 1 {
                    raw_lines.len() - 1
                } else {
                    raw_lines.len()
                };
                let header_lines: Vec<String> = raw_lines[..header_end.min(effective_len)]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                // Reconstruct the message portion with trailing newline if original had one.
                let message_lines = &raw_lines[header_end.min(effective_len)..];
                let message_str = if !message_lines.is_empty() {
                    let mut s = message_lines.join("\n");
                    if raw_content.ends_with('\n') {
                        s.push('\n');
                    }
                    s
                } else {
                    String::new()
                };
                (header_lines, message_str)
            } else {
                (Vec::new(), raw_content.to_string())
            }
        } else {
            (Vec::new(), raw_content.to_string())
        };

        let document = Document::parse(&content_for_document, comment_char);
        let editable = document.editable_lines();
        let mut textarea = TextArea::new(editable);

        textarea.set_cursor_line_style(Style::default());
        textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
        textarea.set_block(Block::default().borders(Borders::ALL));

        let (rebase_lines, selectable_indices) = if matches!(context, GitContext::Rebase) {
            let lines = parse_rebase_todo(raw_content, comment_char);
            let selectable: Vec<usize> = lines
                .iter()
                .enumerate()
                .filter_map(|(i, line)| {
                    if matches!(line, RebaseLine::Action { .. }) {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();
            (lines, selectable)
        } else {
            (Vec::new(), Vec::new())
        };

        Self {
            document,
            textarea,
            context,
            show_help: false,
            rebase_lines,
            selected_rebase_idx: 0,
            selectable_indices,
            squash_log,
        }
    }

    /// Apply an action and return the resulting outcome.
    pub fn apply(&mut self, action: Action) -> Outcome {
        match action {
            Action::Save => Outcome::Save,
            Action::Cancel => Outcome::Cancel,
            Action::Noop => Outcome::Continue,
            Action::Help => {
                self.show_help = true;
                Outcome::Continue
            }
            Action::DismissHelp => {
                self.show_help = false;
                Outcome::Continue
            }
            Action::CycleRebaseAction => {
                if let Some(&line_idx) = self.selectable_indices.get(self.selected_rebase_idx) {
                    if let RebaseLine::Action { action, .. } = &mut self.rebase_lines[line_idx] {
                        *action = action.cycle();
                    }
                }
                Outcome::Continue
            }
            Action::MoveRebaseDown => {
                if !self.selectable_indices.is_empty() {
                    let max = self.selectable_indices.len() - 1;
                    if self.selected_rebase_idx < max {
                        self.selected_rebase_idx += 1;
                    }
                }
                Outcome::Continue
            }
            Action::MoveRebaseUp => {
                if self.selected_rebase_idx > 0 {
                    self.selected_rebase_idx -= 1;
                }
                Outcome::Continue
            }
        }
    }

    /// Return whether the help overlay is currently visible.
    pub fn is_help_visible(&self) -> bool {
        self.show_help
    }

    /// Return the parsed Document.
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Return an immutable reference to the TextArea.
    pub fn textarea(&self) -> &TextArea {
        &self.textarea
    }

    /// Return a mutable reference to the TextArea (for passing key events).
    pub fn textarea_mut(&mut self) -> &mut TextArea<'static> {
        &mut self.textarea
    }

    /// Return the detected git context.
    pub fn context(&self) -> &GitContext {
        &self.context
    }

    /// Return the rebase lines (only populated in Rebase context).
    pub fn rebase_lines(&self) -> &[RebaseLine] {
        &self.rebase_lines
    }

    /// Return the current selection index into selectable_indices.
    pub fn selected_rebase_idx(&self) -> usize {
        self.selected_rebase_idx
    }

    /// Return the actual index into rebase_lines for the currently selected line.
    pub fn selected_rebase_line_idx(&self) -> Option<usize> {
        self.selectable_indices.get(self.selected_rebase_idx).copied()
    }

    /// Return the selectable indices (indices of Action lines in rebase_lines).
    pub fn selectable_indices(&self) -> &[usize] {
        &self.selectable_indices
    }

    /// Return the squash log lines (read-only header in squash mode).
    /// Empty slice if not in squash mode or no squash header was detected.
    pub fn squash_log(&self) -> &[String] {
        &self.squash_log
    }

    /// Get serialized content for saving.
    /// In Rebase context, delegates to serialize_rebase_todo().
    /// In Squash context with a non-empty squash_log, prepends the log lines before Document serialization.
    /// Otherwise, merges textarea edits back into Document.
    pub fn serialized_content(&self) -> String {
        if matches!(self.context, GitContext::Rebase) {
            serialize_rebase_todo(&self.rebase_lines)
        } else if matches!(self.context, GitContext::Squash) && !self.squash_log.is_empty() {
            // Reconstruct complete SQUASH_MSG: squash log header + edited message
            let mut output = String::new();
            for line in &self.squash_log {
                output.push_str(line);
                output.push('\n');
            }
            output.push_str(&self.document.serialize(self.textarea.lines()));
            output
        } else {
            self.document.serialize(self.textarea.lines())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::RebaseAction;

    fn make_app() -> App {
        App::new("hello\n", GitContext::Commit)
    }

    const REBASE_CONTENT: &str =
        "pick abc1234 Fix bug\npick def5678 Add test\n# Rebase instructions\ndrop ghi9012 Remove old\n";

    #[test]
    fn apply_save_returns_save() {
        let mut app = make_app();
        assert_eq!(app.apply(Action::Save), Outcome::Save);
    }

    #[test]
    fn apply_cancel_returns_cancel() {
        let mut app = make_app();
        assert_eq!(app.apply(Action::Cancel), Outcome::Cancel);
    }

    #[test]
    fn apply_noop_returns_continue() {
        let mut app = make_app();
        assert_eq!(app.apply(Action::Noop), Outcome::Continue);
    }

    #[test]
    fn serialized_content_returns_initial_content() {
        let app = make_app();
        assert_eq!(app.serialized_content(), "hello\n");
    }

    #[test]
    fn context_returns_detected_context() {
        let app = make_app();
        assert_eq!(*app.context(), GitContext::Commit);
    }

    // -------------------------------------------------------------------------
    // Rebase mode tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_app_rebase_mode_parses_lines() {
        let app = App::new(REBASE_CONTENT, GitContext::Rebase);
        let lines = app.rebase_lines();
        // 4 lines: 3 Action (pick, pick, drop), 1 Comment
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn test_app_rebase_cycle_action() {
        let mut app = App::new(REBASE_CONTENT, GitContext::Rebase);
        // selected_rebase_idx starts at 0 -> first action line (pick abc1234)
        app.apply(Action::CycleRebaseAction);
        // pick -> squash
        let lines = app.rebase_lines();
        if let crate::document::RebaseLine::Action { action, .. } = &lines[0] {
            assert_eq!(*action, RebaseAction::Squash);
        } else {
            panic!("Expected Action line at index 0");
        }
    }

    #[test]
    fn test_app_rebase_cycle_on_comment_noop() {
        // Build content with only comment lines
        let comment_only = "# comment line\n# another\n";
        let mut app = App::new(comment_only, GitContext::Rebase);
        // No selectable lines; CycleRebaseAction should be a no-op
        let result = app.apply(Action::CycleRebaseAction);
        assert_eq!(result, Outcome::Continue);
    }

    #[test]
    fn test_app_rebase_cycle_on_exec_noop() {
        let exec_content = "exec make test\n";
        let mut app = App::new(exec_content, GitContext::Rebase);
        app.apply(Action::CycleRebaseAction);
        // exec stays exec
        if let crate::document::RebaseLine::Action { action, .. } = &app.rebase_lines()[0] {
            assert_eq!(*action, RebaseAction::Exec);
        }
    }

    #[test]
    fn test_app_rebase_selected_idx_default() {
        let app = App::new(REBASE_CONTENT, GitContext::Rebase);
        assert_eq!(app.selected_rebase_idx(), 0);
    }

    #[test]
    fn test_app_rebase_move_down() {
        let mut app = App::new(REBASE_CONTENT, GitContext::Rebase);
        app.apply(Action::MoveRebaseDown);
        // Should advance to next selectable line (pick def5678)
        assert_eq!(app.selected_rebase_idx(), 1);
    }

    #[test]
    fn test_app_rebase_move_up() {
        let mut app = App::new(REBASE_CONTENT, GitContext::Rebase);
        app.apply(Action::MoveRebaseDown);
        app.apply(Action::MoveRebaseUp);
        assert_eq!(app.selected_rebase_idx(), 0);
    }

    #[test]
    fn test_app_rebase_move_clamps() {
        let mut app = App::new(REBASE_CONTENT, GitContext::Rebase);
        // Move up at 0 should stay at 0
        app.apply(Action::MoveRebaseUp);
        assert_eq!(app.selected_rebase_idx(), 0);

        // Move to end and try to go further
        app.apply(Action::MoveRebaseDown);
        app.apply(Action::MoveRebaseDown);
        app.apply(Action::MoveRebaseDown);
        // 3 selectable lines (pick, pick, drop); max index = 2
        let max = app.selected_rebase_idx();
        app.apply(Action::MoveRebaseDown);
        assert_eq!(app.selected_rebase_idx(), max);
    }

    #[test]
    fn test_app_rebase_serialized_content() {
        let app = App::new(REBASE_CONTENT, GitContext::Rebase);
        let content = app.serialized_content();
        // Should reconstruct the original (all long-form actions)
        assert_eq!(content, REBASE_CONTENT);
    }

    #[test]
    fn test_app_commit_mode_no_rebase_lines() {
        let app = App::new("commit message\n", GitContext::Commit);
        assert!(app.rebase_lines().is_empty());
    }

    // -------------------------------------------------------------------------
    // Squash mode tests
    // -------------------------------------------------------------------------

    const SQUASH_CONTENT: &str = "# This is a combination of 2 commits.\n# This is the 1st commit message:\n# First commit\n#\n# This is the commit message #2:\n# Second commit\n\nCombined message\n";

    #[test]
    fn test_app_squash_mode_has_log() {
        let app = App::new(SQUASH_CONTENT, GitContext::Squash);
        let log = app.squash_log();
        // The squash_log should contain the comment-only header lines
        assert!(!log.is_empty());
        // First line should start with the squash marker
        assert!(log[0].contains("This is a combination of"));
    }

    #[test]
    fn test_app_squash_mode_editable() {
        let app = App::new(SQUASH_CONTENT, GitContext::Squash);
        // TextArea should only contain the editable portion (blank separator + "Combined message")
        // The blank line separating the header from the message is preserved as part of the message.
        let lines = app.textarea().lines();
        // Last non-empty line must be "Combined message"
        let last_content: Vec<&str> = lines.iter().filter(|l| !l.is_empty()).map(|s| s.as_str()).collect();
        assert!(!last_content.is_empty());
        assert_eq!(*last_content.last().unwrap(), "Combined message");
        // The header comment lines should NOT appear in textarea
        for line in lines.iter() {
            assert!(!line.contains("This is a combination of"), "Header line found in textarea: {:?}", line);
        }
    }

    #[test]
    fn test_app_squash_serialized_content() {
        let app = App::new(SQUASH_CONTENT, GitContext::Squash);
        let serialized = app.serialized_content();
        // Should include both the squash log and the editable message
        assert!(serialized.contains("# This is a combination of 2 commits."));
        assert!(serialized.contains("Combined message"));
    }

    #[test]
    fn test_app_squash_no_header_falls_back_to_normal() {
        // A SQUASH_MSG with no squash header should work like a normal commit message
        let content = "Regular commit message\n";
        let app = App::new(content, GitContext::Squash);
        let log = app.squash_log();
        assert!(log.is_empty());
        // Textarea should have the full content
        let lines = app.textarea().lines();
        assert_eq!(lines[0], "Regular commit message");
    }
}
