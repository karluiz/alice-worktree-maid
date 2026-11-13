# Alice Worktrees

## Project overview

Alice is a personal Git worktree cleaner for developers who accumulate worktrees while running coding agents. Its purpose is to reclaim disk space without deleting unfinished work. The native application presents Git evidence, separates worktrees into sweep/ask/keep piles, and asks for confirmation before every cleanup batch.

The repository contains two distinct implementations:

- The product is the Rust application at the repository root. It has a UI-independent library, a native Rune desktop app, and a CLI.
- `idea/` is a React/Vite Magic Patterns design prototype with simulated data. It is the visual reference, not a web version of the product and not connected to the Rust engine.

`PRODUCT.md` defines product scope and safety principles. `DESIGN.md` is the design-system authority. The composition and three-pile report in `idea/` supersede older visual directions, but implemented product behavior overrides unsupported prototype claims.

The first desktop target is Linux. The project is at version 0.1.0; there is no installer, package definition, release pipeline, or deployment configuration in the repository.

## Technology stack and configuration

### Rust product

- `Cargo.toml`: Rust 2024 package `alice-worktrees`; library crate name `alice`.
- Default feature `app` enables the optional `rune` dependency and the `alice` desktop binary.
- Rune is fetched from GitHub at immutable revision `b556a38a1ff88aa8690d21b45ebf9b9167038674`, with `native-file-dialog` enabled. Keep this revision pinned and keep `Cargo.lock` committed.
- `alice-cli` does not require the `app` feature. `--no-default-features` builds only the core library and CLI, avoiding Rune and native GUI dependencies.
- The core uses the Rust standard library plus external `git` and `df` processes. There is no libgit2 dependency.
- No `rust-toolchain.toml`, custom rustfmt configuration, or Clippy configuration is present.

Required runtime tools are Git and the POSIX `df` command on `PATH`. Native Linux builds/runtime also need the system libraries required by Rune, notably XKB/Wayland and a working GPU/Vulkan setup; headless snapshots do not need a display. Kimi Code CLI (`kimi`) is optional and is needed only for WIP explanations. The optional Omarchy tray integration additionally uses `omarchy`, `hyprctl`, `wtype`, and `findmnt` for settings, window focus, keyboard activation, and real disk usage.

### React design prototype

- `idea/package.json` and `idea/package-lock.json` define a separate npm project.
- React 18, TypeScript, Vite 8, Tailwind CSS 4, Radix icons, Lucide, Framer Motion, date-fns, and React Router are declared dependencies.
- `idea/vite.config.ts` enables the React and Tailwind Vite plugins.
- `idea/index.html` enters through `src/xukcWHQ5H3ynrEFjAtHXn2.prototype/__mp-prototype-entry.tsx`.
- There is no `tsconfig.json`, frontend test runner, lint script, or standalone TypeScript-check script. Vite's build transpiles the prototype; do not describe it as a type check.

## Build and run commands

Run Rust commands from the repository root. Using `--locked` is recommended for reproducible builds.

```sh
cargo build --locked
cargo build --release --locked
cargo build --no-default-features --locked
```

The release build produces `target/release/alice` and `target/release/alice-cli`. The native app needs a desktop session for normal windowed use.

```sh
# Open the app with the current directory as its initial location.
cargo run --bin alice

# Choose one or more initial repositories/folders and optionally start scanning on ready.
cargo run --bin alice -- /path/to/location /another/location --scan

# Core CLI without compiling the desktop app.
cargo run --no-default-features --bin alice-cli -- scan [repository]
cargo run --no-default-features --bin alice-cli -- usage [repository-or-folder] [more folders...]
cargo run --no-default-features --bin alice-cli -- clean [repository]
cargo run --no-default-features --bin alice-cli -- explain <repository> <branch>
cargo run --no-default-features --bin alice-cli -- --help
```

With no command, `alice-cli` defaults to `scan`; with no repository, it uses `.`. `usage` accepts one or more roots, scans each repository or folder, deduplicates overlapping repositories and filesystems, and prints one JSON object containing `worktree_bytes`, `disk_total_bytes`, `worktree_count`, `rust_target_bytes`, `rust_target_count`, `rust_disk_total_bytes`, `tmp_bytes`, `tmp_total_bytes`, and `tmp_top` (the three largest direct `/tmp` entries as `{"name", "bytes"}` objects); linked-worktree apparent bytes divided by aggregated disk capacity supplies the tray percentage. `clean` selects every currently cleanable worktree, prints the batch, and requires the user to type `clean`. Its `--yes` option bypasses that prompt. `explain` invokes the optional Kimi adapter for one non-cleanable linked worktree.

