# Phase 4: Rebase + Squash Modes - Research

**Researched:** 2026-03-23
**Domain:** Git interactive rebase/squash operations, structured table UI, dual-pane layout
**Confidence:** HIGH

## Summary

Phase 4 adds two complementary git operation modes to gitmedit: interactive rebase (action cycling on git-rebase-todo lines) and squash (read-only commit log + editable message). Both build directly on Phase 2's foundation (ContentLine enum, Document parsing, atomic file writing) and Phase 3's architecture (GitContext enum, rendering patterns). The work centers on three technical areas: rebase-todo format parsing and table rendering, action cycling via Tab key, and squash context detection with dual-pane layout.

**Primary recommendation:** Extend Document to support RebaseLine enum with action+hash+subject variants; render via ratatui Table widget; implement Tab as action-cycling hotkey by extending Action enum; detect squash context in existing code path and render protected commit log above editable message.

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Rebase Todo Parsing:** Parse git-rebase-todo line-by-line into structured `RebaseLine` enum with action + commit hash + message variants
- **Display Format:** Show rebase as structured table (columns: action, hash, subject) — not free-form text
- **Comment Handling:** Comment lines preserved and displayed but not editable (reuse ContentLine pattern from Phase 2)
- **Action Cycling:** Tab key cycles pick→squash→fixup→drop→pick; only non-comment lines cycle; selected line visually highlighted
- **Rebase File Writing:** Reconstruct rebase-todo in exact git format `{action} {hash} {subject}` per line; comment lines written back byte-for-byte; atomic write (establish Phase 1 pattern)
- **Squash Context Display:** Detect SQUASH_MSG file; parse accumulated commit log (read-only); editable area below; commit log marked protected
- **Mode Detection:** Reuse existing context detection (Phase 1); add `GitContext::Rebase` and `GitContext::Squash` variants (already stubbed)
- **UI Switching:** Structured rebase table view for rebase-todo (not text editing); dual-pane/stacked for SQUASH_MSG (read-only log + editable message)

### Claude's Discretion

- Exact table column widths and alignment for rebase display
- Visual styling colors for protected sections and highlighted lines
- Cursor movement in table (arrow keys move between rows vs columns)
- Tab vs other hotkey for action cycling (Tab chosen for ergonomics)
- Line wrapping behavior for long commit subjects in table view
- Handling of rebase-todo files larger than screen height (scrolling strategy)

### Deferred Ideas (OUT OF SCOPE)

- Line reordering in rebase (move commits up/down) — v2 feature
- Squash message templates — v2 feature
- Rebase-specific hotkey help overlay — Phase 4 reuses Phase 3 help (update context filtering)
- Exec line support with arguments — v2 enhancement

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| REBASE-01 | Rebase todo file parsed into lines with actions (pick, squash, fixup, drop, exec) | RebaseLine enum with action+hash+subject variants; classify_line() extended to detect action prefix |
| REBASE-02 | Rebase mode displays lines in structured table (not free-form text) | ratatui Table widget renders action, abbreviated hash, and subject columns |
| REBASE-03 | User cycles through action types with Tab hotkey (p→s→f→d→p) | Action enum extended with CycleRebaseAction; Tab key maps to this; action cycling logic in App::apply() |
| REBASE-04 | Non-comment lines can have their action changed | RebaseLine Content variant holds mutable action field; Tab handler updates selected line's action |
| REBASE-05 | Comment lines and order preserved on save | ContentLine::Comment and RebaseLine variants maintain original structure; serialize() reconstructs byte-for-byte |
| REBASE-06 | Save writes rebase todo back in exact git format | write_atomic() reconstructs `{action} {hash} {subject}` per line; comment lines unchanged |
| SQUASH-01 | Squash mode detects when editing SQUASH_MSG file | GitContext::Squash already detected by detect_context(); needs handling in App/Renderer |
| SQUASH-02 | Squash mode displays original commit log (read-only) | Parse "This is a combination of N commits" block from SQUASH_MSG; render in protected section |
| SQUASH-03 | Squash mode allows editing combined commit message | TextArea positioned below or beside read-only log; normal editing applies to editable area only |
| SQUASH-04 | Squash commits list highlighted and protected from editing | Render commit log section with distinct background color; prevent ContentLine::Comment editing on this block |

## Standard Stack

