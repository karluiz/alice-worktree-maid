---
name: Alice
description: A careful robot maid for agent-created Git worktrees
colors:
  paper: "#ffffff"
  wash: "#f6f6f3"
  rule: "#e7e6e1"
  ink: "#141414"
  ink-2: "#5a5b60"
  ink-3: "#74757a"
  sweep: "#c8331f"
  sweep-wash: "#fbece9"
  ask: "#e8821e"
  ask-ink: "#a5560a"
  ask-wash: "#fdf2e6"
  keep: "#2457c5"
  keep-wash: "#ebf0fb"
  dark-paper: "#000000"
  dark-wash: "#141414"
  dark-rule: "#303030"
  dark-ink: "#f4f2ed"
  dark-ink-2: "#b5b5b5"
  dark-ink-3: "#949494"
  dark-sweep: "#ff7663"
  dark-sweep-wash: "#2a1210"
  dark-ask: "#f59e42"
  dark-ask-ink: "#ffb15c"
  dark-ask-wash: "#25190c"
  dark-keep: "#82aaff"
  dark-keep-wash: "#101b30"
typography:
  display:
    fontFamily: "Inter, sans-serif"
    fontSize: "49px"
    fontWeight: 650
    letterSpacing: "-1px"
  headline:
    fontFamily: "Inter, sans-serif"
    fontSize: "35px"
    fontWeight: 650
    lineHeight: "38px"
    letterSpacing: "-0.7px"
  verdict:
    fontFamily: "Kalam, cursive"
    fontSize: "25px"
    fontWeight: 700
  body:
    fontFamily: "Inter, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: "21px"
  note:
    fontFamily: "Kalam, cursive"
    fontSize: "17px"
    fontWeight: 400
    lineHeight: "20px"
  mono:
    fontFamily: "JetBrains Mono, monospace"
    fontSize: "12px"
    fontWeight: 500
rounded:
  sm: "4px"
  md: "7px"
  lg: "10px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "16px"
  lg: "26px"
  xl: "54px"
components:
  button-primary:
    backgroundColor: "{colors.ink}"
    textColor: "{colors.paper}"
    rounded: "{rounded.md}"
    padding: "0 18px"
    height: "44px"
  button-primary-hover:
    backgroundColor: "{colors.ink-2}"
  button-secondary:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink}"
    rounded: "{rounded.md}"
    padding: "0 18px"
    height: "44px"
  input-folder:
    backgroundColor: "{colors.wash}"
    textColor: "{colors.ink}"
    rounded: "{rounded.md}"
    padding: "0 12px"
    height: "46px"
  navigation-repository-active:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink}"
    rounded: "{rounded.sm}"
    padding: "0 9px"
    height: "38px"
---

# Design System: Alice

## Overview

**Creative North Star: "The robot maid's housekeeping report"**

Alice turns Git maintenance into a domestic sorting ritual. The interface is quiet and exact so the evidence remains readable; Alice's handwritten notes and local illustrations supply the personality. The complete `idea/` prototype is the visual authority, not the discarded dashboard direction.

The native app uses three connected scenes: a welcoming subscription-and-folder setup, a three-pile worktree report, and a cleanup result. Product truth overrides prototype theater: unsupported actions are disabled or absent, demo scenes are labeled, and no bin, undo, schedule, or patch-saving claim appears without an implementation.

**Key Characteristics:**
- White-paper workspace with warm gray utility surfaces and thin rules.
- Sweep, ask, and keep as the three semantic color families.
- Inter for tasks, Kalam for Alice's voice, JetBrains Mono for Git identifiers.
- Local raster illustrations for greeting, sweeping, and rest.
- Standard desktop controls and a protected-focus confirmation dialog.

## Colors

Color carries verdict meaning first; decoration comes second.

### Primary
- **Sweep Red** (`sweep`): merged-work selection, cleanup emphasis, and the report's reclaimable segment.
- **Sweep Wash** (`sweep-wash`): expanded evidence for sweep candidates.

### Secondary
- **Ask Orange** (`ask`): uncertain-work boundaries and warnings.
- **Ask Ink** (`ask-ink`): handwritten notes for unpushed, dirty, or unintegrated work.
- **Ask Wash** (`ask-wash`): expanded evidence and repository scan warnings.

### Tertiary
- **Keep Blue** (`keep`): active/protected work, worker presence, and successful rest.
- **Keep Wash** (`keep-wash`): expanded evidence for protected work.