For the visual prototype:

```sh
cd idea
npm ci
npm run dev
npm run build
npm run preview
```

`npm run build` writes static prototype output to `idea/dist/`. This is not a deployment of the native product.

On the current development machine, Cargo may fail before compilation because `RUSTC_WRAPPER` names an unavailable `sccache`. If that occurs, install `sccache` or prefix the individual command with `RUSTC_WRAPPER=`; this is an environment issue, not a project dependency.

## Source organization and runtime architecture

### Core library

`src/lib.rs` keeps the core modules private and re-exports their public API. Both frontends consume this library rather than duplicating Git rules.

- `src/git.rs`: thin wrappers around `git -C <repository>`, returning stdout or stderr as Rust `String` results.
- `src/scan.rs`: worktree porcelain parsing, base-branch selection, evidence collection, disposition assignment, recursive apparent disk usage, and byte formatting.
- `src/batch.rs`: single-repository or multi-repository scans, bounded repository discovery, per-location disk usage through `df -kP`, cached external Rust target observation and eligibility, and cleanup across a scanned batch.
- `src/cleanup.rs`: fresh safety scans, non-forced worktree removal, and verified generated-entry removal for Cargo targets.
- `src/worker.rs`: optional local worker adapters for Kimi Code, Claude Code, and GitHub Copilot. Detection uses each CLI's `--version`; the selected provider persists in `~/.config/alice/worker` (respecting `XDG_CONFIG_HOME`). Prompts run noninteractively with tools disabled: Kimi `--prompt`, Claude `-p` with `--tools ""`, and Copilot `-p` with `--available-tools ""`, `--no-ask-user`, and `--no-custom-instructions`.
- `src/tmp.rs`: read-only `/tmp` observation. Apparent per-entry sizing with the same symlink-aware walk as worktrees, tmpfs capacity through the shared `df` helper, and the three largest direct entries for the tray. Never deletes.

`scan_repository` resolves the repository top level, reads `git worktree list --porcelain`, chooses a base branch, and assesses every registered worktree. Base-branch preference is local `main`, `master`, or `trunk`, then the current symbolic HEAD, then `origin/HEAD`.

The deletion boundary is deterministic. A linked worktree is `Ready` and cleanable only when it has branch identity (not detached), a clean working tree including no untracked files, HEAD contained in the base branch, and no commits ahead of its upstream. Primary, bare, locked, prunable, missing, detached, dirty, unmerged, unpushed, and unreadable worktrees are preserved. Clean, unmerged, fully pushed work with a commit at most seven days old is `Active`; other uncertain cases are `NeedsReview`. Kimi output never changes these classifications or deletion eligibility.

`scan_location` first tries the supplied location as a repository. If that fails, it searches for repository directories below it to depth 6. Discovery skips hidden entries, symlinks, `.git`, `node_modules`, `target`, `.worktrees`, and `worktrees`; registered worktrees below hidden or excluded directories are still obtained from each primary repository's Git metadata. `scan_locations` merges independent roots, deduplicates repositories and filesystem/mount pairs, and turns failed roots into warnings when any root succeeds. Disk usage is optional if `df` fails.

`ScanBatch::rust_targets` returns the cached `RustTargetUsage` computed once per scan. Observation reads external Cargo `build.target-dir` values from the nearest `.cargo/config` or `.cargo/config.toml` above each checkout, canonicalizes and deduplicates shared or nested targets, and excludes targets inside or containing checkouts. Each `RustTarget` records observed bytes, the generated reclaimable subset, repositories, checkouts, reasons, and eligibility. Targets need Cargo fingerprint evidence, a cache tag or rustc metadata, nonzero generated bytes, and no active Cargo/rustc process using them. Never recompute directory sizes during rendering.

### Desktop app

- `src/main.rs`: Rune app specification, global state, initial path handling, background operations, demo scenes, embedded images/fonts, and keyboard handling.
- `src/ui.rs`: shared palette, typography, top bar, buttons, and vector icons.
- `src/ui/connect.rs`: provider display, folder selection, and scan setup.
- `src/ui/report.rs`: shared tabs and hero, size-sorted nonzero worktree repository rail and verdict groups, full-row direct-verdict evidence in two desktop columns or one compact column, Kimi explanation, selection, and confirmation dispatch. Merged rows omit repeated notes; branch text and a separate 44×44 chevron button expand evidence without an offset margin.
- `src/ui/builds.rs`: compact branch/crate icon tabs, full-width Cargo-target list, size/name/status sorting, plain-language verdicts, generated-byte selection, evidence, optional selected-worker explanation, review footer, and build-specific confirmation.
- `src/ui/swept.rs`: worktree or Cargo cleanup result and refreshed free-space summary.