### Core Libraries

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ratatui | 0.30 | TUI rendering with widget system | Established Phase 1-3 stack; Table widget supports structured rebase display |
| ratatui-textarea | 0.8 | Text input widget | Used for commit message editing; can be positioned in dual-pane layout |
| crossterm | 0.29 | Terminal I/O abstraction | Established Phase 1-3; handles raw mode, event capture |

### Supporting Libraries (No New Deps)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::fs | (stdlib) | File I/O | Atomic write pattern (Phase 1) reused for rebase-todo output |
| std::process | (stdlib) | Subprocess (git config) | Read comment char; already proven in Phase 2 |

**No new dependencies required** — all phase work uses existing Phase 1-3 stack.

**Installation:** No new packages needed.
```bash
# Verify existing stack is intact
cargo check
cargo test --lib context::tests
cargo test --lib document::tests
```

## Architecture Patterns

### Rebase Line Representation

**What:** Structured enum to represent a single line from git-rebase-todo

**Pattern:**
```rust
// Extends ContentLine pattern from Phase 2
#[derive(Debug, Clone, PartialEq)]
pub enum RebaseLine {
    Content {
        action: Action,       // pick, squash, fixup, drop, exec
        hash: String,         // abbreviated commit hash (7-40 chars)
        subject: String,      // commit message subject
    },
    Comment(String),          // "# ..." lines, not editable
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RebaseAction {
    Pick,
    Squash,
    Fixup,
    Drop,
    Exec,
}

impl RebaseAction {
    pub fn cycle(&self) -> Self {
        match self {
            Pick => Squash,
            Squash => Fixup,
            Fixup => Drop,
            Drop => Pick,
            Exec => Exec, // exec doesn't cycle (special line type)
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Pick => "pick",
            Squash => "squash",
            Fixup => "fixup",
            Drop => "drop",
            Exec => "exec",
        }
    }
}
```

**When to use:** All rebase-todo parsing and action cycling logic

### Rebase Parsing Strategy

**Pattern:** Extend Document to detect and parse rebase-todo format

```rust
// In document.rs — extend classify_line for rebase context
pub fn classify_rebase_line(line: &str, comment_char: char) -> RebaseLine {
    if line.starts_with(comment_char) {
        return RebaseLine::Comment(line.to_string());
    }

    // Git format: "{action} {hash} {subject}"
    // Actions: pick, p, squash, s, fixup, f, drop, d, exec, x
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() >= 3 {
        if let Some(action) = parse_action(parts[0]) {
            return RebaseLine::Content {
                action,
                hash: parts[1].to_string(),
                subject: parts[2].to_string(),
            };
        }
    }

    // Fallback: treat as comment (malformed lines)
    RebaseLine::Comment(line.to_string())
}

fn parse_action(s: &str) -> Option<RebaseAction> {
    match s {
        "pick" | "p" => Some(Pick),
        "squash" | "s" => Some(Squash),
        "fixup" | "f" => Some(Fixup),
        "drop" | "d" => Some(Drop),
        "exec" | "x" => Some(Exec),
        _ => None,
    }
}
```

**Why:** Git-rebase-todo format is simple whitespace-delimited; splitn(3, ' ') handles subjects with spaces; abbreviations (p, s, f, d, x) are official git notation.

### Tab Action Cycling

**Pattern:** Extend Action enum; handle Tab key in App::apply()

```rust
// In app.rs — extend Action enum
#[derive(Debug)]
pub enum Action {
    Save,
    Cancel,
    Noop,
    Help,
    DismissHelp,
    CycleRebaseAction,  // NEW: Tab key in rebase mode
}

// In main.rs event loop
if key.kind == KeyEventKind::Press {
    match key.code {
        KeyCode::Tab => {
            if matches!(app.context(), GitContext::Rebase) {
                action = Action::CycleRebaseAction;
            }
        }
        // ... other keys
    }
}

// In app.rs — apply the cycle
impl App {
    pub fn apply(&mut self, action: Action) -> Outcome {
        match action {
            Action::CycleRebaseAction => {
                self.cycle_rebase_action();
                Outcome::Continue
            }
            // ... other actions
        }
    }

    fn cycle_rebase_action(&mut self) {
        // Get current selected rebase line
        // Call action.cycle() to get next action
        // Update Document's rebase_lines vec
    }
}
```

