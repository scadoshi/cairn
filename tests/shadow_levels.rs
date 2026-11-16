//! Cairn's surfaces take orangui's shadow levels by what they are: a card that sits on the page, the
//! home hero included, casts the small one; the medium one is for what floats over the page.

/// The body of the top-level rule for `selector` in Cairn's own stylesheet.
fn rule(selector: &str) -> &'static str {
    let css = include_str!("../assets/main.css");
    css.split_once(&format!("\n{selector} {{\n"))
        .and_then(|(_, rest)| rest.split_once("\n}"))
        .map_or("", |(body, _)| body)
}

#[test]
fn every_card_on_the_page_casts_the_small_shadow() {
    for card in [".home-hero", ".tile", ".odometer"] {
        let body = rule(card);
        assert!(
            body.contains("box-shadow: var(--shadow-sm);"),
            "{card} sits on the page: {body:?}"
        );
    }
}
