use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum GitContext {
    Commit,
    Merge,
    Rebase,
    Squash,
    Unknown,
}

pub fn detect_context(_path: &Path) -> GitContext {
    GitContext::Unknown
}
