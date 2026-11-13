mod builds;
mod connect;
mod report;
mod swept;
mod tmp;

use crate::State;
use rune::tok::motion;
use rune::{
    Beat, Color, Cx, Cycle, El, Family, Image, LineCap, LoopSpec, Sequence, Shape, SurfaceFit,
    TextStyle, Transition, canvas, col, el, row, text,
};

const ALICE_ARRIVAL: &[Beat] = &[Beat::new(0.0).opacity(0.72).scale(0.985), Beat::new(0.65)];
const SWEEP_FRAME_0: &[(f32, f32)] = &[(0.0, 1.0), (0.124, 1.0), (0.125, 0.0), (1.0, 0.0)];
const SWEEP_FRAME_1: &[(f32, f32)] = &[
    (0.0, 0.0),
    (0.124, 0.0),
    (0.125, 1.0),
    (0.249, 1.0),
    (0.25, 0.0),
    (1.0, 0.0),
];
const SWEEP_FRAME_2: &[(f32, f32)] = &[
    (0.0, 0.0),
    (0.249, 0.0),
    (0.25, 1.0),
    (0.374, 1.0),
    (0.375, 0.0),
    (1.0, 0.0),
];
const SWEEP_FRAME_3: &[(f32, f32)] = &[
    (0.0, 0.0),
    (0.374, 0.0),
    (0.375, 1.0),
    (0.499, 1.0),
    (0.5, 0.0),
    (1.0, 0.0),
];
const SWEEP_FRAME_4: &[(f32, f32)] = &[
    (0.0, 0.0),
    (0.499, 0.0),
    (0.5, 1.0),
    (0.624, 1.0),
    (0.625, 0.0),
    (1.0, 0.0),
];
const SWEEP_FRAME_5: &[(f32, f32)] = &[
    (0.0, 0.0),
    (0.624, 0.0),
    (0.625, 1.0),
    (0.749, 1.0),
    (0.75, 0.0),
    (1.0, 0.0),
];
const SWEEP_FRAME_6: &[(f32, f32)] = &[
    (0.0, 0.0),
    (0.749, 0.0),
    (0.75, 1.0),
    (0.874, 1.0),
    (0.875, 0.0),
    (1.0, 0.0),
];
const SWEEP_FRAME_7: &[(f32, f32)] = &[(0.0, 0.0), (0.874, 0.0), (0.875, 1.0), (1.0, 1.0)];

