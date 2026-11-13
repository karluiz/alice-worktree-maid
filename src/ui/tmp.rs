use super::{Paint, font, hand, mono};
use crate::State;
use alice::{ScanBatch, TmpEntry, TmpUsage, format_bytes};
use rune::{El, ScrollSpec, col, row, text};

pub(super) fn ready(state: &State, batch: &ScanBatch, p: Paint) -> El {
    col().grow(1.0).min_w(0.0).min_h(0.0).child(
        col()
            .grow(1.0)
            .min_h(0.0)
            .scroll(ScrollSpec {
                stick_bottom: false,
            })
            .scroll_edge_fade(12.0)
            .child(body(state, batch, p)),
    )
}

fn body(state: &State, batch: &ScanBatch, p: Paint) -> El {
    let usage = &state.tmp;
    let mut content = col()
        .w_full()
        .padding(if p.compact { 16.0 } else { 26.0 })
        .child(super::builds::tabs(state, batch, p))
        .child(header(usage, p));
    if usage.entries.is_empty() {
        content = content.child(
            text("Nothing inside /tmp right now.")
                .font(font(12.0, 400.0))
                .py(14.0)
                .color(p.ink3),
        );
    }
    for entry in &usage.entries {
        content = content.child(entry_row(entry, p));
    }
    content
}

fn header(usage: &TmpUsage, p: Paint) -> El {
    let total = usage.total_bytes.unwrap_or(0);
    let percent = if total > 0 {
        ((usage.bytes as f64 / total as f64) * 100.0).round() as u64
    } else {
        0
    };
    col()
        .w_full()
        .gap(8.0)
        .pb(14.0)
        .mb(2.0)
        .border_sides([0.0, 0.0, 1.0, 0.0], p.rule)
        .child(
            row()
                .w_full()
                .justify_between()
                .center_y()
                .child(text("inside /tmp").font(hand(25.0, 700.0)).color(p.ink))
                .child(
                    text(if total > 0 {
                        format!("{percent}% of tmpfs")
                    } else {
                        "tmpfs".to_owned()
                    })
                    .font(font(12.0, 600.0))
                    .color(p.ink2),
                ),
        )
        .child(
            text(summary_line(usage))
                .font(font(13.0, 400.0).lh(19.0))
                .wrap()
                .color(p.ink2),
        )
        .child(
            text("Alice only watches this folder. Sockets, locks, and files in use belong to running programs, so nothing here is selected or deleted. Command-R refreshes.")
                .font(font(11.0, 400.0).lh(15.0))
                .wrap()
                .color(p.ink3),
        )
}

fn summary_line(usage: &TmpUsage) -> String {
    match usage.total_bytes {
        Some(total) => format!(
            "/tmp holds {} of the {} tmpfs.",
            format_bytes(usage.bytes),
            format_bytes(total)
        ),
        None => format!("/tmp holds {}.", format_bytes(usage.bytes)),
    }
}

fn entry_row(entry: &TmpEntry, p: Paint) -> El {
    row()
        .w_full()
        .py(11.0)
        .gap(13.0)
        .center_y()
        .border_sides([0.0, 0.0, 1.0, 0.0], p.rule)
        .child(
            text(entry.name.clone())
                .grow(1.0)
                .min_w(0.0)
                .font(mono(12.0, 400.0))
                .truncate_middle()
                .color(p.ink),
        )
        .child(
            text(format_bytes(entry.bytes))
                .font(font(12.0, 600.0))
                .color(p.ink2),
        )
}
