use super::{ButtonKind, Paint, button, check, chevron, font, hand, lock, mono};
use crate::{
    Activity, BuildSort, ReportTab, State, start_build_explanation, start_cleanup, start_scan,
};
use alice::{RustTarget, ScanBatch, format_bytes};
use rune::tok::motion;
use rune::{Color, El, LineCap, ScrollSpec, Shape, Transition, canvas, col, el, row, text};
use std::path::PathBuf;

pub(super) fn tabs(state: &State, batch: &ScanBatch, p: Paint) -> El {
    row()
        .w_full()
        .pb(16.0)
        .gap(20.0)
        .child(tab(
            "tab-worktrees",
            format!("Worktrees · {}", batch.linked().count()),
            ReportTab::Worktrees,
            state.report_tab == ReportTab::Worktrees,
            p,
        ))
        .child(tab(
            "tab-builds",
            format!("Build files · {}", batch.rust.target_count),
            ReportTab::Builds,
            state.report_tab == ReportTab::Builds,
            p,
        ))
        .child(tab(
            "tab-tmp",
            format!("Tmp · {}", format_bytes(state.tmp.bytes)),
            ReportTab::Tmp,
            state.report_tab == ReportTab::Tmp,
            p,
        ))
}

fn tab(key: &str, label: String, tab: ReportTab, active: bool, p: Paint) -> El {
    let color = if active { p.ink } else { p.ink3 };
    el().key(key)
        .role("tab")
        .label(label.clone())
        .checked(active)
        .focusable(true)
        .h(44.0)
        .px(4.0)
        .center_y()
        .border_sides(
            [0.0, 0.0, if active { 2.0 } else { 0.0 }, 0.0],
            if active { p.ink } else { Color::TRANSPARENT },
        )
        .hover(p.wash)
        .motion(motion::SNAPPY)
        .press_scale(0.985)
        .on_press(move |state: &mut State| state.report_tab = tab)
        .child(
            row()
                .gap(7.0)
                .center_y()
                .child(match tab {
                    ReportTab::Worktrees => worktree_icon(color),
                    ReportTab::Builds => build_icon(color),
                    ReportTab::Tmp => tmp_icon(color),
                })
                .child(
                    text(label)
                        .font(font(12.0, if active { 600.0 } else { 400.0 }))
                        .color(color)
                        .a11y_hidden(),
                ),
        )
}

fn worktree_icon(color: Color) -> El {
    let shape = Shape::new()
        .stroke_with(
            vec![[3.0, 2.0], [3.0, 14.0]],
            false,
            1.5,
            LineCap::Round,
            color,
        )
        .stroke_with(
            vec![[3.0, 6.0], [12.0, 6.0]],
            false,
            1.5,
            LineCap::Round,
            color,
        )
        .stroke_with(
            vec![[12.0, 6.0], [12.0, 14.0]],
            false,
            1.5,
            LineCap::Round,
            color,
        )
        .stroke_with(
            vec![[1.0, 2.0], [5.0, 2.0]],
            false,
            1.5,
            LineCap::Round,
            color,
        )
        .stroke_with(
            vec![[10.0, 14.0], [14.0, 14.0]],
            false,
            1.5,
            LineCap::Round,
            color,
        )
        .view_box(16.0, 16.0);
    canvas(shape).size(16.0, 16.0).a11y_hidden()
}

fn build_icon(color: Color) -> El {
    let shape = Shape::new()
        .stroke_with(
            vec![
                [2.0, 5.0],
                [8.0, 2.0],
                [14.0, 5.0],
                [14.0, 12.0],
                [8.0, 15.0],
                [2.0, 12.0],
            ],
            true,
            1.5,
            LineCap::Round,
            color,
        )
        .stroke_with(
            vec![[2.0, 5.0], [8.0, 8.0], [14.0, 5.0]],
            false,
            1.5,
            LineCap::Round,
            color,
        )
        .stroke_with(
            vec![[8.0, 8.0], [8.0, 15.0]],
            false,
            1.5,
            LineCap::Round,
            color,
        )
        .view_box(16.0, 16.0);
    canvas(shape).size(16.0, 16.0).a11y_hidden()
}

