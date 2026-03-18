use crate::context::GitContext;

/// Actions that can be applied to the App state machine.
#[derive(Debug)]
pub enum Action {
    Save,
    Cancel,
    Noop,
}

/// Outcomes returned by `App::apply()`.
#[derive(Debug, PartialEq)]
pub enum Outcome {
    Save,
    Cancel,
    Continue,
}

/// Application state: holds file content and detected git context.
pub struct App {
    content: String,
    context: GitContext,
}

impl App {
    pub fn new(content: String, context: GitContext) -> Self {
        Self { content, context }
    }

    /// Apply an action and return the resulting outcome.
    pub fn apply(&mut self, action: Action) -> Outcome {
        match action {
            Action::Save => Outcome::Save,
            Action::Cancel => Outcome::Cancel,
            Action::Noop => Outcome::Continue,
        }
    }

    /// Return the current text content.
    pub fn content(&self) -> &str {
        self.content.as_str()
    }

    /// Return the detected git context.
    pub fn context(&self) -> &GitContext {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app() -> App {
        App::new("hello\n".to_string(), GitContext::Commit)
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
    fn content_returns_initial_content() {
        let app = make_app();
        assert_eq!(app.content(), "hello\n");
    }

    #[test]
    fn context_returns_detected_context() {
        let app = make_app();
        assert_eq!(*app.context(), GitContext::Commit);
    }
}
