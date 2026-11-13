use std::path::Path;

/// One direct child of the observed temporary directory.
#[derive(Clone, Debug)]
pub struct TmpEntry {
    pub name: String,
    pub bytes: u64,
}

/// Apparent usage of the temporary directory. Read-only observation: Alice
/// never removes from `/tmp`, where live sockets and locks belong to running
/// programs.
#[derive(Clone, Debug, Default)]
pub struct TmpUsage {
    pub bytes: u64,
    pub total_bytes: Option<u64>,
    /// Every direct entry, largest first. Callers truncate for display.
    pub entries: Vec<TmpEntry>,
}

/// Sizes every direct child of `path` and sorts them largest first.
pub fn observe_tmp(path: &Path) -> TmpUsage {
    let mut usage = TmpUsage {
        total_bytes: crate::batch::disk_usage(path).map(|usage| usage.total_bytes),
        ..TmpUsage::default()
    };
    for entry in std::fs::read_dir(path).into_iter().flatten().flatten() {
        let Ok(metadata) = std::fs::symlink_metadata(entry.path()) else {
            continue;
        };
        let bytes = if metadata.is_dir() && !metadata.file_type().is_symlink() {
            crate::scan::directory_size(&entry.path()).unwrap_or(0)
        } else {
            metadata.len()
        };
        usage.bytes += bytes;
        usage.entries.push(TmpEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            bytes,
        });
    }
    usage.entries.sort_by(|left, right| {
        right
            .bytes
            .cmp(&left.bytes)
            .then_with(|| left.name.cmp(&right.name))
    });
    usage
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn tmp_observation_sums_children_and_ranks_the_largest() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let fixture =
            std::env::temp_dir().join(format!("alice-tmp-test-{}-{}", std::process::id(), nonce));
        std::fs::create_dir_all(fixture.join("big/nested")).unwrap();
        std::fs::write(fixture.join("big/nested/blob"), [0_u8; 40]).unwrap();
        std::fs::write(fixture.join("small"), [0_u8; 10]).unwrap();
        std::fs::write(fixture.join("mid"), [0_u8; 25]).unwrap();

        let usage = observe_tmp(&fixture);

        assert_eq!(usage.bytes, 75);
        assert_eq!(usage.entries.len(), 3);
        assert_eq!(usage.entries[0].name, "big");
        assert_eq!(usage.entries[0].bytes, 40);
        assert_eq!(usage.entries[1].name, "mid");
        assert_eq!(usage.entries[1].bytes, 25);
        assert_eq!(usage.entries[2].name, "small");
        assert_eq!(usage.entries[2].bytes, 10);
        let _ = std::fs::remove_dir_all(fixture);
    }
}
