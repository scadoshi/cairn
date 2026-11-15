//! The heatmap's weekday strip, pinned at the left while the year scrolls.

/// The body of the rule for `selector` in Cairn's own stylesheet.
fn rule(selector: &str) -> &'static str {
    let css = include_str!("../assets/main.css");
    let Some(at) = css.find(&format!("\n{selector} {{")) else {
        return "";
    };
    css.get(at..)
        .and_then(|rest| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn the_weekdays_stay_at_the_left_on_the_card_color() {
    let axis = rule(".heat-axis");
    assert!(axis.contains("position: sticky;"), "{axis:?}");
    assert!(axis.contains("left: 0;"), "{axis:?}");
    assert!(
        axis.contains("background-color: var(--bg-primary);"),
        "{axis:?}"
    );
}

#[test]
fn the_left_fade_starts_past_the_weekdays() {
    let scroll = rule(".heat-scroll");
    assert!(scroll.contains("--axis-end:"), "{scroll:?}");
    assert!(
        scroll.contains("#000 var(--axis-end), transparent var(--axis-end)"),
        "the strip stays opaque and the weeks fade in beside it: {scroll:?}"
    );
}
