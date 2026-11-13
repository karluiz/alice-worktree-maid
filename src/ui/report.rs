use super::{ButtonKind, Paint, animated_art, button, check, chevron, font, hand, lock, mono};
use crate::{
    Activity, LoadState, ReportTab, Screen, State, change_folders, start_cleanup,
    start_explanation, start_scan,
};
use alice::{Assessment, Disposition, ScanBatch, ScanReport, format_bytes};
use rune::tok::motion;
use rune::{Color, El, LoopSpec, ScrollSpec, Transition, col, el, row, text};
use std::path::PathBuf;

pub(super) fn report(state: &State, p: Paint) -> El {
    let content_width = (p.width - 40.0).min(560.0);
    if state.activity == Activity::Scanning {
        return col().grow(1.0).center().child(
            col()
                .key("scan-loading")
                .w(content_width)
                .items_center()
                .gap(18.0)
                .child(animated_art(
                    &state.sweeping_frames,
                    &state.sweeping_dark_frames,
                    p,
                    content_width,
                    content_width * 0.558,
                    true,
                ))
                .child(
                    text("Reading repositories and inspecting every worktree…")
                        .w_full()
                        .center()
                        .align(rune::Align::Center)
                        .font(font(14.0, 500.0))
                        .wrap()
                        .color(p.ink2),
                )
                .child(
                    el().w(240.0).h(4.0).radius(2.0).fill(p.wash).child(
                        el().key("scan-progress")
                            .w_frac(0.25)
                            .h_full()
                            .radius(2.0)
                            .fill(p.sweep)
                            .looping(LoopSpec::slide(1.6, [-1.0, 0.0], [4.0, 0.0])),
                    ),
                ),
        );
    }
    match &state.load {
        LoadState::Idle => col().grow(1.0).center().child(
            text("Preparing Alice…")
                .font(font(16.0, 500.0))
                .color(p.ink2),
        ),
        LoadState::Failed(error) => col()
            .grow(1.0)
            .center()
            .gap(16.0)
            .child(
                text("Alice could not finish the scan")
                    .w(content_width)
                    .font(font(27.0, 650.0))
                    .wrap()
                    .color(p.ink),
            )
            .child(
                text(error)
                    .w(content_width)
                    .font(font(14.0, 400.0).lh(20.0))
                    .wrap()
                    .color(p.sweep),
            )
            .child(
                row()
                    .gap(10.0)
                    .child(button(
                        "retry-scan",
                        "Try again",
                        p,
                        ButtonKind::Dark,
                        true,
                        start_scan,
                    ))
                    .child(button(
                        "back-folder",
                        "Change folder",
                        p,
                        ButtonKind::Outline,
                        true,
                        |state| {
                            state.screen = Screen::Connect;
                            state.load = LoadState::Idle;
                        },
                    )),
            ),
        LoadState::Ready(batch) => ready(state, batch, p),
    }
}

fn ready(state: &State, batch: &ScanBatch, p: Paint) -> El {
    if state.report_tab == ReportTab::Builds {
        return super::builds::ready(state, batch, p);
    }
    if state.report_tab == ReportTab::Tmp {
        return super::tmp::ready(state, batch, p);
    }
    if batch.linked().next().is_none() {
        let empty = col()
            .grow(1.0)
            .center()
            .gap(12.0)
            .child(animated_art(
                &state.rest_frames,
                &state.rest_dark_frames,
                p,
                330.0,
                246.0,
                true,
            ))
            .child(text("Spotless.").font(font(34.0, 650.0)).color(p.ink))
            .child(
                text("No linked worktrees were found in this scan.")
                    .font(font(13.0, 400.0))
                    .color(p.ink2),
            )
            .child(button(
                "spotless-scan",
                "Scan again",
                p,
                ButtonKind::Outline,
                true,
                start_scan,
            ));
        return col()
            .grow(1.0)
            .min_h(0.0)
            .child(super::builds::tabs(state, batch, p))
            .child(empty);
    }
    let main = col()
        .grow(1.0)
        .min_w(0.0)
        .min_h(0.0)
        .child(
            col()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .scroll_edge_fade(12.0)
                .child(report_body(state, batch, p)),
        )
        .child(commit_bar(state, p));
    if p.compact {
        col()
            .grow(1.0)
            .min_h(0.0)
            .child(repo_sidebar(state, batch, p))
            .child(main)
    } else {
        row()
            .grow(1.0)
            .min_h(0.0)
            .child(repo_sidebar(state, batch, p))
            .child(main)
    }
}

