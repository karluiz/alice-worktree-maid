use crate::git;
use crate::scan_repository;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct CleanupOutcome {
    pub path: PathBuf,
    pub reclaimed_bytes: u64,
    pub result: Result<(), String>,
}

#[derive(Clone, Debug, Default)]
pub struct CleanupResult {
    pub outcomes: Vec<CleanupOutcome>,
}

impl CleanupResult {
    pub fn reclaimed_bytes(&self) -> u64 {
        self.outcomes
            .iter()
            .filter(|outcome| outcome.result.is_ok())
            .map(|outcome| outcome.reclaimed_bytes)
            .sum()
    }

    pub fn succeeded(&self) -> bool {
        self.outcomes.iter().all(|outcome| outcome.result.is_ok())
    }
}

pub fn cleanup_worktrees(
    repository: &Path,
    requested: &[PathBuf],
) -> Result<CleanupResult, String> {
    if requested.is_empty() {
        return Err("Choose at least one worktree before cleaning".to_owned());
    }
    let report = scan_repository(repository)?;
    let mut result = CleanupResult::default();
    for path in requested {
        let Some(assessment) = report
            .worktrees
            .iter()
            .find(|item| &item.worktree.path == path)
        else {
            result.outcomes.push(CleanupOutcome {
                path: path.clone(),
                reclaimed_bytes: 0,
                result: Err("Not a currently registered linked worktree".to_owned()),
            });
            continue;
        };
        if !assessment.cleanable() {
            result.outcomes.push(CleanupOutcome {
                path: path.clone(),
                reclaimed_bytes: assessment.disk_bytes,
                result: Err(format!(
                    "Protected by a fresh scan: {}",
                    assessment.reasons.join("; ")
                )),
            });
            continue;
        }
        let removal = git::run(
            &report.repository,
            &["worktree", "remove", &path.to_string_lossy()],
        )
        .map(|_| ())
        .map_err(|error| format!("Git refused the cleanup: {error}"));
        result.outcomes.push(CleanupOutcome {
            path: path.clone(),
            reclaimed_bytes: assessment.disk_bytes,
            result: removal,
        });
    }
    Ok(result)
}

pub fn cleanup_rust_targets(
    batch: &crate::ScanBatch,
    requested: &[PathBuf],
) -> Result<CleanupResult, String> {
    if requested.is_empty() {
        return Err("Choose at least one Cargo target before cleaning".to_owned());
    }
    let mut result = CleanupResult::default();
    let mut seen = std::collections::BTreeSet::new();
    for path in requested {
        if !seen.insert(path) {
            continue;
        }
        let Some(target) = batch
            .rust
            .targets
            .iter()
            .find(|target| &target.path == path)
        else {
            result.outcomes.push(CleanupOutcome {
                path: path.clone(),
                reclaimed_bytes: 0,
                result: Err("Not a currently observed Cargo target".to_owned()),
            });
            continue;
        };
        if !target.cleanable() || target.in_use() {
            result.outcomes.push(CleanupOutcome {
                path: path.clone(),
                reclaimed_bytes: 0,
                result: Err(format!(
                    "Protected by a fresh scan: {}",
                    target.reasons.join("; ")
                )),
            });
            continue;
        }
        let artifacts = crate::batch::generated_artifact_paths(path);
        let fingerprint = artifacts
            .iter()
            .any(|artifact| artifact.join(".fingerprint").is_dir());
        let cargo_identity = path.join(".rustc_info.json").is_file()
            || std::fs::read_to_string(path.join("CACHEDIR.TAG"))
                .is_ok_and(|tag| tag.starts_with("Signature: 8a477f597d28d172789f06886806bc55"));
        let contains_checkout = target
            .checkouts
            .iter()
            .any(|checkout| path.starts_with(checkout) || checkout.starts_with(path));
        let canonical = std::fs::canonicalize(path).is_ok_and(|current| current == target.path);
        let direct_children = artifacts
            .iter()
            .all(|artifact| artifact.parent() == Some(path.as_path()));
        let mut reclaimed_bytes = 0;
        let mut errors = Vec::new();
        if !canonical || !fingerprint || !cargo_identity || contains_checkout || !direct_children {
            errors.push("The target failed its fresh path and Cargo-evidence check".to_owned());
        } else {
            let mut preserved_bytes = 0;
            for artifact in artifacts {
                let removal = std::fs::symlink_metadata(&artifact).and_then(|metadata| {
                    if metadata.is_dir() && !metadata.file_type().is_symlink() {
                        std::fs::remove_dir_all(&artifact)
                    } else {
                        std::fs::remove_file(&artifact)
                    }
                });
                if let Err(error) = removal {
                    // The artifact is still on disk; measure it so only actual
                    // removals count toward the fresh assessment's total.
                    preserved_bytes += artifact_size(&artifact);
                    errors.push(format!("{}: {error}", artifact.display()));
                }
            }
            reclaimed_bytes = target.reclaimable_bytes.saturating_sub(preserved_bytes);
        }
        result.outcomes.push(CleanupOutcome {
            path: path.clone(),
            reclaimed_bytes,
            result: if errors.is_empty() {
                Ok(())
            } else {
                Err(errors.join("; "))
            },
        });
    }
    Ok(result)
}

