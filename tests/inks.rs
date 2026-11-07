//! Text in a tone takes the tone's ink. A theme sets each `--ink-<tone>` to
//! read at 4.5:1 on the page, the sink and the tone's tint (orangui's ink
//! gate), where the bare tone may not. Every text color in the stylesheet
//! that names a tone must fall back to it from its ink, apart from the
//! graphics listed here, which are drawings, not text.

/// Selectors whose color is a drawing: the wordmark and the app icon's
/// preview of it, the level-up word that flashes for a second, and the
/// chart's overlay line.
const GRAPHICS: [&str; 4] = [
    ".logo",
    ".app-icon-tile pre",
    ".levelup-word",
    ".chart-overlay",
];

/// The tones a theme gives an ink.
const TONES: [&str; 6] = [
    "accent-",
    "palette-",
    "text-muted",
    "text-subtle",
    "color-",
    "logo",
];

#[test]
fn text_in_a_tone_takes_its_ink() {
    let css = include_str!("../assets/main.css");
    let mut offenders = Vec::new();
    let mut rest = css;
    while let Some(open) = rest.find('{') {
        let (head, after) = rest.split_at(open);
        let Some(close) = after.find('}') else { break };
        let (body, tail) = after.split_at(close);
        let selector = head
            .rsplit(['}', '/'])
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        for decl in body.split(';') {
            let decl = decl.trim().trim_start_matches('{').trim();
            let Some(value) = decl.strip_prefix("color:") else {
                continue;
            };
            let value = value.trim();
            let bare_tone = TONES
                .iter()
                .any(|tone| value.starts_with(&format!("var(--{tone}")));
            if bare_tone && !GRAPHICS.iter().any(|g| selector.ends_with(g)) {
                offenders.push(format!("{selector} {{ color: {value} }}"));
            }
        }
        rest = tail;
    }
    assert!(
        offenders.is_empty(),
        "text in a bare tone (use var(--ink-<tone>, var(--<tone>))):\n  {}",
        offenders.join("\n  ")
    );
}
