use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GitContext {
    Commit,
    Merge,
    Rebase,
    Squash,
    Tag,
    Unknown,
}

impl GitContext {
    /// Commit, merge, squash, and tag files share the message layout.
    pub fn is_message(self) -> bool {
        matches!(
            self,
            GitContext::Commit | GitContext::Merge | GitContext::Squash | GitContext::Tag
        )
    }
}

pub fn detect_context(path: &Path) -> GitContext {
    match path.file_name().and_then(OsStr::to_str) {
        Some("COMMIT_EDITMSG") => GitContext::Commit,
        Some("TAG_EDITMSG") => GitContext::Tag,
        Some("MERGE_MSG") => GitContext::Merge,
        Some("git-rebase-todo") => GitContext::Rebase,
        Some("SQUASH_MSG") => GitContext::Squash,
        _ => GitContext::Unknown,
    }
}

/// The git directory holding `path`: the parent for message files, the parent of
/// `rebase-merge` for a todo, and none for unknown files.
pub fn git_dir(path: &Path, context: GitContext) -> Option<PathBuf> {
    let parent = path.parent()?;
    match context {
        GitContext::Unknown => None,
        GitContext::Rebase => parent.parent().map(Path::to_path_buf),
        _ => Some(parent.to_path_buf()),
    }
}

/// Read `core.commentChar` from git, falling back to `#` on any failure.
pub fn read_comment_char() -> char {
    let output = Command::new("git")
        .args(["config", "--get", "core.commentchar"])
        .stdin(Stdio::null())
        .output();
    match output {
        Ok(o) if o.status.success() => parse_comment_char(&String::from_utf8_lossy(&o.stdout)),
        _ => '#',
    }
}

/// Empty and `auto` fall back to `#`, since automatic selection is not supported.
pub fn parse_comment_char(value: &str) -> char {
    match value.trim() {
        "" | "auto" => '#',
        v => v.chars().next().unwrap_or('#'),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_each_git_file() {
        assert_eq!(
            detect_context(Path::new("COMMIT_EDITMSG")),
            GitContext::Commit
        );
        assert_eq!(detect_context(Path::new("MERGE_MSG")), GitContext::Merge);
        assert_eq!(
            detect_context(Path::new("git-rebase-todo")),
            GitContext::Rebase
        );
        assert_eq!(detect_context(Path::new("SQUASH_MSG")), GitContext::Squash);
        assert_eq!(detect_context(Path::new("TAG_EDITMSG")), GitContext::Tag);
        assert_eq!(
            detect_context(Path::new("unknown.txt")),
            GitContext::Unknown
        );
    }

    #[test]
    fn only_the_filename_matters() {
        assert_eq!(
            detect_context(Path::new("/tmp/.git/COMMIT_EDITMSG")),
            GitContext::Commit
        );
        assert_eq!(
            detect_context(Path::new("/some/path/unknown.txt")),
            GitContext::Unknown
        );
    }

    #[test]
    fn message_contexts_share_the_message_layout() {
        for ctx in [
            GitContext::Commit,
            GitContext::Merge,
            GitContext::Squash,
            GitContext::Tag,
        ] {
            assert!(ctx.is_message(), "{ctx:?}");
        }
        assert!(!GitContext::Rebase.is_message());
        assert!(!GitContext::Unknown.is_message());
    }

    #[test]
    fn git_dir_for_message_file_in_worktree() {
        let p = Path::new("/repo/.git/worktrees/feature/COMMIT_EDITMSG");
        assert_eq!(
            git_dir(p, GitContext::Commit),
            Some(PathBuf::from("/repo/.git/worktrees/feature"))
        );
    }

    #[test]
    fn git_dir_for_rebase_todo_is_parent_of_rebase_merge() {
        let p = Path::new("/repo/.git/rebase-merge/git-rebase-todo");
        assert_eq!(
            git_dir(p, GitContext::Rebase),
            Some(PathBuf::from("/repo/.git"))
        );
    }

    #[test]
    fn git_dir_for_unknown_file_is_none() {
        assert_eq!(
            git_dir(Path::new("/tmp/notes.txt"), GitContext::Unknown),
            None
        );
    }

    #[test]
    fn comment_char_parsing() {
        assert_eq!(parse_comment_char(";\n"), ';');
        assert_eq!(parse_comment_char(""), '#');
        assert_eq!(parse_comment_char("auto\n"), '#');
    }
}
