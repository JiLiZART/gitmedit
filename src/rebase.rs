#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Pick,
    Reword,
    Edit,
    Squash,
    Fixup,
    Drop,
}

impl Action {
    pub fn parse(word: &str) -> Option<Self> {
        Some(match word {
            "pick" | "p" => Action::Pick,
            "reword" | "r" => Action::Reword,
            "edit" | "e" => Action::Edit,
            "squash" | "s" => Action::Squash,
            "fixup" | "f" => Action::Fixup,
            "drop" | "d" => Action::Drop,
            _ => return None,
        })
    }

    /// Letter keys p r e s f d set the action directly.
    pub fn from_key(c: char) -> Option<Self> {
        match c {
            'p' | 'r' | 'e' | 's' | 'f' | 'd' => Self::parse(&c.to_string()),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Action::Pick => "pick",
            Action::Reword => "reword",
            Action::Edit => "edit",
            Action::Squash => "squash",
            Action::Fixup => "fixup",
            Action::Drop => "drop",
        }
    }

    /// Tab cycle: pick → squash → fixup → drop → pick; reword and edit jump to squash.
    pub fn cycled(self) -> Self {
        match self {
            Action::Pick | Action::Reword | Action::Edit => Action::Squash,
            Action::Squash => Action::Fixup,
            Action::Fixup => Action::Drop,
            Action::Drop => Action::Pick,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommitLine {
    pub action: Action,
    /// `-C` or `-c` on a fixup.
    pub flag: Option<String>,
    pub hash: String,
    /// Everything after the hash, verbatim. git writes `# subject`.
    pub rest: String,
    raw: String,
    original_action: Action,
    original_flag: Option<String>,
}

impl CommitLine {
    /// Subject for display: `rest` without git's leading `# `.
    pub fn subject(&self) -> &str {
        self.rest.strip_prefix("# ").unwrap_or(&self.rest)
    }

    pub fn set_action(&mut self, action: Action) {
        self.action = action;
        if action != Action::Fixup {
            self.flag = None;
        }
    }

    fn serialize(&self) -> String {
        if self.action == self.original_action && self.flag == self.original_flag {
            return self.raw.clone();
        }
        let mut parts = vec![self.action.as_str()];
        if let Some(flag) = &self.flag {
            parts.push(flag);
        }
        parts.push(&self.hash);
        if !self.rest.is_empty() {
            parts.push(&self.rest);
        }
        parts.join(" ")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TodoLine {
    Commit(CommitLine),
    /// exec, break, label, reset, merge, update-ref, noop: kept verbatim.
    Other(String),
    Comment(String),
    Blank(String),
    Unknown(String),
}

impl TodoLine {
    pub fn is_instruction(&self) -> bool {
        matches!(self, TodoLine::Commit(_) | TodoLine::Other(_))
    }

    fn serialize(&self) -> String {
        match self {
            TodoLine::Commit(c) => c.serialize(),
            TodoLine::Other(s)
            | TodoLine::Comment(s)
            | TodoLine::Blank(s)
            | TodoLine::Unknown(s) => s.clone(),
        }
    }
}

const OTHER_COMMANDS: &[&str] = &[
    "exec",
    "x",
    "break",
    "b",
    "label",
    "l",
    "reset",
    "t",
    "merge",
    "m",
    "update-ref",
    "u",
    "noop",
];

#[derive(Debug, Clone, PartialEq)]
pub struct Todo {
    pub lines: Vec<TodoLine>,
    pub final_newline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Summary {
    pub total: usize,
    pub result: usize,
    pub squash_fixup: usize,
    pub drop: usize,
    pub reword: usize,
    pub first_is_squash: bool,
}

pub fn parse(raw: &str, cc: char) -> Todo {
    let final_newline = raw.ends_with('\n');
    let body = raw.strip_suffix('\n').unwrap_or(raw);
    let lines = if raw.is_empty() {
        Vec::new()
    } else {
        body.split('\n').map(|l| parse_line(l, cc)).collect()
    };
    Todo {
        lines,
        final_newline,
    }
}

fn parse_line(line: &str, cc: char) -> TodoLine {
    let t = line.trim_start();
    if t.is_empty() {
        return TodoLine::Blank(line.to_string());
    }
    if t.starts_with(cc) {
        return TodoLine::Comment(line.to_string());
    }
    let (word, after) = split_word(t);
    if let Some(action) = Action::parse(word) {
        let (flag, after) = match after.split_at_checked(3) {
            Some((f @ ("-C " | "-c "), rest)) if action == Action::Fixup => {
                (Some(f.trim_end().to_string()), rest.trim_start())
            }
            _ => (None, after),
        };
        let (hash, rest) = split_word(after);
        if hash.is_empty() {
            return TodoLine::Unknown(line.to_string());
        }
        return TodoLine::Commit(CommitLine {
            action,
            flag: flag.clone(),
            hash: hash.to_string(),
            rest: rest.to_string(),
            raw: line.to_string(),
            original_action: action,
            original_flag: flag,
        });
    }
    if OTHER_COMMANDS.contains(&word) {
        TodoLine::Other(line.to_string())
    } else {
        TodoLine::Unknown(line.to_string())
    }
}

fn split_word(s: &str) -> (&str, &str) {
    match s.find(char::is_whitespace) {
        Some(i) => (&s[..i], s[i..].trim_start()),
        None => (s, ""),
    }
}

impl Todo {
    pub fn serialize(&self) -> String {
        let mut s = self
            .lines
            .iter()
            .map(TodoLine::serialize)
            .collect::<Vec<_>>()
            .join("\n");
        if self.final_newline {
            s.push('\n');
        }
        s
    }

    /// The range git names in its `Rebase a..b onto c` comment.
    pub fn range(&self, cc: char) -> Option<String> {
        self.lines.iter().find_map(|l| match l {
            TodoLine::Comment(c) => c
                .trim_start()
                .strip_prefix(cc)?
                .trim()
                .strip_prefix("Rebase ")?
                .split_whitespace()
                .next()
                .map(str::to_string),
            _ => None,
        })
    }

    /// Indices into `lines` of instructions (commit and other commands), in file order.
    pub fn instruction_indices(&self) -> Vec<usize> {
        self.lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.is_instruction())
            .map(|(i, _)| i)
            .collect()
    }

    pub fn commit(&self, line: usize) -> Option<&CommitLine> {
        match self.lines.get(line)? {
            TodoLine::Commit(c) => Some(c),
            _ => None,
        }
    }

    pub fn commit_mut(&mut self, line: usize) -> Option<&mut CommitLine> {
        match self.lines.get_mut(line)? {
            TodoLine::Commit(c) => Some(c),
            _ => None,
        }
    }

    /// Swap the instruction at `line` with the previous or next instruction. Comments, blanks and
    /// unrecognized lines keep their positions. Returns the moved instruction's new line index.
    pub fn move_instruction(&mut self, line: usize, up: bool) -> Option<usize> {
        let instructions = self.instruction_indices();
        let pos = instructions.iter().position(|&i| i == line)?;
        let target = if up {
            *instructions.get(pos.checked_sub(1)?)?
        } else {
            *instructions.get(pos + 1)?
        };
        self.lines.swap(line, target);
        Some(target)
    }

    pub fn summary(&self) -> Summary {
        let commits: Vec<&CommitLine> = self
            .lines
            .iter()
            .filter_map(|l| match l {
                TodoLine::Commit(c) => Some(c),
                _ => None,
            })
            .collect();
        let count = |pred: fn(Action) -> bool| commits.iter().filter(|c| pred(c.action)).count();
        Summary {
            total: commits.len(),
            result: count(|a| matches!(a, Action::Pick | Action::Reword | Action::Edit)),
            squash_fixup: count(|a| matches!(a, Action::Squash | Action::Fixup)),
            drop: count(|a| a == Action::Drop),
            reword: count(|a| a == Action::Reword),
            first_is_squash: commits
                .first()
                .is_some_and(|c| matches!(c.action, Action::Squash | Action::Fixup)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TODO: &str = include_str!("../fixtures/squash_fixture.txt");

    fn commit(todo: &Todo, i: usize) -> &CommitLine {
        match &todo.lines[i] {
            TodoLine::Commit(c) => c,
            other => panic!("line {i} is not a commit: {other:?}"),
        }
    }

    fn hashes(todo: &Todo) -> Vec<String> {
        todo.lines
            .iter()
            .filter_map(|l| match l {
                TodoLine::Commit(c) => Some(c.hash.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn fixture_round_trips_byte_for_byte() {
        assert_eq!(parse(TODO, '#').serialize(), TODO);
    }

    #[test]
    fn parses_commit_instruction() {
        let todo = parse(TODO, '#');
        let c = commit(&todo, 0);
        assert_eq!(c.action, Action::Pick);
        assert_eq!(c.hash, "54763e6");
        assert_eq!(c.rest, "# docs(state): record phase 8 context session");
        assert_eq!(c.subject(), "docs(state): record phase 8 context session");
    }

    #[test]
    fn classifies_comments_blanks_other_and_unknown() {
        let todo = parse(
            "pick abc Fix\n\n# comment\nexec cargo test\nbreak\ngarbage line\n",
            '#',
        );
        assert!(matches!(todo.lines[1], TodoLine::Blank(_)));
        assert!(matches!(todo.lines[2], TodoLine::Comment(_)));
        assert_eq!(todo.lines[3], TodoLine::Other("exec cargo test".into()));
        assert_eq!(todo.lines[4], TodoLine::Other("break".into()));
        assert_eq!(todo.lines[5], TodoLine::Unknown("garbage line".into()));
        assert!(todo.lines[3].is_instruction() && !todo.lines[5].is_instruction());
    }

    #[test]
    fn abbreviated_actions_parse_and_round_trip() {
        let todo = parse("p abc1234 Fix bug\nf def5678 Tidy\n", '#');
        assert_eq!(commit(&todo, 0).action, Action::Pick);
        assert_eq!(commit(&todo, 1).action, Action::Fixup);
        assert_eq!(todo.serialize(), "p abc1234 Fix bug\nf def5678 Tidy\n");
    }

    #[test]
    fn fixup_flag_is_parsed() {
        let todo = parse("fixup -C 7314ba6 subject\n", '#');
        let c = commit(&todo, 0);
        assert_eq!(c.flag.as_deref(), Some("-C"));
        assert_eq!(c.hash, "7314ba6");
        assert_eq!(todo.serialize(), "fixup -C 7314ba6 subject\n");
    }

    #[test]
    fn changed_action_rewrites_only_that_line() {
        let mut todo = parse(TODO, '#');
        todo.commit_mut(1).unwrap().set_action(Action::Fixup);
        let out = todo.serialize();
        let diff: Vec<(&str, &str)> = TODO
            .lines()
            .zip(out.lines())
            .filter(|(a, b)| a != b)
            .collect();
        assert_eq!(
            diff,
            vec![(
                "pick 45f8bcd # docs(08): create phase plan",
                "fixup 45f8bcd # docs(08): create phase plan"
            )]
        );
    }

    #[test]
    fn leaving_fixup_drops_the_flag() {
        let mut todo = parse("fixup -c 7314ba6 subject\n", '#');
        todo.commit_mut(0).unwrap().set_action(Action::Pick);
        assert_eq!(todo.serialize(), "pick 7314ba6 subject\n");
    }

    #[test]
    fn empty_and_newline_only_files_round_trip() {
        assert_eq!(parse("", '#').serialize(), "");
        assert_eq!(parse("\n", '#').serialize(), "\n");
        assert_eq!(parse("pick abc x", '#').serialize(), "pick abc x");
    }

    #[test]
    fn range_comes_from_rebase_comment() {
        assert_eq!(
            parse(TODO, '#').range('#').as_deref(),
            Some("36d7eda..aa619f8")
        );
        assert_eq!(parse("pick abc x\n", '#').range('#'), None);
    }

    #[test]
    fn keys_and_cycle() {
        assert_eq!(Action::from_key('r'), Some(Action::Reword));
        assert_eq!(Action::from_key('x'), None);
        assert_eq!(Action::Pick.cycled(), Action::Squash);
        assert_eq!(Action::Squash.cycled(), Action::Fixup);
        assert_eq!(Action::Fixup.cycled(), Action::Drop);
        assert_eq!(Action::Drop.cycled(), Action::Pick);
        assert_eq!(Action::Reword.cycled(), Action::Squash);
        assert_eq!(Action::Edit.cycled(), Action::Squash);
    }

    #[test]
    fn instruction_indices_skip_comments_blanks_and_unknown() {
        let todo = parse("pick a x\n# c\n\nexec make\ngarbage\npick b y\n", '#');
        assert_eq!(todo.instruction_indices(), vec![0, 3, 5]);
    }

    #[test]
    fn moving_swaps_instructions_and_leaves_comments_in_place() {
        let mut todo = parse("pick A a\n# comment\npick B b\n", '#');
        assert_eq!(todo.move_instruction(2, true), Some(0));
        assert_eq!(todo.serialize(), "pick B b\n# comment\npick A a\n");
    }

    #[test]
    fn moving_past_the_ends_does_nothing() {
        let mut todo = parse("pick A a\npick B b\n", '#');
        assert_eq!(todo.move_instruction(0, true), None);
        assert_eq!(todo.move_instruction(1, false), None);
        assert_eq!(todo.serialize(), "pick A a\npick B b\n");
    }

    #[test]
    fn many_moves_never_lose_or_duplicate_instructions() {
        let mut todo = parse(TODO, '#');
        let mut before = hashes(&todo);
        let mut line = 0;
        for step in 0..40 {
            line = match todo.move_instruction(line, step % 3 == 0) {
                Some(new_line) => new_line,
                None => todo.instruction_indices()[step % 14],
            };
        }
        let mut after = hashes(&todo);
        before.sort();
        after.sort();
        assert_eq!(before, after);
    }

    #[test]
    fn summary_counts_resulting_commits() {
        let mut todo = parse(TODO, '#');
        todo.commit_mut(1).unwrap().set_action(Action::Squash);
        todo.commit_mut(2).unwrap().set_action(Action::Squash);
        todo.commit_mut(3).unwrap().set_action(Action::Drop);
        todo.commit_mut(4).unwrap().set_action(Action::Reword);
        let s = todo.summary();
        assert_eq!(
            (s.total, s.result, s.squash_fixup, s.drop, s.reword),
            (14, 11, 2, 1, 1)
        );
        assert!(!s.first_is_squash);
    }

    #[test]
    fn summary_flags_leading_squash() {
        let mut todo = parse(TODO, '#');
        todo.commit_mut(0).unwrap().set_action(Action::Fixup);
        assert!(todo.summary().first_is_squash);
    }

    #[test]
    fn commit_accessors_only_return_commit_lines() {
        let mut todo = parse("pick a x\nexec make\n", '#');
        assert!(todo.commit(0).is_some());
        assert!(todo.commit(1).is_none());
        assert!(todo.commit_mut(9).is_none());
    }
}
