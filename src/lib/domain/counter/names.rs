//! Counter names are unique, compared trimmed and without case: "Pull-ups"
//! and "pull-ups " are one name. A store refuses a second, an import renames
//! its own repeats, and a store from before the rule renames its later
//! repeats once.

use std::collections::BTreeSet;

use super::models::{CounterId, CounterName};

/// The form a name is compared in: [`CounterName`] is already trimmed, so
/// this lowercases it.
pub fn key(name: &CounterName) -> String {
    name.as_str().to_lowercase()
}

/// The message when `name` is taken.
pub fn name_taken(name: &CounterName) -> String {
    format!("a counter named {name} already exists")
}

/// `name` with the first of " (1)", " (2)", ... whose result no key in
/// `taken` matches, the base cut short so the whole fits
/// [`CounterName::MAX_LEN`].
pub fn free_name(name: &CounterName, taken: &BTreeSet<String>) -> CounterName {
    // One more try than there are taken names always finds a free one.
    for n in 1..=taken.len().saturating_add(1) {
        let suffix = format!(" ({n})");
        let room = CounterName::MAX_LEN.saturating_sub(suffix.chars().count());
        let base: String = name.as_str().chars().take(room).collect();
        let Ok(candidate) = CounterName::new(&format!("{}{suffix}", base.trim_end())) else {
            continue;
        };
        if !taken.contains(&key(&candidate)) {
            return candidate;
        }
    }
    name.clone()
}

/// New names for the counters, given in creation order, that repeat an
/// earlier counter's name: the first keeps it, each later one takes
/// [`free_name`], skipping any name in the list or given out before it.
pub fn dedupe(counters: &[(CounterId, CounterName)]) -> Vec<(CounterId, CounterName)> {
    let mut taken: BTreeSet<String> = counters.iter().map(|(_, name)| key(name)).collect();
    let mut seen = BTreeSet::new();
    let mut renamed = Vec::new();
    for (id, name) in counters {
        if seen.insert(key(name)) {
            continue;
        }
        let free = free_name(name, &taken);
        taken.insert(key(&free));
        renamed.push((*id, free));
    }
    renamed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(s: &str) -> CounterName {
        CounterName::new(s).unwrap()
    }

    #[test]
    fn names_compare_trimmed_and_without_case() {
        assert_eq!(key(&n(" Pull-Ups ")), key(&n("pull-ups")));
        assert_ne!(key(&n("Pull-ups")), key(&n("Push-ups")));
        assert_eq!(
            name_taken(&n("Pull-ups")),
            "a counter named Pull-ups already exists"
        );
    }

    #[test]
    fn a_free_name_takes_the_first_open_suffix() {
        let taken: BTreeSet<String> = ["pull-ups", "pull-ups (1)"].map(str::to_owned).into();
        assert_eq!(free_name(&n("Pull-ups"), &taken), n("Pull-ups (2)"));
        assert_eq!(free_name(&n("Squats"), &BTreeSet::new()), n("Squats (1)"));
    }

    #[test]
    fn a_long_name_is_cut_to_fit_its_suffix() {
        let long = n("Twenty-four letters long");
        assert_eq!(long.as_str().chars().count(), CounterName::MAX_LEN);
        let free = free_name(&long, &BTreeSet::new());
        assert_eq!(free.as_str(), "Twenty-four letters (1)");
        assert!(free.as_str().chars().count() <= CounterName::MAX_LEN);
    }

    #[test]
    fn later_repeats_are_renamed_in_creation_order() {
        let seeded = [
            (CounterId(1), n("Pull-ups")),
            (CounterId(2), n("pull-ups")),
            (CounterId(3), n("Squats")),
            (CounterId(4), n("Pull-ups (1)")),
            (CounterId(5), n("PULL-UPS")),
            (CounterId(6), n("squats")),
        ];
        assert_eq!(
            dedupe(&seeded),
            [
                (CounterId(2), n("pull-ups (2)")),
                (CounterId(5), n("PULL-UPS (3)")),
                (CounterId(6), n("squats (1)")),
            ],
            "the first keeps its name; a suffix already in the list is skipped"
        );
        assert_eq!(
            dedupe(&[(CounterId(1), n("a")), (CounterId(2), n("b"))]),
            []
        );
    }
}
