//! Number formatting for display: thousands separators, so 5475 reads as
//! 5,475 wherever a count appears.

/// Groups digits in threes with commas.
pub fn thousands(n: u32) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Same for a signed value, keeping the sign.
pub fn thousands_i64(n: i64) -> String {
    let magnitude = u32::try_from(n.unsigned_abs()).unwrap_or(u32::MAX);
    if n < 0 {
        format!("-{}", thousands(magnitude))
    } else {
        thousands(magnitude)
    }
}

/// One decimal, thousands-grouped in the integer part: 1234.56 as 1,234.6.
pub fn rate(v: f64) -> String {
    let rounded = (v * 10.0).round() / 10.0;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let whole = rounded.trunc().abs() as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let tenths = ((rounded.abs() - rounded.trunc().abs()) * 10.0).round() as u32;
    let sign = if rounded < 0.0 { "-" } else { "" };
    format!("{sign}{}.{tenths}", thousands(whole))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_in_threes() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1000), "1,000");
        assert_eq!(thousands(5475), "5,475");
        assert_eq!(thousands(1_234_567), "1,234,567");
        assert_eq!(thousands_i64(-1200), "-1,200");
    }

    #[test]
    fn rate_keeps_one_decimal_and_groups() {
        assert_eq!(rate(14.26), "14.3");
        assert_eq!(rate(1234.56), "1,234.6");
        assert_eq!(rate(0.04), "0.0");
    }
}
