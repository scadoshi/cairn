//! A keystroke script that types a line out the way a person would.
//!
//! The script is a pure function of the text and a seed: the same pair gives
//! the same keystrokes, and replaying any script always leaves exactly the
//! text. Speed drifts from word to word, the hand pauses after a word or a
//! comma, hesitates now and then inside one, and sometimes hits a neighboring
//! key, types on a little before noticing, backspaces and fixes it.

/// One key press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// Types this character.
    Char(char),
    /// Deletes the last character typed.
    Backspace,
}

/// A key press and how long to wait before it lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stroke {
    /// What is pressed.
    pub key: Key,
    /// Milliseconds after the previous stroke.
    pub wait_ms: u32,
}

/// The wait before the first key, with the cursor already blinking.
const LEAD_IN_MS: (u32, u32) = (450, 850);
/// The gap between two keys inside a word, before the word's pace scales it.
const KEY_MS: (u32, u32) = (14, 39);
/// Each word's pace, in hundredths: under 100 is a quicker word.
const WORD_PACE: (u32, u32) = (70, 125);
/// Percent of words that start after a pause to think.
const WORD_PAUSE_PCT: u32 = 18;
const WORD_PAUSE_MS: (u32, u32) = (50, 150);
/// After a comma, semicolon, colon or dash.
const CLAUSE_PAUSE_MS: (u32, u32) = (80, 190);
/// After the end of a sentence.
const SENTENCE_PAUSE_MS: (u32, u32) = (150, 300);
/// Percent of letters, past a word's first, preceded by a stall mid-word.
const HESITATE_PCT: u32 = 3;
const HESITATE_MS: (u32, u32) = (80, 210);
/// Percent of letters that come out as a neighboring key.
const TYPO_PCT: u32 = 2;
/// Letters to type after a typo before another can happen.
const TYPO_GAP: usize = 12;
/// Most correct letters typed past a typo before it is noticed.
const TYPO_RUN_ON: u32 = 2;
/// The stare at the mistake before backspacing.
const NOTICE_MS: (u32, u32) = (110, 240);
const BACKSPACE_MS: (u32, u32) = (27, 52);
/// The beat after the last backspace before typing resumes.
const RESUME_MS: (u32, u32) = (45, 110);

/// The keystrokes that type `text`, varied by `seed`.
pub fn script(text: &str, seed: u64) -> Vec<Stroke> {
    let chars: Vec<char> = text.chars().collect();
    let mut rng = SplitMix64(seed);
    let mut out = Vec::with_capacity(chars.len() + chars.len() / 10);
    let mut pace = rng.range(WORD_PACE);
    let mut since_typo = 0;
    let mut wait = rng.range(LEAD_IN_MS);

    for (i, &c) in chars.iter().enumerate() {
        let prev = i.checked_sub(1).and_then(|p| chars.get(p)).copied();
        if prev.is_some_and(char::is_whitespace) && !c.is_whitespace() {
            pace = rng.range(WORD_PACE);
            if rng.percent(WORD_PAUSE_PCT) {
                wait += rng.range(WORD_PAUSE_MS);
            }
        } else if prev.is_some_and(char::is_alphabetic)
            && c.is_alphabetic()
            && rng.percent(HESITATE_PCT)
        {
            wait += rng.range(HESITATE_MS);
        }

        since_typo += 1;
        let slip = (i > 0 && since_typo > TYPO_GAP && rng.percent(TYPO_PCT))
            .then(|| neighbor(c, &mut rng))
            .flatten();
        if let Some(wrong) = slip {
            since_typo = 0;
            out.push(Stroke {
                key: Key::Char(wrong),
                wait_ms: wait + key_gap(&mut rng, pace),
            });
            // The fingers run on through the rest of the word before the eye
            // catches up, so the slip can be a letter or two back.
            let run_on = chars
                .get(i + 1..)
                .unwrap_or_default()
                .iter()
                .take(rng.range((0, TYPO_RUN_ON)) as usize)
                .take_while(|n| n.is_alphabetic());
            let mut typed = 1;
            for &n in run_on {
                out.push(Stroke {
                    key: Key::Char(n),
                    wait_ms: key_gap(&mut rng, pace),
                });
                typed += 1;
            }
            let mut back = rng.range(NOTICE_MS);
            for _ in 0..typed {
                out.push(Stroke {
                    key: Key::Backspace,
                    wait_ms: back,
                });
                back = rng.range(BACKSPACE_MS);
            }
            wait = rng.range(RESUME_MS);
        }

        out.push(Stroke {
            key: Key::Char(c),
            wait_ms: wait + key_gap(&mut rng, pace),
        });
        wait = match c {
            '.' | '!' | '?' => rng.range(SENTENCE_PAUSE_MS),
            ',' | ';' | ':' | '-' => rng.range(CLAUSE_PAUSE_MS),
            _ => 0,
        };
    }
    out
}

