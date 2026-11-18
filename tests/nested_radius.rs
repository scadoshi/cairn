//! Nested corners: a rounded box inside a card takes the card's radius minus the inset between
//! them, so the gap between the two curves stays even around the corner.

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
fn the_nested_radius_is_the_card_radius_less_the_inset() {
    let root = rule(":root");
    assert!(root.contains("--card-radius: 1rem;"), "{root:?}");
    assert!(root.contains("--card-inset: 0.5rem;"), "{root:?}");
    assert!(
        root.contains("calc(var(--card-radius) - 1px - var(--card-inset))"),
        "inner = outer less the card's border and the inset: {root:?}"
    );
}

#[test]
fn tiles_and_the_odometer_take_the_nested_radius() {
    for selector in [".tile", ".odometer"] {
        let body = rule(selector);
        assert!(
            body.contains("border-radius: var(--nested-radius);"),
            "{selector}: {body:?}"
        );
    }
}

#[test]
fn the_insets_are_the_ones_the_radius_is_made_from() {
    // The same inset on every side, so the gap the radius is made from is the gap there is.
    assert!(rule(".tile-grid").contains("padding: var(--card-inset);"));
    assert!(rule(".odometer").contains("margin: var(--card-inset) var(--card-inset) 0;"));
}

#[test]
fn a_tile_s_figure_takes_the_room_the_smaller_inset_gives() {
    assert!(rule(".tile-value").contains("font-size: 1.6rem;"));
    assert!(rule(".tile-value-word").contains("font-size: min(1.6rem, 17cqi);"));
}
