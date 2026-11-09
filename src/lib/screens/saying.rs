//! What a celebration says.

/// What a celebration says.
///
/// Two kinds, because the line sets rotate so the same words do not come up
/// twice running, while a figure worked out at the tap is whatever it is.
#[derive(Debug, Clone)]
pub enum Saying {
    /// This exact text, such as "40% of the year".
    Fixed(String),
    /// One of a set, picked by how many celebrations have fired.
    Rotating(fn(u64) -> &'static str),
}
