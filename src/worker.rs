use crate::{Assessment, RustTarget};
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkerProvider {
    Kimi,
    Claude,
    Copilot,
}

impl WorkerProvider {
    pub const ALL: [Self; 3] = [Self::Kimi, Self::Claude, Self::Copilot];

    pub fn id(self) -> &'static str {
        match self {
            Self::Kimi => "kimi",
            Self::Claude => "claude",
            Self::Copilot => "copilot",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Kimi => "Kimi Code",
            Self::Claude => "Claude Code",
            Self::Copilot => "GitHub Copilot",
        }
    }

    pub fn cli(self) -> &'static str {
        self.id()
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|provider| provider.id() == id)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerStatus {
    Unavailable,
    Ready,
}

#[derive(Clone, Debug)]
pub struct WorkerOutcome {
    pub status: WorkerStatus,
    pub summary: String,
}

pub fn configured_worker() -> WorkerProvider {
    worker_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|id| WorkerProvider::from_id(id.trim()))
        .unwrap_or(WorkerProvider::Kimi)
}

pub fn set_worker_provider(provider: WorkerProvider) -> Result<(), String> {
    let path = worker_path().ok_or_else(|| "Alice could not find the config folder".to_owned())?;
    let parent = path
        .parent()
        .ok_or_else(|| "Alice could not find the config folder".to_owned())?;
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    std::fs::write(path, format!("{}\n", provider.id())).map_err(|error| error.to_string())
}

pub fn worker_available(provider: WorkerProvider) -> bool {
    Command::new(provider.cli())
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

pub fn available_workers() -> Vec<WorkerProvider> {
    WorkerProvider::ALL
        .into_iter()
        .filter(|provider| worker_available(*provider))
        .collect()
}

pub fn summarize_with_kimi(assessment: &Assessment) -> Result<WorkerOutcome, String> {
    summarize_with_provider(WorkerProvider::Kimi, assessment)
}

pub fn summarize_with_provider(
    provider: WorkerProvider,
    assessment: &Assessment,
) -> Result<WorkerOutcome, String> {
    if assessment.cleanable() || assessment.worktree.main {
        return Err("Worker summaries are reserved for non-cleanable linked worktrees".to_owned());
    }
    let prompt = format!(
        "You are Alice, a careful Git worktree assistant. Explain this worktree in at most 80 words. Do not tell the user to delete it, do not run commands, and do not inspect files. Git facts: branch {:?}; status {}; dirty={}; changed_files={}; merged_into_base={}; upstream {:?}; ahead {:?}; behind {:?}; last commit {:?}; reasons: {}. End with one question the user should answer before deciding what to do.",
        assessment.worktree.branch,
        assessment.disposition.label(),
        assessment.dirty,
        assessment.changed_files,
        assessment.merged,
        assessment.upstream,
        assessment.ahead,
        assessment.behind,
        assessment.last_commit_subject,
        assessment.reasons.join("; ")
    );
    run_prompt(provider, prompt)
}

pub fn explain_rust_target(target: &RustTarget) -> Result<WorkerOutcome, String> {
    explain_rust_target_with_provider(WorkerProvider::Kimi, target)
}

pub fn explain_rust_target_with_provider(
    provider: WorkerProvider,
    target: &RustTarget,
) -> Result<WorkerOutcome, String> {
    let prompt = format!(
        "You are Alice, a careful build-artifact assistant. Explain this Cargo target in at most 100 words. Say what it is, why Alice considers it cleanable or protected, which generated entries would be removed, what would be preserved, and what happens on the next build. Do not change the eligibility, do not recommend deletion, do not run commands, and do not inspect files. Facts: path {:?}; observed_bytes={}; generated_reclaimable_bytes={}; cleanable={}; repositories {:?}; checkout_count={}; reasons: {}. End with one concrete consideration for the user.",
        target.path,
        target.disk_bytes,
        target.reclaimable_bytes,
        target.cleanable(),
        target.repositories,
        target.checkouts.len(),
        target.reasons.join("; ")
    );
    run_prompt(provider, prompt)
}

fn run_prompt(provider: WorkerProvider, prompt: String) -> Result<WorkerOutcome, String> {
    let output = match Command::new(provider.cli())
        .args(prompt_args(provider, &prompt))
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            return Ok(WorkerOutcome {
                status: WorkerStatus::Unavailable,
                summary: format!("{} could not be started: {error}", provider.label()),
            });
        }
    };
    if !output.status.success() {
        return Ok(WorkerOutcome {
            status: WorkerStatus::Unavailable,
            summary: format!(
                "{} could not answer: {}",
                provider.label(),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        });
    }
    Ok(WorkerOutcome {
        status: WorkerStatus::Ready,
        summary: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
    })
}

fn prompt_args<'a>(provider: WorkerProvider, prompt: &'a str) -> Vec<&'a str> {
    match provider {
        WorkerProvider::Kimi => vec!["--prompt", prompt, "--output-format", "text"],
        WorkerProvider::Claude => vec!["-p", prompt, "--tools", "", "--output-format", "text"],
        WorkerProvider::Copilot => vec![
            "-p",
            prompt,
            "--available-tools",
            "",
            "--no-ask-user",
            "--no-custom-instructions",
            "--output-format",
            "text",
        ],
    }
}

fn worker_path() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|config| config.join("alice").join("worker"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn providers_have_distinct_noninteractive_tool_free_arguments() {
        assert_eq!(
            prompt_args(WorkerProvider::Kimi, "test"),
            ["--prompt", "test", "--output-format", "text"]
        );
        let claude = prompt_args(WorkerProvider::Claude, "test");
        assert!(claude.contains(&"-p"));
        assert!(claude.contains(&"--tools"));
        let copilot = prompt_args(WorkerProvider::Copilot, "test");
        assert!(copilot.contains(&"-p"));
        assert!(copilot.contains(&"--available-tools"));
        assert!(copilot.contains(&"--no-custom-instructions"));
    }
}
