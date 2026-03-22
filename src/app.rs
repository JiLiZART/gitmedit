use crate::context::GitContext;
use crate::document::{Document, read_comment_char};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use ratatui_textarea::TextArea;

/// Actions that can be applied to the App state machine.
#[derive(Debug)]
pub enum Action {
    Save,
    Cancel,
    Noop,
    Help,         // triggered by Ctrl+H
    DismissHelp,  // triggered by Esc while help visible
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
    show_help: bool,  // tracks if help overlay is visible
}

impl App {
    pub fn new(raw_content: &str, context: GitContext) -> Self {
        let comment_char = read_comment_char();
        let document = Document::parse(raw_content, comment_char);
        let editable = document.editable_lines();
        let mut textarea = TextArea::new(editable);

        textarea.set_cursor_line_style(Style::default());
        textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
        textarea.set_block(Block::default().borders(Borders::ALL));

        Self { document, textarea, context, show_help: false }
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

    /// Get serialized content for saving: merges textarea edits back into Document.
    pub fn serialized_content(&self) -> String {
        self.document.serialize(self.textarea.lines())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app() -> App {
        App::new("hello\n", GitContext::Commit)
    }

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
}
