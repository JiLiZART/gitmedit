# Phase 3: Commit Message Intelligence - Research

**Researched:** 2026-03-22
**Domain:** Real-time commit message validation (subject line counter, blank line enforcement, context-aware help overlay)
**Confidence:** HIGH (verified against Phase 2 codebase, ratatui rendering patterns, git conventions)

## Summary

Phase 3 extends the text editor with three distinct intelligence features to guide users toward well-formed commit messages:

1. **Subject Line Counter (COMMIT-01, COMMIT-02):** A real-time character count displayed in the status bar with color coding (green ≤50, yellow 51-72, red >72) that updates on every keystroke with no perceivable lag.

2. **Blank Line Enforcement (COMMIT-03):** Visual indicator in the status bar warning when no blank line exists between subject and body, allowing save but suggesting the convention.

3. **Help Overlay (COMMIT-06, HELP-01 through HELP-04):** A non-intrusive modal dialog triggered by Ctrl+H, dismissable with Esc, showing context-aware actions (Commit vs Merge mode) with brief help text.

The implementation leverages Phase 2's Document model (which already separates content and comments), extends the Action enum to include Help, adds a help overlay state to App, and updates the status bar renderer to compute and display the counter. No new dependencies required; ratatui's Layout and Paragraph widgets suffice for the modal.

