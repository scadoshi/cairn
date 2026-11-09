//! Small text helpers the screens share.

/// `message` with its first letter uppercased, for a domain error line shown
/// to the user. Domain messages are lowercase.
pub fn sentence_case(message: &str) -> String {
    let mut chars = message.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
