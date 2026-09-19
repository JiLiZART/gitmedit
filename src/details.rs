use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug, Clone, PartialEq)]
pub struct CommitDetails {
    pub message: Vec<String>,
    /// (status letter, path) — renames and copies read `old -> new`.
    pub files: Vec<(char, String)>,
}

/// Parse the output of `git show --format=%B%x00 --name-status`.
pub fn parse_show(out: &str) -> CommitDetails {
    let (message, files) = out.split_once('\0').unwrap_or((out, ""));
    let message = message.trim_end().lines().map(str::to_string).collect();
    let files = files
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let status = parts.next()?.chars().next()?;
            let paths: Vec<&str> = parts.collect();
            (!paths.is_empty()).then(|| (status, paths.join(" -> ")))
        })
        .collect();
    CommitDetails { message, files }
}

/// Loads commit details on a background thread so the table never waits on git.
// ponytail: one worker serves requests in order; holding Down queues one git call per commit.
pub struct DetailsLoader {
    requests: Sender<String>,
    results: Receiver<(String, Option<CommitDetails>)>,
    cache: HashMap<String, Option<CommitDetails>>,
    requested: HashSet<String>,
}

impl DetailsLoader {
    pub fn spawn(git_dir: Option<PathBuf>) -> Self {
        let (requests, request_rx) = mpsc::channel::<String>();
        let (result_tx, results) = mpsc::channel();
        std::thread::spawn(move || {
            for hash in request_rx {
                let details = fetch(git_dir.as_deref(), &hash);
                if result_tx.send((hash, details)).is_err() {
                    break;
                }
            }
        });
        Self {
            requests,
            results,
            cache: HashMap::new(),
            requested: HashSet::new(),
        }
    }

    /// Ask for a commit's details; repeated requests for the same hash are ignored.
    pub fn request(&mut self, hash: &str) {
        if self.requested.insert(hash.to_string()) {
            let _ = self.requests.send(hash.to_string());
        }
    }

    /// Move finished lookups into the cache. Returns true when anything arrived.
    pub fn poll(&mut self) -> bool {
        let mut changed = false;
        while let Ok((hash, details)) = self.results.try_recv() {
            self.cache.insert(hash, details);
            changed = true;
        }
        changed
    }

    /// `None` while loading, `Some(None)` when git could not provide details.
    pub fn get(&self, hash: &str) -> Option<&Option<CommitDetails>> {
        self.cache.get(hash)
    }
}

fn fetch(git_dir: Option<&Path>, hash: &str) -> Option<CommitDetails> {
    let mut cmd = Command::new("git");
    if let Some(dir) = git_dir {
        cmd.arg("--git-dir").arg(dir);
    }
    let out = cmd
        .args([
            "show",
            "--no-color",
            "--format=%B%x00",
            "--name-status",
            "--end-of-options",
            hash,
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| parse_show(&String::from_utf8_lossy(&out.stdout)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn parses_message_and_name_status() {
        let d = parse_show("subject\n\nbody\n\0\nA\ta.txt\nM\tsrc/b.rs\nR100\told.rs\tnew.rs\n");
        assert_eq!(d.message, vec!["subject", "", "body"]);
        assert_eq!(
            d.files,
            vec![
                ('A', "a.txt".into()),
                ('M', "src/b.rs".into()),
                ('R', "old.rs -> new.rs".into())
            ]
        );
    }

    #[test]
    fn commit_without_files() {
        let d = parse_show("subject\n\0\n");
        assert_eq!(d.message, vec!["subject"]);
        assert!(d.files.is_empty());
    }

    fn wait_for(loader: &mut DetailsLoader, hash: &str) -> Option<Option<CommitDetails>> {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            loader.poll();
            if let Some(details) = loader.get(hash) {
                return Some(details.clone());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        None
    }

    #[test]
    fn loader_fetches_details_from_git() {
        let dir = tempfile::tempdir().unwrap();
        let git = |args: &[&str]| {
            Command::new("git")
                .current_dir(dir.path())
                .args(args)
                .stdin(Stdio::null())
                .output()
        };
        let Ok(init) = git(&["init", "-q"]) else {
            return;
        };
        if !init.status.success() {
            return;
        }
        std::fs::write(dir.path().join("a.txt"), "a").unwrap();
        git(&["add", "a.txt"]).unwrap();
        let commit = git(&[
            "-c",
            "commit.gpgsign=false",
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            "subject",
            "-m",
            "body",
        ])
        .unwrap();
        assert!(
            commit.status.success(),
            "{}",
            String::from_utf8_lossy(&commit.stderr)
        );

        let mut loader = DetailsLoader::spawn(Some(dir.path().join(".git")));
        loader.request("HEAD");
        loader.request("0000000");
        let details = wait_for(&mut loader, "HEAD")
            .flatten()
            .expect("details loaded");
        assert_eq!(details.message, vec!["subject", "", "body"]);
        assert_eq!(details.files, vec![('A', "a.txt".to_string())]);
        assert_eq!(wait_for(&mut loader, "0000000"), Some(None));
    }
}
