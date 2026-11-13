---
version: 1
slug: "src-main-rs"
primary_target: "src/main.rs"
related_targets: ["src/main.rs","src/ui.rs","src/ui/builds.rs","src/ui/connect.rs","src/ui/report.rs","src/ui/swept.rs","src/scan.rs","src/batch.rs","src/cleanup.rs","src/worker.rs"]
---

# Alice desktop worktree and build cleaner

Scope: native Rune onboarding, multi-repository worktree and Cargo-target reports, evidence expansion, separate cleanup confirmations, and result screens.
Mode: Operate. The user reclaims disk space from agent-created worktrees and regenerable Cargo artifacts without losing source or active work.
Proof: real Git status, merge containment, upstream counts, disk size, last-commit evidence, configured Cargo target paths, generated-entry evidence, active Cargo/rustc process checks, plain-language verdicts, size/name/status sorting, and optional selected-worker explanations from Kimi Code, Claude Code, or GitHub Copilot using the same target facts. Demo scenes are labeled DEMO DATA.
Constraints: confirmation for every batch; only clean, merged, contained linked worktrees are selectable; dirty, unpushed, locked, primary, detached, and uncertain work stays protected. Build cleanup uses a fresh observation and removes only verified generated entries from configured external Cargo targets; active, unverified, internal, checkout-containing, and empty targets stay protected. Source, branches, target roots, and unrecognized entries stay. Kimi is optional and advisory.

## Direction contract

THESIS: A small robot maid does the dirty job agents leave behind. The complete user-supplied `idea/` prototype replaces the previous visual direction; its three-pile report and hand-written verdicts are the authority. The approved `design/build-files-preview/` extends the same ruled-list grammar to Cargo build artifacts instead of introducing a dashboard.

OWN-WORLD: Pure white paper, warm gray wash, thin neutral rules, black Inter, JetBrains Mono for branches and paths, Kalam for Alice's notes. Dark is a separately composed TRUE DARK scheme: `#000000` paper, discrete gray utility surfaces, light ink, and transparent light-stroke illustrations rather than inverted white rectangles. Red means sweep, orange means ask, blue means keep in both appearances. Local Magic Patterns illustrations show Alice greeting, sweeping, and having tea. Her motion is composed and state-driven: a small arrival, an eight-pose anticipation–stroke–follow-through sweeping flipbook with moving body/broom/lifted debris, distinct eight-pose greeting and tea flipbooks with grip/plug/plant and cup/leg motion, tactile controls, rotating chevrons, smooth evidence, and a worker-status pulse; reduced motion holds the first sprite/resting pose.

STORY: The user selects an available local worker subscription, adds and removes independent repository roots, scans them together with overlap deduplication, reviews worktree sweep/ask/keep piles or the Build files tab with target evidence, confirms the selected cleanup kind across freshly scanned roots, and sees real reclaimed space. Folder and worker preferences persist across launches. No fake bin, undo, schedule, or unsupported provider actions.

FIRST VIEWPORT: At 1200×780, a 48-point unified bar leads with a compact Theme: Light/Dark control that changes the window immediately and persists across launches. Worktrees use a 218-point repository rail; Build files replaces it with a full-width target list. Both views share tabs, a selected-space hero with disk bar and sweeping illustration, ruled evidence groups, and a fixed review footer. Below 900 points and down to the 360×560 minimum, layouts become fluid single columns: repositories scroll horizontally, notes and group controls wrap, dialogs fit the window, and vertical scrolling preserves every action. Ctrl/Cmd + Plus, Minus, and 0 provide a persistent 80–200% browser-style zoom that reflows these breakpoints without clipping. Confirmation uses a protected-focus dialog with the exact cleanup kind and paths.

FORM: User-pinned Magic Patterns prototype in `idea/`; supersedes the earlier roll (seed:ef2c40eb02d6f736). The Build files composition is approved through the working taste-skill preview in `design/build-files-preview/`. Code-led native implementation in Rune.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
