//! Running gitmedit with no arguments delegates to `git commit`, which decides the outcome.
//!
//! Both tests return early when git cannot be run, so they do not fail on a machine without git.

use std::path::Path;
use std::process::{Command, Stdio};

fn git(dir: &Path, args: &[&str]) -> Option<std::process::Output> {
    Command::new("git")
        .current_dir(dir)
        .args(args)
        // Never let git open a real editor while testing.
        .env("GIT_EDITOR", "true")
        .stdin(Stdio::null())
        .output()
        .ok()
}

fn gitmedit(dir: &Path) -> Option<std::process::Output> {
    Command::new(env!("CARGO_BIN_EXE_gitmedit"))
        .current_dir(dir)
        .env("GIT_CEILING_DIRECTORIES", dir.parent().unwrap_or(dir))
        .stdin(Stdio::null())
        .output()
        .ok()
}

#[test]
fn nothing_staged_reports_what_git_reports() {
    let dir = tempfile::tempdir().expect("temp dir");
    let Some(init) = git(dir.path(), &["init", "-q"]) else {
        return;
    };
    if !init.status.success() {
        return;
    }
    git(dir.path(), &["config", "user.email", "t@t"]);
    git(dir.path(), &["config", "user.name", "t"]);

    let expected = git(dir.path(), &["commit"]).expect("git commit");
    let actual = gitmedit(dir.path()).expect("gitmedit");

    assert_eq!(
        actual.status.code(),
        expected.status.code(),
        "exit code should be git's"
    );
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&actual.stdout),
        String::from_utf8_lossy(&actual.stderr)
    );
    assert!(
        text.contains("nothing to commit"),
        "expected git's report, got: {text}"
    );
}

#[test]
fn outside_a_repository_reports_gits_error() {
    let dir = tempfile::tempdir().expect("temp dir");
    let Some(out) = gitmedit(dir.path()) else {
        return;
    };

    assert!(!out.status.success(), "should fail outside a repository");
    let text = String::from_utf8_lossy(&out.stderr).to_lowercase();
    assert!(
        text.contains("not a git repository"),
        "expected git's error, got: {text}"
    );
}
