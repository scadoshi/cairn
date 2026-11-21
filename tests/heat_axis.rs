//! The heatmap's weekday strip, pinned at the left while the year scrolls, as orangui draws it
//! for Cairn: the kit's sheet holds the rules, and Cairn's own sheet adds none.

/// The body of the first rule for `selector` in `css`.
fn rule(css: &'static str, selector: &str) -> &'static str {
    let Some(at) = css.find(&format!("\n{selector} {{")) else {
        return "";
    };
    css.get(at..)
        .and_then(|rest| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

fn kit(selector: &str) -> &'static str {
    rule(orangui::COMPONENTS_CSS, selector)
}

#[test]
fn the_weekdays_stay_at_the_left_on_the_card_color() {
    let axis = kit(".heat-axis");
    assert!(axis.contains("position: sticky;"), "{axis:?}");
    assert!(axis.contains("left: 0;"), "{axis:?}");
    assert!(
        axis.contains("background-color: var(--bg-primary);"),
        "{axis:?}"
    );
}

#[test]
fn the_weeks_fade_at_both_edges_past_the_weekdays() {
    let scroll = kit(".heat-scroll");
    assert!(scroll.contains("var(--fade-left, 0px)"), "{scroll:?}");
    assert!(scroll.contains("var(--fade-right, 0px)"), "{scroll:?}");
    assert!(
        scroll.contains("#000 var(--heat-gutter), transparent var(--heat-gutter)"),
        "the strip stays opaque and the weeks fade in beside it: {scroll:?}"
    );
}

#[test]
fn the_year_scrolls_with_no_scroll_bar() {
    assert!(kit(".heat-scroll").contains("scrollbar-width: none;"));
    assert!(kit(".heat-scroll::-webkit-scrollbar").contains("display: none;"));
}

#[test]
fn cairn_adds_no_heatmap_rules_of_its_own() {
    let own = include_str!("../assets/main.css");
    for selector in [".heat-scroll", ".heat-axis", ".heat-plot", ".heat-readout"] {
        assert!(rule(own, selector).is_empty(), "{selector} in main.css");
    }
}
