//! A settings row's label gives way before its value: the value's words are cut short only once
//! the label can wrap no narrower.

/// The rule that sets how fast a button row's label shrinks, in Cairn's own stylesheet.
fn label_rule() -> Option<&'static str> {
    let css = include_str!("../assets/main.css");
    let at =
        css.find("\n.settings-row:has(> .settings-row-value > button) > .row-label-with-hint {")?;
    css.get(at..)?.split_once('}').map(|(body, _)| body)
}

/// The least shrink factor that keeps the value whole while the label has room to give. Flex
/// shrinks each item by its factor times its width, so the value always takes a share of the
/// overflow; at 100 that share of a couple of pixels was 0.05 px, enough to draw "Level …" in a
/// value that fit. A layout unit is 1/64 px, so with up to 50 px of overflow against widths of the
/// same size, a factor past 50 × 64 leaves the value's share under one unit.
const LEAST_SHRINK: f64 = 50.0 * 64.0;

#[test]
fn a_button_rows_label_gives_way_entirely_before_its_value() {
    let rule = label_rule().unwrap_or_default();
    let shrink: f64 = rule
        .lines()
        .find_map(|line| line.trim().strip_prefix("flex-shrink:"))
        .and_then(|value| value.trim().trim_end_matches(';').parse().ok())
        .unwrap_or_default();
    assert!(
        shrink >= LEAST_SHRINK,
        "the label's flex-shrink is {shrink}, so the value is cut short while the label could still wrap: {rule}"
    );
}
