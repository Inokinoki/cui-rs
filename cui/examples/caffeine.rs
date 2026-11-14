//! Caffeine — a 1:1 port of examples/caffeine.c from the C repo. A
//! dot-matrix chart with annotations, an event track under it, readouts and
//! a dot-matrix display.

use cui::{Align, App, AppDesc, Axis, Color, DotChart, DotSeries, EventTrack, Readout, Size, Ui};

const MAX_CUPS: usize = 12;
const SAMPLES: usize = 145; // every 10 minutes from 06:00 to 06:00
const DAY_START: f64 = 6.0;
const DAY_END: f64 = 30.0;
const BEDTIME: f64 = 23.0;
const LIMIT_MG: f64 = 50.0; // what still lets you fall asleep, for this example
const CUP_MG: f64 = 95.0;
const HALF_LIFE: f64 = 5.0; // hours

struct AppState {
    cups: [f64; MAX_CUPS], // hours since midnight, sorted by the event track
    count: i32,
    selected: i32,
    curve: [[f32; SAMPLES]; MAX_CUPS], // each cup's share, one series per cup
}

/// One cup's caffeine at `t` hours after drinking it: absorbed over about 45
/// minutes, then cleared with a half-life of five hours.
fn cup_level(t: f64) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    let ke = 2.0f64.ln() / HALF_LIFE;
    let ka = 2.0f64.ln() / 0.25;
    CUP_MG * ka / (ka - ke) * ((-ke * t).exp() - (-ka * t).exp())
}

fn level_at(cups: &[f64], at: f64) -> f64 {
    cups.iter().take(cups.len()).map(|c| cup_level(at - c)).sum()
}

fn clock_text(h: f64) -> String {
    let m = (h * 60.0).round() as i32 % (24 * 60);
    format!("{:02}:{:02}", m / 60, m % 60)
}

