use super::{ButtonKind, Paint, animated_art, button, font, hand, mono};
use crate::{CleanupKind, Screen, State, start_scan};
use alice::format_bytes;
use rune::{El, ScrollSpec, col, row, text};

pub(super) fn swept(state: &State, p: Paint) -> El {
    let Some(summary) = &state.summary else {
        return col()
            .grow(1.0)
            .center()
            .gap(14.0)
            .child(
                text("No cleanup result is available")
                    .font(font(20.0, 600.0))
                    .color(p.ink),
            )
            .child(button(
                "back-no-result",
                "Back to the report",
                p,
                ButtonKind::Dark,
                true,
                |state| state.screen = Screen::Report,
            ));
    };
    let mut branches = col().w_full().gap(0.0);
    for (branch, bytes) in &summary.branches {
        branches = branches.child(
            row()
                .w_full()
                .py(10.0)
                .center_y()
                .justify_between()
                .border_sides([0.0, 0.0, 1.0, 0.0], p.rule)
                .child(
                    text(branch)
                        .grow(1.0)
                        .min_w(0.0)
                        .font(mono(11.0, 400.0))
                        .truncate()
                        .color(p.ink2),
                )
                .child(
                    text(format_bytes(*bytes))
                        .font(font(11.0, 400.0))
                        .color(p.ink3),
                ),
        );
    }
    let builds = summary.kind == CleanupKind::Builds;
    let target_noun = if summary.count == 1 {
        "Cargo target"
    } else {
        "Cargo targets"
    };
    let outcome = match (builds, summary.free_after) {
        (true, Some(free)) => format!(
            "{} {target_noun} cleaned. You now have {} free. Source and Git history were not touched.",
            summary.count,
            format_bytes(free)
        ),
        (true, None) => format!(
            "{} {target_noun} cleaned. Source and Git history were not touched.",
            summary.count
        ),
        (false, Some(free)) => format!(
            "{} worktree directories removed. You now have {} free. Every branch remains in Git.",
            summary.count,
            format_bytes(free)
        ),
        (false, None) => format!(
            "{} worktree directories removed. Every branch remains in Git.",
            summary.count
        ),
    };
    let summary_view = col()
        .w(if p.compact {
            (p.width - 40.0).min(560.0)
        } else {
            470.0
        })
        .gap(16.0)
        .child(
            text("all tidy. she's having tea.")
                .font(hand(20.0, 400.0))
                .color(p.keep),
        )
        .child(
            text(format!("{} back", format_bytes(summary.bytes)))
                .font(font(49.0, 650.0).tracking(-1.0))
                .color(p.ink),
        )
        .child(
            text(outcome)
                .font(font(14.0, 400.0).lh(20.0))
                .wrap()
                .color(p.ink2),
        )
        .child(
            col()
                .pt(8.0)
                .border_sides([1.0, 0.0, 0.0, 0.0], p.rule)
                .child(branches),
        )
        .child(
            text(if builds {
                "Alice removed only verified generated entries. Target roots and unrecognized files remain."
            } else {
                "Alice confirmed each removal with Git. No bin, no automatic branch deletion."
            })
            .font(font(12.0, 400.0).lh(17.0))
            .wrap()
            .color(p.ink2),
        )
        .child(
            row()
                .pt(6.0)
                .gap(10.0)
                .child(button(
                    "back-report",
                    "Back to the report",
                    p,
                    ButtonKind::Dark,
                    true,
                    |state| state.screen = Screen::Report,
                ))
                .child(button(
                    "scan-after-sweep",
                    "Scan again",
                    p,
                    ButtonKind::Outline,
                    true,
                    start_scan,
                )),
        );
    let content = if p.compact {
        col()
            .w_full()
            .min_h(680.0)
            .px(20.0)
            .py(24.0)
            .items_center()
            .gap(24.0)
            .child(summary_view)
            .child(animated_art(
                &state.rest_frames,
                &state.rest_dark_frames,
                p,
                280.0,
                209.0,
                true,
            ))
    } else {
        row()
            .w_full()
            .min_h(680.0)
            .px(54.0)
            .py(28.0)
            .center()
            .gap(54.0)
            .child(summary_view)
            .child(animated_art(
                &state.rest_frames,
                &state.rest_dark_frames,
                p,
                410.0,
                306.0,
                true,
            ))
    };
    col()
        .grow(1.0)
        .min_h(0.0)
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .child(content)
}