fn repo_sidebar(state: &State, batch: &ScanBatch, p: Paint) -> El {
    let mut nav = if p.compact {
        row().gap(6.0)
    } else {
        col().w_full().gap(3.0)
    };
    nav = nav.child(repo_item(
        "All repos",
        None,
        batch.linked().map(|(_, item)| item.disk_bytes).sum(),
        state.active_repo.is_none(),
        p,
    ));
    let mut reports: Vec<_> = batch
        .reports
        .iter()
        .filter_map(|report| {
            let bytes: u64 = report
                .worktrees
                .iter()
                .filter(|item| !item.worktree.main)
                .map(|item| item.disk_bytes)
                .sum();
            (bytes > 0).then_some((report, bytes))
        })
        .collect();
    reports.sort_by(|a, b| b.1.cmp(&a.1));
    let visible_count = reports.len();
    for (report, bytes) in reports {
        let path = report.repository.clone();
        nav = nav.child(repo_item(
            report
                .repository
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("repository"),
            Some(path.clone()),
            bytes,
            state.active_repo.as_ref() == Some(&path),
            p,
        ));
    }
    nav = nav.child(button(
        "edit-folders",
        "Edit folders",
        p,
        ButtonKind::Text,
        true,
        change_folders,
    ));
    if p.compact {
        return col()
            .w_full()
            .fill(p.wash)
            .border_sides([0.0, 0.0, 1.0, 0.0], p.rule)
            .child(
                el().w_full()
                    .h(58.0)
                    .scroll_x()
                    .child(col().w_full().px(8.0).py(10.0).child(nav)),
            );
    }
    col()
        .w(218.0)
        .h_full()
        .fill(p.wash)
        .border_sides([0.0, 1.0, 0.0, 0.0], p.rule)
        .child(
            col()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(
                    col()
                        .padding(12.0)
                        .gap(8.0)
                        .child(
                            text("Worktrees by repo")
                                .font(font(11.0, 400.0))
                                .color(p.ink3),
                        )
                        .child(nav),
                ),
        )
        .child(
            col()
                .padding(15.0)
                .gap(5.0)
                .border_sides([1.0, 0.0, 0.0, 0.0], p.rule)
                .child(
                    text(format!("{visible_count} repositories with worktrees"))
                        .font(font(11.0, 600.0))
                        .color(p.ink2),
                )
                .child(
                    text("Every cleanup needs confirmation")
                        .font(font(10.0, 400.0).lh(15.0))
                        .wrap()
                        .color(p.ink3),
                ),
        )
}

fn repo_item(label: &str, path: Option<PathBuf>, bytes: u64, active: bool, p: Paint) -> El {
    el().key(format!(
        "repo-{}",
        path.as_ref()
            .map_or("all".to_owned(), |path| path.display().to_string())
    ))
    .role("button")
    .label(label)
    .checked(active)
    .focusable(true)
    .w(if p.compact { 150.0 } else { 194.0 })
    .h(38.0)
    .px(9.0)
    .center_y()
    .radius(5.0)
    .fill(if active { p.paper } else { Color::TRANSPARENT })
    .border(1.0, if active { p.rule } else { Color::TRANSPARENT })
    .hover(p.paper)
    .on_press(move |state: &mut State| state.active_repo = path.clone())
    .child(
        text(label)
            .grow(1.0)
            .min_w(0.0)
            .mr(6.0)
            .font(font(12.0, if active { 600.0 } else { 400.0 }))
            .truncate()
            .color(p.ink),
    )
    .child(
        text(format_bytes(bytes))
            .font(font(10.0, 400.0))
            .color(p.ink3),
    )
}

