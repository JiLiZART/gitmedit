use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ratatui_textarea::{CursorMove, Input, TextArea};

use crate::context::GitContext;
use crate::details::DetailsLoader;
use crate::layout::{self, Pane, PaneRects};
use crate::message::{self, MessageFile};
use crate::rebase::{self, Todo, TodoLine};
use crate::status::{self, Status};
use crate::{reword, wrap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollBy {
    Lines(isize),
    Pages(isize),
    Top,
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Save,
    Cancel,
    ToggleHelp,
    ScrollHelp(isize),
    FocusLeft,
    FocusRight,
    ToggleFocus,
    ClickAt { column: u16, row: u16 },
    WheelAt { column: u16, row: u16, delta: isize },
    ScrollRight(ScrollBy),
    // Text editing: message, plain, raw todo, or inline subject editor.
    Edit(Input),
    CursorUp,
    CursorDown,
    DeleteLine,
    Undo,
    Redo,
    DeleteWord,
    DeleteNextWord,
    Copy,
    Cut,
    Paste,
    // Rebase table.
    SelectBy(ScrollBy),
    SetInstruction(rebase::Action),
    CycleInstruction,
    MoveInstruction { up: bool },
    StartInline,
    CommitInline,
    CancelInline,
    ToggleRaw,
}

#[derive(Debug, PartialEq)]
pub enum Outcome {
    Save,
    Cancel,
    Continue,
}

pub struct MessageBody {
    pub file: MessageFile,
    pub editor: TextArea<'static>,
    pub status: Status,
    /// Stored reword message used to prefill this commit; deleted after a successful save.
    pub reword_file: Option<PathBuf>,
}

pub struct RebaseBody {
    pub todo: Todo,
    /// Index into `todo.instruction_indices()`.
    pub selected: usize,
    /// Pending rewords: hash as written in the todo → new subject.
    pub rewords: HashMap<String, String>,
    pub inline: Option<TextArea<'static>>,
    pub raw: Option<TextArea<'static>>,
    pub details: DetailsLoader,
}

pub enum Body {
    Message(MessageBody),
    Plain(TextArea<'static>),
    Rebase(Box<RebaseBody>),
}

pub struct App {
    pub body: Body,
    pub git_dir: Option<PathBuf>,
    pub comment_char: char,
    pub file_name: String,
    pub focus: Pane,
    pub show_help: bool,
    pub help_scroll: usize,
    pub right_scroll: usize,
    // Written by the renderer every frame.
    pub rects: PaneRects,
    pub editor_width: usize,
    pub right_page: usize,
    pub editor_top: usize,
    pub table_top: usize,
}

impl App {
    pub fn new(raw: &str, context: GitContext, git_dir: Option<PathBuf>, comment_char: char) -> Self {
        let body = if context == GitContext::Rebase {
            Body::Rebase(Box::new(RebaseBody::new(raw, git_dir.as_deref(), comment_char)))
        } else if context.is_message() {
            Body::Message(new_message_body(raw, context, git_dir.as_deref(), comment_char))
        } else {
            Body::Plain(TextArea::new(raw.lines().map(str::to_string).collect()))
        };
        Self {
            body,
            git_dir,
            comment_char,
            file_name: String::new(),
            focus: Pane::Left,
            show_help: false,
            help_scroll: 0,
            right_scroll: 0,
            rects: PaneRects::default(),
            editor_width: 80,
            right_page: 10,
            editor_top: 0,
            table_top: 0,
        }
    }

    pub fn has_right(&self) -> bool {
        match &self.body {
            Body::Message(m) => !m.status.is_empty(),
            Body::Plain(_) => false,
            Body::Rebase(_) => true,
        }
    }

    /// The text editor keystrokes go to, when the left pane is showing one.
    pub fn active_editor_mut(&mut self) -> Option<&mut TextArea<'static>> {
        match &mut self.body {
            Body::Message(m) => Some(&mut m.editor),
            Body::Plain(editor) => Some(editor),
            Body::Rebase(r) => r.inline.as_mut().or(r.raw.as_mut()),
        }
    }

    pub fn apply(&mut self, action: Action) -> Outcome {
        match action {
            Action::Save => return self.prepare_save(),
            Action::Cancel => return Outcome::Cancel,
            Action::ToggleHelp => {
                self.show_help = !self.show_help;
                self.help_scroll = 0;
            }
            Action::ScrollHelp(delta) => self.help_scroll = self.help_scroll.saturating_add_signed(delta),
            Action::FocusLeft => self.focus = Pane::Left,
            Action::FocusRight => {
                if self.has_right() {
                    self.focus = Pane::Right;
                }
            }
            Action::ToggleFocus => {
                self.focus = match self.focus {
                    Pane::Left if self.has_right() => Pane::Right,
                    _ => Pane::Left,
                };
            }
            Action::ClickAt { column, row } => {
                if let Some(pane) = layout::pane_at(&self.rects, column, row) {
                    self.focus = pane;
                }
            }
            Action::WheelAt { column, row, delta } => match layout::pane_at(&self.rects, column, row) {
                Some(Pane::Right) => self.scroll_right(ScrollBy::Lines(delta)),
                Some(Pane::Left) => self.wheel_left(delta),
                None => {}
            },
            Action::ScrollRight(by) => self.scroll_right(by),
            Action::CursorUp => self.move_cursor_row(false),
            Action::CursorDown => self.move_cursor_row(true),
            Action::Copy => self.copy_selection(false),
            Action::Cut => self.copy_selection(true),
            Action::Paste => self.paste(),
            Action::SelectBy(by) => {
                if let Body::Rebase(r) = &mut self.body {
                    r.move_selection(by);
                    self.right_scroll = 0;
                }
            }
            Action::SetInstruction(action) => {
                if let Body::Rebase(r) = &mut self.body {
                    r.set_action(action);
                }
            }
            Action::CycleInstruction => {
                if let Body::Rebase(r) = &mut self.body {
                    r.cycle_action();
                }
            }
            Action::MoveInstruction { up } => {
                if let Body::Rebase(r) = &mut self.body {
                    r.move_instruction(up);
                }
            }
            Action::StartInline => {
                if let Body::Rebase(r) = &mut self.body {
                    r.start_inline();
                }
            }
            Action::CommitInline => {
                if let Body::Rebase(r) = &mut self.body {
                    r.commit_inline();
                }
            }
            Action::CancelInline => {
                if let Body::Rebase(r) = &mut self.body {
                    r.inline = None;
                }
            }
            Action::ToggleRaw => {
                let cc = self.comment_char;
                if let Body::Rebase(r) = &mut self.body {
                    r.toggle_raw(cc);
                }
            }
            other => {
                if let Some(editor) = self.active_editor_mut() {
                    edit(editor, other);
                }
            }
        }
        Outcome::Continue
    }

    pub fn serialized_content(&self) -> String {
        match &self.body {
            Body::Message(m) => m.file.assemble(m.editor.lines()),
            Body::Plain(editor) => {
                let mut s = editor.lines().join("\n");
                s.push('\n');
                s
            }
            Body::Rebase(r) => r.todo.serialize(),
        }
    }

    /// Side effects that follow a successful write: store pending rewords, or consume the stored
    /// message this commit was prefilled from.
    pub fn finish_save(&self) -> anyhow::Result<()> {
        match &self.body {
            Body::Message(m) => {
                if let Some(path) = &m.reword_file {
                    let _ = std::fs::remove_file(path);
                }
            }
            Body::Rebase(r) => {
                if let Some(dir) = &self.git_dir {
                    reword::store(dir, &r.rewords, |hash| reword::git_full_message(dir, hash))?;
                }
            }
            Body::Plain(_) => {}
        }
        Ok(())
    }

    /// Pull in finished background work. Returns true when something new arrived.
    pub fn poll_background(&mut self) -> bool {
        match &mut self.body {
            Body::Rebase(r) => r.details.poll(),
            _ => false,
        }
    }

    fn prepare_save(&mut self) -> Outcome {
        let cc = self.comment_char;
        if let Body::Rebase(r) = &mut self.body {
            if r.inline.is_some() {
                r.commit_inline();
            }
            if r.raw.is_some() {
                r.toggle_raw(cc);
            }
        }
        Outcome::Save
    }

    fn scroll_right(&mut self, by: ScrollBy) {
        let page = self.right_page.max(1) as isize;
        self.right_scroll = match by {
            ScrollBy::Lines(n) => self.right_scroll.saturating_add_signed(n),
            ScrollBy::Pages(n) => self.right_scroll.saturating_add_signed(n * page),
            ScrollBy::Top => 0,
            // The renderer clamps this to the last page.
            ScrollBy::End => usize::MAX,
        };
    }

    fn wheel_left(&mut self, delta: isize) {
        if let Body::Rebase(r) = &mut self.body
            && r.inline.is_none()
            && r.raw.is_none()
        {
            r.move_selection(ScrollBy::Lines(delta));
            self.right_scroll = 0;
            return;
        }
        for _ in 0..delta.unsigned_abs() {
            self.move_cursor_row(delta > 0);
        }
    }

    fn move_cursor_row(&mut self, down: bool) {
        let width = self.editor_width;
        if let Some(editor) = self.active_editor_mut() {
            move_display_row(editor, width, down);
        }
    }

    fn copy_selection(&mut self, cut: bool) {
        let Some(editor) = self.active_editor_mut() else { return };
        if cut {
            editor.cut();
        } else {
            editor.copy();
        }
        let text = editor.yank_text();
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text);
        }
    }

    fn paste(&mut self) {
        let Some(text) = arboard::Clipboard::new().ok().and_then(|mut c| c.get_text().ok()) else {
            return;
        };
        if let Some(editor) = self.active_editor_mut() {
            editor.insert_str(text);
        }
    }
}

