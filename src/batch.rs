use crate::{Assessment, CleanupResult, ScanReport, cleanup_worktrees, scan_repository};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug)]
pub struct DiskUsage {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Clone, Debug)]
pub struct ScanBatch {
    pub location: PathBuf,
    pub reports: Vec<ScanReport>,
    pub disk: Option<DiskUsage>,
    pub warnings: Vec<String>,
    pub rust: RustTargetUsage,
}

#[derive(Clone, Debug)]
pub struct RustTarget {
    pub path: PathBuf,
    pub disk_bytes: u64,
    pub reclaimable_bytes: u64,
    pub repositories: Vec<PathBuf>,
    pub checkouts: Vec<PathBuf>,
    pub reasons: Vec<String>,
    pub eligible: bool,
}

impl RustTarget {
    pub fn cleanable(&self) -> bool {
        self.eligible
    }

    pub fn in_use(&self) -> bool {
        rust_target_in_use(self)
    }
}

#[derive(Clone, Debug, Default)]
pub struct RustTargetUsage {
    pub bytes: u64,
    pub target_count: usize,
    pub disk_total_bytes: Option<u64>,
    pub targets: Vec<RustTarget>,
}

impl RustTargetUsage {
    pub fn reclaimable_bytes(&self) -> u64 {
        self.targets
            .iter()
            .filter(|target| target.cleanable())
            .map(|target| target.reclaimable_bytes)
            .sum()
    }
}

impl ScanBatch {
    pub fn worktrees(&self) -> impl Iterator<Item = (&ScanReport, &Assessment)> {
        self.reports.iter().flat_map(|report| {
            report
                .worktrees
                .iter()
                .map(move |assessment| (report, assessment))
        })
    }

    pub fn linked(&self) -> impl Iterator<Item = (&ScanReport, &Assessment)> {
        self.worktrees()
            .filter(|(_, assessment)| !assessment.worktree.main)
    }

    pub fn reclaimable_bytes(&self) -> u64 {
        self.reports.iter().map(ScanReport::reclaimable_bytes).sum()
    }

    pub fn rust_targets(&self) -> &RustTargetUsage {
        &self.rust
    }

    fn observe_rust_targets(mut self) -> Self {
        self.rust = rust_target_usage(self.worktrees().map(|(report, assessment)| {
            (
                assessment.worktree.path.as_path(),
                report.repository.as_path(),
            )
        }));
        self
    }
}

pub fn scan_location(location: &Path) -> Result<ScanBatch, String> {
    scan_location_repositories(location).map(ScanBatch::observe_rust_targets)
}

fn scan_location_repositories(location: &Path) -> Result<ScanBatch, String> {
    let location = if location.is_absolute() {
        location.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|error| error.to_string())?
            .join(location)
    };
    if let Ok(report) = scan_repository(&location) {
        return Ok(ScanBatch {
            disk: disk_usage(&location),
            location,
            reports: vec![report],
            warnings: Vec::new(),
            rust: RustTargetUsage::default(),
        });
    }
    if !location.is_dir() {
        return Err(format!("{} is not a folder", location.display()));
    }
    let mut candidates = Vec::new();
    discover_repositories(&location, 6, &mut candidates)?;
    let mut reports = Vec::new();
    let mut warnings = Vec::new();
    for repository in candidates {
        match scan_repository(&repository) {
            Ok(report) => reports.push(report),
            Err(error) => warnings.push(format!("{}: {error}", repository.display())),
        }
    }
    if reports.is_empty() {
        return Err(format!(
            "No Git repositories found under {}",
            location.display()
        ));
    }
    Ok(ScanBatch {
        disk: disk_usage(&location),
        location,
        reports,
        warnings,
        rust: RustTargetUsage::default(),
    })
}

pub fn scan_locations(locations: &[PathBuf]) -> Result<ScanBatch, String> {
    let Some(first) = locations.first() else {
        return Err("Add at least one folder for Alice to scan".to_owned());
    };
    let mut reports = Vec::new();
    let mut warnings = Vec::new();
    let mut seen_repositories = BTreeSet::new();
    let mut disks: BTreeMap<(String, String), DiskUsage> = BTreeMap::new();
    for location in locations {
        match scan_location_repositories(location) {
            Ok(batch) => {
                warnings.extend(batch.warnings);
                for report in batch.reports {
                    if seen_repositories.insert(report.repository.clone()) {
                        reports.push(report);
                    }
                }
                if let Some((filesystem, mount, usage)) = disk_usage_details(&batch.location) {
                    disks.entry((filesystem, mount)).or_insert(usage);
                }
            }
            Err(error) => warnings.push(error),
        }
    }
    if reports.is_empty() {
        return Err(warnings.join("\n"));
    }
    let disk = if disks.is_empty() {
        None
    } else {
        Some(DiskUsage {
            total_bytes: disks.values().map(|disk| disk.total_bytes).sum(),
            used_bytes: disks.values().map(|disk| disk.used_bytes).sum(),
            available_bytes: disks.values().map(|disk| disk.available_bytes).sum(),
        })
    };
    Ok(ScanBatch {
        location: first.clone(),
        reports,
        disk,
        warnings,
        rust: RustTargetUsage::default(),
    }
    .observe_rust_targets())
}