fn report_body(state: &State, batch: &ScanBatch, p: Paint) -> El {
    let visible: Vec<_> = batch
        .linked()
        .filter(|(report, _)| {
            state
                .active_repo
                .as_ref()
                .is_none_or(|path| path == &report.repository)
        })
        .collect();
    let picked: Vec<_> = visible
        .iter()
        .filter(|(_, item)| state.selected.contains(&item.worktree.path))
        .collect();
    let picked_bytes: u64 = picked.iter().map(|(_, item)| item.disk_bytes).sum();
    let sweep: Vec<_> = visible
        .iter()
        .copied()
        .filter(|(_, item)| item.disposition == Disposition::Ready)
        .collect();
    let ask: Vec<_> = visible
        .iter()
        .copied()
        .filter(|(_, item)| item.disposition == Disposition::NeedsReview)
        .collect();
    let keep: Vec<_> = visible
        .iter()
        .copied()
        .filter(|(_, item)| {
            matches!(
                item.disposition,
                Disposition::Active | Disposition::Protected
            )
        })
        .collect();
    let mut body = col()
        .w_full()
        .padding(if p.compact { 16.0 } else { 26.0 })
        .gap(0.0)
        .child(super::builds::tabs(state, batch, p))
        .child(hero(state, batch, picked_bytes, p, false));
    body = body.child(group(
        state,
        "sweep it",
        "Merged and safely contained",
        sweep,
        p,
    ));
    body = body.child(group(
        state,
        "ask me first",
        "Unpushed or uncommitted work",
        ask,
        p,
    ));
    body = body.child(group(
        state,
        "leave alone",
        "Recent work and protected checkouts",
        keep,
        p,
    ));
    if batch.warnings.len() > 0 {
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

pub(super) fn hero(
    state: &State,
    batch: &ScanBatch,
    picked_bytes: u64,
    p: Paint,
    builds: bool,
) -> El {
    let (used_pct, sweep_pct, used, total, after) =
        batch.disk.as_ref().map_or((0.0, 0.0, 0, 0, 0), |disk| {
            let total = disk.total_bytes.max(1);
            (
                disk.used_bytes as f32 / total as f32,
                picked_bytes as f32 / total as f32,
                disk.used_bytes,
                disk.total_bytes,
                disk.used_bytes.saturating_sub(picked_bytes),
            )
        });
    let bar_width = if p.compact {
        (p.width - 32.0).max(240.0)
    } else {
        480.0
    };
    row()
        .w_full()
        .pb(16.0)
        .gap(20.0)
        .center_y()
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .gap(7.0)
                .child(
                    text(if builds {
                        "she found the build cache"
                    } else if state.demo {
                        "she found the demo mess"
                    } else {
                        "she found the mess"
                    })
                    .font(hand(19.0, 400.0))
                    .color(p.sweep),
                )
                .child(
                    row()
                        .gap(12.0)
                        .center_y()
                        .child(
                            text(format_bytes(picked_bytes))
                                .font(font(48.0, 650.0).tracking(-1.0))
                                .color(p.ink),
                        )
                        .child(
                            text(if builds {
                                "ready to clean"
                            } else {
                                "ready to sweep"
                            })
                            .font(font(16.0, 400.0))
                            .color(p.ink2),
                        ),
                )
                .child(
                    col()
                        .pt(12.0)
                        .gap(7.0)
                        .child(
                            el().w(bar_width)
                                .h(12.0)
                                .radius(6.0)
                                .fill(p.wash)
                                .child(
                                    el().absolute(0.0, 0.0)
                                        .w(bar_width * used_pct.min(1.0))
                                        .h(12.0)
                                        .radius(6.0)
                                        .fill(p.ink3.alpha(0.72))
                                        .motion(motion::SNAPPY),
                                )
                                .child(
                                    el().absolute(
                                        (bar_width * (used_pct - sweep_pct).max(0.0))
                                            .min(bar_width),
                                        0.0,
                                    )
                                    .w((bar_width * sweep_pct).min(bar_width * used_pct))
                                    .h(12.0)
                                    .radius(6.0)
                                    .fill(p.sweep)
                                    .motion(motion::SNAPPY),
                                ),
                        )
                        .child(
                            row()
                                .w(bar_width)
                                .justify_between()
                                .child(
                                    text(if total > 0 {
                                        format!(
                                            "{} of {} used",
                                            format_bytes(used),
                                            format_bytes(total)
                                        )
                                    } else {
                                        "Disk usage unavailable".to_owned()
                                    })
                                    .font(font(10.0, 400.0))
                                    .color(p.ink2),
                                )
                                .child(
                                    text(if total > 0 {
                                        format!("{} after", format_bytes(after))
                                    } else {
                                        "".to_owned()
                                    })
                                    .font(font(10.0, 400.0))
                                    .color(p.ink),
                                ),
                        ),
                ),
        )
        .child(if p.compact {
            el()
        } else {
            animated_art(
                &state.sweeping_frames,
                &state.sweeping_dark_frames,
                p,
                290.0,
                162.0,
                state.activity == Activity::Idle,
            )
        })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pile {
    Sweep,
    Ask,
    Keep,
}

fn group<'a>(
    state: &State,
    title: &str,
    blurb: &str,
    items: Vec<(&'a ScanReport, &'a Assessment)>,
    p: Paint,
) -> El {
    if items.is_empty() {
        return el();
    }
    let pile = match title {
        "sweep it" => Pile::Sweep,
        "ask me first" => Pile::Ask,
        _ => Pile::Keep,
    };
    let color = match pile {
        Pile::Sweep => p.sweep,
        Pile::Ask => p.ask_ink,
        Pile::Keep => p.keep,
    };
    let total: u64 = items.iter().map(|(_, item)| item.disk_bytes).sum();
    let all_on = pile == Pile::Sweep
        && items
            .iter()
            .all(|(_, item)| state.selected.contains(&item.worktree.path));
    let paths: Vec<_> = items
        .iter()
        .map(|(_, item)| item.worktree.path.clone())
        .collect();
    let mut list = col().w_full();
    for (report, assessment) in items {
        list = list.child(worktree_row(state, report, assessment, pile, p));
    }
    col()
        .w_full()
        .mt(24.0)
        .child(
            row()
                .w_full()
                .pb(8.0)
                .center_y()
                .border_sides([0.0, 0.0, 1.0, 0.0], p.ink)
                .child(text(title).font(hand(25.0, 700.0)).color(color))
                .child(if p.compact {
                    el().grow(1.0)
                } else {
                    text(blurb)
                        .ml(12.0)
                        .grow(1.0)
                        .font(font(11.0, 400.0))
                        .color(p.ink2)
                })
                .child(
                    text(format!("{} · {}", paths.len(), format_bytes(total)))
                        .font(font(11.0, 400.0))
                        .color(p.ink2),
                )
                .child(if pile == Pile::Sweep {
                    button(
                        "select-pile",
                        if all_on { "Deselect all" } else { "Select all" },
                        p,
                        ButtonKind::Text,
                        true,
                        move |state| {
                            if all_on {
                                for path in &paths {
                                    state.selected.remove(path);
                                }
                            } else {
                                state.selected.extend(paths.iter().cloned());
                            }
                        },
                    )
                } else {
                    el()
                }),
        )
        .child(list)
}

fn worktree_row(
    state: &State,
    report: &ScanReport,
    assessment: &Assessment,
    pile: Pile,
    p: Paint,
) -> El {
    let path = assessment.worktree.path.clone();
    let on = state.selected.contains(&path);
    let open = state.expanded.as_ref() == Some(&path);
    let color = match pile {
        Pile::Sweep => p.sweep,
        Pile::Ask => p.ask_ink,
        Pile::Keep => p.keep,
    };
    let checkbox = match pile {
        Pile::Sweep => {
            let toggle = path.clone();
            el().key(format!("select-{}", path.display()))
                .role("checkbox")
                .label(format!(
                    "Sweep {}",
                    assessment.worktree.branch.as_deref().unwrap_or("worktree")
                ))
                .checked(on)
                .focusable(true)
                .size(20.0, 20.0)
                .center()
                .radius(4.0)
                .fill(if on { color } else { p.paper })
                .border(1.0, if on { color } else { p.ink3 })
                .motion(motion::SNAPPY)
                .press_scale(0.94)
                .on_press(move |state: &mut State| {
                    if !state.selected.insert(toggle.clone()) {
                        state.selected.remove(&toggle);
                    }
                })
                .child(if on { check(p.paper) } else { el() })
        }
        Pile::Ask => el()
            .size(20.0, 20.0)
            .radius(4.0)
            .border(1.0, p.ask)
            .disabled(true),
        Pile::Keep => el().size(20.0, 20.0).center().child(lock(p.keep)),
    };
    let repo = report
        .repository
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("repository");
    let branch = assessment
        .worktree
        .branch
        .as_deref()
        .unwrap_or("detached HEAD");
    let expand = path.clone();
    let chevron_expand = expand.clone();
    let mut row_view = row()
        .w_full()
        .min_h(66.0)
        .py(10.0)
        .gap(13.0)
        .center_y()
        .child(checkbox)
        .child(
            el().key(format!("expand-worktree-{}", path.display()))
                .grow(1.0)
                .min_w(0.0)
                .role("button")
                .label(format!("Explain {branch}"))
                .focusable(true)
                .on_press(move |state: &mut State| {
                    state.expanded = if state.expanded.as_ref() == Some(&expand) {
                        None
                    } else {
                        Some(expand.clone())
                    };
                })
                .child(
                    col()
                        .gap(4.0)
                        .child(text(branch).font(mono(12.0, 500.0)).truncate().color(p.ink))
                        .child(text(repo).font(font(10.0, 400.0)).color(p.ink2)),
                ),
        )
        .child(
            if p.compact || assessment.disposition == Disposition::Ready {
                el()
            } else {
                text(note(assessment))
                    .w(245.0)
                    .font(hand(17.0, 400.0).lh(20.0))
                    .wrap()
                    .color(color)
            },
        )
        .child(if p.compact {
            el()
        } else {
            text(format_age(assessment.last_commit_at))
                .w(65.0)
                .font(font(10.0, 400.0))
                .color(p.ink2)
        })
        .child(
            text(format_bytes(assessment.disk_bytes))
                .w(if p.compact { 60.0 } else { 70.0 })
                .font(font(11.0, 600.0))
                .color(p.ink),
        )
        .child(
            el().key(format!("expand-chevron-{}", path.display()))
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
                    state.expanded = if state.expanded.as_ref() == Some(&chevron_expand) {
                        None
                    } else {
                        Some(chevron_expand.clone())
                    };
                })
                .child(chevron(open, p.ink2)),
        );
    row_view = row_view.border_sides([0.0, 0.0, 1.0, 0.0], p.rule);
    let mut container = col().w_full().motion(motion::SNAPPY).child(row_view);
    if p.compact && assessment.disposition != Disposition::Ready {
        container = container.child(
            row()
                .ml(33.0)
                .pb(10.0)
                .gap(8.0)
                .border_sides([0.0, 0.0, 1.0, 0.0], p.rule)
                .child(
                    text(note(assessment))
                        .grow(1.0)
                        .min_w(0.0)
                        .font(hand(17.0, 400.0).lh(20.0))
                        .wrap()
                        .color(color),
                )
                .child(
                    text(format_age(assessment.last_commit_at))
                        .font(font(10.0, 400.0))
                        .color(p.ink2),
                ),
        );
    }
    if open {
        container = container.child(evidence(state, report, assessment, pile, p));
    }
    container
}

