//! The quote that sits under the counters, changing once an hour.
//!
//! There is no stored state and nothing to invalidate. Which quote shows is a
//! pure function of the hour, so every launch inside the same hour picks the
//! same one, the app can be killed and reopened without resetting anything,
//! and the countdown is just the time left on the clock.

use chrono::{DateTime, Duration, TimeZone, Timelike};

/// One quote, with where it came from. Attribution ships with the text
/// because these are real people and most are alive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quote {
    /// The words.
    pub text: &'static str,
    /// Who said them.
    pub author: &'static str,
    /// The book, episode, or talk it came from.
    pub source: &'static str,
}

/// Steps through `QUOTES` an hour at a time.
///
/// Coprime with the list length, so repeatedly adding it visits every quote
/// before any repeat. A plain `hours % len` would work too, but the quotes
/// would then march in the order they are written here, and two adjacent
/// hours would always show two adjacent quotes.
const STEP: usize = 7;

/// The quote for the hour that `at` falls in.
///
/// Whole hours since the Unix epoch, so it turns over on the hour rather
/// than an hour after launch. Timezone comes from `at`. `None` only when
/// the list is empty.
pub fn for_time<Tz: TimeZone>(at: &DateTime<Tz>) -> Option<&'static Quote> {
    at_hour(at.timestamp().div_euclid(3600))
}

/// The quote for a given count of whole hours since the epoch.
///
/// Split out from [`for_time`] so the rotation can be tested without
/// building timestamps.
pub fn at_hour(hours: i64) -> Option<&'static Quote> {
    let len = QUOTES.len();
    // The empty check is load-bearing, not defensive: rem_euclid(0) is a
    // divide by zero, which took the whole screen down while this list was
    // still being filled in.
    if len == 0 {
        return None;
    }
    // rem_euclid rather than %: hours is negative before 1970, and a clock
    // set wrong should still land on a quote instead of panicking.
    let slot = usize::try_from(hours.rem_euclid(i64::try_from(len).ok()?)).ok()?;
    QUOTES.get(slot.wrapping_mul(STEP) % len)
}

/// How long until the quote changes.
pub fn until_next<Tz: TimeZone>(at: &DateTime<Tz>) -> Duration {
    let into_hour = i64::from(at.minute()) * 60 + i64::from(at.second());
    Duration::seconds(3600 - into_hour)
}