fn new_message_body(raw: &str, context: GitContext, git_dir: Option<&Path>, cc: char) -> MessageBody {
    let mut file = message::split(raw, cc);
    let mut reword_file = None;
    if context == GitContext::Commit
        && let Some((path, stored)) = git_dir.and_then(reword::pending_for_commit)
    {
        file.message = stored.lines().map(str::to_string).collect();
        reword_file = Some(path);
    }
    let status = status::parse(&file.trailer, cc);
    let editor = TextArea::new(file.message.clone());
    MessageBody { file, editor, status, reword_file }
}

impl RebaseBody {
    fn new(raw: &str, git_dir: Option<&Path>, cc: char) -> Self {
        let todo = rebase::parse(raw, cc);
        let rewords = match git_dir {
            Some(dir) => {
                let hashes: Vec<&str> = todo
                    .lines
                    .iter()
                    .filter_map(|l| match l {
                        TodoLine::Commit(c) if c.action == rebase::Action::Reword => Some(c.hash.as_str()),
                        _ => None,
                    })
                    .collect();
                reword::load(dir, &hashes)
            }
            None => HashMap::new(),
        };
        let mut body = Self {
            todo,
            selected: 0,
            rewords,
            inline: None,
            raw: None,
            details: DetailsLoader::spawn(git_dir.map(Path::to_path_buf)),
        };
        body.request_selected_details();
        body
    }

