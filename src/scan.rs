use crate::git;
use std::fs;
use std::path::{Path, PathBuf};

const ACTIVE_WINDOW_SECONDS: i64 = 7 * 24 * 60 * 60;

#[derive(Clone, Debug)]
pub struct Worktree {
    pub path: PathBuf,
    pub head: String,
    pub branch: Option<String>,
    pub detached: bool,
    pub bare: bool,
    pub main: bool,
    pub locked: Option<String>,
    pub prunable: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Disposition {
    Ready,
    Active,
    NeedsReview,
    Protected,
}

impl Disposition {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "Ready to clean",
            Self::Active => "Active",
            Self::NeedsReview => "Needs review",
            Self::Protected => "Protected",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Assessment {
    pub worktree: Worktree,
    pub disposition: Disposition,
    pub reasons: Vec<String>,
    pub dirty: bool,
    pub changed_files: u64,
    pub merged: bool,
    pub upstream: Option<String>,
    pub ahead: Option<u64>,
    pub behind: Option<u64>,
    pub disk_bytes: u64,
    pub last_commit_at: Option<i64>,
    pub last_commit_subject: String,
}

impl Assessment {
    pub fn cleanable(&self) -> bool {
        self.disposition == Disposition::Ready
    }
}

#[derive(Clone, Debug)]
pub struct ScanReport {
    pub repository: PathBuf,
    pub base_branch: String,
    pub worktrees: Vec<Assessment>,
}

impl ScanReport {
    pub fn reclaimable_bytes(&self) -> u64 {
        self.worktrees
            .iter()
            .filter(|item| item.cleanable())
            .map(|item| item.disk_bytes)
            .sum()
    }