fn discover_repositories(
    folder: &Path,
    depth: usize,
    found: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if depth == 0 {
        return Ok(());
    }
    for entry in fs::read_dir(folder).map_err(|error| format!("{}: {error}", folder.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == ".git"
            || name == "node_modules"
            || name == "target"
            || name == ".worktrees"
            || name == "worktrees"
        {
            continue;
        }
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if !file_type.is_dir() || file_type.is_symlink() || name.starts_with('.') {
            continue;
        }
        if path.join(".git").is_dir() {
            found.push(path);
        } else {
            discover_repositories(&path, depth - 1, found)?;
        }
    }
    Ok(())
}

fn rust_target_usage<'a>(checkouts: impl Iterator<Item = (&'a Path, &'a Path)>) -> RustTargetUsage {
    let checkouts: Vec<_> = checkouts.collect();
    let checkout_roots: Vec<_> = checkouts
        .iter()
        .map(|(checkout, _)| fs::canonicalize(checkout).unwrap_or_else(|_| (*checkout).to_owned()))
        .collect();
    let mut candidates: BTreeMap<PathBuf, Vec<usize>> = BTreeMap::new();
    for (index, (checkout, _)) in checkouts.iter().enumerate() {
        if let Some(target) = configured_target_dir(checkout)
            && let Ok(target) = fs::canonicalize(target)
            && !checkout_roots
                .iter()
                .any(|checkout| target.starts_with(checkout) || checkout.starts_with(&target))
        {
            candidates.entry(target).or_default().push(index);
        }
    }

    let mut selected: Vec<PathBuf> = Vec::new();
    for target in candidates.keys() {
        if !selected.iter().any(|parent| target.starts_with(parent)) {
            selected.push(target.clone());
        }
    }

    let mut usage = RustTargetUsage::default();
    let mut disks = BTreeMap::new();
    for path in selected {
        let associated = candidates
            .iter()
            .filter(|(candidate, _)| candidate.starts_with(&path))
            .flat_map(|(_, indices)| indices.iter().copied());
        let mut repositories = BTreeSet::new();
        let mut target_checkouts = BTreeSet::new();
        for index in associated {
            let (checkout, repository) = checkouts[index];
            repositories.insert(repository.to_owned());
            target_checkouts.insert(checkout.to_owned());
        }
        let artifacts = generated_artifact_paths(&path);
        let (disk_bytes, reclaimable_bytes) = target_sizes(&path, &artifacts);
        let fingerprint = artifacts
            .iter()
            .any(|artifact| artifact.join(".fingerprint").is_dir());
        let cargo_identity = path.join(".rustc_info.json").is_file()
            || fs::read_to_string(path.join("CACHEDIR.TAG"))
                .is_ok_and(|tag| tag.starts_with("Signature: 8a477f597d28d172789f06886806bc55"));
        let mut target = RustTarget {
            path: path.clone(),
            disk_bytes,
            reclaimable_bytes,
            repositories: repositories.into_iter().collect(),
            checkouts: target_checkouts.into_iter().collect(),
            reasons: vec![
                "Observed from Cargo build.target-dir configuration".to_owned(),
                "Only generated artifacts are eligible; source and Git history stay".to_owned(),
            ],
            eligible: false,
        };
        if !fingerprint {
            target
                .reasons
                .push("No Cargo fingerprint directory confirms generated output".to_owned());
        }
        if !cargo_identity {
            target
                .reasons
                .push("No Cargo cache tag or rustc metadata confirms this target".to_owned());
        }
        if reclaimable_bytes == 0 {
            target
                .reasons
                .push("No generated Cargo artifacts were found".to_owned());
        }
        if disk_bytes > reclaimable_bytes {
            target
                .reasons
                .push("The target root and unrecognized entries are preserved".to_owned());
        }
        let in_use = fingerprint && target.in_use();
        if in_use {
            target
                .reasons
                .push("Cargo or rustc is currently using this target".to_owned());
        } else if fingerprint {
            target
                .reasons
                .push("No active Cargo or rustc process is using this target".to_owned());
        }
        target.eligible = fingerprint && cargo_identity && reclaimable_bytes > 0 && !in_use;
        usage.bytes += disk_bytes;
        if let Some((filesystem, mount, disk)) = disk_usage_details(&path) {
            disks.entry((filesystem, mount)).or_insert(disk);
        }
        usage.targets.push(target);
    }
    usage.target_count = usage.targets.len();
    if !disks.is_empty() {
        usage.disk_total_bytes = Some(disks.values().map(|disk| disk.total_bytes).sum());
    }
    usage
}