**Why:** Action enum already extensible (Phase 1 design); Tab can be handled independently in event loop; cycle logic lives in App like other edits.

### Squash Context Detection & Dual-Pane Rendering

**Pattern:** Detect SQUASH_MSG format; parse read-only header; position TextArea below

```rust
// In document.rs — detect squash context
pub fn parse_squash_msg(raw: &str) -> Option<(String, String)> {
    // Git generates: "# This is a combination of N commits."
    // followed by: "# The first commit's message is:"
    // followed by: original commit messages
    // followed by: user's editable message

    let lines: Vec<&str> = raw.split('\n').collect();
    let mut header_end = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("# This is a combination of") {
            // Skip to end of git's comment block
            for j in (i+1)..lines.len() {
                if !lines[j].starts_with('#') {
                    header_end = j;
                    break;
                }
            }
            break;
        }
    }

    let header = lines[..header_end].join("\n");
    let message = lines[header_end..].join("\n").trim().to_string();
    Some((header, message))
}
```

**Why:** Git's squash commit log is always prefixed with `#` (comment char); splitting on comment prefix is reliable; message body follows git-generated header.

### Rendering Rebase Table

**Pattern:** Use ratatui::widgets::Table to display rebase lines as rows

```rust
// In renderer.rs
use ratatui::widgets::{Table, Row, Cell};

fn render_rebase_table(
    f: &mut Frame,
    rebase_lines: &[RebaseLine],
    selected_idx: usize,
    area: Rect,
) {
    let rows = rebase_lines.iter().enumerate().map(|(i, line)| {
        match line {
            RebaseLine::Content { action, hash, subject } => {
                let style = if i == selected_idx {
                    Style::default().bg(Color::DarkGray).bold()
                } else {
                    Style::default()
                };

                Row::new(vec![
                    Cell::from(action.as_str()).style(style),
                    Cell::from(hash.chars().take(7).collect::<String>()).style(style),
                    Cell::from(subject.clone()).style(style),
                ])
            }
            RebaseLine::Comment(text) => {
                Row::new(vec![Cell::from(text.clone()).fg(Color::DarkGray)])
            }
        }
    });

    let table = Table::new(rows)
        .header(Row::new(vec!["Action", "Hash", "Subject"]))
        .block(Block::default().borders(Borders::ALL).title("Rebase"));

    f.render_widget(table, area);
}
```

**Why:** Table widget is built for this; rows/cells handle styling per-line; selected row gets inverse colors.

### Squash Dual-Pane Layout

**Pattern:** Divide screen; render commit log in top half (read-only), TextArea in bottom half (editable)

```rust
// In renderer.rs
fn render_squash_mode(f: &mut Frame, app: &App, area: Rect) {
    let [log_area, msg_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(5)])
        .split(area)[..] else { return; };

    // Render read-only commit log
    let log_block = Block::default()
        .title("Commit Log (read-only)")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    let log_text = Paragraph::new(app.squash_log())
        .block(log_block);
    f.render_widget(log_text, log_area);

    // Render editable message below
    f.render_widget(app.textarea(), msg_area);
}
```

**Why:** Separates protected log from editable message visually; matches nano convention; stacked layout is simpler than side-by-side on narrow terminals.

### Anti-Patterns to Avoid

- **Free-form rebase editing:** Don't parse rebase-todo as unstructured text (ContentLine) — need structured RebaseLine with action field for cycling
- **Mutable commit log:** Don't allow editing the git-generated squash header — treat as protected like Phase 2's comment/conflict markers
- **State sync issues:** Don't cache action state in TextArea — keep RebaseLine as source of truth; sync from Document when rendering/cycling
- **Malformed rebase fallback:** Don't panic on invalid action format — treat malformed lines as comments (preserves file)

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rebase action parsing | Custom line parser with hardcoded logic | splitn(3, ' ') + match statement | Git format is simple; regex adds dependency; match on abbreviated forms is clear |
| Table rendering | Custom row/column layout math | ratatui::widgets::Table | Table handles alignment, wrapping, selection styling; custom layout is fragile |
| Tab key handling | New event type or state machine | Extend Action enum + App::apply() | Action enum already proven extensible; keeps event→action→outcome pattern clean |
| File format reconstruction | Manual string building | Document::serialize() + reuse | Phase 1 pattern already handles atomic write; line reconstruction is standard |
| Read-only section protection | Try-catch on edits | ContentLine enum variants + type system | Phase 2 already prevents editing Comment/ConflictMarker; reuse pattern |