The three main screens are `Connect`, `Report`, and `Swept`; the report has Worktrees and Build files tabs with independent selections and confirmation flows. Connect edits a persistent list of independent roots stored one per line in `~/.config/alice/folders` (respecting `XDG_CONFIG_HOME`); explicit launch paths override that list, and Edit folders returns to it from the report. Scanning, cleaning, and Kimi explanation run on spawned threads and return results through `AppHandle::update`; keep blocking Git, filesystem, and worker work off the UI thread. Build cleanup freshly scans every root before removal and again afterward. Command-R starts another multi-root scan.

The initial window is 1200×780 points with a 360×560 minimum. `ui::view` applies the persisted 80–200% UI zoom through `Cx::zoom` and reads the resulting `window` layout width through `Cx::layout`; Ctrl/Cmd + Plus, Minus, and 0 adjust and reset it in 10% steps, stored in `~/.config/alice/zoom` (respecting `XDG_CONFIG_HOME`). Below 900 points, shared `Paint::compact` switches onboarding, report, result, and confirmation to fluid single-column layouts. Compact report repositories use horizontal scrolling, worktree notes and build-group controls wrap onto additional lines, and long content uses vertical scrolling rather than clipping. Fonts and illustrations are embedded with `include_bytes!`, so native scenes render offline. Dark uses true-black `#000000`, dedicated semantic colors, and transparent dark illustration assets. Explicit `--theme dark|light` wins at launch; otherwise the top bar's Theme control changes the palette immediately and persists `dark` or `light` in `~/.config/alice/theme` (respecting `XDG_CONFIG_HOME`), with `Cx::system_dark` as the unset fallback. The tray has no appearance controls and does not pass a theme to the app. Shipping raster provenance is recorded in `assets/alice-illustrations/provenance.json`.

The `AppSpec` scenes are `demo`, `scanning`, `confirm`, `swept`, `builds`, `confirm-builds`, `swept-builds`, `empty`, and `error`. They use synthetic data and must remain visibly labeled `DEMO DATA`; never present them as real repository scans. Scanning centers illustration and loading text and animates a 25% progress sweep with Rune's native `LoopSpec` only while `Activity::Scanning` is active. The sweeping illustration is an embedded eight-pose Light/Dark flipbook generated by `assets/alice-illustrations/animate-rigid-sweeping.sh`; it plays an anticipation–stroke–follow-through–recovery sequence over 2.4 seconds through Rune's 12 Hz wall-clock `Cycle`, with a rigid broom pivot, independently translated debris, and redrawn blinking. Greeting and rest use embedded eight-pose Light/Dark flipbooks generated by `assets/alice-illustrations/animate-rigid-greeting.sh` and `animate-rigid-rest.sh`: greeting rigidly moves the hand, plug, and plant, while tea moves the cup, leg, and arrow. No whole-image Shepards deformation is used. Worker activity pauses the report's sweeping sprite and uses the top-bar status pulse instead. Buttons, checks, chevrons, evidence, and disk changes use native transitions/springs, with reduced motion holding the first sprite/resting pose and disabling spatial press feedback.

### CLI

`src/bin/alice-cli.rs` is a small synchronous frontend over `scan_repository`, `scan_locations`, `cleanup_worktrees`, `cleanup_rust_targets`, and `summarize_with_provider` using the saved worker preference. It prints the same assessments and safety reasons as the native engine, exposes worktree, external Rust-target, and /tmp usage JSON for the tray, and exits nonzero on errors or partial cleanup. `clean-builds <folder> [more folders...]` selects only currently eligible targets, requires the typed phrase `clean builds` unless `--yes` is explicitly supplied, freshly rescans before removal, and preserves partial failures.

### Prototype and design assets