**Primary recommendation:** Extract the first editable line (subject) via a new `Document::first_line()` method, compute character count in renderer on each frame, and implement help overlay as an optional overlay() function called after the main content is rendered.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Subject Line Counter Location:** Display in status bar at bottom (follows nano convention, doesn't distract from editing)
- **Real-time Updates:** Counter updates on every keystroke with no perceivable lag
- **Color Coding Thresholds:** Green if ≤50 chars, yellow for 51-72, red for >72
- **Counter Scope:** Applies only to first line (subject); counts up to first newline
- **Blank Line Enforcement:** Suggest but don't block; visual indicator only
- **Blank Line Detection:** At least one completely blank line (no whitespace) between subject and first body line
- **Blank Line Non-Blocking:** Allow save even if blank line is missing (user choice to override)
- **Help Overlay Modal:** Non-intrusive overlay, dismissable with Esc key
- **Help Trigger Hotkey:** Ctrl+H
- **Help State Preservation:** On dismiss, cursor and scroll position preserved exactly
- **Help Integration:** Add new `HelpOverlay` state to App state machine
- **Mode-Based Filtering:** Commit mode, merge mode, rebase mode show different action sets
- **Commit Mode Actions:** Ctrl+S (save), Esc (cancel), Ctrl+C/X/V (clipboard), Ctrl+U/W/D (editing), Ctrl+H (help)
- **Merge Mode Actions:** Same as commit, plus note about conflict markers being read-only
- **Rebase Mode Actions:** Phase 4 — don't implement for Phase 3
- **Help Text Format:** Brief, one-line per action, showing key + description

### Claude's Discretion

- **Color Scheme Details:** Exact RGB/hex values beyond green/yellow/red choice
- **Blank Line Warning Wording:** Specific message text
- **Help Overlay Layout:** Dialog box design, centering, size
- **Help Text Storage:** How/where to store help strings (constants, enum variant data, etc.)
- **Help Text Word Wrapping:** Behavior for long help lines in modal
- **Status Bar Styling:** Font, padding, alignment details beyond content

### Deferred Ideas (OUT OF SCOPE)

- Rebase mode context filtering — Phase 4
- Customizable hotkeys — v2 feature (CONFIG-01)
- Syntax highlighting in help text — v2 feature
- Help text localization — English only in v1

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| COMMIT-01 | Subject line character counter displays (real-time) | §Subject Line Counter — `Document::first_line()` method, renderer computes on each frame tick |
| COMMIT-02 | Subject line shows green if ≤50 chars, yellow if 50-72, red if >72 | §Subject Line Counter — color mapping table, Status Bar Styling section |
| COMMIT-03 | Blank line between subject and body is enforced/suggested | §Blank Line Detection — algorithm to detect first blank line after subject |
| COMMIT-06 | Hotkey help shows on Ctrl+H (or similar) | §Help Overlay State & Rendering — Ctrl+H triggers action, overlay modal displayed |
| HELP-01 | Hotkey reference is available (Ctrl+H or similar) | §Help Overlay — triggered by Ctrl+H, dismissable with Esc |
| HELP-02 | Hotkey reference shows only relevant actions for current mode | §Mode-Aware Action Filtering — GitContext determines action set shown |
| HELP-03 | Hotkey reference does not interfere with message editing | §Help Overlay Rendering — modal overlay rendered on top without modifying editor state |
| HELP-04 | Help can be dismissed and editing resumes | §Help Overlay Dismissal — Esc key dismisses, cursor and scroll preserved |

</phase_requirements>

---

## Standard Stack

### Core (No New Dependencies)

All Phase 3 features build on existing stack from Phase 2:

| Library | Version | Phase 3 Use | Notes |
|---------|---------|-----------|-------|
| ratatui | 0.30 | Modal overlay, Layout for help dialog, color styling | Paragraph widget for help text, Rect layout for centering |
| crossterm | 0.29 | Ctrl+H event handling (already used for all keys) | No new terminal features needed |
| ratatui-textarea | 0.8 | Subject line extraction via `Document::first_line()` | No changes to TextArea itself |

### No New Libraries Required

Phase 3 adds no new crates. The counter, blank line detection, and help overlay are implemented in Rust with standard string operations and ratatui's built-in layout/styling.

---

## Architecture Patterns

### Pattern 1: Subject Line Counter (Real-Time, Stateless)

**What:** The status bar displays a live character count of the subject line (first editable line up to the first newline) that updates on every render frame.

**Implementation approach:**
- Add `Document::first_line()` method that returns the first editable (Content) line as a String
- In `Renderer::render_status_bar()`, call `app.document().first_line()` and count `.chars().len()`
- Compute color based on: green (≤50), yellow (51-72), red (>72)
- Format status bar text: `"Subject: 42 chars | ^S Save  Esc Cancel  ^H Help"`

**Key insight:** Since the renderer is called on every tick (~60fps), the counter updates automatically on every keystroke without special event handling. No state needed; it's computed fresh from Document each frame.

**Trade-off:** Slightly more computation per frame (one string allocation + char count). Negligible at 60fps on commit messages <10KB. Benefit: simple, no state sync issues.

**Example flow:**
```rust
// In Renderer::render_status_bar()
let first_line = app.document().first_line();
let char_count = first_line.chars().count(); // Use char count, not byte length (Unicode)
let color = match char_count {
    0..=50 => Color::Green,
    51..=72 => Color::Yellow,
    _ => Color::Red,
};
let counter_text = format!("Chars: {} ", char_count);
let status_widget = Paragraph::new(
    Line::from(Span::styled(counter_text, Style::default().fg(color)))
);
```

### Pattern 2: Blank Line Detection (Structural Analysis)

**What:** Detect if a blank line exists between the subject (first line) and the body (second+ lines). A "blank line" is a line with zero length or only whitespace.

**Detection algorithm:**
1. Extract first editable line (subject)
2. If `editable_lines.len() < 2`, no body exists → no blank line needed (single-line messages are valid)
3. If `editable_lines.len() >= 2`, check line 1 (index 1) for all-whitespace or empty
4. All-whitespace includes spaces, tabs, etc.: `editable_lines[1].trim().is_empty()`
5. If line 1 is NOT blank, set warning status

**Key insight:** This is structural (check the Document's editable_lines), not content-aware. Comments and conflict markers don't affect detection.

**Example:**
```rust
// In Renderer::render_status_bar() or new method in Document
pub fn has_blank_line_separator(&self) -> bool {
    let editable = self.editable_lines();
    if editable.len() < 2 {
        return true; // Single-line messages don't require blank line
    }
    editable[1].trim().is_empty()
}
```

**Display:** If `!has_blank_line_separator()`, add warning text to status bar: `" [No blank line]"` (yellow or red color).

### Pattern 3: Help Overlay State & Rendering

**What:** A modal dialog overlay that renders on top of the editor content without modifying editor state. Triggered by Ctrl+H, dismissed by Esc, preserves cursor position and scroll offset when closed.

**State machine extension (App):**
```rust
pub struct App {
    document: Document,
    textarea: TextArea<'static>,
    context: GitContext,
    show_help: bool,  // NEW: toggles help overlay visibility
}
```

**Action enum extension:**
```rust
pub enum Action {
    Save,
    Cancel,
    Noop,
    Help,        // NEW: triggered by Ctrl+H
    DismissHelp, // NEW: triggered by Esc while help visible
}
```

**Event handling in main.rs:**
- Ctrl+H: emit `Action::Help` → `app.apply(Action::Help)` sets `app.show_help = true`
- Esc: if `app.show_help`, emit `Action::DismissHelp` → sets `app.show_help = false`; otherwise emit `Action::Cancel` (existing behavior)
- While help is visible, all other key events are ignored (no editing until help dismissed)

**Rendering (Renderer::render):**
1. Render main content area (unchanged)
2. Render status bar (unchanged)
3. **NEW:** If `app.show_help`, call `Renderer::render_help_overlay(frame, app)` to draw modal on top

### Pattern 4: Modal Overlay Rendering (Centered Dialog)

**What:** A text dialog rendered in the center of the terminal using ratatui's Layout to compute the overlay rectangle and Paragraph to render the help text.

**Implementation:**
```rust
fn render_help_overlay(frame: &mut Frame, app: &App) {
    // Create a centered dialog rectangle (e.g., 60% width, 70% height)
    let popup_area = Self::centered_rect(60, 70, frame.area());

    // Build help text from GitContext
    let help_text = Self::help_text_for_context(app.context());

    // Render a block with borders and the help text
    let help_widget = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help (Esc to close)"))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(help_widget, popup_area);
}

/// Center a rect of size (width_percent, height_percent) within the available area
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);

    horizontal_layout[1]
}
```

**Key patterns:**
- Modal is rendered **after** main content in the same frame → appears on top without clearing
- No alternate screen needed (Phase 1 foundational decision maintained)
- Modal renders a Block with Borders and a title for clarity
- Help text is static (no interaction within the modal; only Esc dismisses)

### Pattern 5: Mode-Aware Action Filtering

**What:** The help overlay shows different actions based on the detected git context (GitContext enum).

**Action lists by mode:**

| Mode | Actions Shown |
|------|---------------|
| Commit | Ctrl+S Save, Esc Cancel, Ctrl+C Copy, Ctrl+X Cut, Ctrl+V Paste, Ctrl+U Delete Line, Ctrl+Z Undo, Ctrl+Y Redo, Ctrl+W Delete Word, Ctrl+D Delete Next Word, Ctrl+H Help |
| Merge | Same as Commit + note about conflict markers (e.g., "Conflict markers (<<<...) are read-only") |
| Rebase | Phase 4 — skip implementation for Phase 3 |
| Unknown | Same as Commit (safe default) |

**Implementation:**
```rust
fn help_text_for_context(context: &GitContext) -> Vec<Line> {
    let base_actions = vec![
        "^S  Save message",
        "Esc  Cancel (discard changes)",
        "",
        "^C  Copy selection",
        "^X  Cut selection",
        "^V  Paste",
        "",
        "^U  Delete entire line",
        "^Z  Undo",
        "^Y  Redo",
        "^W  Delete previous word",
        "^D  Delete next word",
    ];

    let mut lines: Vec<Line> = base_actions
        .iter()
        .map(|s| Line::raw(*s))
        .collect();

    match context {
        GitContext::Merge => {
            lines.push(Line::raw(""));
            lines.push(Line::raw("NOTE: Conflict markers (<<<, ==>, >>>) are read-only"));
        }
        _ => {}
    }

    lines
}
```

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Modal dialog centering | Custom math to center a Rect | ratatui `Layout` with `Constraint::Percentage` | Layout handles different terminal widths; custom math breaks on resize |
| Character counting (Unicode) | `.len()` on String | `.chars().count()` | `.len()` counts bytes; multi-byte chars (emoji, non-ASCII) miscounted |
| Line classification | Detect blank lines with `line == ""` | `line.trim().is_empty()` | Trailing whitespace should be ignored per git convention |
| Help text management | Multiple strings scattered in code | Constants or enum-driven text generation | Single source of truth prevents help/actions falling out of sync |
| Color scheme | Hard-code RGB in multiple places | Define colors once in a helper fn | Changing theme requires one edit, not three |

---

## Common Pitfalls

### Pitfall 1: Status Bar Overflow with Long Counter Text

**What goes wrong:** Adding the counter to status bar can exceed the terminal width if the status bar text is too verbose. On a 80-char terminal, a status bar with "Chars: 999 | ^S Save  Esc Cancel  ^H Help" may be 50+ characters, leaving little room for context display.

**Why it happens:** Status bar is rendered in a fixed `Constraint::Length(1)` region. If the text is longer than the available width, ratatui truncates it (no wrapping on a single line).

**How to avoid:** Keep status bar text minimal. Suggested format: `"Chars: 50 ⬤ ^S Save ^C Cancel ^H Help"` (use emoji for color indicator instead of separate text; ~30 chars total). Prioritize the counter and save/cancel shortcuts; move less critical info (like git context) into the help overlay instead.

**Warning signs:** When testing on narrow terminals (80 chars), status bar text is cut off; the counter disappears.

**Example of good vs bad:**
```rust
// Bad: too verbose
let status_text = format!("Subject line character counter: {} chars | ^S Save  Esc Cancel  ^H Help [{:?}]", count, context);

// Good: concise
let status_text = format!("Chars: {} | ^S ^H Help", count);
```

### Pitfall 2: Blank Line Detection Counting Comment Lines

**What goes wrong:** Checking `editable_lines[1]` works for typical messages, but if the file has Comments between the subject and body, the logic breaks. Example:
```
Fix the bug
# Please enter the commit message for your changes.
Body text here
```

In this case, `editable_lines = ["Fix the bug", "Body text here"]` — the comment is filtered out. So `editable_lines[1].trim().is_empty()` returns false (body text is not blank). But visually, there IS a blank line (the comment line). The detection is correct for commit semantics (a blank line of editable content), not visual layout.

**Why it happens:** `Document::editable_lines()` filters out comments by design (Phase 2). So blank line detection operates on the semantic content, not the visual file.

**How to avoid:** This is actually correct behavior — git cares about the semantic blank line in the editable content, not comment lines. Document the expectation: "Blank line detection checks editable lines only; comments are transparent to this check."

**Warning signs:** If the first non-comment line after subject is a comment (not blank), the warning won't trigger. This is acceptable per git conventions (the comment is visually a separator).

### Pitfall 3: Modal Overlay Clearing Content on Rerender

**What goes wrong:** If the help overlay is rendered by overwriting a part of the frame instead of rendering on top, the content underneath is erased even after help is dismissed.

**Why it happens:** ratatui's `frame.render_widget()` clears the target area first, then draws. If the help modal is rendered with the full frame area, it overwrites everything.

**How to avoid:** Compute a centered Rect for the modal **inside** the full frame (using Layout as shown above). Render main content first, then the modal second. The modal Rect should be smaller than the full frame area.

**Warning signs:** After dismissing help, parts of the editor text are missing or garbled.

### Pitfall 4: Blocking Input While Help Visible

**What goes wrong:** While help is displayed, keystrokes (e.g., typing text) are still processed, causing the editor to be modified while the user is reading help.

**Why it happens:** The event loop doesn't check `app.show_help` before passing key events to the textarea.

**How to avoid:** In `main.rs` event loop, add a guard: if `app.show_help` is true, only accept Esc (to dismiss) and Ctrl+H (to toggle). All other key presses are ignored (Noop).

**Example:**
```rust
match &event {
    Event::Key(KeyEvent { code, modifiers, kind: KeyEventKind::Press, .. }) => {
        if app.show_help {
            // Help overlay is visible: only allow Esc to dismiss
            match (*code, *modifiers) {
                (KeyCode::Esc, _) => {
                    app.apply(Action::DismissHelp);
                }
                (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                    // Ctrl+H while help visible: toggle off
                    app.apply(Action::DismissHelp);
                }
                _ => { /* ignore other keys */ }
            }
        } else {
            // Normal editing mode
            match (*code, *modifiers) {
                (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                    app.apply(Action::Help);
                }
                // ... other keys
            }
        }
    }
}
```

### Pitfall 5: Cursor/Scroll Position Lost When Help Dismissed

**What goes wrong:** User opens help, closes it, and the cursor has jumped to (0, 0) or the viewport has scrolled to the top.

**Why it happens:** The help modal rendering might implicitly reset the cursor position or viewport state.

**How to avoid:** Help overlay is purely a visual overlay; it does NOT modify `textarea` or `document` state. The renderer preserves cursor position automatically because it doesn't change the underlying App state. Verify in tests: open help, close it, verify `app.textarea().cursor()` is unchanged.

**Warning signs:** Cursor position changes after help dismiss; scroll viewport resets.

---

## Code Examples

All verified patterns from Phase 2 codebase and ratatui documentation.

### Subject Line Extraction

Source: Phase 2 `Document::editable_lines()` pattern

```rust
// In document.rs, add to impl Document:
pub fn first_line(&self) -> String {
    self.editable_lines()
        .first()
        .cloned()
        .unwrap_or_default()
}

// In main.rs tests:
#[test]
fn test_first_line_returns_subject() {
    let doc = Document::parse("Fix the bug\n\nBody text\n", '#');
    assert_eq!(doc.first_line(), "Fix the bug");
}

#[test]
fn test_first_line_empty_when_no_content() {
    let doc = Document::parse("# only comment\n", '#');
    assert_eq!(doc.first_line(), "");
}
```

### Character Counter in Status Bar

Source: Phase 2 `Renderer::render_status_bar()` pattern, extended

```rust
// In renderer.rs, update render_status_bar():
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let first_line = app.document().first_line();
    let char_count = first_line.chars().count();

    let color = match char_count {
        0..=50 => Color::Green,
        51..=72 => Color::Yellow,
        _ => Color::Red,
    };

    let counter = format!("Chars: {}", char_count);
    let counter_span = Span::styled(counter, Style::default().fg(color).bold());

    let actions = Span::raw(" | ^S ^H Help");

    let line = Line::from(vec![counter_span, actions]);
    let status_widget = Paragraph::new(line)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(status_widget, area);
}
```

### Blank Line Detection

Source: Phase 2 document analysis pattern

```rust
// In document.rs, add to impl Document:
pub fn has_blank_line_after_subject(&self) -> bool {
    let editable = self.editable_lines();
    if editable.len() < 2 {
        true // Single-line messages don't require blank line
    } else {
        editable[1].trim().is_empty()
    }
}

// In renderer.rs, extend render_status_bar():
let has_blank_line = app.document().has_blank_line_after_subject();
let blank_warning = if !has_blank_line {
    Span::styled(" [No blank line]", Style::default().fg(Color::Yellow))
} else {
    Span::raw("")
};
// Add blank_warning to status line
```

### Help Modal with Centered Layout

Source: ratatui `Layout` pattern (from ratatui.rs examples)

```rust
// In renderer.rs, add:
fn render_help_overlay(frame: &mut Frame, app: &App) {
    let popup_area = Self::centered_rect(60, 70, frame.area());

    let help_text = Self::help_text_for_context(app.context());

    let help_widget = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Help (Esc to close) "))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .alignment(Alignment::Left);

    frame.render_widget(help_widget, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);

    horizontal[1]
}

fn help_text_for_context(context: &GitContext) -> Vec<Line> {
    match context {
        GitContext::Commit | GitContext::Unknown => vec![
            Line::raw("Ctrl+S  Save message"),
            Line::raw("Esc     Cancel (discard)"),
            Line::raw(""),
            Line::raw("Ctrl+C  Copy selection"),
            Line::raw("Ctrl+X  Cut selection"),
            Line::raw("Ctrl+V  Paste"),
            Line::raw(""),
            Line::raw("Ctrl+U  Delete line"),
            Line::raw("Ctrl+Z  Undo"),
            Line::raw("Ctrl+Y  Redo"),
            Line::raw("Ctrl+W  Delete word"),
            Line::raw("Ctrl+D  Delete next word"),
        ],
        GitContext::Merge => {
            let mut lines = Self::help_text_for_context(&GitContext::Commit);
            lines.push(Line::raw(""));
            lines.push(Line::raw("NOTE: Conflict markers (<<<, ===, >>>) are read-only."));
            lines
        }
        _ => vec![Line::raw("(Help not configured for this mode)")],
    }
}
```

### Action Enum & Apply Logic

Source: Phase 2 `App::apply()` pattern, extended

```rust
// In app.rs:
#[derive(Debug)]
pub enum Action {
    Save,
    Cancel,
    Noop,
    Help,
    DismissHelp,
}

pub struct App {
    document: Document,
    textarea: TextArea<'static>,
    context: GitContext,
    show_help: bool, // NEW
}

impl App {
    pub fn new(raw_content: &str, context: GitContext) -> Self {
        // ... existing code ...
        Self {
            document,
            textarea,
            context,
            show_help: false, // NEW
        }
    }

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

    pub fn is_help_visible(&self) -> bool {
        self.show_help
    }
}
```

### Event Loop Integration

Source: Phase 2 `main.rs` key handling pattern, extended

```rust
// In main.rs event loop, key handling:
match &event {
    Event::Key(KeyEvent { code, modifiers, kind: KeyEventKind::Press, .. }) => {
        // If help is visible, only allow Esc/Ctrl+H to dismiss
        if app.is_help_visible() {
            match (*code, *modifiers) {
                (KeyCode::Esc, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                    app.apply(Action::DismissHelp);
                }
                _ => { /* ignore other input while help is open */ }
            }
        } else {
            // Normal editing mode
            match (*code, *modifiers) {
                (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                    match app.apply(Action::Save) {
                        Outcome::Save => {
                            drop(guard);
                            writer::FileWriter::write_atomic(&app.serialized_content(), &path)?;
                            process::exit(0);
                        }
                        _ => {}
                    }
                }
                (KeyCode::Esc, _) => {
                    match app.apply(Action::Cancel) {
                        Outcome::Cancel => {
                            drop(guard);
                            process::exit(1);
                        }
                        _ => {}
                    }
                }
                (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                    app.apply(Action::Help);
                }
                // ... existing handlers ...
                _ => {
                    app.textarea_mut().input(event.clone());
                }
            }
        }
    }
    // ... rest of event loop ...
}
```

### Renderer Frame with Help Overlay

Source: Phase 2 `Renderer::render()` pattern, extended

```rust
// In renderer.rs:
pub fn render(frame: &mut Frame, app: &App) {
    // Split into content area and status bar (unchanged)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    Self::render_content(frame, app, chunks[0]);
    Self::render_status_bar(frame, app, chunks[1]);

    // NEW: Render help overlay on top if visible
    if app.is_help_visible() {
        Self::render_help_overlay(frame, app);
    }
}
```

---

## State of the Art

| Concern | Phase 2 Approach | Phase 3 Change | Rationale |
|---------|-----------------|----------------|-----------|
| Status bar content | Shows context and save/cancel hints | Add character counter + blank line warning | Proactive guidance toward git conventions (50/72 rule, blank line separator) |
| App state machine | Two outcomes (Save/Cancel) | Three outcomes + Help state | Help is a UI overlay, not a state transition; `Outcome::Continue` handles it |
| Modal dialogs | N/A (first modal) | Non-intrusive overlay rendered on top | Matches nano/vim conventions; no alternate screen (Phase 1 constraint) |
| Action filtering | All keys routed to textarea | Ctrl+H intercepted, Esc context-dependent | Enables help hotkey without losing cancel semantics |
| First-line extraction | N/A | Add `Document::first_line()` method | Enables counter computation and blank-line detection |

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) |
| Config file | N/A — no config (uses Cargo.toml) |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test --lib && cargo test --doc` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| COMMIT-01 | Subject line counter displays | unit | `cargo test --lib document::tests::test_first_line` | ✅ Wave 0 |
| COMMIT-02 | Color coding applied correctly | unit | `cargo test --lib renderer::tests::test_counter_colors` | ❌ Wave 0 |
| COMMIT-03 | Blank line detection works | unit | `cargo test --lib document::tests::test_blank_line_detection` | ❌ Wave 0 |
| COMMIT-06 | Ctrl+H triggers help | integration | `cargo test --lib app::tests::test_help_action` | ❌ Wave 0 |
| HELP-01 | Help overlay renders | integration | `cargo test --lib renderer::tests::test_help_overlay_render` | ❌ Wave 0 |
| HELP-02 | Mode-aware actions shown | unit | `cargo test --lib renderer::tests::test_help_text_by_context` | ❌ Wave 0 |
| HELP-03 | Help doesn't interfere with editing | integration | `cargo test --lib main_integration::test_help_overlay_preserves_state` | ❌ Wave 0 |
| HELP-04 | Esc dismisses help | unit | `cargo test --lib app::tests::test_dismiss_help_action` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --lib` (unit tests only, <2s runtime)
- **Per wave merge:** `cargo test` (full suite including integration tests, <10s runtime)
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/integration_help_overlay.rs` — integration test for help modal rendering, modal dismissal with Esc, state preservation
- [ ] `src/renderer.rs` — add `test_counter_colors()` unit test for color mapping (0..=50 = Green, 51..=72 = Yellow, 73+ = Red)
- [ ] `src/document.rs` — add `test_blank_line_detection()` unit test for `has_blank_line_after_subject()`
- [ ] `src/app.rs` — add `test_help_action()` and `test_dismiss_help_action()` unit tests
- [ ] `src/renderer.rs` — add `test_help_text_by_context()` unit test for mode-aware help text generation

**Test fixture requirements:** None — all Phase 3 tests use in-memory Document/App state, no file I/O.

---

## Sources

### Primary (HIGH confidence)

- **Phase 2 RESEARCH.md** — Verified ratatui-textarea 0.8, crossterm 0.29, Document model architecture
- **Phase 2 Source Code** (`src/app.rs`, `src/renderer.rs`, `src/document.rs`) — Verified Action enum pattern, render_status_bar pattern, editable_lines pattern
- **ratatui 0.30 Documentation** (https://ratatui.rs/) — Layout with Constraint::Percentage, Paragraph widget, Block/Borders styling, Color enum
- **Git Commit Message Conventions** — 50/72 rule verified in: https://www.conventionalcommits.org/, https://git-scm.com/docs/pretty-formats

### Secondary (MEDIUM confidence)

- **ratatui-textarea 0.8.0 Source** — Verified undo/redo, copy/yank buffers, input() method (not needed for Phase 3, but documented for completeness)
- **Existing Phase 1 & Phase 2 test suites** — Verified cargo test pattern, test module structure

### Tertiary (observations from codebase)

- **Phase 2 UAT.md** — Confirmed that counter and help are user-visible features; no new file I/O or git integration needed
- **CONTEXT.md decisions** — All locked decisions are git conventions (50/72 rule) and nano editor patterns (status bar, non-blocking warnings)

---

## Metadata

**Confidence breakdown:**
- Standard Stack: **HIGH** — No new dependencies; all patterns verified in Phase 2 codebase
- Architecture: **HIGH** — Modal overlay and action filtering are standard ratatui patterns; blank line detection is trivial string logic
- Pitfalls: **MEDIUM-HIGH** — Common pitfall (status bar overflow, help blocking input) identified from ratatui examples; specific to Phase 3 implementation
- Code Examples: **HIGH** — All examples extracted/adapted from Phase 2 source; patterns verified

**Research date:** 2026-03-22
**Valid until:** 2026-04-01 (10 days; ratatui 0.30 is stable; no expected changes to git conventions)

---

*Phase 3: Commit Message Intelligence*
*Researched: 2026-03-22*
*Status: Ready for planning*