fn artifact_size(path: &Path) -> u64 {
    std::fs::symlink_metadata(path).map_or(0, |metadata| {
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            crate::scan::directory_size(path).unwrap_or(0)
        } else {
            metadata.len()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RustTarget, RustTargetUsage, ScanBatch};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn build_cleanup_preserves_unknown_entries_and_source() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let fixture = std::env::temp_dir().join(format!(
            "alice-cleanup-test-{}-{}",
            std::process::id(),
            nonce
        ));
        let repository = fixture.join("repository");
        let target = fixture.join("target");
        fs::create_dir_all(repository.join("src")).unwrap();
        fs::create_dir_all(target.join("debug/.fingerprint")).unwrap();
        fs::write(repository.join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(target.join("debug/artifact"), [0_u8; 20]).unwrap();
        fs::write(target.join(".rustc_info.json"), "{}").unwrap();
        fs::write(target.join("keep.txt"), "keep").unwrap();

        let batch = ScanBatch {
            location: repository.clone(),
            reports: Vec::new(),
            disk: None,
            warnings: Vec::new(),
            rust: RustTargetUsage {
                bytes: 45,
                target_count: 1,
                disk_total_bytes: None,
                targets: vec![RustTarget {
                    path: target.clone(),
                    disk_bytes: 45,
                    reclaimable_bytes: 22,
                    repositories: vec![repository.clone()],
                    checkouts: vec![repository.clone()],
                    reasons: Vec::new(),
                    eligible: true,
                }],
            },
        };
        let result = cleanup_rust_targets(&batch, &[target.clone()]).unwrap();

        assert!(result.succeeded());
        assert_eq!(result.reclaimed_bytes(), 22);
        assert!(!target.join("debug").exists());
        assert!(!target.join(".rustc_info.json").exists());
        assert_eq!(fs::read_to_string(target.join("keep.txt")).unwrap(), "keep");
        assert!(repository.join("src/main.rs").exists());
        let _ = fs::remove_dir_all(fixture);
    }

    #[cfg(unix)]
    #[test]
    fn build_cleanup_counts_only_removed_artifacts_on_partial_failure() {
        use std::os::unix::fs::PermissionsExt;

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let fixture = std::env::temp_dir().join(format!(
            "alice-cleanup-partial-{}-{}",
            std::process::id(),
            nonce
        ));
        let repository = fixture.join("repository");
        let target = fixture.join("target");
        fs::create_dir_all(repository.join("src")).unwrap();
        fs::create_dir_all(target.join("debug/.fingerprint")).unwrap();
        fs::create_dir_all(target.join("release/.fingerprint")).unwrap();
        fs::write(target.join("debug/artifact"), [0_u8; 20]).unwrap();
        fs::write(target.join("release/artifact"), [0_u8; 30]).unwrap();
        fs::write(target.join(".rustc_info.json"), "{}").unwrap();
        // Keep `release` readable but not writable so its removal fails.
        fs::set_permissions(target.join("release"), fs::Permissions::from_mode(0o555)).unwrap();

        let batch = ScanBatch {
            location: repository.clone(),
            reports: Vec::new(),
            disk: None,
            warnings: Vec::new(),
            rust: RustTargetUsage {
                bytes: 50,
                target_count: 1,
                disk_total_bytes: None,
                targets: vec![RustTarget {
                    path: target.clone(),
                    disk_bytes: 50,
                    reclaimable_bytes: 50,
                    repositories: vec![repository.clone()],
                    checkouts: vec![repository.clone()],
                    reasons: Vec::new(),
                    eligible: true,
                }],
            },
        };
        let result = cleanup_rust_targets(&batch, &[target.clone()]).unwrap();

        assert!(!result.succeeded());
        assert_eq!(result.outcomes[0].reclaimed_bytes, 20);
        assert!(!target.join("debug").exists());
        assert!(target.join("release/artifact").exists());

        fs::set_permissions(target.join("release"), fs::Permissions::from_mode(0o755)).unwrap();
        let _ = fs::remove_dir_all(fixture);
    }
}