fn evidence(
    state: &State,
    report: &ScanReport,
    assessment: &Assessment,
    pile: Pile,
    p: Paint,
) -> El {
    let wash = match pile {
        Pile::Sweep => p.sweep_wash,
        Pile::Ask => p.ask_wash,
        Pile::Keep => p.keep_wash,
    };
    let path = assessment.worktree.path.clone();
    let mut panel = col()
        .w_full()
        .mt(12.0)
        .mb(12.0)
        .padding(if p.compact { 12.0 } else { 14.0 })
        .gap(7.0)
        .radius(7.0)
        .fill(wash)
        .transition(Transition::offset(0.0, 6.0).combined(Transition::opacity()))
        .child(
            text("What she checked")
                .font(font(11.0, 700.0))
                .color(p.ink),
        );
    let integration = if assessment.merged {
        format!("HEAD is contained in {}", report.base_branch)
    } else {
        format!("HEAD is not contained in {}", report.base_branch)
    };
    let working = if assessment.dirty {
        format!("{} changed or untracked files", assessment.changed_files)
    } else {
        "Clean working tree · no untracked files".to_owned()
    };
    let upstream = match (assessment.ahead, assessment.behind) {
        (Some(ahead), Some(behind)) => format!("Upstream: {ahead} ahead · {behind} behind"),
        _ => "No upstream tracking information".to_owned(),
    };
    let verdict = match assessment.disposition {
        Disposition::Ready => "Ready to remove: clean, contained, and fully pushed.",
        Disposition::NeedsReview if assessment.dirty => {
            "Review first: uncommitted changes are still here."
        }
        Disposition::NeedsReview if assessment.ahead.is_some_and(|ahead| ahead > 0) => {
            "Review first: commits have not been pushed."
        }
        Disposition::NeedsReview => "Review first: this work is not fully integrated.",
        Disposition::Active => "Keep for now: recent committed work is still active.",
        Disposition::Protected => "Protected: Alice will not remove this checkout.",
    };
    panel = panel.child(
        text(verdict)
            .font(font(12.0, 600.0).lh(17.0))
            .wrap()
            .color(match pile {
                Pile::Sweep => p.sweep,
                Pile::Ask => p.ask_ink,
                Pile::Keep => p.keep,
            }),
    );
    let mut facts = col().grow(1.0).min_w(0.0).gap(7.0);
    for line in [working, integration, upstream] {
        facts = facts.child(
            text(line)
                .font(font(12.0, 400.0).lh(17.0))
                .wrap()
                .color(p.ink2),
        );
    }
    let mut reasons = col().grow(1.0).min_w(0.0).gap(7.0);
    for reason in &assessment.reasons {
        reasons = reasons.child(
            text(reason)
                .font(font(12.0, 400.0).lh(17.0))
                .wrap()
                .color(p.ink2),
        );
    }
    if p.compact {
        panel = panel.child(facts).child(reasons);
    } else {
        panel = panel.child(row().w_full().gap(24.0).child(facts).child(reasons));
    }
    panel = panel.child(
        text(assessment.worktree.path.display().to_string())
            .font(mono(10.0, 400.0))
            .truncate_middle()
            .color(p.ink3),
    );
    if pile == Pile::Ask {
        panel = panel.child(row().pt(6.0).gap(10.0).child(button(
            "explain-wip",
            if state.activity == Activity::Explaining {
                format!("Asking {}…", state.worker_provider.label())
            } else {
                format!("Explain with {}", state.worker_provider.label())
            },
            p,
            ButtonKind::Outline,
            state.activity == Activity::Idle,
            start_explanation,
        )));
        if let Some((worker_path, summary)) = &state.worker
            && worker_path == &path
        {
            panel = panel.child(
                text(summary)
                    .font(font(12.0, 400.0).lh(17.0))
                    .wrap()
                    .color(p.ink),
            );
        }
    }
    panel
}