### Neutral
- **Paper** (`paper`): the main field and active repository row.
- **Warm Wash** (`wash`): repository rail and folder input.
- **Hairline Rule** (`rule`): separators, input borders, and secondary buttons.
- **Ink** (`ink`): headings and primary buttons.
- **Ink 2 / Ink 3** (`ink-2`, `ink-3`): supporting evidence and metadata without dropping into faint placeholder text. The disk-used bar uses Ink 3 at 72% opacity so secondary disk information stays quieter than headings and actions.

### TRUE DARK appearance
- **Paper** is exactly `#000000`; wash is `#141414` and rules are `#303030`. Do not substitute blue-gray panel colors.
- **Ink** is `#f4f2ed`, with `#b5b5b5` and `#949494` for supporting evidence. Body, secondary, metadata, and verdict colors meet at least 4.5:1 contrast against their assigned fields.
- **Verdicts** retain their meanings with appearance-specific red, orange, and blue values; tinted evidence uses dark tonal washes rather than light-theme inversion.
- **Artwork** uses transparent PNG derivatives with light neutral strokes and preserved chromatic details. White JPEG rectangles never sit on the black canvas.
- Rune's `--theme dark|light` selects an explicit launch appearance; otherwise the top bar's **Theme: Light/Dark** control changes the window immediately and stores the choice in `~/.config/alice/theme`. With no explicit flag or saved choice, Alice follows the system appearance.

**The Verdict First Rule.** Red, orange, and blue are reserved for their respective verdicts and directly related evidence; do not spend them as arbitrary decoration.

## Typography

**Display Font:** Inter, bundled locally in regular, medium, and semibold files.
**Body Font:** Inter.
**Alice's Hand:** Kalam, bundled locally in regular and bold.
**Git Identifier Font:** JetBrains Mono, bundled locally.

The sans carries decisions and dense evidence. Kalam is a human-scale annotation layer, never the body face. Monospace is reserved for branches, paths, and commit-shaped data.

### Hierarchy
- **Display** (650 weight, 48–49px, tight tracking): reclaimable bytes and cleanup result.
- **Headline** (650 weight, 35px / 38px): onboarding promise.
- **Verdict** (Kalam bold, 25px): sweep, ask, and keep group titles.
- **Body** (Inter regular, 14px / 21px): explanatory copy and evidence.
- **Note** (Kalam regular, 17px / 20px): Alice's one-line read of a worktree.
- **Git Label** (JetBrains Mono medium, 12px): branch and path identity.

**The One Voice Rule.** Handwriting belongs to Alice's short notes and group titles; it never replaces evidence, controls, or long-form explanation.

## Layout

The desktop target is 1200×780 points, with a 360×560 minimum and a compact breakpoint below 900 points. A 48-point unified bar stays fixed; compact windows remove secondary bar text but retain Theme and DEMO DATA. Onboarding centers a two-column composition with a 430-point illustration and 470-point form on desktop. Compact onboarding uses one fluid form column without the illustration and scrolls vertically when height is constrained.

Ctrl/Cmd + Plus zooms in, Ctrl/Cmd + Minus zooms out, and Ctrl/Cmd + 0 restores 100%. Rune's browser-style zoom ranges from 80% to 200% in 10% steps, persists in `~/.config/alice/zoom`, and lays out in the smaller or larger logical viewport so text stays crisp, compact breakpoints still apply, and overflowing content scrolls rather than clipping.

The desktop report uses a 218-point repository rail and a fluid main column. Its hero pairs the selected-space total and 480-point disk bar with a 290-point illustration. Compact report navigation becomes a 58-point horizontally scrolling repository band, the disk bar becomes fluid, nonessential hero art disappears, worktree notes move to a second line, and the footer stacks. Verdict groups keep the same ruled list grammar. Expanded evidence remains in flow and pushes later rows down. Build files replaces the repository rail with a full-width Cargo-target list and shares the same tabs, hero, ruled rows, and fixed review footer; compact group controls wrap onto a second line.

The result returns to a centered two-column composition on desktop: summary and removed branches left, resting Alice right. Compact result uses a top-aligned single column with a smaller illustration after the controls. Scanning uses one centered composition for illustration, loading text, and a 240-point progress track; a 25% sweep travels across it only while scanning through Rune's native `LoopSpec`, including reduced-motion handling. Empty and error states use a single centered reading column with fluid text widths. Confirmation never exceeds the window width and scrolls its branch list in compact windows.