/// The gap before a key at a word's pace.
fn key_gap(rng: &mut SplitMix64, pace: u32) -> u32 {
    rng.range(KEY_MS) * pace / 100
}

/// A key next to `c` on a QWERTY keyboard, in the same case. `None` for
/// anything but an ASCII letter.
fn neighbor(c: char, rng: &mut SplitMix64) -> Option<char> {
    let near = match c.to_ascii_lowercase() {
        'q' => "wa",
        'w' => "qes",
        'e' => "wrd",
        'r' => "etf",
        't' => "ryg",
        'y' => "tuh",
        'u' => "yij",
        'i' => "uok",
        'o' => "ipl",
        'p' => "ol",
        'a' => "qsz",
        's' => "adw",
        'd' => "sfe",
        'f' => "dgr",
        'g' => "fht",
        'h' => "gjy",
        'j' => "hku",
        'k' => "jli",
        'l' => "ko",
        'z' => "xa",
        'x' => "zcs",
        'c' => "xvd",
        'v' => "cbf",
        'b' => "vng",
        'n' => "bmh",
        'm' => "nj",
        _ => return None,
    };
    let pick = rng.range((0, u32::try_from(near.len()).ok()? - 1)) as usize;
    let wrong = near.chars().nth(pick)?;
    Some(if c.is_ascii_uppercase() {
        wrong.to_ascii_uppercase()
    } else {
        wrong
    })
}

/// A small seeded generator. Plenty for picking delays, and no dependency.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value in `lo..=hi`.
    fn range(&mut self, (lo, hi): (u32, u32)) -> u32 {
        let span = u64::from(hi.saturating_sub(lo)) + 1;
        lo + u32::try_from(self.next() % span).unwrap_or(0)
    }

    /// True `pct` times in a hundred.
    fn percent(&mut self, pct: u32) -> bool {
        self.range((0, 99)) < pct
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::quote;

    fn replay(strokes: &[Stroke]) -> String {
        let mut s = String::new();
        for stroke in strokes {
            match stroke.key {
                Key::Char(c) => s.push(c),
                Key::Backspace => {
                    s.pop();
                }
            }
        }
        s
    }

    fn quotes() -> impl Iterator<Item = &'static str> {
        (0..200).filter_map(quote::at_hour).map(|q| q.text)
    }

    #[test]
    fn every_quote_replays_to_its_text() {
        for text in quotes() {
            for seed in 0..50 {
                assert_eq!(replay(&script(text, seed)), text, "seed {seed}");
            }
        }
    }

    #[test]
    fn same_seed_same_script() {
        let text = "The obstacle is the way.";
        assert_eq!(script(text, 7), script(text, 7));
        assert_ne!(script(text, 7), script(text, 8));
    }

    #[test]
    fn empty_text_types_nothing() {
        assert_eq!(script("", 1), []);
    }

    #[test]
    fn backspaces_never_reach_past_what_was_typed() {
        for text in quotes() {
            for seed in 0..50 {
                let mut len = 0usize;
                for stroke in script(text, seed) {
                    match stroke.key {
                        Key::Char(_) => len += 1,
                        Key::Backspace => len = len.checked_sub(1).expect("backspace on nothing"),
                    }
                }
            }
        }
    }

    #[test]
    fn typos_happen_and_get_fixed() {
        let typos: usize = quotes()
            .flat_map(|text| (0..20).map(move |seed| script(text, seed)))
            .map(|s| s.iter().filter(|k| k.key == Key::Backspace).count())
            .sum();
        assert!(typos > 0);
    }

    #[test]
    fn a_typo_is_a_neighboring_key_in_the_same_case() {
        let mut rng = SplitMix64(3);
        for _ in 0..100 {
            let wrong = neighbor('G', &mut rng).expect("a letter has neighbors");
            assert!("FHT".contains(wrong));
        }
        assert_eq!(neighbor('7', &mut rng), None);
    }

    #[test]
    fn a_median_quote_takes_seconds_not_minutes() {
        let text = "x".repeat(90);
        let total: u32 = script(&text, 1).iter().map(|s| s.wait_ms).sum();
        assert!((1_500..6_000).contains(&total), "{total}ms");
    }
}
