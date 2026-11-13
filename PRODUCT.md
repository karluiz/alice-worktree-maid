# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Stack

Native desktop app built with Rust and Rune, pinned to a specific revision of Rune's `dev` branch. The first target is the user's Linux machine. Git operations live in a UI-independent core. Kimi Code is invoked through its local CLI as the first optional worker adapter.

## Users

The primary user is a developer running many coding agents across Git worktrees. Their disk is frequently close to full because finished and abandoned agent worktrees accumulate. They need to distinguish merged work, integrated WIP, active work, and unintegrated WIP without manually auditing every directory.

## Product Purpose

Alice is a personal worktree cleaner that reclaims disk space without losing unfinished work. Success means the user can quickly understand why each worktree is considered safe, active, or uncertain, then confirm cleanup with confidence.

## Positioning

Alice combines deterministic Git evidence with an optional bring-your-own-worker (BYOW) assistant. Git rules—not an AI model—control deletion eligibility. The user's existing Kimi Code subscription can explain ambiguous WIP instead of requiring a separate metered AI API.

## Operating Context

Local Git repositories with multiple linked worktrees, often created by coding agents. Alice scans without changing repositories, presents evidence and disk usage, and requests confirmation before every cleanup batch. The first AI worker is the locally installed Kimi Code CLI.

## Capabilities and Constraints

- Add and remove multiple independent scan roots, persist them across launches, and deduplicate repositories when roots overlap.
- Zoom the whole interface from 80% to 200% with Ctrl/Cmd + Plus, Ctrl/Cmd + Minus, and Ctrl/Cmd + 0; preserve the choice across launches and reflow content instead of clipping it.
- Discover linked worktrees and inspect Git status, branch integration, upstream state, and disk usage.
- Show configured external Cargo build-target usage separately from worktree bytes so shared Rust artifacts cannot be mistaken for space that worktree removal will reclaim.
- Clean eligible Cargo targets in a dedicated Build files view after a fresh observation and explicit confirmation, removing only verified generated entries while preserving source, worktrees, branches, target roots, and unrecognized files.
- Sort Cargo targets by size, name, or status; read a plain-language verdict for each target; and optionally ask Kimi to explain the same deterministic evidence and rebuild consequences.
- Classify worktrees into safe-to-clean, active, and needs-review states with human-readable evidence.
- Protect the primary/current worktree, dirty worktrees, unpushed commits, locked worktrees, uncertain states, and Cargo targets currently used by Cargo or rustc.
- Report `/tmp` usage and its largest entries as read-only observation; Alice never removes from the temporary directory, where live sockets and locks belong to running programs.
- Require explicit confirmation for every cleanup; never use forced deletion in the initial release.
- Use the configured local worker for optional interpretation: Kimi Code, Claude Code, or GitHub Copilot. Enable only detected CLIs, persist the selection, and never change cleanup eligibility from a model response. Scanning and cleanup must work without a worker.
- Rune is pre-release, so its dependency must be pinned to an immutable revision.
- Automatic cleanup, branch deletion, remote deletion, and additional worker providers are intentionally undecided future work.

## Brand Commitments

The name is Alice. The character is inspired by Ragnarok Online's Alice: a composed, capable maid who handles the dirty work. Use original artwork and visual language rather than copying the game's copyrighted character art. The complete user-approved visual reference is the Magic Patterns prototype in `idea/`; its composition, three-pile report, voice, and illustrations supersede the earlier laundry-day direction. The user-approved `design/tray-preview/` defines the companion tray menu, and `design/build-files-preview/` defines the approved Build files interaction. The Omarchy plugin is deliberately minimal: it shows the real root-filesystem used percentage next to Alice's color line-art icon and removes the separate disk widget. The popup contains the detailed worktree, Cargo build, and /tmp metrics, worker status, and a clean-in-app affordance. The popup uses Omarchy's `Color.popups` background, text, and border tokens with alpha-derived utility colors, and has no appearance controls. Light/TRUE DARK selection lives inside the native window and persists across launches; the pixel and monochrome alternatives are not part of the tray. The Ian Xiaohei illustration skill guides the hand-drawn visual language; TasteSkill guides product UI discipline and finish. Every displayed Alice illustration moves with its own role: an eight-pose greeting with grip/body, plug, and plant motion, the stronger sweeping stroke, and an eight-pose tea loop with a cup lift and leg swing. The bar and popup line art add a restrained nod. Reduced motion holds the first native pose and disables the tray nod.

## Evidence on Hand

The user's recurring disk-space problem and agent-heavy worktree workflow are confirmed. No customer claims, benchmarks, pricing, testimonials, or production assets exist. Demonstration worktrees must be identified as demo data and must not be presented as a real scan.

## Product Principles

- Evidence before action: every recommendation explains the Git facts behind it.
- Preserve uncertainty: ambiguous or unfinished work stays protected.
- One confirmation per cleanup batch: no silent deletion.
- Useful without AI: the deterministic engine is the product's safety boundary.
- Motion with a job: Alice's gentle presence and interface transitions explain activity, feedback, and state without delaying cleanup or overwhelming evidence; reduced-motion preferences are respected.
- Local by default: repository contents stay on the user's machine except when the user explicitly invokes their chosen worker.
