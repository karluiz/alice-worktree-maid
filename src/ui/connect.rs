use super::{ButtonKind, Paint, animated_art, button, check, folder, font, hand, mono};
use crate::{
    Activity, State, WorkerProvider, add_folder, choose_folder, remove_folder, set_provider,
    start_scan,
};
use rune::{El, ScrollSpec, col, el, row, text, text_input};

pub(super) fn connect(state: &State, p: Paint) -> El {
    let form_width = if p.compact {
        (p.width - 40.0).min(560.0)
    } else {
        470.0
    };
    let choose_x = form_width - 88.0;
    let provider = |provider: WorkerProvider, name: &str, detail: &str| {
        let available = state.available_workers.contains(&provider);
        let selected = state.worker_provider == provider;
        let mut control = row()
            .key(format!("provider-{name}"))
            .role("radio")
            .label(if selected {
                format!("{name}, selected")
            } else if available {
                format!("{name}, available")
            } else {
                format!("{name}, not configured")
            })
            .checked(selected)
            .disabled(!available)
            .focusable(available)
            .w_full()
            .py(12.0)
            .gap(13.0)
            .center_y()
            .border_sides([0.0, 0.0, 1.0, 0.0], p.rule)
            .child(
                el().size(20.0, 20.0)
                    .radius(10.0)
                    .center()
                    .fill(if selected { p.ink } else { p.paper })
                    .border(1.0, if selected { p.ink } else { p.ink3 })
                    .child(if selected { check(p.paper) } else { el() }),
            )
            .child(
                col()
                    .grow(1.0)
                    .min_w(0.0)
                    .gap(3.0)
                    .child(text(name).font(font(14.0, 600.0)).color(p.ink))
                    .child(text(detail).font(font(12.0, 400.0)).wrap().color(p.ink2)),
            )
            .child(if p.compact {
                el()
            } else {
                text(if available {
                    "Uses local CLI"
                } else {
                    "Not configured"
                })
                .font(font(11.0, 400.0))
                .color(p.ink3)
            });
        if available {
            control = control.on_press(move |state| set_provider(state, provider));
        }
        control
    };
    let mut roots = col().w_full().gap(4.0);
    for location in &state.locations {
        let remove = location.clone();
        roots = roots.child(
            row()
                .w_full()
                .h(36.0)
                .gap(9.0)
                .center_y()
                .child(folder(p.ink3))
                .child(
                    text(location.display().to_string())
                        .grow(1.0)
                        .min_w(0.0)
                        .font(mono(11.0, 400.0))
                        .truncate_middle()
                        .color(p.ink2),
                )
                .child(button(
                    format!("remove-folder-{}", location.display()),
                    "Remove",
                    p,
                    ButtonKind::Text,
                    true,
                    move |state| remove_folder(state, remove.clone()),
                )),
        );
    }
    let form = col()
        .w(form_width)
        .gap(18.0)
        .child(
            text("Your agents leave worktrees everywhere. Alice cleans up.")
                .font(font(35.0, 650.0).lh(38.0).tracking(-0.7))
                .wrap()
                .color(p.ink),
        )
        .child(
            text("She reads every worktree, finds what was merged, protects what is still cooking, then asks before she sweeps. Bring the worker you already pay for.")
                .font(font(14.0, 400.0).lh(21.0))
                .wrap()
                .color(p.ink2),
        )
        .child(
            col()
                .pt(10.0)
                .child(text("Bring your own subscription").font(font(13.0, 600.0)).color(p.ink))
                .child(
                    col()
                        .pt(8.0)
                        .border_sides([1.0, 0.0, 0.0, 0.0], p.rule)
                        .child(provider(WorkerProvider::Kimi, "Kimi Code", "Your existing Kimi Code subscription"))
                        .child(provider(WorkerProvider::Claude, "Claude Code", "Pro or Max subscription"))
                        .child(provider(WorkerProvider::Copilot, "GitHub Copilot", "Individual or Business seat")),
                ),
        )
        .child(
            col()
                .gap(8.0)
                .child(roots)
                .child(
                    row()
                        .w_full()
                        .center_y()
                        .justify_between()
                        .child(text("Where should she clean?").font(font(13.0, 600.0)).color(p.ink))
                        .child(button(
                            "add-folder",
                            "Add folder",
                            p,
                            ButtonKind::Text,
                            !state.folder_edit.text.trim().is_empty(),
                            add_folder,
                        )),
                )
                .child(
                    row()
                        .h(46.0)
                        .px(12.0)
                        .gap(10.0)
                        .center_y()
                        .radius(7.0)
                        .fill(p.wash)
                        .border(1.0, p.rule)
                        .child(folder(p.ink2))
                        .child(
                            text_input(&state.folder_edit, mono(12.0, 400.0))
                                .single_line()
                                .key("folder")
                                .label("Folder to scan")
                                .placeholder("~/code", p.ink3)
                                .color(p.ink)
                                .absolute(34.0, 5.5)
                                .w((choose_x - 62.0).max(80.0))
                                .h(38.5)
                                .py(11.0)
                                .on_edit(|state: &mut State| &mut state.folder_edit)
                                .on_submit(|state, _| add_folder(state)),
                        )
                        .child(
                            el()
                                .absolute(choose_x, 7.0)
                                .w(76.0)
                                .key("choose-folder")
                                .role("button")
                                .label("Choose folder")
                                .focusable(true)
                                .h(32.0)
                                .px(10.0)
                                .center()
                                .radius(5.0)
                                .fill(p.paper)
                                .border(1.0, p.rule)
                                .hover(p.rule)
                                .on_press_tx(choose_folder)
                                .child(text("Choose…").font(font(11.0, 600.0)).color(p.ink).a11y_hidden()),
                        ),
                )
                .child(
                    text(state.folder_error.as_deref().unwrap_or("Folders are scanned together · each searches depth 6"))
                        .min_h(20.0)
                        .pb(2.0)
                        .font(font(11.0, 400.0).lh(15.0))
                        .wrap()
                        .color(if state.folder_error.is_some() { p.sweep } else { p.ink3 }),
                ),
        )
        .child(
            row()
                .pt(4.0)
                .gap(16.0)
                .center_y()
                .child(button(
                    "send-alice",
                    if state.activity == Activity::Scanning { "Waking Alice up…" } else { "Send Alice in" },
                    p, ButtonKind::Dark,
                    state.activity == Activity::Idle && (!state.locations.is_empty() || !state.folder_edit.text.trim().is_empty()),
                    start_scan,
                ))
                .child(
                    text("Only branch names, commit messages and file stats go to your worker. Code stays on disk.")
                        .w(if p.compact { (form_width - 150.0).max(120.0) } else { 245.0 })
                        .font(font(11.0, 400.0).lh(15.0))
                        .wrap()
                        .color(p.ink3),
                ),
        );
    let content = if p.compact {
        col()
            .w_full()
            .min_h(730.0)
            .px(20.0)
            .py(22.0)
            .center()
            .child(form)
    } else {
        let art = col().w(430.0).gap(0.0).child(
            el().w(430.0)
                .h(390.0)
                .child(animated_art(
                    &state.greeting_frames,
                    &state.greeting_dark_frames,
                    p,
                    430.0,
                    390.0,
                    true,
                ))
                .child(
                    text("plug in a brain,\nshe does the dirty job")
                        .absolute(260.0, 12.0)
                        .w(175.0)
                        .font(hand(19.0, 400.0).lh(22.0))
                        .wrap()
                        .color(p.sweep),
                ),
        );
        row()
            .w_full()
            .min_h(730.0)
            .px(38.0)
            .py(22.0)
            .center()
            .gap(50.0)
            .child(art)
            .child(form)
    };
    col()
        .grow(1.0)
        .min_h(0.0)
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .child(content)
}