## Elevation & Depth

The working surfaces are flat: hierarchy comes from paper/wash contrast and one-point rules. Shadows belong only to the confirmation dialog, where elevation reinforces protected focus. The dialog uses a soft downward shadow rather than an offset graphic effect.

## Motion

Motion gives Alice a composed presence and explains state rather than decorating every surface. Her illustration arrives once over 650 ms with a small opacity/scale settle. The sweeping scene is an authored eight-pose flipbook over 2.4 seconds: anticipation, a deep left stroke, lifted debris, a right follow-through, and recovery. The broom rotates rigidly around its grip, debris moves independently, and the dustpan and protected plants stay unchanged. Light and TRUE DARK sprites use isolated rigid layers and frame-specific redrawn blinks, never whole-image warping, and play through Rune's wall-clock `Cycle` at 12 Hz with only the visible pose drawn. Greeting rigidly moves the hand, plug, and plant; tea moves the cup, leg, and arrow, with a redrawn blink in each. All three use the same 2.4-second playback cycle. Compact layouts omit the artwork and its loops; during worker activity, the report's Alice rests and the blue worker indicator pulses instead.

Buttons and tabs compress subtly on press, checks arrive with a short scale/fade, chevrons rotate between open and closed, evidence enters with a 6-point offset and fade, and disk segments spring to freshly observed values. Routine feedback uses Rune's `SNAPPY` motion; the scanning composition combines the flipbook with its existing progress sweep. Rune's reduced-motion path lands arrivals and spatial changes at their resting pose, holds the first sprite of every scene, and disables press scaling. No list items animate merely because they scrolled into view.

## Shapes

Controls use restrained 4–10 point radii and thin borders. Checkboxes are 20-point rounded squares with a drawn check; protected rows replace the checkbox with a lock. The disk bar is a 12-point rounded track with muted Ink 3 used space and a red reclaimable segment. No decorative clipped artwork or card-heavy nesting.

## Components

### Buttons
- **Primary:** Ink field on a 44-point control with Paper label and 18-point horizontal padding; hover shifts to Ink 2 and press uses a neutral response.
- **Secondary:** paper field, rule border, same dimensions and type.
- **Text action:** transparent field for Select all / Deselect all.
- **Disabled:** visibly muted and without a press handler. Unsupported providers and unsafe worktrees are not merely styled as available.

### Inputs / Fields
- **Folder input:** 46-point wash field, 7-point radius, folder pictogram, JetBrains Mono path, visible label, and a 32-point Choose button backed by the native desktop portal. Selected roots appear as a ruled list with Remove actions above the field. Add folder and Enter add the typed or chosen path without starting a scan; Send Alice in scans every root, including an uncommitted draft. Roots persist in `~/.config/alice/folders`, and Edit folders returns from the report to this editor. The single-line path editor is 320 points on desktop and fluid in compact forms, so long text cannot push Choose outside the border or wrap into the helper. Its 38.5-point hit area starts 5.5 points below the field's top, placing the glyphs at the folder icon's optical center. Choose has its own 76-point lane inside the field. Helper and portal-error text reserves a full 20-point line and wraps instead of clipping.
- **Worker input:** Kimi Code, Claude Code, and GitHub Copilot radio rows are enabled only when the corresponding local CLI is detected. The selected worker persists in `~/.config/alice/worker`; unavailable providers read Not configured. Explanation buttons, the top bar, and the tray use the selected worker's name.

### Navigation
- **Repository rail:** all-repositories entry plus only repositories with nonzero linked-worktree bytes, sorted largest first on desktop; one horizontally scrolling band with 150-point entries in compact windows. Active rows use paper, a rule border, and medium type; sizes use tabular-looking metadata and truncation for long names. Edit folders is always available after the repository entries. The footer counts only visible repositories with worktrees.

### Verdict Rows
- **Sweep:** red selectable checkbox, branch, repository, handwritten merged note, age, and apparent size.
- **Ask:** disabled orange boundary, uncertainty note, expandable evidence, optional Kimi explanation.
- **Keep:** blue lock and active/protected note. No cleanup control is offered.
- **Evidence:** a tinted inset panel fills the row width without an offset margin and leaves 12 points above and below, leading with a direct verdict, then working-tree, integration, upstream, reason, and path facts in two desktop columns or one compact column. Body text is 12/17. Merged rows omit the repetitive merged note; uncertain and active rows keep it. Branch text and a separate 44×44 chevron button both expand and collapse evidence.