fn tmp_icon(color: Color) -> El {
    let shape = Shape::new()
        .stroke_with(
            vec![
                [8.0, 2.0],
                [12.2, 3.8],
                [14.0, 8.0],
                [12.2, 12.2],
                [8.0, 14.0],
                [3.8, 12.2],
                [2.0, 8.0],
                [3.8, 3.8],
            ],
            true,
            1.5,
            LineCap::Round,
            color,
        )
        .stroke_with(
            vec![[8.0, 4.5], [8.0, 8.5]],
            false,
            1.5,
            LineCap::Round,
            color,
        )
        .stroke_with(
            vec![[8.0, 8.5], [10.5, 10.0]],
            false,
            1.5,
            LineCap::Round,
            color,
        )
        .view_box(16.0, 16.0);
    canvas(shape).size(16.0, 16.0).a11y_hidden()
}

pub(super) fn ready(state: &State, batch: &ScanBatch, p: Paint) -> El {
    let content = if batch.rust.targets.is_empty() {
        col()
            .grow(1.0)
            .center()
            .gap(12.0)
            .child(
                text("No Cargo build targets")
                    .font(font(28.0, 650.0))
                    .color(p.ink),
            )
            .child(
                text("No configured external Cargo target directories were found in this scan.")
                    .font(font(13.0, 400.0))
                    .wrap()
                    .color(p.ink2),
            )
            .child(button(
                "build-empty-scan",
                "Scan again",
                p,
                ButtonKind::Outline,
                true,
                start_scan,
            ))
    } else {
        col()
            .grow(1.0)
            .min_h(0.0)
            .scroll(ScrollSpec {
                stick_bottom: false,
            })
            .scroll_edge_fade(12.0)
            .child(build_body(state, batch, p))
    };
    col()
        .grow(1.0)
        .min_w(0.0)
        .min_h(0.0)
        .child(content)
        .child(commit_bar(state, p))
}

fn build_body(state: &State, batch: &ScanBatch, p: Paint) -> El {
    let selected_bytes: u64 = batch
        .rust
        .targets
        .iter()
        .filter(|target| state.selected_builds.contains(&target.path))
        .map(|target| target.reclaimable_bytes)
        .sum();
    let ready: Vec<_> = batch
        .rust
        .targets
        .iter()
        .filter(|target| target.cleanable())
        .collect();
    let protected: Vec<_> = batch
        .rust
        .targets
        .iter()
        .filter(|target| !target.cleanable())
        .collect();
    let mut body = col()
        .w_full()
        .padding(if p.compact { 16.0 } else { 26.0 })
        .gap(0.0)
        .child(tabs(state, batch, p))
        .child(super::report::hero(state, batch, selected_bytes, p, true))
        .child(build_group(state, "sweep the build files", ready, true, p))
        .child(build_group(state, "leave alone", protected, false, p));
    if !batch.warnings.is_empty() {
        body = body.child(
            col()
                .mt(24.0)
                .padding(14.0)
                .gap(5.0)
                .fill(p.ask_wash)
                .radius(7.0)
                .child(
                    text("Some repositories were skipped")
                        .font(font(12.0, 700.0))
                        .color(p.ask_ink),
                )
                .children(
                    batch
                        .warnings
                        .iter()
                        .map(|warning| text(warning).font(font(11.0, 400.0)).wrap().color(p.ink2)),
                ),
        );
    }
    body
}

