use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Where messages for pending rewords are kept between the todo and git's reword stop.
pub fn store_dir(git_dir: &Path) -> PathBuf {
    git_dir.join("gitmedit").join("reword")
}

/// Todo files abbreviate hashes and `rebase-merge/done` does not, so either may prefix the other.
fn same_commit(a: &str, b: &str) -> bool {
    !a.is_empty() && !b.is_empty() && (a.starts_with(b) || b.starts_with(a))
}

/// The new subject followed by the body of the original message, if it had one.
pub fn replace_subject(original: Option<&str>, subject: &str) -> String {
    let body = original
        .and_then(|m| m.split_once('\n'))
        .map(|(_, body)| body.trim_matches('\n'))
        .unwrap_or("");
    if body.is_empty() {
        format!("{subject}\n")
    } else {
        format!("{subject}\n\n{body}\n")
    }
}

/// Replace every stored message with `rewords` (hash → new subject). `full_message` supplies a
/// commit's current message so its body is kept. Only hex hashes are used as file names, so a
/// hand-edited todo cannot write outside the store.
pub fn store(
    git_dir: &Path,
    rewords: &HashMap<String, String>,
    full_message: impl Fn(&str) -> Option<String>,
) -> io::Result<()> {
    let dir = store_dir(git_dir);
    let _ = fs::remove_dir_all(&dir);
    let valid: Vec<(&String, &String)> = rewords
        .iter()
        .filter(|(hash, _)| !hash.is_empty() && hash.chars().all(|c| c.is_ascii_hexdigit()))
        .collect();
    if valid.is_empty() {
        return Ok(());
    }
    fs::create_dir_all(&dir)?;
    for (hash, subject) in valid {
        fs::write(
            dir.join(hash),
            replace_subject(full_message(hash).as_deref(), subject),
        )?;
    }
    Ok(())
}

/// A commit's full message as git has it.
pub fn git_full_message(git_dir: &Path, hash: &str) -> Option<String> {
    let out = Command::new("git")
        .arg("--git-dir")
        .arg(git_dir)
        .args(["log", "-1", "--format=%B", "--end-of-options", hash])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Load stored subjects for the reword instructions of an opened todo, keyed by the todo's hash.
/// Stored messages matching none of them are left over from an earlier rebase and are deleted.
pub fn load(git_dir: &Path, reword_hashes: &[&str]) -> HashMap<String, String> {
    let mut found = HashMap::new();
    let Ok(entries) = fs::read_dir(store_dir(git_dir)) else {
        return found;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        match reword_hashes.iter().find(|hash| same_commit(hash, &name)) {
            Some(hash) => {
                if let Ok(message) = fs::read_to_string(entry.path()) {
                    found.insert(
                        hash.to_string(),
                        message.lines().next().unwrap_or("").to_string(),
                    );
                }
            }
            None => {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
    found
}

/// When git has stopped to reword a commit that has a stored message: the stored file and message.
pub fn pending_for_commit(git_dir: &Path) -> Option<(PathBuf, String)> {
    let done = fs::read_to_string(git_dir.join("rebase-merge").join("done")).ok()?;
    let last = done.lines().rev().find(|l| !l.trim().is_empty())?;
    let mut words = last.split_whitespace();
    if !matches!(words.next()?, "reword" | "r") {
        return None;
    }
    let hash = words.next()?;
    fs::read_dir(store_dir(git_dir))
        .ok()?
        .flatten()
        .find_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !same_commit(hash, &name) {
                return None;
            }
            let message = fs::read_to_string(entry.path()).ok()?;
            Some((entry.path(), message))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    }

    fn write_done(git_dir: &Path, content: &str) {
        let rebase_merge = git_dir.join("rebase-merge");
        fs::create_dir_all(&rebase_merge).unwrap();
        fs::write(rebase_merge.join("done"), content).unwrap();
    }

    fn write_stored(git_dir: &Path, name: &str, content: &str) {
        let dir = store_dir(git_dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(name), content).unwrap();
    }

    #[test]
    fn replace_subject_keeps_body() {
        assert_eq!(
            replace_subject(Some("c3\n\nbody 3\n"), "new"),
            "new\n\nbody 3\n"
        );
        assert_eq!(replace_subject(Some("c3\n"), "new"), "new\n");
        assert_eq!(replace_subject(None, "new"), "new\n");
    }

    #[test]
    fn store_writes_one_message_per_hash() {
        let dir = tempdir().unwrap();
        store(dir.path(), &map(&[("545ca5d", "new")]), |hash| {
            assert_eq!(hash, "545ca5d");
            Some("c3\n\nbody 3\n".into())
        })
        .unwrap();
        let written = fs::read_to_string(store_dir(dir.path()).join("545ca5d")).unwrap();
        assert_eq!(written, "new\n\nbody 3\n");
    }

    #[test]
    fn store_with_empty_map_removes_directory() {
        let dir = tempdir().unwrap();
        store(dir.path(), &map(&[("abc123", "x")]), |_| None).unwrap();
        assert!(store_dir(dir.path()).exists());
        store(dir.path(), &HashMap::new(), |_| None).unwrap();
        assert!(!store_dir(dir.path()).exists());
    }

    #[test]
    fn store_skips_non_hex_names() {
        let dir = tempdir().unwrap();
        store(dir.path(), &map(&[("../evil", "x")]), |_| None).unwrap();
        assert!(!store_dir(dir.path()).exists());
        assert!(!dir.path().join("gitmedit").join("evil").exists());
    }

    #[test]
    fn load_matches_by_prefix_and_deletes_stale_messages() {
        let dir = tempdir().unwrap();
        write_stored(dir.path(), "54763e6", "stored subject\n\nbody\n");
        write_stored(dir.path(), "deadbeef", "old\n");
        let loaded = load(dir.path(), &["54763e6"]);
        assert_eq!(loaded, map(&[("54763e6", "stored subject")]));
        assert!(store_dir(dir.path()).join("54763e6").exists());
        assert!(!store_dir(dir.path()).join("deadbeef").exists());
    }

    #[test]
    fn pending_reword_matches_full_sha_to_stored_short_hash() {
        let dir = tempdir().unwrap();
        write_done(
            dir.path(),
            "pick ecf1f56e4fa1606da66c4f72bea779f2fb51851f # c2\nreword 545ca5d95e7b6bbb297681be181a60d396ee8ee8 # c3\n",
        );
        write_stored(dir.path(), "545ca5d", "new\n\nbody 3\n");
        let (path, message) = pending_for_commit(dir.path()).expect("pending reword");
        assert_eq!(path, store_dir(dir.path()).join("545ca5d"));
        assert_eq!(message, "new\n\nbody 3\n");
    }

    #[test]
    fn no_pending_reword_for_pick_or_missing_done() {
        let dir = tempdir().unwrap();
        write_stored(dir.path(), "545ca5d", "new\n");
        assert!(pending_for_commit(dir.path()).is_none());
        write_done(
            dir.path(),
            "pick 545ca5d95e7b6bbb297681be181a60d396ee8ee8 # c3\n",
        );
        assert!(pending_for_commit(dir.path()).is_none());
    }
}
