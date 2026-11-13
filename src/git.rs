use std::path::Path;
use std::process::{Command, Output};

pub(crate) fn run(repo: &Path, args: &[&str]) -> Result<Output, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|error| format!("Could not run git: {error}"))?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

pub(crate) fn text(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = run(repo, args)?;
    String::from_utf8(output.stdout).map_err(|_| "Git returned a non-UTF-8 response".to_owned())
}

pub(crate) fn optional_text(repo: &Path, args: &[&str]) -> Option<String> {
    text(repo, args)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