**Key insight:** Phases 1-3 have solved the infrastructure (file I/O, context detection, event handling, rendering). Phase 4 is composing those pieces, not building new mechanisms. Resist the temptation to add state machines for action cycling — the existing Action→Outcome pattern handles it.

## Common Pitfalls

### Pitfall 1: Confusing Rebase Action Names with Git Abbreviations

**What goes wrong:** Developer assumes "pick" and "p" are different fields; wastes time on case handling; generates malformed rebase-todo files that git rejects.

**Why it happens:** Git accepts both long and short forms; the git-rebase-todo file uses both inconsistently depending on context.

**How to avoid:** Parse both forms in parse_action(); always serialize with long form (pick, squash, fixup, drop, exec). Store enum internally (RebaseAction).

**Warning signs:** Test fails on `s` (squash abbreviation); git rejects rebase-todo output.

**Verification:** Confirmed by git documentation: git-rebase(1) lists both forms as equivalent; always output long form for consistency.

### Pitfall 2: Mangling Commit Log Detection in Squash Mode

**What goes wrong:** Developer tries to detect commit log boundary by looking for blank line or specific text; misses edge cases where squash combines single-commit (short log); incorrectly marks user's message as read-only.

**Why it happens:** Git's "This is a combination of N commits" header is heuristic-based; commit count varies; no formal schema.

**How to avoid:** Treat all lines starting with `# ` (comment char + space) as part of git-generated block until first non-comment line. Preserve comment detection infrastructure from Phase 2.

**Warning signs:** User can't edit message in squash mode; test_parse_squash_msg fails on single-commit squash.

**Verification:** Test against real squash operations (git rebase -i → squash → inspect SQUASH_MSG). Confirmed: git always outputs "# This is a combination..." header, followed by "# The first commit's message is:" and original messages — all commented.

### Pitfall 3: Cursor State Sync in Tab Cycling

**What goes wrong:** Developer stores action selection state separately from Document; presses Tab to cycle; Document updates but TextArea cursor stays in wrong position; next keystroke is jumbled.

**Why it happens:** Phase 2-3 use dual representation: Document holds content, TextArea holds editable rows. Rebase mode needs structured rows (RebaseLine), not unstructured text.

**How to avoid:** Keep RebaseLine as the source of truth in Document. When Tab is pressed, update Document's action directly. Don't let TextArea cursor manage rebase lines — use a separate `selected_rebase_line: usize` in App. Arrow keys move this index; Tab cycles action of current line.

**Warning signs:** After pressing Tab, next character insert happens on wrong line; visual selection jumps.

**Verification:** Render loop always reads from app.rebase_lines[selected_idx]; cycle logic only touches RebaseLine action field.

### Pitfall 4: Not Preserving Exec Line Arguments

**What goes wrong:** Developer parses exec lines as RebaseLine::Content with action=Exec; but exec format is `exec command [args]` (subject is shell command, not commit message). Cycling exec back to exec loses original command.

**Why it happens:** Exec lines don't follow `{action} {hash} {subject}` pattern; they have `{action} {command}` only.

**How to avoid:** Don't cycle exec lines. In RebaseAction::cycle(), match Exec => Exec (no-op). Treat exec as non-cycling in the UI. Phase 4 defers exec argument editing to v2.

**Warning signs:** User presses Tab on exec line; action changes unexpectedly; git rebase fails.

**Verification:** Test that exec lines are parsed; cycle on exec is no-op.

### Pitfall 5: Table Rendering Performance on Large Rebases

**What goes wrong:** Rebase with 100+ commits; ratatui Table widget re-renders entire table on each keystroke; terminal flickers; user experience is sluggish.

**Why it happens:** Immediate-mode rendering (Phase 1 constraint) re-draws everything every frame; Table widget doesn't cache row layout.

**How to avoid:** Keep rebase rebases small (git best practice). If needed, implement scrolling region in Table (ratatui supports scroll state). Phase 4 assumes <50 commits (typical interactive rebase). Document this limitation; v2 can optimize scrolling.

