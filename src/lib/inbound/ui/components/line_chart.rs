//! A small SVG line chart, drawn in when it mounts and redrawn when its data
//! changes.
//!
//! No chart library: a polyline over a viewBox, styled by the theme
//! variables. `pathLength` pins the line's length to 1000 units so one CSS
//! keyframe draws any line end to end. The svg is keyed on a hash of the data,
//! so a change remounts it and replays the draw, which is what makes a +1 on
//! the counter visibly move the trend.

use crate::domain::counter::series::Shape;
use dioxus::prelude::*;
use std::hash::{Hash, Hasher};
use zwipe_components::curve;

/// One point: the label shown for it and its value.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    /// Short label, e.g. "Mar" or "07:00".
    pub label: String,
    /// The value; `None` draws a gap.
    pub value: Option<f64>,
}

/// The chart.
#[component]
pub fn LineChart(
    points: Vec<Point>,
    /// A second, smoother line over the same x positions (a rolling average).
    #[props(default)]
    overlay: Vec<Option<f64>>,
    /// Unit shown after the axis numbers.
    #[props(default)]
    unit: String,
    /// The series' overall shape. Given one, the chart draws a band one
    /// standard deviation either side of the average.
    #[props(default)]
    shape: Option<Shape>,
    /// Also draw the fitted line. Only for series where x is time; across
    /// weekdays or hours of the day a slope would be an artifact of the
    /// order the buckets happen to sit in.
    #[props(default)]
    trend: bool,
    /// Label every nth point along the bottom. Zero, the default, labels
    /// the first, the middle and the last, which is all there is room for
    /// on a long series.
    #[props(default)]
    label_every: usize,
) -> Element {
    const W: f64 = 320.0;
    const H: f64 = 140.0;
    const PAD_L: f64 = 30.0;
    const PAD_R: f64 = 8.0;
    const PAD_T: f64 = 10.0;
    const PAD_B: f64 = 22.0;

    let n = points.len();
    let max = points
        .iter()
        .filter_map(|p| p.value)
        .chain(overlay.iter().flatten().copied())
        .fold(0.0_f64, f64::max)
        .max(1.0);

    let x_of = |i: usize| {
        if n <= 1 {
            PAD_L
        } else {
            #[allow(clippy::cast_precision_loss)]
            let t = i as f64 / (n - 1) as f64;
            PAD_L + t * (W - PAD_L - PAD_R)
        }
    };
    let y_of = |v: f64| H - PAD_B - (v / max) * (H - PAD_T - PAD_B);

    // Gaps (None) break the line into segments; each segment becomes a
    // smooth path through its points that never swings past them, so a rest
    // day beside a spike stays at zero.
    let segments = |values: Vec<Option<f64>>| -> Vec<String> {
        let mut out = Vec::new();
        let mut cur: Vec<(f64, f64)> = Vec::new();
        for (i, v) in values.iter().enumerate() {
            if let Some(v) = v {
                cur.push((x_of(i), y_of(*v)));
            } else {
                if cur.len() > 1 {
                    out.push(curve(&cur));
                }
                cur.clear();
            }
        }
        if cur.len() > 1 {
            out.push(curve(&cur));
        }
        out
    };
    let line_segments = segments(points.iter().map(|p| p.value).collect());
    let overlay_segments = segments(overlay.clone());

    // Key on the data so a change remounts the svg and replays the draw.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for p in &points {
        p.label.hash(&mut hasher);
        p.value.map(f64::to_bits).hash(&mut hasher);
    }
    for v in &overlay {
        v.map(f64::to_bits).hash(&mut hasher);
    }
    if let Some(f) = shape {
        (f.slope.to_bits(), f.intercept.to_bits(), f.sd.to_bits()).hash(&mut hasher);
    }
    trend.hash(&mut hasher);
    label_every.hash(&mut hasher);
    let key = hasher.finish();

    // (x position, text, anchor). Either every nth point, or the three that
    // fit on a series too long to label.
    //
    // `label_every` is a floor, not the answer: forty weeks at every fifth
    // still ran "12/29" into "02/02" with no gap between them. The stride
    // widens until the labels have room, which is why no series has to
    // carry a number tuned to its own longest label.
    let step = if label_every > 0 {
        let widest = points
            .iter()
            .map(|p| p.label.chars().count())
            .max()
            .unwrap_or(0);
        // Measured against the real thing rather than from the font
        // metrics: the viewBox is stretched horizontally to the card, so a
        // character costs more than its 8px advance. One character of gap
        // between neighbours on top.
        #[allow(clippy::cast_precision_loss)]
        let needed = (widest + 1) as f64 * 6.2;
        let spacing = if n > 1 {
            #[allow(clippy::cast_precision_loss)]
            let gaps = (n - 1) as f64;
            (W - PAD_L - PAD_R) / gaps
        } else {
            W
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let fits = (needed / spacing.max(0.1)).ceil() as usize;
        label_every.max(fits).max(1)
    } else {
        0
    };
    let label_every = step;
    let ticks: Vec<(f64, String, &str)> = if label_every > 0 {
        points
            .iter()
            .enumerate()
            .filter(|(i, _)| i % label_every == 0)
            .map(|(i, p)| {
                // The ends anchor inwards so they do not hang off the chart.
                let anchor = if i == 0 {
                    "start"
                } else if i + label_every >= n {
                    "end"
                } else {
                    "middle"
                };
                (x_of(i), p.label.clone(), anchor)
            })
            .collect()
    } else {
        let at = |i: usize, anchor: &'static str| {
            points.get(i).map(|p| (x_of(i), p.label.clone(), anchor))
        };
        [
            at(0, "start"),
            at(n / 2, "middle"),
            at(n.saturating_sub(1), "end"),
        ]
        .into_iter()
        .flatten()
        .collect()
    };
    let baseline = y_of(0.0);
    let top = y_of(max);
    // Both are clamped into the plot: a band an average sits near the top of
    // would otherwise reach above the axis, and a falling trend below zero.
    let clamp = |v: f64| v.clamp(0.0, max);
    let band = shape.map(|f| {
        let hi = y_of(clamp(f.mean + f.sd));
        let lo = y_of(clamp(f.mean - f.sd));
        (hi, (lo - hi).max(0.6), y_of(clamp(f.mean)))
    });
    let trend_line = shape
        .filter(|_| trend)
        .map(|f| (y_of(clamp(f.at(0))), y_of(clamp(f.at(n.saturating_sub(1))))));

    rsx! {
        svg {
            key: "{key}",
            class: "line-chart",
            view_box: "0 0 {W} {H}",
            preserve_aspect_ratio: "none",
            role: "img",
            // axis
            line { class: "chart-axis", x1: "{PAD_L}", y1: "{baseline}", x2: "{W - PAD_R}", y2: "{baseline}" }
            line { class: "chart-axis chart-axis-faint", x1: "{PAD_L}", y1: "{top}", x2: "{W - PAD_R}", y2: "{top}" }
            text { class: "chart-tick", x: "{PAD_L - 4.0}", y: "{top + 3.0}", text_anchor: "end", "{max:.0}{unit}" }
            text { class: "chart-tick", x: "{PAD_L - 4.0}", y: "{baseline + 3.0}", text_anchor: "end", "0" }
            for (x, label, anchor) in ticks.iter() {
                text {
                    class: "chart-tick",
                    x: "{x}",
                    y: "{H - 6.0}",
                    text_anchor: "{anchor}",
                    "{label}"
                }
            }
            // Drawn first so the data reads over the top of them.
            if let Some((y, height, mean_y)) = band {
                rect {
                    class: "chart-band",
                    x: "{PAD_L}",
                    y: "{y}",
                    width: "{W - PAD_L - PAD_R}",
                    height: "{height}",
                }
                line { class: "chart-mean", x1: "{PAD_L}", y1: "{mean_y}", x2: "{W - PAD_R}", y2: "{mean_y}" }
            }
            if let Some((y1, y2)) = trend_line {
                line { class: "chart-trend", x1: "{PAD_L}", y1: "{y1}", x2: "{W - PAD_R}", y2: "{y2}" }
            }
            for seg in line_segments.iter() {
                path { class: "chart-line", d: "{seg}", path_length: "1000" }
            }
            for seg in overlay_segments.iter() {
                path { class: "chart-line chart-overlay", d: "{seg}", path_length: "1000" }
            }
        }
    }
}