fn frame(ui: &mut Ui, a: &mut AppState) {
    let count = a.count as usize;

    // the model: every cup's curve, the total's peak and what is left at bedtime
    let mut peak = 0.0f64;
    let mut peak_at = DAY_START;
    for s in 0..SAMPLES {
        let at = DAY_START + (DAY_END - DAY_START) * s as f64 / (SAMPLES - 1) as f64;
        let mut total = 0.0;
        for (c, cup) in a.cups.iter().enumerate().take(count) {
            a.curve[c][s] = cup_level(at - cup) as f32;
            total += a.curve[c][s] as f64;
        }
        if total > peak {
            peak = total;
            peak_at = at;
        }
    }
    let at_bed = level_at(&a.cups, BEDTIME);

    // the verdict: the latest time the last cup could be drunk and still be under the line
    let verdict = if count == 0 {
        "NO CUPS YET".to_string()
    } else if level_at(&a.cups[..count - 1], BEDTIME) > LIMIT_MG {
        "OVER THE LINE EVEN WITHOUT THE LAST CUP".to_string()
    } else {
        let (mut lo, mut hi) = (DAY_START, BEDTIME);
        for _ in 0..40 {
            let mid = (lo + hi) / 2.0;
            let mut cups = a.cups;
            cups[count - 1] = mid;
            if level_at(&cups[..count], BEDTIME) <= LIMIT_MG {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        format!("LAST CUP BY {}", clock_text((lo * 4.0).floor() / 4.0))
    };

    ui.column(
        &cui::Column {
            width: Size::FILL,
            height: Size::FILL,
            bg: Color::BACKGROUND,
            align: Align::Center,
            ..Default::default()
        },
        |ui| {
            ui.column(
                &cui::Column {
                    width: Size::FILL,
                    max_width: 880.,
                    height: Size::FILL,
                    pad: 28.,
                    gap: 22.,
                    ..Default::default()
                },
                |ui| {
                    ui.row(&cui::Row { width: Size::FILL, gap: 28., wrap: true, align: Align::End, ..Default::default() }, |ui| {
                        ui.column(&cui::Column { gap: 4., ..Default::default() }, |ui| {
                            let sub = format!(
                                "{} CUPS · {:.0} MG · HALF-LIFE {:.0}H",
                                count, count as f64 * CUP_MG, HALF_LIFE
                            );
                            ui.text_with("CAFFEINE", &cui::Text { font: cui::font::MONO, ..Default::default() });
                            ui.text_with(
                                sub,
                                &cui::Text {
                                    font: cui::font::MONO,
                                    color: Color::TEXT_MUTED,
                                    size: 12.,
                                    ..Default::default()
                                },
                            );
                        });
                        ui.spacer();
                        let peak_text = if count > 0 { format!("{peak:.0}") } else { "—".to_string() };
                        let peak_time = clock_text(peak_at);
                        ui.readout_with(
                            "PEAK",
                            &peak_text,
                            &Readout {
                                unit: Some("MG"),
                                detail: Some(if count > 0 { peak_time.as_str() } else { "" }),
                                size: cui::ControlSize::Sm,
                                ..Default::default()
                            },
                        );
                        ui.readout_with(
                            "AT BED",
                            format!("{at_bed:.0}"),
                            &Readout {
                                unit: Some("MG"),
                                size: cui::ControlSize::Sm,
                                accent: at_bed > LIMIT_MG,
                                detail: Some(if at_bed > LIMIT_MG { "OVER THE LINE" } else { "UNDER THE LINE" }),
                                ..Default::default()
                            },
                        );
                    });

                    // the chart: one series per cup, the selected one in the accent color
                    let series: Vec<DotSeries> = (0..count)
                        .map(|c| DotSeries {
                            values: &a.curve[c],
                            name: None,
                            accent: c as i32 == a.selected,
                        })
                        .collect();
                    let line1 = format!("PEAK {peak:.0} MG");
                    let line2 = format!("{at_bed:.0} MG STILL IN YOU");
                    let peak_clock = clock_text(peak_at);
                    let notes = [
                        cui::Annotation {
                            kind: cui::AnnotationKind::Y,
                            x: 0.,
                            y: LIMIT_MG,
                            label: Some("50"),
                            detail: None,
                            accent: false,
                        },
                        cui::Annotation {
                            kind: cui::AnnotationKind::X,
                            x: BEDTIME,
                            y: 0.,
                            label: Some("BED 23:00"),
                            detail: Some(line2.as_str()),
                            accent: false,
                        },
                        cui::Annotation {
                            kind: cui::AnnotationKind::X,
                            x: peak_at,
                            y: 0.,
                            label: Some(line1.as_str()),
                            detail: Some(peak_clock.as_str()),
                            accent: false,
                        },
                    ];
                    ui.column(&cui::Column { width: Size::FILL, gap: 0., ..Default::default() }, |ui| {
                        let r = ui.dot_chart(
                            "level",
                            &series,
                            SAMPLES as i32,
                            &DotChart {
                                height: 300.,
                                x_min: DAY_START,
                                x_max: DAY_END,
                                x_units: Axis::Hours,
                                min: 0.,
                                max: 260.,
                                unit: Some("MG"),
                                no_axis: true,
                                annotations: &notes[..if count > 0 { 3 } else { 1 }],
                                ..Default::default()
                            },
                        );
                        // a click on the chart pours a cup there
                        if r.clicked && count < MAX_CUPS {
                            let x = (r.x * 4.0 + 0.5).floor() / 4.0;
                            a.cups[count] = x;
                            a.count += 1;
                            for i in (1..a.count as usize).rev() {
                                if a.cups[i] >= a.cups[i - 1] {
                                    break;
                                }
                                a.cups.swap(i, i - 1);
                                a.selected = i as i32 - 1;
                            }
                            if a.selected < 0 || a.cups[a.selected as usize] != x {
                                a.selected = a.count - 1;
                            }
                        }
                        ui.event_track(
                            "cups",
                            &mut a.cups,
                            &mut a.count,
                            DAY_START,
                            DAY_END,
                            &mut EventTrack {
                                step: 0.25,
                                units: Axis::Hours,
                                label_step: 6.,
                                selected: Some(&mut a.selected),
                                ..Default::default()
                            },
                        );
                    });
                    ui.spacer();
                    ui.dot_display_with(
                        &verdict,
                        &cui::DotDisplay {
                            pitch: 3.,
                            width: Size::FILL,
                            accent: count > 0 && verdict.starts_with('O'),
                            ..Default::default()
                        },
                    );
                },
            );
        },
    );
}

fn main() {
    let mut a = AppState {
        cups: [8.0, 10.75, 12.25, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        count: 3,
        selected: 2,
        curve: [[0.0; SAMPLES]; MAX_CUPS],
    };
    let code = App::run(
        &AppDesc {
            title: Some("Caffeine"),
            width: 900,
            height: 700,
            theme: cui::ThemeMode::Dark,
            accent: cui::Accent::Blue,
            ..Default::default()
        },
        |ui| frame(ui, &mut a),
    );
    std::process::exit(code);
}
