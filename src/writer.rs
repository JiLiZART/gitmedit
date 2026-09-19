use anyhow::Context as _;
use std::path::Path;

/// Stateless file writer with atomic write semantics.
pub struct FileWriter;

impl FileWriter {
    /// Write `content` to `path` atomically.
    ///
    /// Writes to `{path}.tmp` first, then renames to `path`. The original file
    /// is never opened for truncation — if the write or rename fails, the
    /// original file remains untouched.
    ///
    /// Line endings are written as-is (Unix `\n`); no CRLF normalization.
    pub fn write_atomic(content: &str, path: &Path) -> anyhow::Result<()> {
        let tmp_path = path.with_extension("tmp");
        std::fs::write(&tmp_path, content.as_bytes())
            .with_context(|| format!("writing temporary file {:?}", tmp_path))?;
        std::fs::rename(&tmp_path, path).with_context(|| format!("writing {:?}", path))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn write_atomic_creates_file_with_correct_content() {
        let dir = tempdir().expect("failed to create tempdir");
        let target = dir.path().join("COMMIT_EDITMSG");

        FileWriter::write_atomic("hello\n", &target).expect("write_atomic should succeed");

        let actual = fs::read_to_string(&target).expect("should be able to read file");
        assert_eq!(actual, "hello\n");
    }

    #[test]
    fn write_atomic_removes_tmp_file_on_success() {
        let dir = tempdir().expect("failed to create tempdir");
        let target = dir.path().join("COMMIT_EDITMSG");

        FileWriter::write_atomic("hello\n", &target).expect("write_atomic should succeed");

        let tmp = target.with_extension("tmp");
        assert!(
            !tmp.exists(),
            ".tmp file should not exist after successful write"
        );
    }

    #[test]
    fn write_atomic_preserves_unix_line_endings() {
        let dir = tempdir().expect("failed to create tempdir");
        let target = dir.path().join("COMMIT_EDITMSG");
        let content = "line1\nline2\nline3\n";

        FileWriter::write_atomic(content, &target).expect("write_atomic should succeed");

        let bytes = fs::read(&target).expect("should be able to read file");
        // Verify no CRLF sequences were introduced
        assert!(
            !bytes.windows(2).any(|w| w == b"\r\n"),
            "no CRLF should be introduced"
        );
        assert_eq!(bytes, content.as_bytes());
    }
}
