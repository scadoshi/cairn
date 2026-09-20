//! A small SVG line chart, drawn in when it mounts and redrawn when its data
//! changes.
//!
//! No chart library: a polyline over a viewBox, styled by the theme
//! variables. `pathLength` pins the line's length to 1000 units so one CSS
//! keyframe draws any line end to end. The svg is keyed on a hash of the data,
//! so a change remounts it and replays the draw, which is what makes a +1 on
//! the counter visibly move the trend.

use dioxus::prelude::*;
use std::{
    fmt::Write,
    hash::{Hash, Hasher},
};

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
    // smooth path through its points.
    let segments = |values: Vec<Option<f64>>| -> Vec<String> {
        let mut out = Vec::new();
        let mut cur: Vec<(f64, f64)> = Vec::new();
        for (i, v) in values.iter().enumerate() {
            if let Some(v) = v {
                cur.push((x_of(i), y_of(*v)));
            } else {
                if cur.len() > 1 {
                    out.push(smooth_path(&cur));
                }
                cur.clear();
            }
        }
        if cur.len() > 1 {
            out.push(smooth_path(&cur));
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
    let key = hasher.finish();

    let last_label = points.last().map(|p| p.label.clone()).unwrap_or_default();
    let first_label = points.first().map(|p| p.label.clone()).unwrap_or_default();
    let mid_label = points
        .get(n / 2)
        .map(|p| p.label.clone())
        .unwrap_or_default();
    let baseline = y_of(0.0);
    let top = y_of(max);

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
            text { class: "chart-tick", x: "{PAD_L}", y: "{H - 6.0}", text_anchor: "start", "{first_label}" }
            text { class: "chart-tick", x: "{(PAD_L + W - PAD_R) / 2.0}", y: "{H - 6.0}", text_anchor: "middle", "{mid_label}" }
            text { class: "chart-tick", x: "{W - PAD_R}", y: "{H - 6.0}", text_anchor: "end", "{last_label}" }
            for seg in line_segments.iter() {
                path { class: "chart-line", d: "{seg}", path_length: "1000" }
            }
            for seg in overlay_segments.iter() {
                path { class: "chart-line chart-overlay", d: "{seg}", path_length: "1000" }
            }
        }
    }
}

/// A cubic path through the points, Catmull-Rom converted to Bezier control
/// points, so the line bends through every value instead of cornering at it.
fn smooth_path(pts: &[(f64, f64)]) -> String {
    let Some((first, rest)) = pts.split_first() else {
        return String::new();
    };
    let mut d = format!("M{:.1},{:.1}", first.0, first.1);
    let n = pts.len();
    for (i, p1) in rest.iter().enumerate() {
        // p0 is the point before the segment, p1 its end; the neighbours on
        // either side shape the tangents, clamped at the ends.
        let p0 = pts.get(i).copied().unwrap_or(*first);
        let prev = pts.get(i.saturating_sub(1)).copied().unwrap_or(p0);
        let next = pts.get((i + 2).min(n - 1)).copied().unwrap_or(*p1);
        let c1 = (p0.0 + (p1.0 - prev.0) / 6.0, p0.1 + (p1.1 - prev.1) / 6.0);
        let c2 = (p1.0 - (next.0 - p0.0) / 6.0, p1.1 - (next.1 - p0.1) / 6.0);
        // Writing to a String cannot fail.
        let _ = write!(
            d,
            " C{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
            c1.0, c1.1, c2.0, c2.1, p1.0, p1.1
        );
    }
    d
}

#[cfg(test)]
mod tests {
    use super::smooth_path;

    #[test]
    fn smooth_path_starts_with_move_and_curves_through_each_point() {
        let d = smooth_path(&[(0.0, 10.0), (10.0, 0.0), (20.0, 10.0)]);
        assert!(d.starts_with("M0.0,10.0 C"));
        assert_eq!(d.matches(" C").count(), 2);
        assert!(d.ends_with("20.0,10.0"));
    }

    #[test]
    fn smooth_path_of_one_point_is_just_a_move() {
        assert_eq!(smooth_path(&[(3.0, 4.0)]), "M3.0,4.0");
    }
}
