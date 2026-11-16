//! Every weight Cairn's sheet names is a face it ships: `JetBrains Mono` in 400 and 700. A
//! browser draws 600 with the 700 face, so a rule says 700 when it means bold.

const SHEET: &str = include_str!("../assets/main.css");

#[test]
fn no_rule_asks_for_a_weight_that_draws_as_another() {
    for (number, line) in SHEET.lines().enumerate() {
        let squeezed: String = line.split_whitespace().collect();
        assert!(
            !squeezed.contains("font-weight:600"),
            "main.css:{}: 600 draws with the 700 face; say 700",
            number.saturating_add(1)
        );
    }
}
