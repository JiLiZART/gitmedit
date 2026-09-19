use crate::{message, wrap};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SectionKind {
    Branch,
    Merge,
    Rebase,
    Squash,
    Conflicts,
    Staged,
    Unstaged,
    SubmodulesStaged,
    SubmodulesNotUpdated,
    Untracked,
    Other,
    Diff,
}

impl SectionKind {
    pub fn title(self) -> &'static str {
        match self {
            SectionKind::Branch => "Branch",
            SectionKind::Merge => "Merge",
            SectionKind::Rebase => "Rebase",
            SectionKind::Squash => "Squash",
            SectionKind::Conflicts => "Conflicts",
            SectionKind::Staged => "Staged",
            SectionKind::Unstaged => "Unstaged",
            SectionKind::SubmodulesStaged => "Submodules staged",
            SectionKind::SubmodulesNotUpdated => "Submodules not updated",
            SectionKind::Untracked => "Untracked",
            SectionKind::Other => "Other",
            SectionKind::Diff => "Diff",
        }
    }

    /// Sections whose header shows how many entries they list.
    pub fn counted(self) -> bool {
        matches!(
            self,
            SectionKind::Conflicts
                | SectionKind::Staged
                | SectionKind::Unstaged
                | SectionKind::SubmodulesStaged
                | SectionKind::SubmodulesNotUpdated
                | SectionKind::Untracked
        )
    }

    fn from_heading(t: &str) -> Option<Self> {
        Some(match t {
            "Conflicts:" | "Unmerged paths:" => SectionKind::Conflicts,
            "Changes to be committed:" => SectionKind::Staged,
            "Changes not staged for commit:" => SectionKind::Unstaged,
            "Submodule changes to be committed:" => SectionKind::SubmodulesStaged,
            "Submodules changed but not updated:" => SectionKind::SubmodulesNotUpdated,
            "Untracked files:" => SectionKind::Untracked,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    File { badge: char, path: String, suffix: Option<String> },
    Submodule(String),
    Commit(String),
    Warning(String),
    Text(String),
    DiffLine(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub kind: SectionKind,
    pub entries: Vec<Entry>,
}

impl Section {
    pub fn count(&self) -> usize {
        self.entries.iter().filter(|e| matches!(e, Entry::File { .. } | Entry::Submodule(_))).count()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Status {
    pub branch: Option<String>,
    pub sections: Vec<Section>,
}

impl Status {
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    #[cfg(test)]
    pub fn section(&self, kind: SectionKind) -> Option<&Section> {
        self.sections.iter().find(|s| s.kind == kind)
    }
}

const BOILERPLATE: &[&str] = &[
    "If this is not correct, please run",
    "git update-ref -d MERGE_HEAD",
    "and try again.",
    "Do not modify or remove the line above.",
    "Everything below it will be ignored.",
];

#[derive(Default)]
struct BranchInfo {
    name: Option<String>,
    upstream: Option<String>,
    ahead: Option<u32>,
    behind: Option<u32>,
    up_to_date: bool,
    extra: Vec<String>,
}

pub fn parse(trailer: &[String], cc: char) -> Status {
    let mut sections: Vec<Section> = Vec::new();
    let mut branch = BranchInfo::default();
    let mut current: Option<SectionKind> = None;
    let mut in_diff = false;

    for line in trailer {
        if in_diff {
            if !line.starts_with(cc) {
                push(&mut sections, SectionKind::Diff, Entry::DiffLine(line.clone()));
            }
            continue;
        }
        if message::is_scissors(line, cc) {
            section_mut(&mut sections, SectionKind::Diff);
            in_diff = true;
            continue;
        }
        let Some(rest) = line.strip_prefix(cc) else { continue };
        let t = rest.trim();
        if t.is_empty() || is_boilerplate(t) {
            continue;
        }

        if let Some(kind) = SectionKind::from_heading(t) {
            section_mut(&mut sections, kind);
            current = Some(kind);
        } else if parse_branch_line(t, &mut branch) {
            current = None;
        } else if let Some(kind) = state_line_kind(t) {
            push(&mut sections, kind, Entry::Text(t.to_string()));
            let lists_commands = t.starts_with("Last command") || t.starts_with("Next command");
            current = lists_commands.then_some(kind);
        } else {
            match current {
                Some(kind @ (SectionKind::Conflicts | SectionKind::Staged | SectionKind::Unstaged | SectionKind::Untracked)) => {
                    push(&mut sections, kind, parse_file(t, kind))
                }
                Some(kind @ (SectionKind::SubmodulesStaged | SectionKind::SubmodulesNotUpdated)) => {
                    push(&mut sections, kind, parse_submodule_line(t))
                }
                Some(SectionKind::Rebase) => push(&mut sections, SectionKind::Rebase, Entry::Commit(t.to_string())),
                _ => {
                    let verbatim = rest.strip_prefix(' ').unwrap_or(rest).trim_end();
                    push(&mut sections, SectionKind::Other, Entry::Text(verbatim.to_string()))
                }
            }
        }
    }

    let summary = branch_summary(&branch);
    if summary.is_some() || !branch.extra.is_empty() {
        let s = section_mut(&mut sections, SectionKind::Branch);
        s.entries.extend(summary.map(Entry::Text));
        s.entries.extend(branch.extra.iter().cloned().map(Entry::Text));
    }
    sections.sort_by_key(|s| s.kind);
    Status { branch: branch.name, sections }
}

fn section_mut(sections: &mut Vec<Section>, kind: SectionKind) -> &mut Section {
    let idx = match sections.iter().position(|s| s.kind == kind) {
        Some(i) => i,
        None => {
            sections.push(Section { kind, entries: Vec::new() });
            sections.len() - 1
        }
    };
    &mut sections[idx]
}

fn push(sections: &mut Vec<Section>, kind: SectionKind, entry: Entry) {
    section_mut(sections, kind).entries.push(entry);
}

fn is_boilerplate(t: &str) -> bool {
    t.starts_with("Please enter the commit message")
        || t.starts_with("Please enter a commit message")
        || t.starts_with("with '")
        || t.starts_with("Lines starting with")
        || t.starts_with("Write a message for tag")
        || (t.starts_with('(') && t.ends_with(')'))
        || BOILERPLATE.contains(&t)
}

fn state_line_kind(t: &str) -> Option<SectionKind> {
    if t.starts_with("It looks like you may be committing a merge")
        || t.starts_with("All conflicts fixed but you are still merging")
        || t.starts_with("You have unmerged paths")
    {
        Some(SectionKind::Merge)
    } else if t.starts_with("interactive rebase in progress")
        || t.starts_with("Last command")
        || t.starts_with("Next command")
        || t.starts_with("No commands remaining")
        || t.starts_with("You are currently")
    {
        Some(SectionKind::Rebase)
    } else if t.starts_with("This is a combination of")
        || t.starts_with("This is the ")
        || (t.starts_with("The ") && t.contains("commit message"))
    {
        Some(SectionKind::Squash)
    } else {
        None
    }
}

fn parse_branch_line(t: &str, b: &mut BranchInfo) -> bool {
    if let Some(name) = t.strip_prefix("On branch ") {
        b.name = Some(name.to_string());
    } else if let Some(rev) = t.strip_prefix("HEAD detached at ").or_else(|| t.strip_prefix("HEAD detached from ")) {
        b.name = Some(format!("detached {rev}"));
    } else if t.starts_with("Your branch is ahead of ") {
        b.upstream = quoted(t);
        b.ahead = count_after_by(t);
    } else if t.starts_with("Your branch is behind ") {
        b.upstream = quoted(t);
        b.behind = count_after_by(t);
    } else if t.starts_with("Your branch is up to date with ") {
        b.upstream = quoted(t);
        b.up_to_date = true;
    } else if t.starts_with("Your branch and ") && t.contains("have diverged") {
        b.upstream = quoted(t);
    } else if let Some(rest) = t.strip_prefix("and have ") {
        let nums: Vec<u32> = rest.split_whitespace().filter_map(|w| w.parse().ok()).collect();
        if let [ahead, behind, ..] = nums.as_slice() {
            b.ahead = Some(*ahead);
            b.behind = Some(*behind);
        }
    } else if let Some(v) = t.strip_prefix("Date:") {
        b.extra.push(format!("Date: {}", v.trim()));
    } else if let Some(v) = t.strip_prefix("Author:") {
        b.extra.push(format!("Author: {}", v.trim()));
    } else if t.starts_with("Your branch is based on ") || t.starts_with("Committer:") {
        b.extra.push(t.to_string());
    } else {
        return false;
    }
    true
}

fn quoted(t: &str) -> Option<String> {
    let start = t.find('\'')? + 1;
    let len = t[start..].find('\'')?;
    Some(t[start..start + len].to_string())
}

fn count_after_by(t: &str) -> Option<u32> {
    t.split(" by ").nth(1)?.split_whitespace().next()?.parse().ok()
}

fn branch_summary(b: &BranchInfo) -> Option<String> {
    let mut s = b.name.clone()?;
    if let Some(upstream) = &b.upstream {
        s.push_str(&format!(" → {upstream}"));
    }
    match (b.ahead, b.behind) {
        (Some(a), Some(bh)) => s.push_str(&format!("  ahead {a}, behind {bh}")),
        (Some(a), None) => s.push_str(&format!("  ahead {a}")),
        (None, Some(bh)) => s.push_str(&format!("  behind {bh}")),
        (None, None) if b.up_to_date => s.push_str("  up to date"),
        _ => {}
    }
    Some(s)
}

fn parse_file(t: &str, kind: SectionKind) -> Entry {
    let parsed = t
        .split_once(':')
        .and_then(|(status, rest)| Some((status_badge(status)?, rest.trim())));
    let (badge, rest) = match parsed {
        Some(found) => found,
        None => (if kind == SectionKind::Untracked { '?' } else { 'U' }, t),
    };
    let (path, suffix) = match rest.rsplit_once(" (") {
        Some((p, s)) if s.ends_with(')') => (p.to_string(), Some(format!("({s}"))),
        _ => (rest.to_string(), None),
    };
    Entry::File { badge, path, suffix }
}

fn status_badge(status: &str) -> Option<char> {
    Some(match status {
        "modified" => 'M',
        "new file" => 'A',
        "deleted" => 'D',
        "renamed" => 'R',
        "copied" => 'C',
        "typechange" => 'T',
        "both modified" | "both added" | "both deleted" | "added by us" | "added by them"
        | "deleted by us" | "deleted by them" | "unmerged" => 'U',
        _ => return None,
    })
}

fn parse_submodule_line(t: &str) -> Entry {
    if let Some(s) = t.strip_prefix("* ") {
        Entry::Submodule(s.to_string())
    } else if t.starts_with("> ") || t.starts_with("< ") {
        Entry::Commit(t.to_string())
    } else if t.starts_with("Warn:") {
        Entry::Warning(t.to_string())
    } else {
        Entry::Text(t.to_string())
    }
}

pub fn badge_style(badge: char) -> Style {
    match badge {
        'A' => Style::default().fg(Color::Green),
        'D' => Style::default().fg(Color::Red),
        'M' => Style::default().fg(Color::Yellow),
        'U' => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        'R' | 'C' | 'T' => Style::default().fg(Color::Blue),
        _ => Style::default().fg(Color::DarkGray),
    }
}

/// Styled rows for the status pane, wrapped to `width` columns.
pub fn render_lines(status: &Status, width: usize) -> Vec<Line<'static>> {
    let header = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let mut out: Vec<Line<'static>> = Vec::new();
    for section in &status.sections {
        if !out.is_empty() {
            out.push(Line::default());
        }
        let title = if section.kind.counted() {
            format!("{} ({})", section.kind.title(), section.count())
        } else {
            section.kind.title().to_string()
        };
        out.push(Line::from(Span::styled(title, header)));
        for entry in &section.entries {
            push_entry(&mut out, entry, width);
        }
    }
    out
}

fn push_entry(out: &mut Vec<Line<'static>>, entry: &Entry, width: usize) {
    match entry {
        Entry::File { badge, path, suffix } => {
            let rows = wrap::wrap(path, width.saturating_sub(3));
            let last = rows.len() - 1;
            for (i, row) in rows.into_iter().enumerate() {
                let mut spans = if i == 0 {
                    vec![Span::raw(" "), Span::styled(badge.to_string(), badge_style(*badge)), Span::raw(" "), Span::raw(row.text)]
                } else {
                    vec![Span::raw("   "), Span::raw(row.text)]
                };
                // ponytail: the suffix is appended, not wrapped; a very narrow pane clips it
                if i == last
                    && let Some(suffix) = suffix
                {
                    spans.push(Span::styled(format!(" {suffix}"), Style::default().fg(Color::DarkGray)));
                }
                out.push(Line::from(spans));
            }
        }
        Entry::Submodule(s) => out.push(Line::from(Span::styled(format!(" * {s}"), Style::default().fg(Color::Magenta)))),
        Entry::Commit(s) => push_wrapped(out, s, "   ", width, Style::default()),
        Entry::Warning(s) => push_wrapped(out, s, "   ", width, Style::default().fg(Color::Yellow)),
        Entry::Text(s) => push_wrapped(out, s, " ", width, Style::default()),
        Entry::DiffLine(s) => out.push(Line::from(Span::styled(s.clone(), diff_style(s)))),
    }
}

fn push_wrapped(out: &mut Vec<Line<'static>>, text: &str, indent: &'static str, width: usize, style: Style) {
    for row in wrap::wrap(text, width.saturating_sub(indent.len())) {
        out.push(Line::from(vec![Span::raw(indent), Span::styled(row.text, style)]));
    }
}

fn diff_style(s: &str) -> Style {
    if s.starts_with("+++") || s.starts_with("---") {
        Style::default().add_modifier(Modifier::BOLD)
    } else if s.starts_with('+') {
        Style::default().fg(Color::Green)
    } else if s.starts_with('-') {
        Style::default().fg(Color::Red)
    } else if s.starts_with("@@") {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;
    use ratatui::text::Line;

    fn parse_fixture(raw: &str) -> Status {
        parse(&message::split(raw, '#').trailer, '#')
    }

    fn kinds(status: &Status) -> Vec<SectionKind> {
        status.sections.iter().map(|s| s.kind).collect()
    }

    fn owned(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn line_text(line: &Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn amend_with_diverged_branch_and_staged_files() {
        let s = parse_fixture(include_str!("../fixtures/ammend_fixture.txt"));
        assert_eq!(kinds(&s), vec![SectionKind::Branch, SectionKind::Staged]);
        assert_eq!(s.branch.as_deref(), Some("EX-3211-arc-epic"));
        let branch = s.section(SectionKind::Branch).unwrap();
        assert_eq!(branch.entries[0], Entry::Text("EX-3211-arc-epic → origin/EX-3211-arc-epic  ahead 1, behind 2".into()));
        assert_eq!(branch.entries[1], Entry::Text("Date: Tue May 12 01:43:54 2026 +0200".into()));
        assert_eq!(s.section(SectionKind::Staged).unwrap().count(), 34);
        assert!(!format!("{:?}", s.sections).contains("Please enter"));
    }

    #[test]
    fn amend_ahead_of_upstream() {
        let s = parse_fixture(include_str!("../fixtures/ammend2_fixture.txt"));
        let branch = s.section(SectionKind::Branch).unwrap();
        assert_eq!(branch.entries[0], Entry::Text("TASK-1111-fix-stage-view → origin/TASK-1111-fix-stage-view  ahead 1".into()));
        assert_eq!(s.section(SectionKind::Staged).unwrap().count(), 6);
    }

    #[test]
    fn merge_with_conflicts_many_deletions_and_submodules() {
        let s = parse_fixture(include_str!("../fixtures/merge_fixture2.txt"));
        assert_eq!(
            kinds(&s),
            vec![
                SectionKind::Branch,
                SectionKind::Merge,
                SectionKind::Conflicts,
                SectionKind::Staged,
                SectionKind::Unstaged,
                SectionKind::SubmodulesStaged,
                SectionKind::SubmodulesNotUpdated,
            ]
        );
        assert_eq!(s.section(SectionKind::Conflicts).unwrap().count(), 4);
        assert_eq!(s.section(SectionKind::Staged).unwrap().count(), 160);
        assert_eq!(s.section(SectionKind::Unstaged).unwrap().count(), 1);
        assert_eq!(s.section(SectionKind::SubmodulesStaged).unwrap().count(), 1);
        assert_eq!(s.section(SectionKind::SubmodulesNotUpdated).unwrap().count(), 1);
        assert_eq!(
            s.section(SectionKind::Unstaged).unwrap().entries[0],
            Entry::File { badge: 'M', path: "packages/@dev-kit".into(), suffix: Some("(new commits)".into()) }
        );
    }

    #[test]
    fn rebase_in_progress() {
        let s = parse_fixture(include_str!("../fixtures/rebase_fixture.txt"));
        assert_eq!(
            kinds(&s),
            vec![
                SectionKind::Branch,
                SectionKind::Rebase,
                SectionKind::Conflicts,
                SectionKind::Staged,
                SectionKind::Unstaged,
                SectionKind::SubmodulesStaged,
                SectionKind::SubmodulesNotUpdated,
                SectionKind::Untracked,
            ]
        );
        let rebase = &s.section(SectionKind::Rebase).unwrap().entries;
        assert_eq!(rebase[0], Entry::Text("interactive rebase in progress; onto c3d42a1f4".into()));
        assert_eq!(rebase[2], Entry::Commit("pick c0ad61ef0 # Update chat message stop words".into()));
        assert_eq!(s.section(SectionKind::Unstaged).unwrap().count(), 6);
        let not_updated = s.section(SectionKind::SubmodulesNotUpdated).unwrap();
        assert_eq!(not_updated.count(), 5);
        assert_eq!(not_updated.entries.iter().filter(|e| matches!(e, Entry::Warning(_))).count(), 4);
        assert_eq!(
            s.section(SectionKind::Untracked).unwrap().entries,
            vec![Entry::File { badge: '?', path: "vendor/money-tree/".into(), suffix: None }]
        );
    }

    #[test]
    fn conflicts_only() {
        let s = parse_fixture(include_str!("../fixtures/pull_rebase_fixture.txt"));
        assert_eq!(kinds(&s), vec![SectionKind::Conflicts]);
        let conflicts = s.section(SectionKind::Conflicts).unwrap();
        assert_eq!(conflicts.count(), 6);
        assert!(conflicts.entries.iter().all(|e| matches!(e, Entry::File { badge: 'U', .. })));
    }

    #[test]
    fn file_statuses_map_to_badges() {
        let trailer = owned(&[
            "# Changes to be committed:",
            "#\tmodified:   a.rs",
            "#\tnew file:   b.rs",
            "#\tdeleted:    c.rs",
            "#\trenamed:    old.rs -> new.rs",
            "#\tboth modified:   d.rs",
        ]);
        let entries = &parse(&trailer, '#').sections[0].entries;
        let badges: Vec<char> = entries
            .iter()
            .map(|e| match e {
                Entry::File { badge, .. } => *badge,
                _ => ' ',
            })
            .collect();
        assert_eq!(badges, vec!['M', 'A', 'D', 'R', 'U']);
        assert_eq!(entries[3], Entry::File { badge: 'R', path: "old.rs -> new.rs".into(), suffix: None });
    }

    #[test]
    fn heading_without_entries_is_kept_with_zero_count() {
        let s = parse(&owned(&["# Changes to be committed:", "#"]), '#');
        assert_eq!(s.section(SectionKind::Staged).unwrap().count(), 0);
    }

    #[test]
    fn unrecognized_comment_lines_go_to_other() {
        let s = parse(&owned(&["# something unexpected"]), '#');
        assert_eq!(s.section(SectionKind::Other).unwrap().entries, vec![Entry::Text("something unexpected".into())]);
    }

    #[test]
    fn diff_below_scissors() {
        let s = parse(
            &owned(&[
                "# Please enter the commit message for your changes. Lines starting",
                "# ------------------------ >8 ------------------------",
                "# Do not modify or remove the line above.",
                "diff --git a/x b/x",
                "+added",
            ]),
            '#',
        );
        assert_eq!(kinds(&s), vec![SectionKind::Diff]);
        assert_eq!(
            s.section(SectionKind::Diff).unwrap().entries,
            vec![Entry::DiffLine("diff --git a/x b/x".into()), Entry::DiffLine("+added".into())]
        );
    }

    #[test]
    fn custom_comment_char() {
        let s = parse(&owned(&["; Changes to be committed:", ";\tmodified:   a.rs"]), ';');
        assert_eq!(s.section(SectionKind::Staged).unwrap().count(), 1);
    }

    #[test]
    fn empty_trailer_is_empty_status() {
        assert!(parse(&[], '#').is_empty());
    }

    fn sample() -> Status {
        parse(
            &owned(&[
                "# On branch main",
                "# Changes to be committed:",
                "#\tmodified:   packages/@dev-kit (new commits)",
                "#\tdeleted:    old.rs",
                "# Untracked files:",
                "#\tnotes.txt",
            ]),
            '#',
        )
    }

    #[test]
    fn headers_show_counts_and_sections_are_separated() {
        let lines: Vec<String> = render_lines(&sample(), 80).iter().map(line_text).collect();
        assert_eq!(lines[0], "Branch");
        assert_eq!(lines[1], " main");
        assert_eq!(lines[2], "");
        assert_eq!(lines[3], "Staged (2)");
        assert_eq!(lines[4], " M packages/@dev-kit (new commits)");
        assert_eq!(lines[5], " D old.rs");
        assert_eq!(lines[7], "Untracked (1)");
        assert_eq!(lines[8], " ? notes.txt");
    }

    #[test]
    fn deleted_badge_is_red_and_suffix_is_dim() {
        let lines = render_lines(&sample(), 80);
        assert_eq!(lines[5].spans[1].content, "D");
        assert_eq!(lines[5].spans[1].style.fg, Some(Color::Red));
        assert_eq!(lines[4].spans.last().unwrap().style.fg, Some(Color::DarkGray));
    }

    #[test]
    fn long_paths_wrap_with_indent() {
        let status = parse(&owned(&["# Changes to be committed:", "#\tmodified:   aaaa/bbbb/cccc/dddd"]), '#');
        // width 12 leaves 9 path columns; no spaces, so the path hard-breaks every 9 chars
        let lines: Vec<String> = render_lines(&status, 12).iter().map(line_text).collect();
        assert_eq!(lines, vec!["Staged (1)", " M aaaa/bbbb", "   /cccc/ddd", "   d"]);
    }

    #[test]
    fn diff_lines_are_colored() {
        let status = parse(&owned(&["# ------------------------ >8 ------------------------", "+added", "-removed", "@@ -1 +1 @@"]), '#');
        let lines = render_lines(&status, 80);
        assert_eq!(lines[1].spans[0].style.fg, Some(Color::Green));
        assert_eq!(lines[2].spans[0].style.fg, Some(Color::Red));
        assert_eq!(lines[3].spans[0].style.fg, Some(Color::Cyan));
    }
}