- `idea/src/*frame/`: onboarding, report, and result frames plus report components.
- `idea/src/*prototype/`: Magic Patterns entry, navigation between frames, metadata, and Tailwind theme.
- `idea/src/shared/alice/`: shared top bar, provider/art references, and hard-coded worktree fixtures.
- `assets/`: native icons, bundled fonts, and local illustrations.
- `design/tray-preview/`: standalone interactive tray-menu preview and artwork explorations; it does not build or modify the native app or Omarchy plugin.
- `design/build-files-preview/`: approved interactive Build files concept, taste-skill plan, local fonts/artwork, responsive captures, and confirmation/result states. It is a design prototype with simulated actions, not the native product.
- `~/.config/omarchy/plugins/karluiz.alice/`: user-owned native tray implementation. `Alice.qml` is the real `BarWidget`; `Panel.qml` is the nested popup. It uses Omarchy's `KeyboardPanel`, only the color line-art assets, real Open/Scan actions for every saved root, and a real root-filesystem used-percentage label next to Alice's icon, with no separate disk widget. The bar and popup art use a small bottom-pivot nod; the popup nod runs only while open, and `Style.reduceMotion` stops and resets both. Detailed worktree and Cargo usage feeds the popup and tooltip through multi-root `alice-cli usage` on shell start, popup open, folder-file changes, and a five-minute timer with queued refreshes. A conditional popup row reports deduplicated external Rust targets with a clean-in-app note; deletion happens only after native confirmation, never in the tray. A second conditional row reports observed `/tmp` bytes against tmpfs capacity plus the largest entries; `/tmp` is observation-only everywhere, including the CLI JSON. A watched `FileView` reads `~/.config/alice/folders`, falling back to the legacy `folder` setting when that file is absent. Its palette uses `Commons.Color.popups.background`, `text`, and `border`, with alpha-derived wash, rules, and secondary text; the tooltip uses `Commons.Color.tooltip`. Keep both `import qs.Commons` and `import qs.Commons as Commons` so unqualified `Style`/`Border` remain available. It has no appearance controls or theme processes. Keep plugin changes in this user directory, never in `/usr/share/omarchy/`.
- `.impeccable/`: design contracts, review captures, semantic trees, and Rune interaction scripts; review and mock output are ignored by `.gitignore`.

The prototype contains simulated capabilities that the product deliberately does not implement, including a bin, undo, scheduled scans, patch saving, and additional worker providers. Do not port those claims or controls into the native app without a real implementation. Prototype artwork and Google Fonts are remote; native artwork and fonts are local.

## Development conventions

- Use English for code, comments, documentation, and user-facing copy.
- Format Rust with default `cargo fmt`. Follow the existing 2024-edition idioms, `Path`/`PathBuf` APIs, explicit structs, `Result<_, String>` core errors, and standard-library process/filesystem calls.
- Keep UI-independent behavior in the library. Do not make `scan`, `cleanup`, or worker modules depend on Rune.
- Preserve the public re-export pattern in `src/lib.rs` when adding core API.
- Keep Git invocations argument-based through `std::process::Command`; do not introduce shell interpolation for repository paths or branch names.
- Native UI uses Rune's chained element builders, keyed interactive elements, roles/labels, disabled unsafe controls, and shared helpers from `src/ui.rs`.
- Prototype React code uses function components, two-space indentation, single quotes, no semicolons, and Tailwind utility classes. Treat the Magic Patterns entry and opaque frame/prototype directory names as exporter-generated structure unless intentionally changing the prototype.
- Follow `DESIGN.md`: white paper and warm gray utility surfaces in Light, TRUE DARK `#000000` with discrete gray surfaces in Dark, thin rules, Inter for interface text, Kalam only for Alice's short notes and group titles, and JetBrains Mono for Git identifiers. Red means sweep, orange means ask, and blue means keep; do not reuse verdict colors as arbitrary decoration.
- Keep the report as ruled lists rather than card-heavy nesting. Preserve evidence readability, vertical overflow scrolling, the fixed cleanup footer, and the protected-focus confirmation dialog.
- Update `PRODUCT.md`, `DESIGN.md`, the relevant `.impeccable` design contract, and asset provenance when changing the behavior or visuals they document.

## Testing and verification

The Rust library has six focused unit tests for deep repository discovery, external Rust-target deduplication, exclusion of in-checkout and checkout-containing targets, generated-entry cleanup with unknown-file/source preservation, and distinct noninteractive worker arguments. The native app also has four focused tests covering zoom shortcut steps/reset/clamping, crisp viewport reflow, reduced-motion resting pose, and sweeping sprite sequence/resting-frame selection. There are no frontend tests or CI workflows. For meaningful changes, add focused tests using the existing Rust test conventions, especially temporary Git repositories for scanning/classification and fresh-scan cleanup behavior; do not test destructive cleanup against a developer's real repositories.