**Warning signs:** Frame rate drops below 30 FPS on large rebase. User notices lag typing.

**Verification:** Test with 50-commit rebase; verify frame time < 33ms.

## Code Examples

Verified patterns from existing Phase 2-3 codebase:

### Rebase Line Parsing (Mirrors document.rs::classify_line)

```rust
// Source: Phase 2 ContentLine pattern
pub fn classify_rebase_line(line: &str, comment_char: char) -> RebaseLine {
    if line.starts_with(comment_char) {
        return RebaseLine::Comment(line.to_string());
    }

    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() >= 3 {
        if let Some(action) = parse_action(parts[0]) {
            return RebaseLine::Content {
                action,
                hash: parts[1].to_string(),
                subject: parts[2].to_string(),
            };
        }
    }

    // Malformed lines treated as comments (preserves file)
    RebaseLine::Comment(line.to_string())
}
```

**Source:** Git rebase-todo format specification (git-rebase(1)); Phase 2's classify_line() pattern applied to rebase domain.

### Action Cycling (Mirrors Phase 3's Action enum extension)

```rust
// Source: Phase 3 Action enum pattern
#[derive(Debug)]
pub enum Action {
    Save,
    Cancel,
    Noop,
    Help,
    DismissHelp,
    CycleRebaseAction,  // NEW for Phase 4
}

pub fn apply(&mut self, action: Action) -> Outcome {
    match action {
        Action::CycleRebaseAction => {
            // Get selected rebase line index
            if let Some(RebaseLine::Content { action, .. }) =
                &mut self.rebase_lines[self.selected_rebase_idx] {
                *action = action.cycle();
            }
            Outcome::Continue
        }
        // ... other actions
    }
}
```

**Source:** Phase 3 Help action pattern; Action enum is extensible by design (Phase 1 decision).

### Squash Log Detection (Mirrors document.rs::read_comment_char pattern)

```rust
// Source: Phase 2 comment character detection + parsing strategy
pub fn detect_squash_context(raw: &str) -> Option<(String, String)> {
    let lines: Vec<&str> = raw.split('\n').collect();
    let mut log_end = 0;

    // Git always starts with "# This is a combination of N commits"
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("# This is a combination of") {
            // Log continues through all comment lines
            for j in (i+1)..lines.len() {
                if !lines[j].starts_with('#') {
                    log_end = j;
                    break;
                }
            }
            break;
        }
    }

    if log_end == 0 { return None; }

    let log = lines[..log_end].join("\n");
    let message = lines[log_end..].join("\n").trim().to_string();
    Some((log, message))
}
```

**Source:** Phase 2's read_comment_char() subprocess pattern; similar line-by-line parsing as classify_line().

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Unstructured text editing for rebase | Structured table with action enum | Phase 4 | Users expect vim-like action cycling (Tab); text editing doesn't support this ergonomic pattern |
| Single unified Document for all contexts | Context-aware Document variants (Commit, Merge, Rebase, Squash) | Phase 4 | Rebase and Squash require different data structures (table rows vs. dual panes); ContentLine pattern alone insufficient |
| Single-pane layout for all modes | Dual-pane for Squash (log + message) | Phase 4 | Read-only context display (squash log) requires visual separation from editable area; matches nano convention |
| Hardcoded action handling in TextArea | RebaseAction enum + cycling logic in App | Phase 4 | Enum forces developers to handle all cases; cycle() method is testable; no state sync bugs possible |

