//! Puts the commit hash in the environment so the statusline can show true
//! data rather than a decorative placeholder.
//!
//! Falls back through: an explicit `GIT_COMMIT`, which is how a build without
//! a repository supplies it; then `git rev-parse`. When neither works the
//! variable stays unset and the statusline omits the segment, because a
//! hardcoded hash is the detail a technical visitor checks.

use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=.git/HEAD");
    println!("cargo::rerun-if-env-changed=GIT_COMMIT");

    if let Some(hash) = std::env::var("GIT_COMMIT")
        .ok()
        .filter(|hash| !hash.trim().is_empty())
        .or_else(git_hash)
    {
        println!("cargo::rustc-env=GIT_COMMIT={}", hash.trim());
    }
}

fn git_hash() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;

    output
        .status
        .success()
        .then(|| String::from_utf8(output.stdout).ok())
        .flatten()
}
