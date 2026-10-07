# Changelog

Unreleased entries collect at the top. At cut time they move under a version
heading with the date, and `Cargo.toml` bumps in the same commit. See
`development/versioning.md` for what bumps what.

## Unreleased

- Line charts stay at zero on rest days next to a big day, instead of dipping below the axis.

- Picking a theme after flipping Dark mode in Config keeps the dark mode you set. It used to switch it back.

- The quote's author, source and countdown are tags in one wrapping row, which arrives with its divider once the quote finishes typing.

- New quotes from Yoda and Apollo Creed.

- A "!" in the header's left corner opens About the developer, with a link to scottyfermo.com.

- Tapping the mark on Home plays its decode again.

- On app open, the quote of the hour types itself out behind a block cursor, like a person at a keyboard: a varying pace, pauses between words and after punctuation, and the odd slip that gets backspaced and retyped. The system's Reduce Motion setting shows it whole.

- iOS: buttons respond while a screen is still scrolling, rather than once the scroll comes to rest.

- iOS: swiping back from the left edge works while a screen is still scrolling.

- Every figure rolls its digits into place as its card comes into view, and
  again when a tap changes it: the odometer, every tile, the goal chips, the
  year table, the chart's average and spread. The mark and the charts hold
  their entrance the same way, so a card below the fold arrives as you reach
  it rather than unseen at load. The system's Reduce Motion setting turns all
  of it off.

- A counter's screen has a new card, Every day: the last year as a grid, a
  column per week, each day shaded by how it ranks against the counter's
  other logged days, the biggest days lit. Tap a day for its count. It
  scrolls sideways and opens on the newest weeks.

- The home mark resolves from static as the screen opens, and again when the
  Mark setting switches between the C and the S.


- Celebrations now fire on three things, not one: a counter finishing its
  day, the tap that finishes the last counter of the day, and passing
  another tenth of the year's goal. Same animation for all three, different
  words: the day's own lines, a set for the whole day being cleared, and the
  figure itself for a tenth ("40% of the year"). Rarest wins when a tap
  crosses more than one.

- Something happens when a counter's daily goal is met. Six of them: a level
  up, with the words, a beam of light and sparks rising with it; falling
  confetti; party poppers from the bottom corners; the success line typed out
  behind a block cursor; the line stamped on; or rings pulsing inward from the
  screen edge. Random rotates through them.
  It fires from the tap that crosses, so reopening the app on a finished day
  celebrates nothing. Set app-wide under Goal animation, and per counter in
  its Edit sheet, where Default follows the app-wide choice. Schema v7.
- Toasts moved to the top left and stack downwards, fading in and out. They
  used to sit above the bottom bar, over the buttons of the last counter in
  the list.
- Step now offers 20, 30 and 40 alongside the sizes it had.
- An optional big step per counter, giving a bar of `-20 -10 +10 +20 Edit`.
  Schema v6; counters without one keep three buttons.

- Renamed to Cairn. The bundle id `com.scadoshi.count` and the data folder
  `scadoshi-count/count.db` deliberately did not move: both are permanent once
  real data exists.
- One screen instead of two. Home and the counter list merged, with the mark
  and the date sharing the hero's top row.
- Cross-counter figures: logged today, goals met, lifetime across everything,
  and a streak counting days anything at all was logged.
- A goal tag on each card counting down to the day's share, green once met.
- Goals per week, alongside per day and per year.
- A quote that turns over on the clock hour, 41 of them, each sourced.
- A "?" in every screen and sheet header, explaining what that screen does.
- A Mark setting, switching the home logo between the C and the owner's S.
- Tapping a counter's name or numbers opens it; the Open button is gone.
- Toasts report what a tap actually did, so an empty day reads "-0".
- Shipped to a real phone, with backup and restore tooling around it.

- Counters with per-day entries, a lifetime odometer, and per-year breakdown.
- Goals per day or per year, with pace: target today, ahead or behind, needed
  per day, projected year end.
- A step per counter, one of 1, 5, 10, 20, 25, 30, 40, 50, 100.
- Trend charts: this week, 60 days with a 7-day average, weekly totals,
  monthly averages, weekday pattern, hour of day.
- A numbers card: streaks, consistency, last 7 days against the 7 before,
  best week and month, days since last logged.
- Theme picker and dark mode, persisted. Toasts on every significant action.
- CSV export, one file per counter, day and count.
- OS back gesture on iOS and Android, closing open sheets first.
