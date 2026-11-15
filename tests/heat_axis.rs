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
fn the_weeks_fade_at_both_edges_with_the_kit_scroll_fade() {
    let scroll = rule(".heat-scroll");
    assert!(scroll.contains("var(--fade-left, 0px)"), "{scroll:?}");
    assert!(scroll.contains("var(--fade-right, 0px)"), "{scroll:?}");
}

#[test]
fn the_strip_is_a_box_around_its_svg() {
    let svg = rule(".heat-axis > svg");
    assert!(svg.contains("display: block;"), "{svg:?}");
}

#[test]
fn the_left_fade_starts_past_the_weekdays() {
    let scroll = rule(".heat-scroll");
    assert!(
        scroll.contains("--heat-gutter: calc(0.75rem + 24px);"),
        "{scroll:?}"
    );
    assert!(
        scroll.contains("#000 var(--heat-gutter), transparent var(--heat-gutter)"),
        "the strip stays opaque and the weeks fade in beside it: {scroll:?}"
    );
}

#[test]
fn the_strip_covers_the_scroller_s_left_padding() {
    let scroll = rule(".heat-scroll");
    let axis = rule(".heat-axis");
    assert!(scroll.contains("padding: 0.25rem 0.75rem 0;"), "{scroll:?}");
    assert!(
        axis.contains("box-shadow: -0.75rem 0 0 var(--bg-primary);"),
        "{axis:?}"
    );
}