pub(crate) fn generated_artifact_paths(target: &Path) -> Vec<PathBuf> {
    let mut artifacts = Vec::new();
    for entry in fs::read_dir(target).into_iter().flatten().flatten() {
        let path = entry.path();
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.is_dir()
            && !metadata.file_type().is_symlink()
            && (entry.file_name() == "tmp" || path.join(".fingerprint").is_dir())
        {
            artifacts.push(path);
        }
    }
    let rustc_info = target.join(".rustc_info.json");
    if fs::symlink_metadata(&rustc_info).is_ok_and(|metadata| metadata.is_file()) {
        artifacts.push(rustc_info);
    }
    artifacts
}

/// Sizes a target in a single walk: every direct child counts toward the
/// total, and generated artifact children also count toward the reclaimable
/// subset.
fn target_sizes(target: &Path, artifacts: &[PathBuf]) -> (u64, u64) {
    let mut total = 0;
    let mut reclaimable = 0;
    for entry in fs::read_dir(target).into_iter().flatten().flatten() {
        let path = entry.path();
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        let size = if metadata.is_dir() && !metadata.file_type().is_symlink() {
            crate::scan::directory_size(&path).unwrap_or(0)
        } else {
            metadata.len()
        };
        total += size;
        if artifacts.contains(&path) {
            reclaimable += size;
        }
    }
    (total, reclaimable)
}

fn rust_target_in_use(target: &RustTarget) -> bool {
    rust_target_in_use_impl(target)
}

#[cfg(target_os = "linux")]
fn rust_target_in_use_impl(target: &RustTarget) -> bool {
    let Ok(processes) = fs::read_dir("/proc") else {
        return true;
    };
    for process in processes.flatten() {
        if !process
            .file_name()
            .to_string_lossy()
            .chars()
            .all(|c| c.is_ascii_digit())
        {
            continue;
        }
        let path = process.path();
        let Ok(command) = fs::read_to_string(path.join("comm")) else {
            continue;
        };
        if !matches!(command.trim(), "cargo" | "rustc") {
            continue;
        }
        if fs::read_link(path.join("cwd")).is_ok_and(|cwd| {
            target
                .checkouts
                .iter()
                .any(|checkout| cwd.starts_with(checkout))
        }) {
            return true;
        }
        let Ok(descriptors) = fs::read_dir(path.join("fd")) else {
            continue;
        };
        for descriptor in descriptors.flatten() {
            if fs::read_link(descriptor.path())
                .is_ok_and(|open_path| open_path.starts_with(&target.path))
            {
                return true;
            }
        }
    }
    false
}

#[cfg(not(target_os = "linux"))]
fn rust_target_in_use_impl(_target: &RustTarget) -> bool {
    false
}

fn configured_target_dir(checkout: &Path) -> Option<PathBuf> {
    for ancestor in checkout.ancestors() {
        for filename in ["config", "config.toml"] {
            let config = ancestor.join(".cargo").join(filename);
            let Ok(contents) = fs::read_to_string(config) else {
                continue;
            };
            if let Some(target) = parse_target_dir(&contents) {
                let target = PathBuf::from(target);
                return Some(if target.is_absolute() {
                    target
                } else {
                    ancestor.join(target)
                });
            }
        }
    }
    None
}

fn parse_target_dir(contents: &str) -> Option<&str> {
    let mut in_build = false;
    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') {
            in_build = line.split('#').next().unwrap_or("").trim() == "[build]";
            continue;
        }
        if !in_build {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != "target-dir" {
            continue;
        }
        let value = value.trim();
        let quote = value.chars().next()?;
        if quote != '"' && quote != '\'' {
            return None;
        }
        let rest = &value[1..];
        let end = rest.find(quote)?;
        let suffix = rest[end + 1..].trim();
        if suffix.is_empty() || suffix.starts_with('#') {
            return Some(&rest[..end]);
        }
    }
    None
}

