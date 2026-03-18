use std::ffi::OsStr;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum GitContext {
    Commit,
    Merge,
    Rebase,
    Squash,
    Unknown,
}

pub fn detect_context(path: &Path) -> GitContext {
    match path.file_name().and_then(OsStr::to_str) {
        Some("COMMIT_EDITMSG") | Some("TAG_EDITMSG") => GitContext::Commit,
        Some("MERGE_MSG") => GitContext::Merge,
        Some("git-rebase-todo") => GitContext::Rebase,
        Some("SQUASH_MSG") => GitContext::Squash,
        _ => GitContext::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_editmsg_returns_commit() {
        assert_eq!(
            detect_context(Path::new("COMMIT_EDITMSG")),
            GitContext::Commit
        );
    }

    #[test]
    fn test_merge_msg_returns_merge() {
        assert_eq!(detect_context(Path::new("MERGE_MSG")), GitContext::Merge);
    }

    #[test]
    fn test_rebase_todo_returns_rebase() {
        assert_eq!(
            detect_context(Path::new("git-rebase-todo")),
            GitContext::Rebase
        );
    }

    #[test]
    fn test_squash_msg_returns_squash() {
        assert_eq!(
            detect_context(Path::new("SQUASH_MSG")),
            GitContext::Squash
        );
    }

    #[test]
    fn test_tag_editmsg_returns_commit() {
        assert_eq!(
            detect_context(Path::new("TAG_EDITMSG")),
            GitContext::Commit
        );
    }

    #[test]
    fn test_unknown_file_returns_unknown() {
        assert_eq!(
            detect_context(Path::new("unknown.txt")),
            GitContext::Unknown
        );
    }

    #[test]
    fn test_full_path_uses_only_filename() {
        assert_eq!(
            detect_context(Path::new("/tmp/.git/COMMIT_EDITMSG")),
            GitContext::Commit
        );
        assert_eq!(
            detect_context(Path::new("/some/other/path/unknown.txt")),
            GitContext::Unknown
        );
    }
}
