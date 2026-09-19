//! The export shape: a `day,count` header and one ISO date row per entry.

use super::DayCount;
use std::fmt::Write;

/// Renders entries as CSV text, oldest first. Dates are `YYYY-MM-DD`, so the
/// file sorts correctly as plain text and needs no quoting.
pub fn render(entries: &[DayCount]) -> String {
    let mut sorted: Vec<&DayCount> = entries.iter().collect();
    sorted.sort_by_key(|e| e.day);
    let mut out = String::from("day,count\n");
    for e in sorted {
        // Writing to a String cannot fail.
        let _ = writeln!(out, "{},{}", e.day.format("%Y-%m-%d"), e.count);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn header_only_when_empty() {
        assert_eq!(render(&[]), "day,count\n");
    }

    #[test]
    fn rows_sorted_oldest_first() {
        let entries = [
            DayCount {
                day: d(2026, 3, 2),
                count: 20,
            },
            DayCount {
                day: d(2026, 3, 1),
                count: 15,
            },
        ];
        assert_eq!(
            render(&entries),
            "day,count\n2026-03-01,15\n2026-03-02,20\n"
        );
    }
}