Run these checks from the root for Rust changes:

```sh
cargo fmt --check
cargo test --all-targets --locked
cargo test --no-default-features --locked
```

For prototype changes, run `npm run build` in `idea/`. Manually exercise the relevant frame because no automated frontend checks exist.

Vertical scroll viewports must use `col()`, not generic `el()`: the latter can produce a zero scroll maximum even when its children overflow. Verify wheel input and the resulting nonzero scroll maximum when changing report, onboarding, result, or dialog scrolling.

Rune provides headless visual and semantic inspection using the same native renderer. After building, useful commands include:

```sh
cargo run --bin alice -- --list-scenes
cargo run --bin alice -- --scene demo --snap target/demo.png
cargo run --bin alice -- --scene confirm --snap target/confirm.png
cargo run --bin alice -- --scene swept --snap target/swept.png
cargo run --bin alice -- --scene empty --snap target/empty.png
cargo run --bin alice -- --scene error --snap target/error.png
cargo run --bin alice -- --scene demo --tree target/report-tree.json
```

Review native UI changes at 1200×780 and 1080×700 on desktop, plus 720×1000, 390×700, and the 360×560 minimum, including long paths, expanded evidence, empty/error states, dialog focus/cancellation, and keyboard operation. Remove generated snapshots or place them in ignored review output rather than treating them as test fixtures. For tray changes, restart the shell after QML base-type or failed-load changes, inspect `quickshell log` for Alice errors, and exercise both appearance modes, Escape, outside click, Open, and Scan against a disposable Git fixture. The focus selector uses `hyprctl eval 'hl.dispatch(hl.dsp.focus({ window = "class:dev.alice.worktrees" }))'` on this Hyprland version.

Strict Clippy is not currently a clean baseline: `cargo clippy --all-targets -- -D warnings` reports `clippy::collapsible_if` at `src/scan.rs:236`. Do not claim that check passes until this existing lint is addressed; it is separate from formatting and tests.

## Safety and security considerations

- Scanning is read-only. Cleanup is destructive and has no bin or undo workflow.
- Every native batch requires explicit confirmation. CLI cleanup requires typed confirmation unless the caller explicitly supplies `--yes`; use `--yes` only in disposable fixtures or when the user has explicitly requested noninteractive deletion.
- Always perform a fresh scan immediately before removal. Never bypass `Assessment::cleanable`, broaden eligibility based on a worker response, or remove a path that is no longer a currently registered linked worktree.
- Use only `git worktree remove <path>` for worktrees. Never add `--force`, delete branches or remotes, or remove primary/current, dirty, unpushed, locked, detached, bare, missing, or uncertain worktrees.
- Build cleanup freshly scans every root immediately before removal and only accepts a currently `RustTarget::cleanable` path. Recheck active processes, canonical path, checkout boundaries, Cargo fingerprint evidence, and cache tag or rustc metadata. Remove only direct generated children: profile directories containing `.fingerprint`, Cargo `tmp`, and `.rustc_info.json`. Never manufacture a cache tag, delete a target root, delete source, or delete unrecognized entries. Legacy targets without `CACHEDIR.TAG` use this same constrained path rather than bypassing Cargo's safety check.
- Reclaimed byte totals are apparent recursive file sizes from the fresh assessment, not a post-deletion filesystem measurement. Directory sizing does not recurse through directory symlinks. Build cleanup reports only the generated reclaimable subset, not the entire observed target.
- Alice is local by default. Worker invocations use `kimi --prompt`, `claude -p`, or `copilot -p` with tools disabled as documented above. Worktree prompts contain branch name, disposition, dirty status and changed-file count, merge state, upstream/ahead/behind facts, last commit subject, and assessment reasons. Cargo prompts contain target path, observed and reclaimable bytes, cleanability, repositories, checkout count, and protection reasons. Workers are explicitly forbidden from running commands or inspecting files, and their answers never change classification or cleanup eligibility. Scanning and cleanup must remain fully functional when any worker is absent or fails.
- Do not add API keys or credentials to the repository. Worker authentication belongs to the user's local CLI installation.
- Keep demo fixtures and screenshots clearly separated from real repository evidence, and do not commit real session content or sensitive repository metadata into design-review artifacts.

## Deployment status

There is no automated deployment process. Use `cargo build --release --locked` for local release binaries. No native bundle, desktop-file installation, distribution package, code signing, auto-update, or release publishing is configured. The Vite build only previews the design prototype and should not be presented as the product's deployment path.
