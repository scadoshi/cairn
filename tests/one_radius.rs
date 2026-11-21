//! One radius, the kit's rule in Cairn: the home card is a container at the panel radius with the
//! panel padding inside, the tiles and the odometer sit that padding in from its edge and round at
//! the chip radius so their corners nest in the card's, and every button takes the chip radius. No
//! rule sets a radius of its own, except the shapes that are no container, chip or button.

#![expect(
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    reason = "a test: walks CSS text by byte offsets it finds itself"
)]

/// Cairn's own sheet.
const CSS: &str = include_str!("../assets/main.css");

/// The radii a rule may set: the kit's panel and chip radii.
const ALLOWED: [&str; 2] = ["var(--panel-radius)", "var(--chip-radius)"];

/// Rules whose shape is no container, chip or button, each keeping its own radius.
const OUT_OF_SCOPE: [&str; 7] = [
    // The store icon tile.
    ".app-icon-tile",
    // The chart key's swatches.
    ".chart-key",
    ".chart-key-band",
    // Celebration bits: a spark, confetti, popper bits and the pulse ring round the count.
    ".levelup-spark",
    ".confetti-piece",
    ".popper-bit",
    ".pulse-ring",
];

/// `css` without its comments, the same length so offsets still line up.
fn without_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find("*/")
            .map_or(rest.len(), |e| start + e + 2);
        out.extend(
            rest[start..end]
                .chars()
                .map(|c| if c == '\n' { '\n' } else { ' ' }),
        );
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// Every style rule in `css` as its selector and its declarations, at-rules looked inside.
fn rules(css: &str) -> Vec<(String, String)> {
    let css = without_comments(css);
    let mut out = Vec::new();
    let mut heads: Vec<String> = Vec::new();
    let mut start = 0;
    for (at, c) in css.char_indices() {
        match c {
            '{' => {
                heads.push(css[start..at].trim().to_owned());
                start = at + 1;
            }
            '}' => {
                if let Some(head) = heads.pop()
                    && !head.starts_with('@')
                {
                    let head = head.split_whitespace().collect::<Vec<_>>().join(" ");
                    out.push((head, css[start..at].to_owned()));
                }
                start = at + 1;
            }
            _ => {}
        }
    }
    out
}

/// The value `body` gives `property`, if it sets it.
fn value(body: &str, property: &str) -> Option<String> {
    body.split(';').find_map(|decl| {
        let (name, value) = decl.split_once(':')?;
        (name.trim() == property).then(|| value.split_whitespace().collect::<Vec<_>>().join(" "))
    })
}

/// The declarations of the first rule whose selector is exactly `selector`.
fn rule(selector: &str) -> String {
    rules(CSS)
        .into_iter()
        .find(|(head, _)| head == selector)
        .map_or_else(|| panic!("a rule for {selector}"), |(_, body)| body)
}

#[test]
fn no_rule_sets_a_radius_of_its_own() {
    let own: Vec<String> = rules(CSS)
        .into_iter()
        .filter_map(|(selector, body)| {
            let radius = value(&body, "border-radius")?;
            (!ALLOWED.contains(&radius.as_str()) && !OUT_OF_SCOPE.contains(&selector.as_str()))
                .then(|| format!("{selector} {{ border-radius: {radius} }}"))
        })
        .collect();
    assert!(
        own.is_empty(),
        "radii outside the rule:\n{}",
        own.join("\n")
    );
}

#[test]
fn the_home_card_is_a_panel_and_its_tiles_nest_at_its_padding() {
    assert_eq!(
        value(&rule(".home-hero"), "border-radius").as_deref(),
        Some("var(--panel-radius)")
    );
    assert_eq!(
        value(&rule(".tile-grid"), "padding").as_deref(),
        Some("var(--panel-pad)")
    );
    assert_eq!(
        value(&rule(".odometer"), "margin").as_deref(),
        Some("var(--panel-pad) var(--panel-pad) 0")
    );
    for selector in [".tile", ".odometer"] {
        assert_eq!(
            value(&rule(selector), "border-radius").as_deref(),
            Some("var(--chip-radius)"),
            "{selector}"
        );
    }
}

#[test]
fn cairn_keeps_no_corner_tokens_of_its_own() {
    for token in ["--card-radius", "--card-inset", "--nested-radius"] {
        assert!(
            !CSS.contains(token),
            "the kit's tokens stand in for {token}"
        );
    }
}
