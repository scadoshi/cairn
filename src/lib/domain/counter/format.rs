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

/// Short form for confined spaces: full digits with commas below ten
/// thousand, then "12.3k", "123k", "1.23M", "12.3M", "123M", "1.2B". Three
/// significant figures at most, so a tile never grows past seven characters.
pub fn compact(n: u32) -> String {
    if n < 10_000 {
        return thousands(n);
    }
    let tiers: [(f64, &str); 3] = [(1e3, "k"), (1e6, "M"), (1e9, "B")];
    let mut value = f64::from(n);
    let mut suffix = "";
    for (div, sfx) in tiers {
        let scaled = f64::from(n) / div;
        // Stop at the first tier that keeps the figure under a thousand once
        // rounded, so 999,999,999 reads as 1B rather than 1000M.
        if scaled.round() < 1000.0 || sfx == "B" {
            value = scaled;
            suffix = sfx;
            break;
        }
    }
    let text = if value < 10.0 {
        format!("{value:.2}")
    } else if value < 100.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.0}")
    };
    let text = if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        text
    };
    format!("{text}{suffix}")
}

/// [`compact`] for a signed value, keeping the sign.
pub fn compact_i64(n: i64) -> String {
    let magnitude = u32::try_from(n.unsigned_abs()).unwrap_or(u32::MAX);
    if n < 0 {
        format!("-{}", compact(magnitude))
    } else {
        compact(magnitude)
    }
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
    fn compact_shrinks_past_ten_thousand() {
        assert_eq!(compact(9_999), "9,999");
        assert_eq!(compact(10_000), "10k");
        assert_eq!(compact(12_345), "12.3k");
        assert_eq!(compact(123_456), "123k");
        assert_eq!(compact(1_234_567), "1.23M");
        assert_eq!(compact(12_345_678), "12.3M");
        assert_eq!(compact(999_999_999), "1B");
        assert_eq!(compact(100_000), "100k");
        assert_eq!(compact(1_000_000), "1M");
        assert_eq!(compact(1_500_000_000), "1.5B");
        assert_eq!(compact_i64(-45_000), "-45k");
    }

    #[test]
    fn rate_keeps_one_decimal_and_groups() {
        assert_eq!(rate(14.26), "14.3");
        assert_eq!(rate(1234.56), "1,234.6");
        assert_eq!(rate(0.04), "0.0");
    }
}