fn sweep_cycle(index: usize) -> Cycle {
    Cycle {
        period: 2.4,
        phase: 0.0,
        stops: [
            SWEEP_FRAME_0,
            SWEEP_FRAME_1,
            SWEEP_FRAME_2,
            SWEEP_FRAME_3,
            SWEEP_FRAME_4,
            SWEEP_FRAME_5,
            SWEEP_FRAME_6,
            SWEEP_FRAME_7,
        ][index],
        still: if index == 0 { 1.0 } else { 0.0 },
        fps: 12.0,
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Paint {
    pub dark: bool,
    pub width: f32,
    pub compact: bool,
    pub paper: Color,
    pub wash: Color,
    pub rule: Color,
    pub ink: Color,
    pub ink2: Color,
    pub ink3: Color,
    pub sweep: Color,
    pub sweep_wash: Color,
    pub ask: Color,
    pub ask_ink: Color,
    pub ask_wash: Color,
    pub keep: Color,
    pub keep_wash: Color,
    pub scrim: Color,
}

pub(crate) fn paint(dark: bool, width: f32) -> Paint {
    if dark {
        return Paint {
            dark,
            width,
            compact: width < 900.0,
            paper: Color::BLACK,
            wash: rune::color::hex("#141414").unwrap(),
            rule: rune::color::hex("#303030").unwrap(),
            ink: rune::color::hex("#f4f2ed").unwrap(),
            ink2: rune::color::hex("#b5b5b5").unwrap(),
            ink3: rune::color::hex("#949494").unwrap(),
            sweep: rune::color::hex("#ff7663").unwrap(),
            sweep_wash: rune::color::hex("#2a1210").unwrap(),
            ask: rune::color::hex("#f59e42").unwrap(),
            ask_ink: rune::color::hex("#ffb15c").unwrap(),
            ask_wash: rune::color::hex("#25190c").unwrap(),
            keep: rune::color::hex("#82aaff").unwrap(),
            keep_wash: rune::color::hex("#101b30").unwrap(),
            scrim: Color::BLACK.alpha(0.72),
        };
    }
    Paint {
        dark,
        width,
        compact: width < 900.0,
        paper: Color::WHITE,
        wash: rune::color::hex("#f6f6f3").unwrap(),
        rule: rune::color::hex("#e7e6e1").unwrap(),
        ink: rune::color::hex("#141414").unwrap(),
        ink2: rune::color::hex("#5a5b60").unwrap(),
        ink3: rune::color::hex("#74757a").unwrap(),
        sweep: rune::color::hex("#c8331f").unwrap(),
        sweep_wash: rune::color::hex("#fbece9").unwrap(),
        ask: rune::color::hex("#e8821e").unwrap(),
        ask_ink: rune::color::hex("#a5560a").unwrap(),
        ask_wash: rune::color::hex("#fdf2e6").unwrap(),
        keep: rune::color::hex("#2457c5").unwrap(),
        keep_wash: rune::color::hex("#ebf0fb").unwrap(),
        scrim: rune::color::hex("#141414").unwrap().alpha(0.28),
    }
}

pub(crate) fn font(size: f32, weight: f32) -> TextStyle {
    TextStyle::new(size, weight)
}

pub(crate) fn hand(size: f32, weight: f32) -> TextStyle {
    font(size, weight).family(Family::Named("Kalam"))
}

pub(crate) fn mono(size: f32, weight: f32) -> TextStyle {
    font(size, weight).mono()
}

pub(crate) fn view(state: &State, cx: &mut Cx<State>) -> El {
    cx.zoom(state.zoom);
    let width = cx.layout("window").map_or(1_200.0, |rect| rect[2]);
    let p = paint(state.dark.unwrap_or_else(|| cx.system_dark()), width);
    let title = match state.screen {
        crate::Screen::Connect => "First run".to_owned(),
        _ if state.locations.len() > 1 => format!("{} folders", state.locations.len()),
        _ => state.location.display().to_string(),
    };
    let body = match state.screen {
        crate::Screen::Connect => connect::connect(state, p),
        crate::Screen::Report => report::report(state, p),
        crate::Screen::Swept => swept::swept(state, p),
    };
    let mut root = col()
        .key("window")
        .role("window")
        .w_full()
        .h_full()
        .fill(p.paper)
        .child(top_bar(state, &title, p))
        .child(body);
    if state.confirming {
        root = root.child(report::confirmation(state, p));
    }
    root
}

fn top_bar(state: &State, title: &str, p: Paint) -> El {
    let connecting = !matches!(state.screen, crate::Screen::Connect);
    row()
        .key("topbar")
        .h(48.0)
        .w_full()
        .px(16.0)
        .center_y()
        .justify_between()
        .border_sides([0.0, 0.0, 1.0, 0.0], p.rule)
        .fill(p.paper)
        .window_drag()
        .child(
            row()
                .gap(13.0)
                .center_y()
                .child(
                    row()
                        .gap(7.0)
                        .child(dot("#ff5f57"))
                        .child(dot("#febc2e"))
                        .child(dot("#28c840")),
                )
                .child(text("alice").font(hand(22.0, 700.0)).color(p.ink))
                .child(if p.compact {
                    el()
                } else {
                    text(title)
                        .font(font(11.0, 400.0))
                        .truncate_middle()
                        .color(p.ink3)
                }),
        )
        .child(
            row()
                .gap(8.0)
                .center_y()
                .child(button(
                    "toggle-theme",
                    if p.dark {
                        "Theme: Dark"
                    } else {
                        "Theme: Light"
                    },
                    p,
                    ButtonKind::Text,
                    true,
                    move |state| crate::set_dark(state, !p.dark),
                ))
                .child(if connecting && !p.compact {
                    row()
                        .gap(7.0)
                        .center_y()
                        .child({
                            let mut status = el().size(6.0, 6.0).radius(3.0).fill(p.keep);
                            if state.activity == crate::Activity::Explaining {
                                status = status.looping(LoopSpec::opacity(1.4, 0.0, 0.35, 1.0));
                            }
                            status
                        })
                        .child(
                            text(format!("Thinking with {}", state.worker_provider.label()))
                                .font(font(11.0, 500.0))
                                .color(p.ink2),
                        )
                } else {
                    el()
                })
                .child(if state.demo {
                    text("DEMO DATA").font(font(10.0, 700.0)).color(p.sweep)
                } else {
                    text("").color(p.ink)
                }),
        )
}

fn dot(hex: &str) -> El {
    el().size(11.0, 11.0)
        .radius(5.5)
        .fill(rune::color::hex(hex).unwrap())
        .a11y_hidden()
}

pub(crate) fn animated_art(
    light: &[Image],
    dark: &[Image],
    p: Paint,
    width: f32,
    height: f32,
    playing: bool,
) -> El {
    let frames = if p.dark { dark } else { light };
    let mut root = el().size(width, height);
    if !playing {
        if let Some(frame) = frames.first() {
            root = root.child(
                el().size(width, height)
                    .pixels_fit(frame.pixels(), SurfaceFit::Fit),
            );
        }
    } else {
        for (index, frame) in frames.iter().take(8).enumerate() {
            let mut layer = el().size(width, height);
            if index > 0 {
                layer = layer.absolute(0.0, 0.0);
            }
            root = root.child(
                layer
                    .pixels_fit(frame.pixels(), SurfaceFit::Fit)
                    .cycle(sweep_cycle(index))
                    .a11y_hidden(),
            );
        }
    }
    root.sequence(Sequence::new(ALICE_ARRIVAL))
}

#[derive(Clone, Copy)]
pub(crate) enum ButtonKind {
    Dark,
    Outline,
    Text,
}

pub(crate) fn button(
    key: impl Into<String>,
    label: impl Into<String>,
    p: Paint,
    kind: ButtonKind,
    enabled: bool,
    on_press: impl Fn(&mut State) + 'static,
) -> El {
    let label = label.into();
    let mut control = el()
        .key(key)
        .role("button")
        .label(label.clone())
        .h(44.0)
        .px(18.0)
        .center()
        .radius(7.0)
        .fill(match kind {
            ButtonKind::Dark if enabled => p.ink,
            ButtonKind::Dark => p.ink3,
            ButtonKind::Outline => p.paper,
            ButtonKind::Text => Color::TRANSPARENT,
        })
        .border(
            1.0,
            match kind {
                ButtonKind::Outline => p.rule,
                _ => Color::TRANSPARENT,
            },
        )
        .motion(motion::SNAPPY)
        .child(
            text(label)
                .font(font(12.0, 600.0))
                .color(match kind {
                    ButtonKind::Dark => p.paper,
                    ButtonKind::Text => p.ink,
                    _ => p.ink,
                })
                .a11y_hidden(),
        );
    if enabled {
        control = control
            .focusable(true)
            .hover(match kind {
                ButtonKind::Dark => p.ink2,
                _ => p.wash,
            })
            .press(p.rule)
            .press_scale(0.985)
            .on_press(on_press);
    } else {
        control = control.disabled(true);
    }
    control
}

pub(crate) fn check(color: Color) -> El {
    let shape = Shape::new()
        .stroke_with(
            vec![[3.0, 8.0], [7.0, 12.0], [14.0, 3.0]],
            false,
            2.0,
            LineCap::Round,
            color,
        )
        .view_box(17.0, 16.0);
    canvas(shape)
        .size(17.0, 16.0)
        .transition(Transition::scale(0.7).combined(Transition::opacity()))
        .a11y_hidden()
}

pub(crate) fn lock(color: Color) -> El {
    let shape = Shape::new()
        .stroke_with(
            vec![
                [5.0, 8.0],
                [5.0, 5.0],
                [7.0, 2.0],
                [11.0, 2.0],
                [13.0, 5.0],
                [13.0, 8.0],
            ],
            false,
            1.5,
            LineCap::Round,
            color,
        )
        .stroke_with(
            vec![[3.0, 8.0], [15.0, 8.0], [15.0, 17.0], [3.0, 17.0]],
            true,
            1.5,
            LineCap::Round,
            color,
        )
        .view_box(18.0, 19.0);
    canvas(shape).size(18.0, 19.0).a11y_hidden()
}

pub(crate) fn chevron(open: bool, color: Color) -> El {
    canvas(
        Shape::new()
            .stroke(vec![[3.0, 6.0], [8.0, 11.0], [13.0, 6.0]], 1.5, color)
            .view_box(16.0, 16.0),
    )
    .size(16.0, 16.0)
    .motion(motion::SNAPPY)
    .rotate(if open { std::f32::consts::PI } else { 0.0 })
    .a11y_hidden()
}

pub(crate) fn folder(color: Color) -> El {
    let shape = Shape::new()
        .stroke_with(
            vec![
                [1.0, 4.0],
                [7.0, 4.0],
                [9.0, 7.0],
                [17.0, 7.0],
                [16.0, 16.0],
                [2.0, 16.0],
            ],
            true,
            1.4,
            LineCap::Round,
            color,
        )
        .view_box(18.0, 19.0);
    canvas(shape).size(18.0, 19.0).a11y_hidden()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduced_motion_holds_alice_at_her_resting_pose() {
        let cache = rune::ImageCache::new(std::sync::Arc::new(|| {}));
        let frames: Vec<Image> = (0..8)
            .map(|_| {
                cache
                    .load(&include_bytes!("../assets/alice-illustrations/greeting-rigid-0.png")[..])
            })
            .collect();
        let mut runtime = rune::Runtime::new(
            frames,
            |frames: &Vec<Image>, _: &mut Cx<Vec<Image>>| {
                animated_art(frames, &[], paint(false, 1200.0), 290.0, 162.0, true)
            },
            (400.0, 300.0),
            rune::Env::new(false, 1.0),
        );
        runtime.set_system_reduced_motion(true);
        runtime.frame(0.0);
        runtime.frame(0.1);

        let elements = runtime.elements();
        let mut loops = elements
            .iter()
            .filter(|element| element.loop_phase.is_some());
        let arrival = loops.next().expect("Alice's arrival track");
        assert_eq!(arrival.opacity, 1.0);
        let sprites: Vec<_> = loops.collect();
        assert_eq!(sprites.len(), 8);
        assert_eq!(sprites[0].rect, [0.0, 0.0, 290.0, 162.0]);
        assert_eq!(sprites[0].opacity, 1.0);
        assert!(sprites[1..].iter().all(|sprite| sprite.opacity == 0.0));
    }

    #[test]
    fn ui_zoom_reflows_the_viewport_and_keeps_text_crisp() {
        let mut state = State::new();
        state.zoom = 1.5;
        let mut runtime =
            rune::Runtime::new(state, view, (1200.0, 780.0), rune::Env::new(false, 1.0));
        runtime.frame(0.0);

        assert_eq!(runtime.zoom(), 1.5);
        assert_eq!(runtime.viewport, (800.0, 520.0));
        assert_eq!(runtime.env.scale, 1.5);
    }

    #[test]
    fn sweeping_cycles_through_all_poses_and_rests_on_the_first() {
        for (time, visible) in [
            (0.0, 0),
            (0.3, 1),
            (0.6, 2),
            (0.9, 3),
            (1.2, 4),
            (1.5, 5),
            (1.8, 6),
            (2.1, 7),
        ] {
            for index in 0..8 {
                let expected = if index == visible { 1.0 } else { 0.0 };
                assert!((sweep_cycle(index).sample(time) - expected).abs() < 0.001);
            }
        }
        assert_eq!(sweep_cycle(0).still, 1.0);
        for index in 1..8 {
            assert_eq!(sweep_cycle(index).still, 0.0);
        }
    }
}
