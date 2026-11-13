use alice::{
    Assessment, cleanup_rust_targets, cleanup_worktrees, configured_worker, format_bytes,
    observe_tmp, scan_locations, scan_repository, summarize_with_provider,
};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("alice: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "scan".to_owned());
    if command == "--help" || command == "-h" || command == "help" {
        help();
        return Ok(());
    }
    let repository = PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned()));
    match command.as_str() {
        "usage" => {
            let locations = std::iter::once(repository)
                .chain(args.map(PathBuf::from))
                .collect::<Vec<_>>();
            let batch = scan_locations(&locations)?;
            let worktree_bytes: u64 = batch.linked().map(|(_, item)| item.disk_bytes).sum();
            let rust_targets = batch.rust_targets();
            let disk_total_bytes = batch
                .disk
                .as_ref()
                .map_or_else(|| "null".to_owned(), |disk| disk.total_bytes.to_string());
            let rust_disk_total_bytes = rust_targets
                .disk_total_bytes
                .map_or_else(|| "null".to_owned(), |total| total.to_string());
            let tmp = observe_tmp(Path::new("/tmp"));
            let tmp_total_bytes = tmp
                .total_bytes
                .map_or_else(|| "null".to_owned(), |total| total.to_string());
            let tmp_top = tmp
                .entries
                .iter()
                .take(3)
                .map(|entry| {
                    format!(
                        "{{\"name\":\"{}\",\"bytes\":{}}}",
                        json_escape(&entry.name),
                        entry.bytes
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "{{\"worktree_bytes\":{worktree_bytes},\"disk_total_bytes\":{disk_total_bytes},\"worktree_count\":{},\"rust_target_bytes\":{},\"rust_target_count\":{},\"rust_disk_total_bytes\":{rust_disk_total_bytes},\"tmp_bytes\":{},\"tmp_total_bytes\":{tmp_total_bytes},\"tmp_top\":[{tmp_top}]}}",
                batch.linked().count(),
                rust_targets.bytes,
                rust_targets.target_count,
                tmp.bytes,
            );
            Ok(())
        }
        "scan" => {
            let report = scan_repository(&repository)?;
            println!(
                "Repository: {}\nBase branch: {}\n",
                report.repository.display(),
                report.base_branch
            );
            for assessment in &report.worktrees {
                print_assessment(assessment);
            }
            println!(
                "\nReclaimable after confirmation: {}",
                format_bytes(report.reclaimable_bytes())
            );
            Ok(())
        }
        "clean" => {
            let assume_yes = args.next().is_some_and(|argument| argument == "--yes");
            let report = scan_repository(&repository)?;
            let selected: Vec<_> = report
                .ready()
                .map(|assessment| assessment.worktree.path.clone())
                .collect();
            if selected.is_empty() {
                println!("Alice found no clean, merged linked worktrees.");
                return Ok(());
            }
            println!("Alice will remove these worktree directories (branches stay):");
            for assessment in report.ready() {
                println!(
                    "  {}  {}",
                    assessment.worktree.path.display(),
                    format_bytes(assessment.disk_bytes)
                );
            }
            println!("Total: {}", format_bytes(report.reclaimable_bytes()));
            if !assume_yes {
                print!("\nType 'clean' to confirm this batch: ");
                io::stdout().flush().map_err(|error| error.to_string())?;
                let mut answer = String::new();
                io::stdin()
                    .read_line(&mut answer)
                    .map_err(|error| error.to_string())?;
                if answer.trim() != "clean" {
                    println!("Cancelled. Nothing was removed.");
                    return Ok(());
                }
            }
            let result = cleanup_worktrees(&repository, &selected)?;
            for outcome in &result.outcomes {
                match &outcome.result {
                    Ok(()) => println!("Removed {}", outcome.path.display()),
                    Err(error) => eprintln!("Kept {}: {error}", outcome.path.display()),
                }
            }
            println!("Reclaimed {}", format_bytes(result.reclaimed_bytes()));
            if result.succeeded() {
                Ok(())
            } else {
                Err("Some worktrees were preserved after a fresh safety check".to_owned())
            }
        }
        "clean-builds" => {
            let remaining: Vec<String> = args.collect();
            let assume_yes = remaining.iter().any(|argument| argument == "--yes");
            let locations = std::iter::once(repository)
                .chain(
                    remaining
                        .into_iter()
                        .filter(|argument| argument != "--yes")
                        .map(PathBuf::from),
                )
                .collect::<Vec<_>>();
            let batch = scan_locations(&locations)?;
            let selected: Vec<_> = batch
                .rust
                .targets
                .iter()
                .filter(|target| target.cleanable())
                .map(|target| target.path.clone())
                .collect();
            if selected.is_empty() {
                println!("Alice found no idle, verified Cargo targets to clean.");
                return Ok(());
            }
            println!("Alice will clean these generated Cargo targets (source stays):");
            for target in batch
                .rust
                .targets
                .iter()
                .filter(|target| target.cleanable())
            {
                println!(
                    "  {}  {}",
                    target.path.display(),
                    format_bytes(target.disk_bytes)
                );
            }
            println!("Total: {}", format_bytes(batch.rust.reclaimable_bytes()));
            if !assume_yes {
                print!("\nType 'clean builds' to confirm this batch: ");
                io::stdout().flush().map_err(|error| error.to_string())?;
                let mut answer = String::new();
                io::stdin()
                    .read_line(&mut answer)
                    .map_err(|error| error.to_string())?;
                if answer.trim() != "clean builds" {
                    println!("Cancelled. Nothing was removed.");
                    return Ok(());
                }
            }
            let fresh = scan_locations(&locations)?;
            let result = cleanup_rust_targets(&fresh, &selected)?;
            for outcome in &result.outcomes {
                match &outcome.result {
                    Ok(()) => println!("Cleaned {}", outcome.path.display()),
                    Err(error) => eprintln!("Kept {}: {error}", outcome.path.display()),
                }
            }
            println!("Reclaimed {}", format_bytes(result.reclaimed_bytes()));
            if result.succeeded() {
                Ok(())
            } else {
                Err("Some Cargo targets were preserved after a fresh safety check".to_owned())
            }
        }
        "explain" => {
            let branch = args
                .next()
                .ok_or_else(|| "Usage: alice-cli explain <repository> <branch>".to_owned())?;
            let report = scan_repository(&repository)?;
            let assessment = report
                .worktrees
                .iter()
                .find(|item| item.worktree.branch.as_deref() == Some(branch.as_str()))
                .ok_or_else(|| format!("No linked worktree found for branch {branch}"))?;
            let outcome = summarize_with_provider(configured_worker(), assessment)?;
            println!("{}", outcome.summary);
            Ok(())
        }
        _ => Err(format!("Unknown command '{command}'. Run alice-cli --help")),
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            control if (control as u32) < 0x20 => {
                escaped.push_str(&format!("\\u{:04x}", control as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped
}

fn print_assessment(assessment: &Assessment) {
    let branch = assessment
        .worktree
        .branch
        .as_deref()
        .unwrap_or("detached HEAD");
    println!(
        "{}\n  {} · {}\n  {}",
        branch,
        assessment.disposition.label(),
        format_bytes(assessment.disk_bytes),
        assessment.worktree.path.display()
    );
    for reason in &assessment.reasons {
        println!("  - {reason}");
    }
}

fn help() {
    println!(
        "Alice — careful Git worktree and Cargo build cleanup\n\nUSAGE\n  alice-cli scan [repository]\n  alice-cli usage [repository-or-folder] [more folders...]\n  alice-cli clean [repository] [--yes]\n  alice-cli clean-builds [repository-or-folder] [more folders...] [--yes]\n  alice-cli explain <repository> <branch>\n\nScanning is read-only. Cleaning always requires confirmation unless --yes is supplied.\nAlice never deletes branches, source files, or uses git worktree remove --force."
    );
}