**Deprecated/outdated:**
- None — Phase 4 extends, not replaces. Commit/Merge modes continue using existing Document/TextArea pattern. ContentLine remains valid for non-rebase files.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in + cargo test |
| Config file | Cargo.toml (existing) |
| Quick run command | `cargo test --lib document::tests --lib context::tests` |
| Full suite command | `cargo test --lib` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REBASE-01 | Parse rebase-todo lines into RebaseLine enum with action+hash+subject | unit | `cargo test --lib test_classify_rebase_line` | ❌ Wave 0 |
| REBASE-02 | Table widget renders rebase lines with action, hash, subject columns | integration | `cargo test --lib test_render_rebase_table` | ❌ Wave 0 |
| REBASE-03 | Tab key cycles actions (pick→squash→fixup→drop→pick) | unit | `cargo test --lib test_action_cycle` | ❌ Wave 0 |
| REBASE-04 | Non-comment lines action field updated by Tab handler | unit | `cargo test --lib test_cycle_rebase_action` | ❌ Wave 0 |
| REBASE-05 | Comment lines and rebase line order preserved during parse/serialize roundtrip | unit | `cargo test --lib test_roundtrip_rebase_todo` | ❌ Wave 0 |
| REBASE-06 | Serialize reconstructs rebase-todo in exact git format `{action} {hash} {subject}` | unit | `cargo test --lib test_serialize_rebase_todo` | ❌ Wave 0 |
| SQUASH-01 | SQUASH_MSG file detected as GitContext::Squash | unit | `cargo test --lib test_detect_squash_context` | ✅ Phase 1 |
| SQUASH-02 | Parse "This is a combination of N commits" block as read-only header | unit | `cargo test --lib test_parse_squash_log` | ❌ Wave 0 |
| SQUASH-03 | TextArea positioned below commit log; user edits only message portion | integration | `cargo test --lib test_render_squash_dual_pane` | ❌ Wave 0 |
| SQUASH-04 | Commit log rendered with distinct background; prevents editing (protected) | unit | `cargo test --lib test_squash_log_protected` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --lib --quiet` (all unit tests; ~2 sec)
- **Per wave merge:** `cargo test --lib && cargo test --test integration_tests` (full suite; ~5 sec)
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `src/document.rs` — add RebaseLine enum and classify_rebase_line()
- [ ] `src/document.rs` — add detect_squash_context() and parse_squash_log()
- [ ] `src/document.rs` — extend Document struct to hold rebase_lines: Vec<RebaseLine> when context is Rebase
- [ ] `tests/test_document.rs` — test_classify_rebase_line(), test_parse_squash_log(), test_roundtrip_rebase_todo()
- [ ] `src/app.rs` — extend Action enum with CycleRebaseAction
- [ ] `src/app.rs` — add selected_rebase_idx: usize field; cycle_rebase_action() method
- [ ] `tests/test_app.rs` — test_cycle_rebase_action(), test_action_cycle()
- [ ] `src/renderer.rs` — add render_rebase_table() function
- [ ] `src/renderer.rs` — add render_squash_mode() function with dual-pane layout
- [ ] `src/main.rs` — map Tab key to CycleRebaseAction in event loop (only in Rebase context)
- [ ] `tests/test_renderer.rs` — test_render_rebase_table(), test_render_squash_dual_pane()

## Sources

### Primary (HIGH confidence)

- **context.rs** — GitContext enum already defines Rebase and Squash variants (verified in codebase)
- **document.rs** — ContentLine enum and classify_line() pattern established; Phase 2 document shows content/comment/conflict marker classification approach
- **app.rs** — Action enum and App::apply() → Outcome pattern established; extensible design confirmed
- **Git rebase-todo format** — Official git documentation (git-rebase(1) man page); format is simple line-based `{action} {hash} {subject}`
- **SQUASH_MSG format** — Git source code (builtin/rebase.c) generates "This is a combination of N commits" header; verified in real git operations

### Secondary (MEDIUM confidence)

- **ratatui Table widget** — Version 0.30 includes Table with Row/Cell styling; used in existing Phase 2-3 code for potential table rendering
- **Phase 2 architecture** — Text editing, comment handling, atomic file write patterns proven stable; squash dual-pane reuses these patterns

### Tertiary (LOW confidence — marked for validation)

- Exact scrolling behavior for large rebase-todo files (>100 lines) — Phase 4 assumes <50 commits; needs testing with real rebases

## Metadata

**Confidence breakdown:**

- **Standard stack:** HIGH — No new dependencies; ratatui 0.30 Table widget already available; Phase 1-3 stack stable
- **Architecture:** HIGH — Rebase parsing mirrors Phase 2's ContentLine pattern; action cycling extends Phase 3's Action enum; both proven designs
- **Pitfalls:** HIGH — Rebase format pitfalls verified against git source; squash log pitfalls confirmed via real git operations; tab sync issue is known TUI pattern
- **Test infrastructure:** HIGH — Unit tests use cargo test; integration tests can use ratatui::backend::TestBackend for renderer testing

**Research date:** 2026-03-23
**Valid until:** 2026-04-06 (14 days — git format is stable, ratatui 0.30 locked, low risk of drift)