fn build_group(
    state: &State,
    title: &str,
    mut items: Vec<&RustTarget>,
    selectable: bool,
    p: Paint,
) -> El {
    if items.is_empty() {
        return el();
    }
    items.sort_by(|a, b| {
        let ordering = match state.build_sort {
            BuildSort::Largest => b.disk_bytes.cmp(&a.disk_bytes),
            BuildSort::Smallest => a.disk_bytes.cmp(&b.disk_bytes),
            BuildSort::Name => a
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
                .cmp(
                    &b.path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase(),
                ),
            BuildSort::Status => b
                .cleanable()
                .cmp(&a.cleanable())
                .then_with(|| b.disk_bytes.cmp(&a.disk_bytes)),
        };
        ordering.then_with(|| a.path.cmp(&b.path))
    });
    let total: u64 = items.iter().map(|target| target.disk_bytes).sum();
    let paths: Vec<PathBuf> = items.iter().map(|target| target.path.clone()).collect();
    let all_on = selectable
        && paths
            .iter()
            .all(|path| state.selected_builds.contains(path));
    let mut list = col().w_full();
    for target in items {
        list = list.child(build_row(state, target, selectable, p));
    }
    let title_view =
        text(title)
            .font(hand(25.0, 700.0))
            .color(if selectable { p.sweep } else { p.ask_ink });
    let count_view = text(format!("{} · {}", paths.len(), format_bytes(total)))
        .font(font(11.0, 400.0))
        .color(p.ink2);
    let select_view = if selectable {
        button(
            format!("select-build-pile-{all_on}"),
            if all_on { "Deselect all" } else { "Select all" },
            p,
            ButtonKind::Text,
            true,
            move |state| {
                if all_on {
                    for path in &paths {
                        state.selected_builds.remove(path);
                    }
                } else {
                    state.selected_builds.extend(paths.iter().cloned());
                }
            },
        )
    } else {
        el()
    };
    let sort_view = button(
        "sort-build-targets",
        state.build_sort.label(),
        p,
        ButtonKind::Text,
        true,
        |state| state.build_sort = state.build_sort.next(),
    );
    let heading = if p.compact {
        col()
            .w_full()
            .pb(8.0)
            .border_sides([0.0, 0.0, 1.0, 0.0], p.ink)
            .child(title_view)
            .child(
                row()
                    .w_full()
                    .pt(4.0)
                    .center_y()
                    .justify_between()
                    .child(count_view)
                    .child(sort_view)
                    .child(select_view),
            )
    } else {
        row()
            .w_full()
            .pb(8.0)
            .center_y()
            .border_sides([0.0, 0.0, 1.0, 0.0], p.ink)
            .child(title_view)
            .child(el().grow(1.0))
            .child(count_view)
            .child(sort_view)
            .child(select_view)
    };
    col().w_full().mt(24.0).child(heading).child(list)
}

fn has_reason(target: &RustTarget, needle: &str) -> bool {
    target.reasons.iter().any(|reason| reason.contains(needle))
}

fn verdict(target: &RustTarget) -> &'static str {
    if target.cleanable() {
        return "You can clean this target. Alice verified generated Cargo output and no active build is using it. Rust will rebuild these entries when needed.";
    }
    if has_reason(target, "currently using this target") {
        return "Wait for the current build to finish. Alice will not remove a target while Cargo or rustc is using it.";
    }
    if has_reason(target, "No Cargo fingerprint") {
        return "Keep this target. Alice cannot prove these directories are generated Cargo output.";
    }
    if has_reason(target, "No Cargo cache tag") {
        return "Keep this target. Cargo metadata is missing, so Alice cannot verify it.";
    }
    if has_reason(target, "No generated Cargo artifacts") {
        return "Nothing to clean here. Alice found no generated Cargo artifacts.";
    }
    "Keep this target for now. Alice could not prove that cleanup is safe."
}

fn verdict_note(target: &RustTarget) -> &'static str {
    if target.cleanable() {
        return "idle. rust can rebuild this.";
    }
    if has_reason(target, "currently using this target") {
        return "a build is running. not now.";
    }
    if has_reason(target, "No Cargo fingerprint") {
        return "not verified cargo output.";
    }
    if has_reason(target, "No Cargo cache tag") {
        return "cargo metadata missing.";
    }
    if has_reason(target, "No generated Cargo artifacts") {
        return "nothing to reclaim.";
    }
    "needs a fresh look."
}

