# Changelog

Unreleased entries collect at the top. At cut time they move under a version
heading with the date, and `Cargo.toml` bumps in the same commit. See
`development/versioning.md` for what bumps what.

## Unreleased

- Something happens when a counter's daily goal is met. Seven of them: a
  sheen, falling confetti, party poppers from the bottom corners, the success
  line typed out behind a block cursor, the line stamped on, a scanline
  sweeping down, or the screen edge pulsing. Random rotates through them.
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