fn commit_bar(state: &State, p: Paint) -> El {
    let count = state.selected.len();
    let bytes = state.batch().map_or(0, |batch| {
        batch
            .linked()
            .filter(|(_, item)| state.selected.contains(&item.worktree.path))
            .map(|(_, item)| item.disk_bytes)
            .sum()
    });
    let message = text("Only merged, clean worktrees are selected. Branches stay in Git; every batch is confirmed.")
        .grow(1.0).font(font(11.0, 400.0).lh(15.0)).wrap().color(p.ink2);
    let action = button(
        "sweep-selected",
        if state.activity == Activity::Cleaning {
            "Sweeping…".to_owned()
        } else if count == 0 {
            "Pick something to sweep".to_owned()
        } else {
            format!("Review {count} worktrees · {}", format_bytes(bytes))
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
    if state.report_tab == ReportTab::Builds {
        return super::builds::confirmation(state, p);
    }
    let selected: Vec<_> = state
        .batch()
        .into_iter()
        .flat_map(ScanBatch::linked)
        .filter(|(_, item)| state.selected.contains(&item.worktree.path))
        .collect();
    let bytes: u64 = selected.iter().map(|(_, item)| item.disk_bytes).sum();
    let mut branches = col().gap(6.0);
    for (_, item) in selected {
        branches = branches.child(
            text(format!(
                "{} · {}",
                item.worktree.branch.as_deref().unwrap_or("worktree"),
                format_bytes(item.disk_bytes)
            ))
            .font(mono(11.0, 500.0))
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
                .label("Confirm cleanup")
                .w((p.width - 24.0).min(490.0))
                .padding(if p.compact { 18.0 } else { 26.0 })
                .gap(if p.compact { 12.0 } else { 17.0 })
                .radius(10.0)
                .fill(p.paper)
                .shadow(10.0, 28.0, Color::BLACK.alpha(0.5))
                .child(text("Sweep this batch?").font(hand(30.0, 700.0)).color(p.sweep))
                .child(text(format!("Alice will remove {} worktree directories and reclaim {}. Their branches stay in Git.", state.selected.len(), format_bytes(bytes)))
                    .font(font(14.0, 400.0).lh(20.0)).wrap().color(p.ink2))
                .child(if p.compact {
                    col().w_full().h(120.0).scroll(ScrollSpec { stick_bottom: false }).child(branches)
                } else {
                    branches
                })
                .child(text("A fresh Git scan runs first. Newly dirty, unpushed, or unmerged work stays protected. This cannot be undone.")
                    .font(font(12.0, 500.0).lh(17.0)).wrap().color(p.sweep))
                .child(row().justify_end().gap(10.0)
                    .child(button("cancel-sweep", "Keep worktrees", p, ButtonKind::Outline, true, |state| state.confirming = false))
                    .child(button("confirm-sweep", "Sweep worktrees", p, ButtonKind::Dark, true, start_cleanup).default_button())),
        )
}

fn note(assessment: &Assessment) -> String {
    match assessment.disposition {
        Disposition::Ready => "merged. done, done, done.".to_owned(),
        Disposition::NeedsReview if assessment.dirty => format!(
            "{} uncommitted files. I won't touch those.",
            assessment.changed_files
        ),
        Disposition::NeedsReview if assessment.ahead.is_some_and(|ahead| ahead > 0) => format!(
            "{} commits never pushed. yours?",
            assessment.ahead.unwrap_or(0)
        ),
        Disposition::NeedsReview => "not integrated. let's be sure.".to_owned(),
        Disposition::Active => "recent work. still cooking.".to_owned(),
        Disposition::Protected => "locked or primary. off limits.".to_owned(),
    }
}

fn format_age(timestamp: Option<i64>) -> String {
    let Some(timestamp) = timestamp else {
        return "unknown".to_owned();
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |value| value.as_secs() as i64);
    let elapsed = (now - timestamp).max(0);
    if elapsed < 86_400 {
        "today".to_owned()
    } else {
        format!("{} days", elapsed / 86_400)
    }
}