    pub fn selected_line(&self) -> Option<usize> {
        self.todo.instruction_indices().get(self.selected).copied()
    }

    /// Subject shown for a commit: its pending reword, or the todo's own subject.
    pub fn display_subject<'a>(&'a self, commit: &'a rebase::CommitLine) -> &'a str {
        self.rewords.get(&commit.hash).map(String::as_str).unwrap_or(commit.subject())
    }

    fn request_selected_details(&mut self) {
        let hash = self.selected_line().and_then(|l| self.todo.commit(l)).map(|c| c.hash.clone());
        if let Some(hash) = hash {
            self.details.request(&hash);
        }
    }

    fn move_selection(&mut self, by: ScrollBy) {
        let count = self.todo.instruction_indices().len();
        if count == 0 {
            return;
        }
        let last = count - 1;
        self.selected = match by {
            ScrollBy::Lines(n) => self.selected.saturating_add_signed(n).min(last),
            ScrollBy::Pages(n) => self.selected.saturating_add_signed(n * 10).min(last),
            ScrollBy::Top => 0,
            ScrollBy::End => last,
        };
        self.request_selected_details();
    }

    fn set_action(&mut self, action: rebase::Action) {
        let Some(line) = self.selected_line() else { return };
        let Some(commit) = self.todo.commit_mut(line) else { return };
        commit.set_action(action);
        if action != rebase::Action::Reword {
            let hash = commit.hash.clone();
            self.rewords.remove(&hash);
        }
    }

