//! A toast's motion is the kit's, reduced motion included: under it a timed toast still fades
//! out at the end of its life, so Cairn's stylesheet never switches a toast's animation off.

const MAIN: &str = include_str!("../assets/main.css");

#[test]
fn cairn_leaves_a_toast_s_motion_to_the_kit() {
    assert!(
        !MAIN.contains(".toast-container [role=\"alertdialog\"] {\n        animation: none;"),
        "Cairn turns the toast's fade off under reduced motion"
    );
}
