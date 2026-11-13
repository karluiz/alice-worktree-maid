mod ui;

use alice::{
    Assessment, CleanupResult, Disposition, RustTarget, RustTargetUsage, ScanBatch, ScanReport,
    TmpUsage, WorkerProvider, Worktree, available_workers, cleanup_batch, cleanup_rust_targets,
    configured_worker, explain_rust_target_with_provider, observe_tmp, scan_locations,
    set_worker_provider, summarize_with_provider,
};
use rune::{
    App, AppHandle, AppSpec, FileDialogOptions, Image, ImageCache, Key, KeyEvent, TextEdit, Tx,
    Waker,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Activity {
    Idle,
    Scanning,
    Cleaning,
    Explaining,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReportTab {
    Worktrees,
    Builds,
    Tmp,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuildSort {
    Largest,
    Smallest,
    Name,
    Status,
}

impl BuildSort {
    pub(crate) fn next(self) -> Self {
        match self {
            Self::Largest => Self::Smallest,
            Self::Smallest => Self::Name,
            Self::Name => Self::Status,
            Self::Status => Self::Largest,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Largest => "Sort: Largest",
            Self::Smallest => "Sort: Smallest",
            Self::Name => "Sort: A–Z",
            Self::Status => "Sort: Status",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CleanupKind {
    Worktrees,
    Builds,
}

pub(crate) enum Screen {
    Connect,
    Report,
    Swept,
}

pub(crate) enum LoadState {
    Idle,
    Ready(ScanBatch),
    Failed(String),
}

pub(crate) struct SweepSummary {
    kind: CleanupKind,
    count: usize,
    bytes: u64,
    branches: Vec<(String, u64)>,
    free_after: Option<u64>,
}

pub(crate) struct State {
    location: PathBuf,
    locations: Vec<PathBuf>,
    folder_edit: TextEdit,
    folder_error: Option<String>,
    screen: Screen,
    load: LoadState,
    activity: Activity,
    report_tab: ReportTab,
    build_sort: BuildSort,
    tmp: TmpUsage,
    active_repo: Option<PathBuf>,
    selected: BTreeSet<PathBuf>,
    selected_builds: BTreeSet<PathBuf>,
    expanded: Option<PathBuf>,
    expanded_build: Option<PathBuf>,
    confirming: bool,
    summary: Option<SweepSummary>,
    worker: Option<(PathBuf, String)>,
    worker_provider: WorkerProvider,
    available_workers: Vec<WorkerProvider>,
    handle: Option<AppHandle<State>>,
    image_cache: Option<ImageCache>,
    greeting_frames: Vec<Image>,
    greeting_dark_frames: Vec<Image>,
    sweeping_frames: Vec<Image>,
    sweeping_dark_frames: Vec<Image>,
    rest_frames: Vec<Image>,
    rest_dark_frames: Vec<Image>,
    dark: Option<bool>,
    zoom: f32,
    demo: bool,
}

impl State {
    fn new() -> Self {
        let locations = initial_locations();
        let location = locations.first().cloned().unwrap_or_default();
        Self {
            folder_edit: TextEdit::new(""),
            folder_error: None,
            location,
            locations,
            screen: Screen::Connect,
            load: LoadState::Idle,
            activity: Activity::Idle,
            report_tab: ReportTab::Worktrees,
            build_sort: BuildSort::Largest,
            tmp: TmpUsage::default(),
            active_repo: None,
            selected: BTreeSet::new(),
            selected_builds: BTreeSet::new(),
            expanded: None,
            expanded_build: None,
            confirming: false,
            summary: None,
            worker: None,
            worker_provider: configured_worker(),
            available_workers: Vec::new(),
            handle: None,
            image_cache: None,
            greeting_frames: Vec::new(),
            greeting_dark_frames: Vec::new(),
            sweeping_frames: Vec::new(),
            sweeping_dark_frames: Vec::new(),
            rest_frames: Vec::new(),
            rest_dark_frames: Vec::new(),
            dark: initial_dark(),
            zoom: initial_zoom(),
            demo: false,
        }
    }

    pub(crate) fn batch(&self) -> Option<&ScanBatch> {
        match &self.load {
            LoadState::Ready(batch) => Some(batch),
            _ => None,
        }
    }

    fn demo(&mut self) {
        self.location = PathBuf::from("~/code");
        self.locations = vec![PathBuf::from("~/code")];
        self.folder_edit = TextEdit::new("");
        let storefront = demo_report(
            "storefront",
            &[
                (
                    "feat/checkout-retry",
                    Disposition::Ready,
                    7_400_000_000,
                    6,
                    true,
                    0,
                    false,
                    "Squash-merged into main as a41f0c2",
                ),
                (
                    "agent/fix-flaky-e2e-3",
                    Disposition::Ready,
                    6_100_000_000,
                    11,
                    true,
                    0,
                    false,
                    "Merged sibling attempt won the race",
                ),
                (
                    "agent/fix-flaky-e2e-1",
                    Disposition::Ready,
                    5_800_000_000,
                    11,
                    true,
                    0,
                    false,
                    "Superseded by merged attempt 2",
                ),
                (
                    "spike/rsc-cache",
                    Disposition::Ready,
                    5_300_000_000,
                    24,
                    true,
                    0,
                    false,
                    "Findings are already in main",
                ),
                (
                    "feat/search-v2",
                    Disposition::NeedsReview,
                    3_700_000_000,
                    27,
                    false,
                    0,
                    true,
                    "41 modified files are preserved",
                ),
            ],
        );
        let api = demo_report(
            "api",
            &[
                (
                    "chore/bump-prisma",
                    Disposition::Ready,
                    4_200_000_000,
                    9,
                    true,
                    0,
                    false,
                    "Merged into main in #219",
                ),
                (
                    "agent/refactor-auth-2",
                    Disposition::Ready,
                    2_600_000_000,
                    17,
                    true,
                    0,
                    false,
                    "Clean and contained in main",
                ),
                (
                    "agent/payments-webhooks",
                    Disposition::NeedsReview,
                    4_800_000_000,
                    19,
                    false,
                    3,
                    false,
                    "3 local commits are not pushed",
                ),
                (
                    "agent/rate-limiter",
                    Disposition::Active,
                    2_200_000_000,
                    1,
                    false,
                    0,
                    true,
                    "Files changed 4 minutes ago",
                ),
            ],
        );
        let admin = demo_report(
            "web-admin",
            &[
                (
                    "agent/i18n-strings",
                    Disposition::Ready,
                    3_900_000_000,
                    5,
                    true,
                    0,
                    false,
                    "Merged into main in #91",
                ),
                (
                    "agent/pdf-export",
                    Disposition::Ready,
                    1_400_000_000,
                    13,
                    true,
                    0,
                    false,
                    "Closed work is clean and contained",
                ),
                (
                    "feat/onboarding-emails",
                    Disposition::Active,
                    2_800_000_000,
                    2,
                    false,
                    0,
                    false,
                    "Recent commit remains active",
                ),
            ],
        );
        let docs = demo_report(
            "docs",
            &[(
                "agent/docs-sweep",
                Disposition::Ready,
                1_900_000_000,
                8,
                true,
                0,
                false,
                "Merged into main in #58",
            )],
        );
        self.load = LoadState::Ready(ScanBatch {
            location: self.location.clone(),
            reports: vec![storefront, api, admin, docs],
            disk: Some(alice::DiskUsage {
                total_bytes: 512_000_000_000,
                used_bytes: 489_000_000_000,
                available_bytes: 23_000_000_000,
            }),
            warnings: Vec::new(),
            rust: demo_rust_usage(),
        });
        self.screen = Screen::Report;
        self.activity = Activity::Idle;
        self.selected = self
            .batch()
            .into_iter()
            .flat_map(ScanBatch::linked)
            .filter(|(_, item)| item.cleanable())
            .map(|(_, item)| item.worktree.path.clone())
            .collect();
        self.selected_builds = self
            .batch()
            .into_iter()
            .flat_map(|batch| batch.rust.targets.iter())
            .filter(|target| target.cleanable())
            .map(|target| target.path.clone())
            .collect();
        self.report_tab = ReportTab::Worktrees;
        self.active_repo = None;
        self.expanded = None;
        self.expanded_build = None;
        self.tmp = demo_tmp_usage();
        self.demo = true;
    }

    fn confirm_demo(&mut self) {
        self.demo();
        self.confirming = true;
    }

    fn swept_demo(&mut self) {
        self.demo();
        let branches = self
            .batch()
            .into_iter()
            .flat_map(ScanBatch::linked)
            .filter(|(_, item)| self.selected.contains(&item.worktree.path))
            .map(|(_, item)| {
                (
                    item.worktree.branch.clone().unwrap_or_default(),
                    item.disk_bytes,
                )
            })
            .collect();
        self.summary = Some(SweepSummary {
            kind: CleanupKind::Worktrees,
            count: self.selected.len(),
            bytes: self.batch().map_or(0, ScanBatch::reclaimable_bytes),
            branches,
            free_after: Some(61_600_000_000),
        });
        self.screen = Screen::Swept;
    }

    fn scanning_demo(&mut self) {
        self.demo();
        self.activity = Activity::Scanning;
    }

    fn builds_demo(&mut self) {
        self.demo();
        self.report_tab = ReportTab::Builds;
    }

    fn tmp_demo(&mut self) {
        self.demo();
        self.report_tab = ReportTab::Tmp;
    }

    fn confirm_builds_demo(&mut self) {
        self.builds_demo();
        self.confirming = true;
    }

    fn swept_builds_demo(&mut self) {
        self.builds_demo();
        let branches = self
            .batch()
            .into_iter()
            .flat_map(|batch| batch.rust.targets.iter())
            .filter(|target| self.selected_builds.contains(&target.path))
            .map(|target| {
                (
                    target
                        .path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("Cargo target")
                        .to_owned(),
                    target.reclaimable_bytes,
                )
            })
            .collect();
        self.summary = Some(SweepSummary {
            kind: CleanupKind::Builds,
            count: self.selected_builds.len(),
            bytes: self
                .batch()
                .map_or(0, |batch| batch.rust.reclaimable_bytes()),
            branches,
            free_after: Some(512_000_000_000),
        });
        self.screen = Screen::Swept;
    }

    fn empty_demo(&mut self) {
        self.demo();
        if let LoadState::Ready(batch) = &mut self.load {
            for report in &mut batch.reports {
                report.worktrees.clear();
            }
        }
        self.selected.clear();
    }

    fn error_demo(&mut self) {
        self.demo();
        self.selected.clear();
        self.load = LoadState::Failed(
            "Alice could not read ~/code. Check the folder and try again.".to_owned(),
        );
    }
}

fn demo_tmp_usage() -> TmpUsage {
    TmpUsage {
        bytes: 5_700_000_000,
        total_bytes: Some(16_500_000_000),
        entries: vec![
            alice::TmpEntry {
                name: "pr-target".to_owned(),
                bytes: 2_700_000_000,
            },
            alice::TmpEntry {
                name: "rimay-platform-v2-0.3.86".to_owned(),
                bytes: 565_000_000,
            },
            alice::TmpEntry {
                name: "rimay-platform-v2-cambios-rapidos".to_owned(),
                bytes: 554_000_000,
            },
            alice::TmpEntry {
                name: "qhipa-test-gjd3k9".to_owned(),
                bytes: 320_000_000,
            },
            alice::TmpEntry {
                name: "sccache".to_owned(),
                bytes: 210_000_000,
            },
            alice::TmpEntry {
                name: "playwright-artifacts".to_owned(),
                bytes: 96_000_000,
            },
        ],
    }
}

fn demo_rust_usage() -> RustTargetUsage {
    RustTargetUsage {
        bytes: 645_000_000_000,
        target_count: 2,
        disk_total_bytes: Some(512_000_000_000),
        targets: vec![
            RustTarget {
                path: PathBuf::from("/demo/cache/qhipa-ai"),
                disk_bytes: 599_000_000_000,
                reclaimable_bytes: 599_000_000_000,
                repositories: vec![PathBuf::from("/demo/storefront")],
                checkouts: vec![PathBuf::from("/demo/storefront/.worktrees/agent-cache")],
                reasons: vec![
                    "Observed from Cargo build.target-dir configuration".to_owned(),
                    "No active Cargo or rustc process is using this target".to_owned(),
                    "The target root and unrecognized entries are preserved".to_owned(),
                ],
                eligible: true,
            },
            RustTarget {
                path: PathBuf::from("/demo/cache/api-active"),
                disk_bytes: 46_000_000_000,
                reclaimable_bytes: 44_000_000_000,
                repositories: vec![PathBuf::from("/demo/api")],
                checkouts: vec![PathBuf::from("/demo/api/.worktrees/agent-auth")],
                reasons: vec![
                    "Observed from Cargo build.target-dir configuration".to_owned(),
                    "Cargo or rustc is currently using this target".to_owned(),
                ],
                eligible: false,
            },
        ],
    }
}

fn demo_report(
    repo: &str,
    entries: &[(&str, Disposition, u64, i64, bool, u64, bool, &str)],
) -> ScanReport {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |value| value.as_secs() as i64);
    ScanReport {
        repository: PathBuf::from(format!("/demo/{repo}")),
        base_branch: "main".to_owned(),
        worktrees: entries
            .iter()
            .map(
                |(branch, disposition, bytes, days, merged, ahead, dirty, reason)| {
                    let path = PathBuf::from(format!("/demo/{repo}/.worktrees/{branch}"));
                    Assessment {
                        worktree: Worktree {
                            path: path.clone(),
                            head: "0123456789abcdef".to_owned(),
                            branch: Some((*branch).to_owned()),
                            detached: false,
                            bare: false,
                            main: false,
                            locked: None,
                            prunable: None,
                        },
                        disposition: *disposition,
                        reasons: vec![
                            (*reason).to_owned(),
                            "No forced removal; the branch stays in Git".to_owned(),
                        ],
                        dirty: *dirty,
                        changed_files: if *dirty { 41 } else { 0 },
                        merged: *merged,
                        upstream: Some(format!("origin/{branch}")),
                        ahead: Some(*ahead),
                        behind: Some(0),
                        disk_bytes: *bytes,
                        last_commit_at: Some(now - days * 86_400),
                        last_commit_subject: (*reason).to_owned(),
                    }
                },
            )
            .collect(),
    }
}

fn config_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|config| config.join("alice"))
}

fn theme_path() -> Option<PathBuf> {
    config_dir().map(|config| config.join("theme"))
}

fn zoom_path() -> Option<PathBuf> {
    config_dir().map(|config| config.join("zoom"))
}

fn initial_zoom() -> f32 {
    zoom_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|zoom| zoom.trim().parse::<f32>().ok())
        .filter(|zoom| zoom.is_finite())
        .map(|zoom| zoom.clamp(0.8, 2.0))
        .unwrap_or(1.0)
}

fn set_zoom(state: &mut State, zoom: f32) {
    state.zoom = zoom;
    let Some(path) = zoom_path() else {
        return;
    };
    let Some(parent) = path.parent() else {
        return;
    };
    let result =
        std::fs::create_dir_all(parent).and_then(|()| std::fs::write(path, format!("{zoom:.1}\n")));
    if let Err(error) = result {
        eprintln!("alice: could not save the zoom preference: {error}");
    }
}

fn zoom_shortcut(current: f32, key: &Key) -> Option<f32> {
    let zoom = match key {
        Key::Command('=' | '+') => current + 0.1,
        Key::Command('-') => current - 0.1,
        Key::Command('0') => 1.0,
        _ => return None,
    };
    Some(((zoom * 10.0).round() / 10.0).clamp(0.8, 2.0))
}

fn folders_path() -> Option<PathBuf> {
    config_dir().map(|config| config.join("folders"))
}

fn initial_locations() -> Vec<PathBuf> {
    let arguments: Vec<PathBuf> = rune::extra_args()
        .iter()
        .filter(|argument| !argument.starts_with('-'))
        .map(PathBuf::from)
        .collect();
    if !arguments.is_empty() {
        return arguments;
    }
    if let Some(saved) = folders_path().and_then(|path| std::fs::read_to_string(path).ok()) {
        let locations: Vec<PathBuf> = saved
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(PathBuf::from)
            .collect();
        if !locations.is_empty() {
            return locations;
        }
    }
    std::env::current_dir()
        .map(|path| vec![path])
        .unwrap_or_else(|_| vec![PathBuf::from(".")])
}

fn save_folders(state: &State) {
    let Some(path) = folders_path() else {
        return;
    };
    let Some(parent) = path.parent() else {
        return;
    };
    let contents = state
        .locations
        .iter()
        .map(|location| location.display().to_string())
        .collect::<Vec<_>>()
        .join("\n");
    let result = std::fs::create_dir_all(parent)
        .and_then(|()| std::fs::write(path, format!("{contents}\n")));
    if let Err(error) = result {
        eprintln!("alice: could not save the folder list: {error}");
    }
}

fn initial_dark() -> Option<bool> {
    let mut arguments = std::env::args();
    while let Some(argument) = arguments.next() {
        if argument == "--theme" {
            return match arguments.next().as_deref() {
                Some("dark") => Some(true),
                Some("light") => Some(false),
                _ => None,
            };
        }
    }
    std::fs::read_to_string(theme_path()?)
        .ok()
        .and_then(|theme| match theme.trim() {
            "dark" => Some(true),
            "light" => Some(false),
            _ => None,
        })
}

pub(crate) fn set_dark(state: &mut State, dark: bool) {
    state.dark = Some(dark);
    let Some(path) = theme_path() else {
        return;
    };
    let Some(parent) = path.parent() else {
        return;
    };
    let result = std::fs::create_dir_all(parent)
        .and_then(|()| std::fs::write(path, if dark { "dark\n" } else { "light\n" }));
    if let Err(error) = result {
        eprintln!("alice: could not save the theme preference: {error}");
    }
}

pub(crate) fn set_provider(state: &mut State, provider: WorkerProvider) {
    if !state.available_workers.contains(&provider) {
        return;
    }
    state.worker_provider = provider;
    if let Err(error) = set_worker_provider(provider) {
        eprintln!("alice: could not save the worker preference: {error}");
    }
}

fn setup(state: &mut State, waker: Waker) {
    let cache = state.image_cache.insert(ImageCache::new(waker));
    state.greeting_frames = [
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-0.png")[..],
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-1.png")[..],
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-2.png")[..],
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-3.png")[..],
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-4.png")[..],
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-5.png")[..],
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-6.png")[..],
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-7.png")[..],
    ]
    .into_iter()
    .map(|bytes| cache.load(bytes))
    .collect();
    state.greeting_dark_frames = [
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-dark-0.png")[..],
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-dark-1.png")[..],
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-dark-2.png")[..],
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-dark-3.png")[..],
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-dark-4.png")[..],
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-dark-5.png")[..],
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-dark-6.png")[..],
        &include_bytes!("../assets/alice-illustrations/greeting-rigid-dark-7.png")[..],
    ]
    .into_iter()
    .map(|bytes| cache.load(bytes))
    .collect();
    state.sweeping_frames = [
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-0.png")[..],
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-1.png")[..],
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-2.png")[..],
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-3.png")[..],
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-4.png")[..],
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-5.png")[..],
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-6.png")[..],
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-7.png")[..],
    ]
    .into_iter()
    .map(|bytes| cache.load(bytes))
    .collect();
    state.sweeping_dark_frames = [
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-dark-0.png")[..],
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-dark-1.png")[..],
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-dark-2.png")[..],
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-dark-3.png")[..],
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-dark-4.png")[..],
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-dark-5.png")[..],
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-dark-6.png")[..],
        &include_bytes!("../assets/alice-illustrations/sweeping-rigid-dark-7.png")[..],
    ]
    .into_iter()
    .map(|bytes| cache.load(bytes))
    .collect();
    state.rest_frames = [
        &include_bytes!("../assets/alice-illustrations/rest-rigid-0.png")[..],
        &include_bytes!("../assets/alice-illustrations/rest-rigid-1.png")[..],
        &include_bytes!("../assets/alice-illustrations/rest-rigid-2.png")[..],
        &include_bytes!("../assets/alice-illustrations/rest-rigid-3.png")[..],
        &include_bytes!("../assets/alice-illustrations/rest-rigid-4.png")[..],
        &include_bytes!("../assets/alice-illustrations/rest-rigid-5.png")[..],
        &include_bytes!("../assets/alice-illustrations/rest-rigid-6.png")[..],
        &include_bytes!("../assets/alice-illustrations/rest-rigid-7.png")[..],
    ]
    .into_iter()
    .map(|bytes| cache.load(bytes))
    .collect();
    state.rest_dark_frames = [
        &include_bytes!("../assets/alice-illustrations/rest-rigid-dark-0.png")[..],
        &include_bytes!("../assets/alice-illustrations/rest-rigid-dark-1.png")[..],
        &include_bytes!("../assets/alice-illustrations/rest-rigid-dark-2.png")[..],
        &include_bytes!("../assets/alice-illustrations/rest-rigid-dark-3.png")[..],
        &include_bytes!("../assets/alice-illustrations/rest-rigid-dark-4.png")[..],
        &include_bytes!("../assets/alice-illustrations/rest-rigid-dark-5.png")[..],
        &include_bytes!("../assets/alice-illustrations/rest-rigid-dark-6.png")[..],
        &include_bytes!("../assets/alice-illustrations/rest-rigid-dark-7.png")[..],
    ]
    .into_iter()
    .map(|bytes| cache.load(bytes))
    .collect();
    if rune::is_headless() {
        for image in state
            .greeting_frames
            .iter()
            .chain(&state.greeting_dark_frames)
            .chain(&state.sweeping_frames)
            .chain(&state.sweeping_dark_frames)
            .chain(&state.rest_frames)
            .chain(&state.rest_dark_frames)
        {
            let _ = image.wait();
        }
    }
}

fn choose_folder(state: &mut State, tx: &mut Tx) {
    let options = FileDialogOptions {
        title: "Choose a folder for Alice".to_owned(),
        directory: Some(state.location.clone()),
        ..Default::default()
    };
    tx.open_folder(options, |state: &mut State, result, _| match result {
        Ok(Some(path)) => {
            state.folder_edit = TextEdit::new(path.display().to_string());
            state.location = path;
            state.folder_error = None;
        }
        Ok(None) => {}
        Err(error) => {
            state.folder_error = Some(format!(
                "The folder chooser could not open ({error}). You can still type the path above."
            ));
        }
    });
}

fn typed_folder(state: &State) -> Option<PathBuf> {
    let typed = state.folder_edit.text.trim();
    if typed.is_empty() {
        return None;
    }
    if typed == "~" {
        return std::env::var_os("HOME").map(PathBuf::from);
    }
    if let Some(relative) = typed.strip_prefix("~/") {
        return std::env::var_os("HOME").map(|home| PathBuf::from(home).join(relative));
    }
    Some(PathBuf::from(typed))
}

pub(crate) fn add_folder(state: &mut State) {
    let Some(location) = typed_folder(state) else {
        state.folder_error = Some("Type or choose a folder to add".to_owned());
        return;
    };
    state.folder_error = None;
    state.folder_edit = TextEdit::new("");
    state.location = location.clone();
    if !state.locations.contains(&location) {
        state.locations.push(location);
        save_folders(state);
    }
}

pub(crate) fn remove_folder(state: &mut State, location: PathBuf) {
    state.locations.retain(|candidate| candidate != &location);
    state.location = state.locations.first().cloned().unwrap_or_default();
    state.folder_error = None;
    save_folders(state);
}

pub(crate) fn change_folders(state: &mut State) {
    state.screen = Screen::Connect;
    state.folder_edit = TextEdit::new("");
    state.folder_error = None;
}

fn start_scan(state: &mut State) {
    if state.activity != Activity::Idle {
        return;
    }
    if !state.folder_edit.text.trim().is_empty() {
        add_folder(state);
    }
    let Some(handle) = state.handle.clone() else {
        return;
    };
    if state.locations.is_empty() {
        state.folder_error = Some("Add at least one folder before scanning".to_owned());
        return;
    }
    let locations = state.locations.clone();
    state.location = locations[0].clone();
    save_folders(state);
    state.folder_error = None;
    state.screen = Screen::Report;
    state.activity = Activity::Scanning;
    state.selected.clear();
    state.selected_builds.clear();
    state.active_repo = None;
    state.expanded = None;
    state.expanded_build = None;
    state.summary = None;
    std::thread::spawn(move || {
        let result = scan_locations(&locations);
        let tmp = observe_tmp(Path::new("/tmp"));
        handle.update(move |state, _| {
            state.activity = Activity::Idle;
            state.tmp = tmp;
            match result {
                Ok(batch) => state.load = LoadState::Ready(batch),
                Err(error) => state.load = LoadState::Failed(error),
            }
        });
    });
}

fn start_cleanup(state: &mut State) {
    if state.report_tab == ReportTab::Builds {
        start_build_cleanup(state);
    } else {
        start_worktree_cleanup(state);
    }
}

fn start_build_cleanup(state: &mut State) {
    if state.activity != Activity::Idle || state.selected_builds.is_empty() {
        return;
    }
    let Some(handle) = state.handle.clone() else {
        return;
    };
    let locations = state.locations.clone();
    let requested: Vec<_> = state.selected_builds.iter().cloned().collect();
    state.activity = Activity::Cleaning;
    state.confirming = false;
    std::thread::spawn(move || {
        let result =
            scan_locations(&locations).and_then(|fresh| cleanup_rust_targets(&fresh, &requested));
        let refreshed = scan_locations(&locations);
        handle.update(move |state, _| {
            state.activity = Activity::Idle;
            match result {
                Ok(result) if result.succeeded() => {
                    state.summary = Some(summary_from_build_cleanup(
                        &result,
                        refreshed.as_ref().ok(),
                    ));
                    state.screen = Screen::Swept;
                }
                Ok(result) => {
                    state.load = LoadState::Failed(format!(
                        "Alice reclaimed {} but preserved some Cargo targets after the fresh safety scan.",
                        alice::format_bytes(result.reclaimed_bytes())
                    ));
                    state.screen = Screen::Report;
                }
                Err(error) => {
                    state.load = LoadState::Failed(error);
                    state.screen = Screen::Report;
                }
            }
            state.selected_builds.clear();
            if matches!(state.screen, Screen::Swept)
                && let Ok(batch) = refreshed
            {
                state.load = LoadState::Ready(batch);
            }
        });
    });
}

fn start_worktree_cleanup(state: &mut State) {
    if state.activity != Activity::Idle || state.selected.is_empty() {
        return;
    }
    let Some(handle) = state.handle.clone() else {
        return;
    };
    let Some(batch) = state.batch().cloned() else {
        return;
    };
    let locations = state.locations.clone();
    let requested: Vec<_> = state.selected.iter().cloned().collect();
    state.activity = Activity::Cleaning;
    state.confirming = false;
    std::thread::spawn(move || {
        let result = cleanup_batch(&batch, &requested);
        let refreshed = scan_locations(&locations);
        handle.update(move |state, _| {
            state.activity = Activity::Idle;
            match result {
                Ok(result) if result.succeeded() => {
                    state.summary = Some(summary_from_cleanup(
                        &result,
                        refreshed.as_ref().ok(),
                        &batch,
                    ));
                    state.screen = Screen::Swept;
                }
                Ok(result) => {
                    state.load = LoadState::Failed(format!(
                        "Alice reclaimed {} but preserved some paths after the fresh safety scan.",
                        alice::format_bytes(result.reclaimed_bytes())
                    ));
                    state.screen = Screen::Report;
                }
                Err(error) => {
                    state.load = LoadState::Failed(error);
                    state.screen = Screen::Report;
                }
            }
            state.selected.clear();
            if matches!(state.screen, Screen::Swept)
                && let Ok(batch) = refreshed
            {
                state.load = LoadState::Ready(batch);
            }
        });
    });
}

fn summary_from_cleanup(
    result: &CleanupResult,
    refreshed: Option<&ScanBatch>,
    original: &ScanBatch,
) -> SweepSummary {
    SweepSummary {
        kind: CleanupKind::Worktrees,
        count: result.outcomes.len(),
        bytes: result.reclaimed_bytes(),
        branches: result
            .outcomes
            .iter()
            .map(|outcome| {
                let branch = original
                    .linked()
                    .find(|(_, item)| item.worktree.path == outcome.path)
                    .and_then(|(_, item)| item.worktree.branch.clone())
                    .unwrap_or_else(|| {
                        outcome
                            .path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("worktree")
                            .to_owned()
                    });
                (branch, outcome.reclaimed_bytes)
            })
            .collect(),
        free_after: refreshed
            .and_then(|batch| batch.disk.as_ref().map(|disk| disk.available_bytes)),
    }
}

fn summary_from_build_cleanup(
    result: &CleanupResult,
    refreshed: Option<&ScanBatch>,
) -> SweepSummary {
    SweepSummary {
        kind: CleanupKind::Builds,
        count: result.outcomes.len(),
        bytes: result.reclaimed_bytes(),
        branches: result
            .outcomes
            .iter()
            .map(|outcome| {
                (
                    outcome
                        .path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("Cargo target")
                        .to_owned(),
                    outcome.reclaimed_bytes,
                )
            })
            .collect(),
        free_after: refreshed
            .and_then(|batch| batch.disk.as_ref().map(|disk| disk.available_bytes)),
    }
}

fn start_build_explanation(state: &mut State) {
    if state.activity != Activity::Idle {
        return;
    }
    let Some(path) = state.expanded_build.clone() else {
        return;
    };
    let Some(target) = state.batch().and_then(|batch| {
        batch
            .rust
            .targets
            .iter()
            .find(|target| target.path == path)
            .cloned()
    }) else {
        return;
    };
    let Some(handle) = state.handle.clone() else {
        return;
    };
    let provider = state.worker_provider;
    state.activity = Activity::Explaining;
    state.worker = None;
    std::thread::spawn(move || {
        let result = explain_rust_target_with_provider(provider, &target);
        handle.update(move |state, _| {
            state.activity = Activity::Idle;
            state.worker = Some((
                path,
                match result {
                    Ok(outcome) => outcome.summary,
                    Err(error) => error,
                },
            ));
        });
    });
}

fn start_explanation(state: &mut State) {
    if state.activity != Activity::Idle {
        return;
    }
    let Some(path) = state.expanded.clone() else {
        return;
    };
    let Some(assessment) = state.batch().and_then(|batch| {
        batch
            .linked()
            .find(|(_, item)| item.worktree.path == path)
            .map(|(_, item)| item.clone())
    }) else {
        return;
    };
    let Some(handle) = state.handle.clone() else {
        return;
    };
    let provider = state.worker_provider;
    state.activity = Activity::Explaining;
    state.worker = None;
    std::thread::spawn(move || {
        let result = summarize_with_provider(provider, &assessment);
        handle.update(move |state, _| {
            state.activity = Activity::Idle;
            state.worker = Some((
                path,
                match result {
                    Ok(outcome) => outcome.summary,
                    Err(error) => error,
                },
            ));
        });
    });
}

fn on_key(state: &mut State, event: &KeyEvent, _: &mut Tx) -> bool {
    if let Some(zoom) = zoom_shortcut(state.zoom, &event.key) {
        set_zoom(state, zoom);
        true
    } else if event.key == Key::Command('r') {
        start_scan(state);
        true
    } else {
        false
    }
}

fn main() {
    rune::set_app_id("dev.alice.worktrees");
    let autostart = rune::extra_args()
        .iter()
        .any(|argument| argument == "--scan");
    let mut app = App::new(AppSpec::<State> {
        scenes: &[
            ("demo", State::demo),
            ("scanning", State::scanning_demo),
            ("confirm", State::confirm_demo),
            ("swept", State::swept_demo),
            ("builds", State::builds_demo),
            ("tmp", State::tmp_demo),
            ("confirm-builds", State::confirm_builds_demo),
            ("swept-builds", State::swept_builds_demo),
            ("empty", State::empty_demo),
            ("error", State::error_demo),
        ],
        setup: Some(setup),
        fonts: &[
            include_bytes!("../assets/fonts/Inter-Regular.ttf"),
            include_bytes!("../assets/fonts/Inter-Medium.ttf"),
            include_bytes!("../assets/fonts/Inter-SemiBold.ttf"),
            include_bytes!("../assets/fonts/Kalam-Regular.ttf"),
            include_bytes!("../assets/fonts/Kalam-Bold.ttf"),
            include_bytes!("../assets/fonts/JetBrainsMono.ttf"),
        ],
        families: ("Inter", "JetBrains Mono"),
        on_key: Some(on_key),
        min_size: Some((360.0, 560.0)),
        ..AppSpec::new(
            "Alice — Worktree cleaner",
            (1_200.0, 780.0),
            State::new,
            ui::view,
        )
    });
    let handle = app.handle();
    handle
        .clone()
        .update(move |state, _| state.handle = Some(handle.clone()));
    let worker_handle = app.handle();
    std::thread::spawn(move || {
        let available = available_workers();
        worker_handle.update(move |state, _| {
            state.available_workers = available.clone();
            // The fallback is in-memory only: a CLI that fails a single
            // --version probe must not overwrite the saved worker preference.
            if !available.contains(&state.worker_provider)
                && let Some(provider) = available.first()
            {
                state.worker_provider = *provider;
            }
        });
    });
    if autostart {
        let ready_handle = app.handle();
        app = app.on_ready(move |_| {
            ready_handle.update(|state, _| start_scan(state));
        });
    }
    app.tab_visits_controls(true).run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_shortcuts_step_reset_and_clamp() {
        assert_eq!(zoom_shortcut(1.0, &Key::Command('=')), Some(1.1));
        assert_eq!(zoom_shortcut(1.0, &Key::Command('+')), Some(1.1));
        assert_eq!(zoom_shortcut(1.0, &Key::Command('-')), Some(0.9));
        assert_eq!(zoom_shortcut(1.7, &Key::Command('0')), Some(1.0));
        assert_eq!(zoom_shortcut(2.0, &Key::Command('+')), Some(2.0));
        assert_eq!(zoom_shortcut(0.8, &Key::Command('-')), Some(0.8));
        assert_eq!(zoom_shortcut(1.0, &Key::Command('r')), None);
    }
}