    fn cycle_action(&mut self) {
        let next = self.selected_line().and_then(|l| self.todo.commit(l)).map(|c| c.action.cycled());
        if let Some(action) = next {
            self.set_action(action);
        }
    }

    fn move_instruction(&mut self, up: bool) {
        let Some(line) = self.selected_line() else { return };
        if self.todo.move_instruction(line, up).is_some() {
            self.selected = if up { self.selected - 1 } else { self.selected + 1 };
        }
    }

    fn start_inline(&mut self) {
        let Some(commit) = self.selected_line().and_then(|l| self.todo.commit(l)) else { return };
        let mut editor = TextArea::new(vec![self.display_subject(commit).to_string()]);
        editor.move_cursor(CursorMove::End);
        self.inline = Some(editor);
    }

    fn commit_inline(&mut self) {
        let Some(editor) = self.inline.take() else { return };
        let text = editor.lines().join(" ").trim().to_string();
        let Some(line) = self.selected_line() else { return };
        let Some(commit) = self.todo.commit_mut(line) else { return };
        let hash = commit.hash.clone();
        if text.is_empty() || text == commit.subject() {
            self.rewords.remove(&hash);
        } else {
            commit.set_action(rebase::Action::Reword);
            self.rewords.insert(hash, text);
        }
    }

    fn toggle_raw(&mut self, cc: char) {
        match self.raw.take() {
            None => {
                self.inline = None;
                let text = self.todo.serialize();
                self.raw = Some(TextArea::new(text.lines().map(str::to_string).collect()));
            }
            Some(editor) => {
                let mut text = editor.lines().join("\n");
                if self.todo.final_newline {
                    text.push('\n');
                }
                self.todo = rebase::parse(&text, cc);
                let todo = &self.todo;
                self.rewords.retain(|hash, _| {
                    todo.lines.iter().any(|l| {
                        matches!(l, TodoLine::Commit(c) if &c.hash == hash && c.action == rebase::Action::Reword)
                    })
                });
                let count = self.todo.instruction_indices().len();
                self.selected = self.selected.min(count.saturating_sub(1));
                self.request_selected_details();
            }
        }
    }
}

/// Move the cursor one display row up or down, following soft-wrapped rows.
fn move_display_row(editor: &mut TextArea<'static>, width: usize, down: bool) {
    let (row, col) = editor.cursor();
    let lines = editor.lines();
    let segments = wrap::wrap(&lines[row], width);
    let (r, c) = wrap::locate(&segments, col);
    let row_len = |s: &wrap::Segment| s.text.chars().count();
    let (new_row, new_col) = if down {
        if r + 1 < segments.len() {
            (row, segments[r + 1].start + c.min(row_len(&segments[r + 1])))
        } else if row + 1 < lines.len() {
            let next = wrap::wrap(&lines[row + 1], width);
            (row + 1, c.min(row_len(&next[0])))
        } else {
            return;
        }
    } else if r > 0 {
        (row, segments[r - 1].start + c.min(row_len(&segments[r - 1])))
    } else if row > 0 {
        let prev = wrap::wrap(&lines[row - 1], width);
        let last = &prev[prev.len() - 1];
        (row - 1, last.start + c.min(row_len(last)))
    } else {
        return;
    };
    editor.move_cursor(CursorMove::Jump(new_row as u16, new_col as u16));
}