    pub fn ready(&self) -> impl Iterator<Item = &Assessment> {
        self.worktrees.iter().filter(|item| item.cleanable())
    }
}

pub fn scan_repository(repository: &Path) -> Result<ScanReport, String> {
    let root = PathBuf::from(git::text(repository, &["rev-parse", "--show-toplevel"])?.trim());
    let raw = git::text(&root, &["worktree", "list", "--porcelain"])?;
    let worktrees = parse_worktrees(&raw)?;
    let base_branch = default_branch(&root)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs() as i64;
    let worktrees = worktrees
        .into_iter()
        .map(|worktree| assess(&worktree, &base_branch, now))
        .collect();
    Ok(ScanReport {
        repository: root,
        base_branch,
        worktrees,
    })
}

fn parse_worktrees(raw: &str) -> Result<Vec<Worktree>, String> {
    let mut worktrees = Vec::new();
    for block in raw.split("\n\n").filter(|block| !block.trim().is_empty()) {
        let mut worktree = Worktree {
            path: PathBuf::new(),
            head: String::new(),
            branch: None,
            detached: false,
            bare: false,
            main: worktrees.is_empty(),
            locked: None,
            prunable: None,
        };
        for line in block.lines() {
            if let Some(path) = line.strip_prefix("worktree ") {
                worktree.path = PathBuf::from(path);
            } else if let Some(head) = line.strip_prefix("HEAD ") {
                worktree.head = head.to_owned();
            } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
                worktree.branch = Some(branch.to_owned());
            } else if line == "detached" {
                worktree.detached = true;
            } else if line == "bare" {
                worktree.bare = true;
            } else if let Some(reason) = line.strip_prefix("locked") {
                worktree.locked = Some(reason.trim().to_owned());
            } else if let Some(reason) = line.strip_prefix("prunable") {
                worktree.prunable = Some(reason.trim().to_owned());
            }
        }
        if worktree.path.as_os_str().is_empty() || worktree.head.is_empty() {
            return Err("Git returned an incomplete worktree entry".to_owned());
        }
        worktrees.push(worktree);
    }
    Ok(worktrees)
}

fn default_branch(repository: &Path) -> Result<String, String> {
    for candidate in ["main", "master", "trunk"] {
        let reference = format!("refs/heads/{candidate}");
        if git::run(repository, &["show-ref", "--verify", "--quiet", &reference]).is_ok() {
            return Ok(candidate.to_owned());
        }
    }
    if let Some(branch) = git::optional_text(repository, &["symbolic-ref", "--short", "HEAD"]) {
        return Ok(branch);
    }
    if let Some(branch) = git::optional_text(
        repository,
        &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    ) {
        return Ok(branch);
    }
    Err("Alice could not find main, master, trunk, or a current base branch".to_owned())
}

fn assess(worktree: &Worktree, base_branch: &str, now: i64) -> Assessment {
    let mut assessment = Assessment {
        worktree: worktree.clone(),
        disposition: Disposition::Protected,
        reasons: Vec::new(),
        dirty: false,
        changed_files: 0,
        merged: false,
        upstream: None,
        ahead: None,
        behind: None,
        disk_bytes: 0,
        last_commit_at: None,
        last_commit_subject: String::new(),
    };
    if worktree.main {
        assessment
            .reasons
            .push("Primary checkout; Alice never removes it".to_owned());
        return assessment;
    }
    if worktree.bare {
        assessment
            .reasons
            .push("Bare worktree; manual inspection required".to_owned());
        return assessment;
    }
    if let Some(reason) = &worktree.locked {
        assessment.reasons.push(if reason.is_empty() {
            "Locked by Git".to_owned()
        } else {
            format!("Locked by Git: {reason}")
        });
        return assessment;
    }
    if let Some(reason) = &worktree.prunable {
        assessment
            .reasons
            .push(format!("Registered path needs attention: {reason}"));
        return assessment;
    }
    if !worktree.path.is_dir() {
        assessment
            .reasons
            .push("Worktree directory is missing".to_owned());
        return assessment;
    }

    assessment.disk_bytes = directory_size(&worktree.path).unwrap_or(0);
    let status = match git::text(
        &worktree.path,
        &["status", "--porcelain=v1", "--untracked-files=all"],
    ) {
        Ok(status) => status,
        Err(error) => {
            assessment
                .reasons
                .push(format!("Could not read Git status: {error}"));
            return assessment;
        }
    };
    assessment.changed_files = status.lines().count() as u64;
    assessment.dirty = assessment.changed_files > 0;
    assessment.merged = git::run(
        &worktree.path,
        &["merge-base", "--is-ancestor", "HEAD", base_branch],
    )
    .map(|_| true)
    .unwrap_or(false);
    assessment.upstream = git::optional_text(
        &worktree.path,
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    );
    if assessment.upstream.is_some() {
        if let Ok(counts) = git::text(
            &worktree.path,
            &["rev-list", "--left-right", "--count", "HEAD...@{upstream}"],
        ) {
            let mut counts = counts.split_whitespace();
            assessment.ahead = counts.next().and_then(|value| value.parse().ok());
            assessment.behind = counts.next().and_then(|value| value.parse().ok());
        }
    }
    if let Ok(log) = git::text(&worktree.path, &["log", "-1", "--format=%ct%x00%s"]) {
        let mut parts = log.trim_end().splitn(2, '\0');
        assessment.last_commit_at = parts.next().and_then(|value| value.parse().ok());
        assessment.last_commit_subject = parts.next().unwrap_or("No commit subject").to_owned();
    }

    if worktree.detached {
        assessment
            .reasons
            .push("Detached HEAD; Alice needs a branch identity".to_owned());
    }
    if assessment.dirty {
        assessment.reasons.push(format!(
            "{} changed or untracked files are preserved",
            assessment.changed_files
        ));
    }
    if !assessment.merged {
        assessment
            .reasons
            .push(format!("HEAD is not contained in {base_branch}"));
    }
    if assessment.ahead.is_some_and(|ahead| ahead > 0) {
        assessment.reasons.push(format!(
            "{} commits are not pushed to its upstream",
            assessment.ahead.unwrap_or(0)
        ));
    }
    if assessment.upstream.is_none() && !assessment.merged {
        assessment
            .reasons
            .push("No upstream tracks this unmerged work".to_owned());
    }

    let unpushed = assessment.ahead.is_some_and(|ahead| ahead > 0);
    assessment.disposition =
        if !worktree.detached && !assessment.dirty && assessment.merged && !unpushed {
            assessment
                .reasons
                .push(format!("Clean and fully contained in {base_branch}"));
            assessment
                .reasons
                .push("Only the worktree directory would be removed; the branch stays".to_owned());
            Disposition::Ready
        } else if assessment.reasons.is_empty() {
            Disposition::NeedsReview
        } else if !assessment.dirty
            && !assessment.merged
            && !unpushed
            && assessment
                .last_commit_at
                .is_some_and(|at| now - at <= ACTIVE_WINDOW_SECONDS)
        {
            assessment
                .reasons
                .push("Recent committed work remains active".to_owned());
            Disposition::Active
        } else {
            Disposition::NeedsReview
        };
    assessment
}

pub(crate) fn directory_size(path: &Path) -> Result<u64, String> {
    let mut total = 0;
    for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            total += directory_size(&entry.path())?;
        } else {
            total += metadata.len();
        }
    }
    Ok(total)
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