### Build Files
- **Tabs:** Worktrees and Build files use compact 44-point text controls with 16-point branch and crate vector icons, counts, and a 2-point active underline instead of full-width segmented boxes. Build files drops the repository rail so configured external targets can use the full content width.
- **Target rows:** red selectable checkbox for idle verified targets, orange lock for protected targets, target name, repository and checkout counts, a direct idle/active/unverified/empty note, observed size, and expandable evidence. Evidence leads with a plain-language verdict and consequence, then names observed and reclaimable bytes, safety reasons, target path, and repositories. Explain with Kimi uses the same facts without changing eligibility.
- **Sorting:** a shared control cycles Largest, Smallest, A–Z, and Status within each verdict group. Sorting never changes selection or eligibility.
- **Cleanup footer:** only generated entries and their reclaimable bytes feed the review action. Active, internal, checkout-containing, unverified, and empty targets never become selectable. Compact group totals and Select all wrap below the group title.

### Confirmation
- **Dialog:** up to 490-point Paper panel with 24 points of window clearance, 10-point radius, soft black shadow, selected branches or target paths and bytes, fresh-scan warning, and explicit Keep/Sweep or Keep/Clean actions. Compact dialogs use tighter padding and a 120-point scrolling item list. Escape, outside dismissal, and keyboard activation preserve the safe choice. Build cleanup explicitly preserves source, worktrees, branches, target roots, and unrecognized entries.

### Tray menu
- **Composition:** a 340-point popup anchored to Alice's color line-art bar icon, visually independent of the clock. The status bar shows the real root-filesystem used percentage next to Alice's icon and no separate disk widget. The popup contains Alice, a short Kalam note, Open Alice and Scan now actions, observed worktree bytes and free disk space, local worker availability, status, and Close. When configured external Cargo targets exist, a conditional Rust builds row shows their deduplicated apparent bytes and disk share, with a clean-in-app affordance. It has no icon-treatment or theme controls.
- **Behavior:** Open launches or focuses every saved root. Scan launches with `--scan`, or focuses the existing window and sends Ctrl+R. Worktree and Rust-target usage refresh on shell start, popup open, folder-file changes, and every five minutes; refreshes arriving during a scan queue rather than being lost, and a failed refresh preserves the last valid value. Outside click and Escape dismiss; Up/Down and Enter operate Open, Scan, and Close. The tooltip explains the WT percentage and B build size and appears only while hovered with the popup closed.
- **Treatment:** only the original color line art appears. The bar uses a dedicated head-and-headdress crop inside the standard status slot, with the real used-disk percentage rendered by a companion button rather than a separate disk widget. The popup keeps the full illustration. Both add a small bottom-pivot nod; the popup nod runs only while open, and `Style.reduceMotion` stops and resets rotation. Surface, text, and border use Omarchy's `Commons.Color.popups` tokens directly. Wash and rules use 6% and 18% text alpha; supporting text uses 72%. The tooltip uses `Commons.Color.tooltip`. Do not force white or black popup surfaces that fight the active OS theme.
- **Integration:** the user-owned Omarchy plugin uses a real `BarWidget` in `Alice.qml` and a nested `Panel.qml` so Omarchy registers hover, click, and popup ownership correctly. A watched `FileView` reads the native app's `~/.config/alice/folders` list, with the legacy `folder` setting as a fallback. `alice-cli usage <folder> [more folders...]` supplies deduplicated worktree bytes, external Rust-target bytes and counts, and their aggregate disk capacities as numeric JSON. Free bytes come from `findmnt -n -b -o USED,SIZE,AVAIL /` and refresh every 30 seconds; the bar percentage comes from `findmnt -n -b -o USED,SIZE /` on the same interval. Worker availability comes from the local `kimi` executable.

## Do's and Don'ts

### Do:
- **Do** keep the three-pile report and handwritten verdict notes at the center of the product.
- **Do** use local raster artwork and bundled fonts so every scene renders offline.
- **Do** label demonstration scenes and report real results without prototype-only promises.
- **Do** preserve the readable list grammar at 1080×700 and allow vertical scrolling for overflow.

### Don't:
- **Don't** resurrect the discarded metric-card dashboard or laundry-fold motif.
- **Don't** make ask or keep rows selectable until a safe workflow exists for them.
- **Don't** invent bin, undo, scheduled-scan, patch-saving, or provider capabilities.
- **Don't** use handwriting for controls, evidence, or long paragraphs.