fn edit(editor: &mut TextArea<'static>, action: Action) {
    match action {
        Action::Edit(input) => {
            editor.input(input);
        }
        Action::DeleteLine => {
            editor.move_cursor(CursorMove::Head);
            editor.delete_line_by_end();
        }
        Action::Undo => {
            editor.undo();
        }
        Action::Redo => {
            editor.redo();
        }
        Action::DeleteWord => {
            editor.delete_word();
        }
        Action::DeleteNextWord => {
            editor.delete_next_word();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    const AMMEND2: &str = include_str!("../fixtures/ammend2_fixture.txt");
    const TODO: &str = include_str!("../fixtures/squash_fixture.txt");

    fn message_app() -> App {
        App::new(AMMEND2, GitContext::Commit, None, '#')
    }

    fn rebase_app() -> App {
        App::new(TODO, GitContext::Rebase, None, '#')
    }

    fn editor_lines(app: &App) -> Vec<String> {
        match &app.body {
            Body::Message(m) => m.editor.lines().to_vec(),
            Body::Plain(e) => e.lines().to_vec(),
            Body::Rebase(_) => panic!("rebase body has no editor"),
        }
    }

    fn rebase(app: &mut App) -> &mut RebaseBody {
        match &mut app.body {
            Body::Rebase(r) => r,
            _ => panic!("not a rebase body"),
        }
    }

    fn first_line(app: &App) -> String {
        app.serialized_content().lines().next().unwrap().to_string()
    }

    fn cursor(app: &mut App) -> (usize, usize) {
        app.active_editor_mut().unwrap().cursor()
    }

    fn reword_first(app: &mut App, subject: &str) {
        app.apply(Action::StartInline);
        let editor = rebase(app).inline.as_mut().unwrap();
        editor.move_cursor(CursorMove::End);
        editor.delete_line_by_head();
        editor.insert_str(subject);
        app.apply(Action::CommitInline);
    }

    #[test]
    fn message_file_opens_with_message_only() {
        let app = message_app();
        assert_eq!(editor_lines(&app), vec!["fix: release volume"]);
        assert!(app.has_right());
        assert_eq!(app.serialized_content(), AMMEND2);
    }

    #[test]
    fn unknown_file_is_plain_and_keeps_comments() {
        let app = App::new("notes\n# kept\n", GitContext::Unknown, None, '#');
        assert_eq!(editor_lines(&app), vec!["notes", "# kept"]);
        assert!(!app.has_right());
        assert_eq!(app.serialized_content(), "notes\n# kept\n");
    }

    #[test]
    fn message_without_comments_has_no_right_pane() {
        assert!(!App::new("just text\n", GitContext::Commit, None, '#').has_right());
    }

    #[test]
    fn edited_message_keeps_trailer() {
        let mut app = message_app();
        let editor = app.active_editor_mut().unwrap();
        editor.move_cursor(CursorMove::End);
        editor.insert_str(" now");
        assert!(app.serialized_content().starts_with("fix: release volume now\n\n# Please enter"));
    }

    #[test]
    fn focus_moves_only_when_right_pane_exists() {
        let mut plain = App::new("x\n", GitContext::Unknown, None, '#');
        plain.apply(Action::FocusRight);
        assert_eq!(plain.focus, Pane::Left);
        plain.apply(Action::ToggleFocus);
        assert_eq!(plain.focus, Pane::Left);

        let mut app = message_app();
        app.apply(Action::FocusRight);
        assert_eq!(app.focus, Pane::Right);
        app.apply(Action::ToggleFocus);
        assert_eq!(app.focus, Pane::Left);
        app.apply(Action::ToggleFocus);
        assert_eq!(app.focus, Pane::Right);
        app.apply(Action::FocusLeft);
        assert_eq!(app.focus, Pane::Left);
    }

    #[test]
    fn click_focuses_pane_under_pointer() {
        let mut app = message_app();
        app.rects = layout::compute(Rect::new(0, 0, 120, 30), true, Pane::Left);
        app.apply(Action::ClickAt { column: 100, row: 5 });
        assert_eq!(app.focus, Pane::Right);
        app.apply(Action::ClickAt { column: 5, row: 5 });
        assert_eq!(app.focus, Pane::Left);
    }

    #[test]
    fn right_pane_scrolling() {
        let mut app = message_app();
        app.right_page = 10;
        app.apply(Action::ScrollRight(ScrollBy::End));
        assert_eq!(app.right_scroll, usize::MAX);
        app.right_scroll = 5;
        app.apply(Action::ScrollRight(ScrollBy::Lines(-1)));
        assert_eq!(app.right_scroll, 4);
        app.apply(Action::ScrollRight(ScrollBy::Pages(1)));
        assert_eq!(app.right_scroll, 14);
        app.apply(Action::ScrollRight(ScrollBy::Top));
        assert_eq!(app.right_scroll, 0);
        app.apply(Action::ScrollRight(ScrollBy::Lines(-1)));
        assert_eq!(app.right_scroll, 0);
    }

    #[test]
    fn wheel_over_right_pane_scrolls_without_focusing_it() {
        let mut app = message_app();
        app.rects = layout::compute(Rect::new(0, 0, 120, 30), true, Pane::Left);
        app.apply(Action::WheelAt { column: 100, row: 5, delta: 1 });
        assert_eq!((app.right_scroll, app.focus), (1, Pane::Left));
    }

    #[test]
    fn help_toggles_and_resets_scroll() {
        let mut app = message_app();
        app.apply(Action::ToggleHelp);
        app.apply(Action::ScrollHelp(3));
        assert_eq!((app.show_help, app.help_scroll), (true, 3));
        app.apply(Action::ToggleHelp);
        assert_eq!((app.show_help, app.help_scroll), (false, 0));
    }

    #[test]
    fn cursor_moves_by_display_row() {
        let mut app = App::new("hello world foo\nx\n", GitContext::Unknown, None, '#');
        app.editor_width = 10;
        app.apply(Action::CursorDown);
        assert_eq!(cursor(&mut app), (0, 6));
        app.apply(Action::CursorDown);
        assert_eq!(cursor(&mut app), (1, 0));
        app.apply(Action::CursorUp);
        assert_eq!(cursor(&mut app), (0, 6));
    }

    #[test]
    fn editing_actions_reach_the_editor() {
        let mut app = App::new("one two\n", GitContext::Unknown, None, '#');
        app.apply(Action::DeleteLine);
        assert_eq!(app.serialized_content(), "\n");
        app.apply(Action::Undo);
        assert_eq!(app.serialized_content(), "one two\n");
    }

    #[test]
    fn stored_reword_prefills_commit_message() {
        let dir = tempfile::tempdir().unwrap();
        let rebase_merge = dir.path().join("rebase-merge");
        std::fs::create_dir_all(&rebase_merge).unwrap();
        std::fs::write(rebase_merge.join("done"), "reword 545ca5d95e7b6bbb297681be181a60d396ee8ee8 # c3\n").unwrap();
        let store = reword::store_dir(dir.path());
        std::fs::create_dir_all(&store).unwrap();
        std::fs::write(store.join("545ca5d"), "new subject\n\nbody 3\n").unwrap();

        let raw = "c3\n\nbody 3\n\n# Please enter the commit message\n#\n";
        let app = App::new(raw, GitContext::Commit, Some(dir.path().to_path_buf()), '#');
        assert_eq!(editor_lines(&app), vec!["new subject", "", "body 3"]);
        assert_eq!(app.serialized_content(), "new subject\n\nbody 3\n\n# Please enter the commit message\n#\n");
        app.finish_save().unwrap();
        assert!(!store.join("545ca5d").exists());
    }

    #[test]
    fn selection_moves_and_stops_at_ends() {
        let mut app = rebase_app();
        app.apply(Action::SelectBy(ScrollBy::Lines(-1)));
        assert_eq!(rebase(&mut app).selected, 0);
        app.apply(Action::SelectBy(ScrollBy::Lines(1)));
        assert_eq!(rebase(&mut app).selected, 1);
        app.apply(Action::SelectBy(ScrollBy::End));
        assert_eq!(rebase(&mut app).selected, 13);
        app.apply(Action::SelectBy(ScrollBy::Lines(1)));
        assert_eq!(rebase(&mut app).selected, 13);
        app.apply(Action::SelectBy(ScrollBy::Pages(-1)));
        assert_eq!(rebase(&mut app).selected, 3);
        app.apply(Action::SelectBy(ScrollBy::Top));
        assert_eq!(rebase(&mut app).selected, 0);
    }

    #[test]
    fn actions_change_the_selected_instruction() {
        let mut app = rebase_app();
        app.apply(Action::SetInstruction(rebase::Action::Fixup));
        assert_eq!(first_line(&app), "fixup 54763e6 # docs(state): record phase 8 context session");
        app.apply(Action::SetInstruction(rebase::Action::Pick));
        app.apply(Action::CycleInstruction);
        assert_eq!(first_line(&app), "squash 54763e6 # docs(state): record phase 8 context session");
    }

    #[test]
    fn moving_keeps_selection_on_the_moved_row() {
        let mut app = rebase_app();
        app.apply(Action::MoveInstruction { up: true });
        assert_eq!(app.serialized_content(), TODO);
        app.apply(Action::MoveInstruction { up: false });
        assert_eq!(rebase(&mut app).selected, 1);
        let out = app.serialized_content();
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "pick 45f8bcd # docs(08): create phase plan");
        assert_eq!(lines[1], "pick 54763e6 # docs(state): record phase 8 context session");
    }

    #[test]
    fn inline_reword_marks_row_and_keeps_subject_text() {
        let mut app = rebase_app();
        reword_first(&mut app, "new subject");
        assert_eq!(rebase(&mut app).rewords.get("54763e6").map(String::as_str), Some("new subject"));
        assert_eq!(first_line(&app), "reword 54763e6 # docs(state): record phase 8 context session");
        app.apply(Action::SetInstruction(rebase::Action::Pick));
        assert!(rebase(&mut app).rewords.is_empty());
    }

    #[test]
    fn cancelled_inline_edit_changes_nothing() {
        let mut app = rebase_app();
        app.apply(Action::StartInline);
        rebase(&mut app).inline.as_mut().unwrap().insert_str(" changed");
        app.apply(Action::CancelInline);
        let r = rebase(&mut app);
        assert!(r.rewords.is_empty() && r.inline.is_none());
        assert_eq!(app.serialized_content(), TODO);
    }

    #[test]
    fn raw_toggle_round_trips_and_reparses_edits() {
        let mut app = rebase_app();
        app.apply(Action::ToggleRaw);
        app.apply(Action::ToggleRaw);
        assert_eq!(app.serialized_content(), TODO);

        app.apply(Action::ToggleRaw);
        let raw = rebase(&mut app).raw.as_mut().unwrap();
        raw.move_cursor(CursorMove::Jump(13, 0));
        raw.move_cursor(CursorMove::End);
        raw.insert_newline();
        raw.insert_str("exec cargo test");
        app.apply(Action::ToggleRaw);
        let r = rebase(&mut app);
        let instructions = r.todo.instruction_indices();
        assert_eq!(instructions.len(), 15);
        assert!(matches!(r.todo.lines[*instructions.last().unwrap()], TodoLine::Other(_)));
    }

    #[test]
    fn raw_toggle_keeps_pending_rewords() {
        let mut app = rebase_app();
        reword_first(&mut app, "new subject");
        app.apply(Action::ToggleRaw);
        app.apply(Action::ToggleRaw);
        assert_eq!(rebase(&mut app).rewords.len(), 1);
    }

    #[test]
    fn save_commits_an_open_inline_edit() {
        let mut app = rebase_app();
        app.apply(Action::StartInline);
        let editor = rebase(&mut app).inline.as_mut().unwrap();
        editor.move_cursor(CursorMove::End);
        editor.delete_line_by_head();
        editor.insert_str("saved subject");
        assert_eq!(app.apply(Action::Save), Outcome::Save);
        assert_eq!(rebase(&mut app).rewords.get("54763e6").map(String::as_str), Some("saved subject"));
    }

    #[test]
    fn stored_rewords_load_on_open_and_store_on_save() {
        let dir = tempfile::tempdir().unwrap();
        let store = reword::store_dir(dir.path());
        std::fs::create_dir_all(&store).unwrap();
        std::fs::write(store.join("54763e6"), "stored subject\n\nbody\n").unwrap();
        std::fs::write(store.join("deadbeef"), "stale\n").unwrap();

        let todo = TODO.replacen("pick 54763e6", "reword 54763e6", 1);
        let mut app = App::new(&todo, GitContext::Rebase, Some(dir.path().to_path_buf()), '#');
        assert_eq!(rebase(&mut app).rewords.get("54763e6").map(String::as_str), Some("stored subject"));
        assert!(!store.join("deadbeef").exists());

        reword_first(&mut app, "new subject");
        app.finish_save().unwrap();
        assert_eq!(std::fs::read_to_string(store.join("54763e6")).unwrap(), "new subject\n");
    }

    #[test]
    fn message_layout_has_no_background_work() {
        assert!(!message_app().poll_background());
    }
}