/// Every quote, each one traced to a book, newsletter, or recorded interview.
///
/// Only lines that could be checked against the source text are here. The
/// ones left out were mostly misattributions that circulate widely: "you
/// don't stop running because you get old" is Jack Kirk rather than
/// McDougall, and "you have power over your mind, not outside events" does
/// not appear in any published translation of Meditations. Anything added
/// later should clear the same bar, because most of these people are alive.
pub const QUOTES: &[Quote] = &[
    Quote {
        text: "Procrastination is often not about indecision. It's a decision to live in theory rather than in practice.",
        author: "Chris Williamson",
        source: "Modern Wisdom, 23 Lessons",
    },
    Quote {
        text: "You will be amazed at what you can complete when you have no other option.",
        author: "Chris Williamson",
        source: "Modern Wisdom, 23 Lessons",
    },
    Quote {
        text: "If you measure inputs, you'll get good at trying. If you measure outcomes, you'll get good at winning.",
        author: "Chris Williamson",
        source: "Modern Wisdom, 23 Lessons",
    },
    Quote {
        text: "Everybody has plans until they get hit for the first time.",
        author: "Mike Tyson",
        source: "on Tyrell Biggs, AP, 1987",
    },
    Quote {
        text: "You are in danger of living a life so comfortable and soft that you will die without ever realizing your true potential.",
        author: "David Goggins",
        source: "Can't Hurt Me",
    },
    Quote {
        text: "The most important conversations you'll ever have are the ones you'll have with yourself.",
        author: "David Goggins",
        source: "Can't Hurt Me",
    },
    Quote {
        text: "The ticket to victory often comes down to bringing your very best when you feel your worst.",
        author: "David Goggins",
        source: "Can't Hurt Me",
    },
    Quote {
        text: "If you want to master the mind and remove your governor, you'll have to become addicted to hard work.",
        author: "David Goggins",
        source: "Can't Hurt Me",
    },
    Quote {
        text: "Life is too dynamic a game. We're either getting better or we're getting worse.",
        author: "David Goggins",
        source: "Can't Hurt Me",
    },
    Quote {
        text: "Pain unlocks a secret doorway in the mind, one that leads to both peak performance and beautiful silence.",
        author: "David Goggins",
        source: "Can't Hurt Me",
    },
    Quote {
        text: "In every failure there is something to be gained, even if it's only practice for the next test.",
        author: "David Goggins",
        source: "Can't Hurt Me",
    },
    Quote {
        text: "Be more than motivated, be more than driven, become literally obsessed to the point where people think you're nuts.",
        author: "David Goggins",
        source: "Can't Hurt Me",
    },
    Quote {
        text: "Extreme Ownership. Leaders must own everything in their world. There is no one else to blame.",
        author: "Jocko Willink",
        source: "Extreme Ownership",
    },
    Quote {
        text: "When it comes to standards, as a leader, it's not what you preach, it's what you tolerate.",
        author: "Jocko Willink",
        source: "Extreme Ownership",
    },
    Quote {
        text: "Discipline can seem like your worst enemy. But in reality it is your best friend.",
        author: "Jocko Willink",
        source: "Discipline Equals Freedom",
    },
    Quote {
        text: "Pursuing too many goals simultaneously often leads to failure of all goals.",
        author: "Andrew Huberman",
        source: "Huberman Lab newsletter",
    },
    Quote {
        text: "Simple goals won't cause sufficient levels of neural arousal and stress to stimulate real growth and learning.",
        author: "Andrew Huberman",
        source: "Huberman Lab newsletter",
    },
    Quote {
        text: "You can't think your way into the mood that you seek or the state of mind that you aspire to inhabit.",
        author: "Rich Roll",
        source: "Tim Ferriss Show 561",
    },
    Quote {
        text: "Action is the only thing that can trigger that change state.",
        author: "Rich Roll",
        source: "Tim Ferriss Show 561",
    },
    Quote {
        text: "If you're waiting until you feel like doing something, chances are, you're probably never going to get to it.",
        author: "Rich Roll",
        source: "Tim Ferriss Show 561",
    },
    Quote {
        text: "You have to play the long game to really reap the huge benefits of this type of training.",
        author: "Rich Roll",
        source: "Tim Ferriss Show 561",
    },
    Quote {
        text: "We were born to run; we were born because we run. We're all Running People, as the Tarahumara have always known.",
        author: "Christopher McDougall",
        source: "Born to Run",
    },
    Quote {
        text: "If you don't have answers to your problems after a four-hour run, you ain't getting them.",
        author: "Christopher McDougall",
        source: "Born to Run",
    },
    Quote {
        text: "Deny your nature, and it will erupt in some other, uglier way.",
        author: "Christopher McDougall",
        source: "Born to Run",
    },
    Quote {
        text: "There's something so universal about that sensation, the way running unites our two most primal impulses: fear and pleasure.",
        author: "Christopher McDougall",
        source: "Born to Run",
    },
    Quote {
        text: "I don't feel like I'm a great hunter or overly talented. All I do is work very hard and it pays off.",
        author: "Cameron Hanes",
        source: "Garmin interview",
    },
    Quote {
        text: "I want to make myself miserable. I want to be at my best on my worst days.",
        author: "Cameron Hanes",
        source: "Garmin interview",
    },
    Quote {
        text: "I never miss a day of training and the whole reason I train is so I can bowhunt.",
        author: "Cameron Hanes",
        source: "Garmin interview",
    },
    Quote {
        text: "Consistently showing up every day and giving your best effort will never be something you regret.",
        author: "Truett Hanes",
        source: "Guinness World Records, 2025",
    },
    Quote {
        text: "The only reason I was able to get to this point and be in this kind of shape was my willingness to become better every day.",
        author: "Truett Hanes",
        source: "Guinness World Records, 2025",
    },
    Quote {
        text: "I don't know what's in store for you, but I know the world needs you and quitting never helped anyone.",
        author: "Truett Hanes",
        source: "Guinness World Records, 2025",
    },
    Quote {
        text: "Waste no more time arguing what a good man should be. Be one.",
        author: "Marcus Aurelius",
        source: "Meditations 10.16",
    },
    Quote {
        text: "If it is not right, do not do it: if it is not true, do not say it.",
        author: "Marcus Aurelius",
        source: "Meditations 12.17",
    },
    Quote {
        text: "If thou art pained by any external thing, it is not this thing that disturbs thee, but thy own judgment about it.",
        author: "Marcus Aurelius",
        source: "Meditations 8.47",
    },
    Quote {
        text: "Do not act as if thou wert going to live ten thousand years. While thou livest, while it is in thy power, be good.",
        author: "Marcus Aurelius",
        source: "Meditations 4.17",
    },
    Quote {
        text: "You do not rise to the level of your goals. You fall to the level of your systems.",
        author: "James Clear",
        source: "Atomic Habits",
    },
    Quote {
        text: "Every action you take is a vote for the type of person you wish to become.",
        author: "James Clear",
        source: "Atomic Habits",
    },
    Quote {
        text: "Be the designer of your world and not merely the consumer of it.",
        author: "James Clear",
        source: "Atomic Habits",
    },
    Quote {
        text: "Time magnifies the margin between success and failure. It will multiply whatever you feed it.",
        author: "James Clear",
        source: "Atomic Habits",
    },
    Quote {
        text: "The first mistake is never the one that ruins you. It is the spiral of repeated mistakes that follows.",
        author: "James Clear",
        source: "Atomic Habits",
    },
    Quote {
        text: "If I used being busy as an excuse not to run, I'd never run again.",
        author: "Haruki Murakami",
        source: "What I Talk About When I Talk About Running",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn every_quote_is_reachable_before_any_repeats() {
        let len = i64::try_from(QUOTES.len()).unwrap();
        let mut seen = vec![false; QUOTES.len()];
        for h in 0..len {
            let q = at_hour(h).expect("the list is not empty");
            let i = QUOTES
                .iter()
                .position(|x| x == q)
                .expect("quote is in the list");
            assert!(
                !seen[i],
                "quote {i} repeated within one cycle: STEP shares a factor with the list length"
            );
            seen[i] = true;
        }
        assert!(seen.into_iter().all(|s| s), "some quote never shows");
    }

    #[test]
    fn the_list_is_not_empty() {
        assert!(!QUOTES.is_empty());
    }

    #[test]
    fn the_quote_holds_for_an_hour_then_moves() {
        let base = Utc.with_ymd_and_hms(2026, 9, 22, 14, 0, 0).unwrap();
        let later = Utc.with_ymd_and_hms(2026, 9, 22, 14, 59, 59).unwrap();
        let next = Utc.with_ymd_and_hms(2026, 9, 22, 15, 0, 0).unwrap();
        assert_eq!(for_time(&base), for_time(&later));
        assert_ne!(for_time(&base), for_time(&next));
    }

    #[test]
    fn countdown_runs_to_the_top_of_the_hour() {
        let at = Utc.with_ymd_and_hms(2026, 9, 22, 14, 15, 30).unwrap();
        assert_eq!(until_next(&at).num_seconds(), 44 * 60 + 30);
        let on_the_hour = Utc.with_ymd_and_hms(2026, 9, 22, 14, 0, 0).unwrap();
        assert_eq!(until_next(&on_the_hour).num_seconds(), 3600);
    }

    #[test]
    fn quotes_fit_a_phone_card() {
        for q in QUOTES {
            assert!(
                q.text.len() <= 190,
                "too long for the card: {} ({} chars)",
                q.author,
                q.text.len()
            );
            assert!(!q.author.is_empty(), "a quote with no author");
            assert!(!q.source.is_empty(), "{} has no source", q.author);
        }
    }
}
