//! The menu's footer row of icon buttons.

/// The rule that lays out the menu's footer row in Cairn's own stylesheet.
fn footer_rule() -> Option<&'static str> {
    let css = include_str!("../assets/main.css");
    let at = css.find("\n.app-nav .nav-trailing-row {")?;
    css.get(at..)?.split_once('}').map(|(body, _)| body)
}

#[test]
fn the_menu_footer_centers_its_icons() {
    let rule = footer_rule().unwrap_or_default();
    assert!(rule.contains("justify-content: center;"), "{rule:?}");
}
