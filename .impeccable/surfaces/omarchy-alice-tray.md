# Omarchy Alice tray

Scope and visitor mode: Operate. Implement the user-approved `design/tray-preview/` experience in the existing Omarchy plugin. Preserve the Rust cleanup engine, native app behavior, and all other shell widgets.

## Direction contract

THESIS: Keep the launcher-only Alice icon and compact native tray menu minimal: original color line art, real Open/Scan actions, observed worktree usage, a separate Rust-target metric with a clean-in-app affordance when present, and no appearance controls. Theme selection belongs to the native window.

OWN-WORLD: The active Omarchy theme owns the surface: `Commons.Color.popups.background`, `text`, and `border`, with alpha-derived wash, rules, and secondary text. Tooltip colors come from `Commons.Color.tooltip`. Thin theme-aware rules, Inter for controls, Kalam only for Alice's short note, and JetBrains Mono for keycaps. Only the original color line art appears, with a small bottom-pivot nod on the bar and popup art when the panel is open; `Style.reduceMotion` disables the nod and resets rotation. Pixel and monochrome treatments are not part of the tray. No circle, glass, gradients, or decorative accents.

STORY: The user opens the tray, sees Alice, observed worktree usage, any configured external Cargo target usage, and the selected local worker's availability, opens or focuses the app, reviews and confirms build cleanup in the native Build files tab, starts a real scan, and dismisses the menu by pointer or keyboard. The tray itself never deletes anything. Worker selection persists in the same config file and can be Kimi Code, Claude Code, or GitHub Copilot. Theme changes happen inside the open app, never in this menu.

FIRST VIEWPORT: A roughly 340px card anchored to Alice's icon, visually independent of the clock. A real `BarWidget` owns the color icon and an explanatory hover tooltip; the real root-filesystem used percentage appears next to Alice in place of the separate disk widget. The nested panel owns the card. Alice art and name sit above two action rows, observed worktree bytes and free space, a conditional shared Rust-target row with a clean-in-app note, the selected local worker's status, then a status line and Close. Open, Scan, and Close are the only keyboard destinations.

FORM: First and only composition, explicitly approved by the user through the working preview. seed: none (unavailable).

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance.