pub fn cleanup_batch(batch: &ScanBatch, requested: &[PathBuf]) -> Result<CleanupResult, String> {
    let requested: BTreeSet<_> = requested.iter().collect();
    let mut combined = CleanupResult::default();
    for report in &batch.reports {
        let paths: Vec<_> = report
            .worktrees
            .iter()
            .filter(|assessment| requested.contains(&assessment.worktree.path))
            .map(|assessment| assessment.worktree.path.clone())
            .collect();
        if paths.is_empty() {
            continue;
        }
        combined
            .outcomes
            .extend(cleanup_worktrees(&report.repository, &paths)?.outcomes);
    }
    if combined.outcomes.len() != requested.len() {
        return Err("Some selected paths no longer belong to this scan".to_owned());
    }
    Ok(combined)
}

pub(crate) fn disk_usage(path: &Path) -> Option<DiskUsage> {
    disk_usage_details(path).map(|(_, _, usage)| usage)
}

fn disk_usage_details(path: &Path) -> Option<(String, String, DiskUsage)> {
    let output = Command::new("df").arg("-kP").arg(path).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let line = stdout.lines().last()?;
    let fields: Vec<_> = line.split_whitespace().collect();
    if fields.len() < 6 {
        return None;
    }
    let parse = |index: usize| {
        fields
            .get(index)?
            .parse::<u64>()
            .ok()
            .map(|value| value * 1024)
    };
    Some((
        fields[0].to_owned(),
        fields[5].to_owned(),
        DiskUsage {
            total_bytes: parse(1)?,
            used_bytes: parse(2)?,
            available_bytes: parse(3)?,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "alice-{}-{}-{}",
                name,
                std::process::id(),
                nonce
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn discovery_reaches_deeply_nested_repositories() {
        let temp = TestDir::new("deep-discovery");
        let repository = temp.0.join("one/two/three/four/five/repository");
        fs::create_dir_all(repository.join(".git")).unwrap();

        let mut shallow = Vec::new();
        discover_repositories(&temp.0, 3, &mut shallow).unwrap();
        assert!(shallow.is_empty());

        let mut deep = Vec::new();
        discover_repositories(&temp.0, 6, &mut deep).unwrap();
        assert_eq!(deep, vec![repository]);
    }

    #[test]
    fn shared_external_rust_targets_are_counted_once() {
        let temp = TestDir::new("rust-targets");
        let checkout = temp.0.join("trees/one/repository");
        let other_checkout = temp.0.join("trees/two/repository");
        let repository = temp.0.join("primary");
        let cache = temp.0.join("cache");
        fs::create_dir_all(&checkout).unwrap();
        fs::create_dir_all(&other_checkout).unwrap();
        fs::create_dir_all(temp.0.join(".cargo")).unwrap();
        fs::create_dir_all(cache.join("debug/.fingerprint")).unwrap();
        fs::write(cache.join("debug/artifact"), [0_u8; 128]).unwrap();
        fs::write(
            temp.0.join(".cargo/config.toml"),
            format!("[build]\ntarget-dir = \"{}\"\n", cache.display()),
        )
        .unwrap();

        let usage = rust_target_usage(
            [
                (checkout.as_path(), repository.as_path()),
                (other_checkout.as_path(), repository.as_path()),
            ]
            .into_iter(),
        );
        assert_eq!(usage.bytes, 128);
        assert_eq!(usage.target_count, 1);
        assert_eq!(usage.targets[0].checkouts.len(), 2);
        assert!(usage.disk_total_bytes.is_some());
    }

    #[test]
    fn targets_inside_checkouts_are_not_reported_as_external() {
        let temp = TestDir::new("internal-rust-target");
        let checkout = temp.0.join("repository");
        fs::create_dir_all(checkout.join(".cargo")).unwrap();
        fs::create_dir_all(checkout.join("target/debug")).unwrap();
        fs::write(checkout.join("target/debug/artifact"), [0_u8; 128]).unwrap();
        fs::write(
            checkout.join(".cargo/config.toml"),
            "[build]\ntarget-dir = \"target\"\n",
        )
        .unwrap();

        let usage = rust_target_usage([(checkout.as_path(), checkout.as_path())].into_iter());
        assert_eq!(usage.bytes, 0);
        assert_eq!(usage.target_count, 0);
    }

    #[test]
    fn targets_that_contain_checkouts_are_rejected() {
        let temp = TestDir::new("ancestor-rust-target");
        let checkout = temp.0.join("repository");
        fs::create_dir_all(checkout.join(".cargo")).unwrap();
        fs::write(
            checkout.join(".cargo/config.toml"),
            format!("[build]\ntarget-dir = \"{}\"\n", temp.0.display()),
        )
        .unwrap();

        let usage = rust_target_usage([(checkout.as_path(), checkout.as_path())].into_iter());
        assert_eq!(usage.bytes, 0);
        assert_eq!(usage.target_count, 0);
    }
}