fn build_row(state: &State, target: &RustTarget, selectable: bool, p: Paint) -> El {
    let path = target.path.clone();
    let on = state.selected_builds.contains(&path);
    let open = state.expanded_build.as_ref() == Some(&path);
    let name = target
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Cargo target");
    let checkbox = if selectable {
        let toggle = path.clone();
        el().key(format!("select-build-{}", path.display()))
            .role("checkbox")
            .label(format!("Clean {name}"))
            .checked(on)
            .focusable(true)
            .size(20.0, 20.0)
            .center()
            .radius(4.0)
            .fill(if on { p.sweep } else { p.paper })
            .border(1.0, if on { p.sweep } else { p.ink3 })
            .motion(motion::SNAPPY)
            .press_scale(0.94)
            .on_press(move |state: &mut State| {
                if !state.selected_builds.insert(toggle.clone()) {
                    state.selected_builds.remove(&toggle);
                }
            })
            .child(if on { check(p.paper) } else { el() })
    } else {
        el().size(20.0, 20.0).center().child(lock(p.ask_ink))
    };
    let expand = path.clone();
    let chevron_expand = expand.clone();
    let note = verdict_note(target);
    let row_view = row()
        .w_full()
        .min_h(66.0)
        .py(10.0)
        .gap(13.0)
        .center_y()
        .border_sides([0.0, 0.0, 1.0, 0.0], p.rule)
        .child(checkbox)
        .child(
            el().key(format!("inspect-build-{}", path.display()))
                .grow(1.0)
                .min_w(0.0)
                .role("button")
                .label(format!("Inspect {name}"))
                .focusable(true)
                .on_press(move |state: &mut State| {
                    state.expanded_build = if state.expanded_build.as_ref() == Some(&expand) {
                        None
                    } else {
                        Some(expand.clone())
                    };
                })
                .child(
                    col()
                        .gap(4.0)
                        .child(text(name).font(mono(12.0, 500.0)).truncate().color(p.ink))
                        .child(
                            text(format!(
                                "Cargo · {} repositories · {} checkouts",
                                target.repositories.len(),
                                target.checkouts.len()
                            ))
                            .font(font(10.0, 400.0))
                            .color(p.ink2),
                        ),
                ),
        )
        .child(if p.compact {
            el()
        } else {
            text(note)
                .w(220.0)
                .font(hand(17.0, 400.0).lh(20.0))
                .wrap()
                .color(if selectable { p.sweep } else { p.ask_ink })
        })
        .child(
            text(format_bytes(target.disk_bytes))
                .w(if p.compact { 66.0 } else { 76.0 })
                .font(font(11.0, 600.0))
                .color(p.ink),
        )
        .child(
            el().key(format!("inspect-build-chevron-{}", path.display()))
                .role("button")
                .label(if open { "Collapse" } else { "Expand" })
                .focusable(true)
                .size(44.0, 44.0)
                .center()
                .radius(5.0)
                .hover(p.wash)
                .motion(motion::SNAPPY)
                .press_scale(0.96)
                .on_press(move |state: &mut State| {
                    state.expanded_build = if state.expanded_build.as_ref() == Some(&chevron_expand)
                    {
                        None
                    } else {
                        Some(chevron_expand.clone())
                    };
                })
                .child(chevron(open, p.ink2)),
        );
    let mut container = col().w_full().motion(motion::SNAPPY).child(row_view);
    if p.compact {
        container = container.child(
            text(note)
                .ml(33.0)
                .pb(10.0)
                .font(hand(17.0, 400.0).lh(20.0))
                .wrap()
                .color(if selectable { p.sweep } else { p.ask_ink }),
        );
    }
    if open {
        container = container.child(build_evidence(state, target, selectable, p));
    }
    container
}

fn build_evidence(state: &State, target: &RustTarget, selectable: bool, p: Paint) -> El {
    let mut panel = col()
        .w_full()
        .mt(12.0)
        .mb(12.0)
        .padding(if p.compact { 12.0 } else { 14.0 })
        .gap(7.0)
        .radius(7.0)
        .fill(if selectable { p.sweep_wash } else { p.ask_wash })
        .transition(Transition::offset(0.0, 6.0).combined(Transition::opacity()))
        .child(
            text("What she checked")
                .font(font(11.0, 700.0))
                .color(p.ink),
        )
        .child(
            text(verdict(target))
                .font(font(12.0, 600.0).lh(17.0))
                .wrap()
                .color(if selectable { p.sweep } else { p.ask_ink }),
        )
        .child(
            text(format!(
                "{} observed · {} in generated entries",
                format_bytes(target.disk_bytes),
                format_bytes(target.reclaimable_bytes)
            ))
            .font(font(11.0, 400.0).lh(15.0))
            .wrap()
            .color(p.ink2),
        );
    for reason in &target.reasons {
        panel = panel.child(
            text(reason)
                .font(font(11.0, 400.0).lh(15.0))
                .wrap()
                .color(p.ink2),
        );
    }
    panel = panel
        .child(
            text(target.path.display().to_string())
                .font(mono(10.0, 400.0))
                .truncate_middle()
                .color(p.ink3),
        )
        .child(
            text(
                target
                    .repositories
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(" · "),
            )
            .font(mono(10.0, 400.0))
            .truncate_middle()
            .color(p.ink3),
        )
        .child(row().pt(6.0).child(button(
            "explain-build-target",
            if state.activity == Activity::Explaining {
                format!("Asking {}…", state.worker_provider.label())
            } else {
                format!("Explain with {}", state.worker_provider.label())
            },
            p,
            ButtonKind::Outline,
            state.activity == Activity::Idle,
            start_build_explanation,
        )));
    if let Some((worker_path, summary)) = &state.worker
        && worker_path == &target.path
    {
        panel = panel.child(
            text(summary)
                .font(font(12.0, 400.0).lh(17.0))
                .wrap()
                .color(p.ink),
        );
    }
    panel
}

fn commit_bar(state: &State, p: Paint) -> El {
    let count = state.selected_builds.len();
    let bytes = state.batch().map_or(0, |batch| {
        batch
            .rust
            .targets
            .iter()
            .filter(|target| state.selected_builds.contains(&target.path))
            .map(|target| target.reclaimable_bytes)
            .sum()
    });
    let message = text("Only generated Cargo entries are selected. Source, worktrees, and Git history stay; every batch is confirmed.")
        .grow(1.0).font(font(11.0, 400.0).lh(15.0)).wrap().color(p.ink2);
    let action = button(
        "clean-builds-selected",
        if state.activity == Activity::Cleaning {
            "Cleaning…".to_owned()
        } else if count == 0 {
            "Pick build files to clean".to_owned()
        } else {
            format!(
                "Review {count} {} · {}",
                if count == 1 {
                    "Cargo target"
                } else {
                    "Cargo targets"
                },
                format_bytes(bytes)
            )
        },
        p,
        ButtonKind::Dark,
        count > 0 && state.activity == Activity::Idle,
        |state| state.confirming = true,
    );
    if p.compact {
        col()
            .w_full()
            .px(16.0)
            .py(10.0)
            .gap(8.0)
            .border_sides([1.0, 0.0, 0.0, 0.0], p.rule)
            .child(message)
            .child(action)
    } else {
        row()
            .w_full()
            .px(24.0)
            .py(12.0)
            .gap(18.0)
            .center_y()
            .justify_between()
            .border_sides([1.0, 0.0, 0.0, 0.0], p.rule)
            .child(message)
            .child(action)
    }
}

pub(super) fn confirmation(state: &State, p: Paint) -> El {
    let selected: Vec<_> = state
        .batch()
        .into_iter()
        .flat_map(|batch| batch.rust.targets.iter())
        .filter(|target| state.selected_builds.contains(&target.path))
        .collect();
    let bytes: u64 = selected.iter().map(|target| target.reclaimable_bytes).sum();
    let mut targets = col().gap(6.0);
    for target in selected {
        targets = targets.child(
            text(format!(
                "{} · {}",
                target.path.display(),
                format_bytes(target.reclaimable_bytes)
            ))
            .font(mono(11.0, 500.0))
            .wrap()
            .color(p.ink),
        );
    }
    el()
        .key("confirmation")
        .absolute(0.0, 0.0)
        .w_full()
        .h_full()
        .center()
        .fill(p.scrim)
        .modal(true, |state: &mut State| state.confirming = false)
        .child(
            col()
                .key("dialog")
                .role("dialog")
                .label("Confirm Cargo build cleanup")
                .w((p.width - 24.0).min(490.0))
                .padding(if p.compact { 18.0 } else { 26.0 })
                .gap(if p.compact { 12.0 } else { 17.0 })
                .radius(10.0)
                .fill(p.paper)
                .shadow(10.0, 28.0, Color::BLACK.alpha(0.5))
                .child(
                    text("Clean these build files?")
                        .font(hand(30.0, 700.0))
                        .color(p.sweep),
                )
                .child(
                    text(format!(
                        "Alice will remove generated entries from {} {} and reclaim {}. Source, worktrees, branches, and target roots stay.",
                        state.selected_builds.len(),
                        if state.selected_builds.len() == 1 { "Cargo target" } else { "Cargo targets" },
                        format_bytes(bytes)
                    ))
                    .font(font(14.0, 400.0).lh(20.0))
                    .wrap()
                    .color(p.ink2),
                )
                .child(if p.compact {
                    col()
                        .w_full()
                        .h(120.0)
                        .scroll(ScrollSpec {
                            stick_bottom: false,
                        })
                        .child(targets)
                } else {
                    targets
                })
                .child(
                    text("A fresh observation runs first. Active Cargo or rustc use protects the target. Unrecognized entries stay. This cannot be undone.")
                        .font(font(12.0, 500.0).lh(17.0))
                        .wrap()
                        .color(p.sweep),
                )
                .child(
                    row()
                        .justify_end()
                        .gap(10.0)
                        .child(button(
                            "cancel-build-sweep",
                            "Keep build files",
                            p,
                            ButtonKind::Outline,
                            true,
                            |state| state.confirming = false,
                        ))
                        .child(
                            button(
                                "confirm-build-sweep",
                                "Clean build files",
                                p,
                                ButtonKind::Dark,
                                true,
                                start_cleanup,
                            )
                            .default_button(),
                        ),
                ),
        )
}
