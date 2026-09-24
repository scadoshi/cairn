# Overview

What is live, as of 2026-09-24. `todo.md` is what's next; `backlog.md` is
someday.

## State

Cairn is installed on a real iPhone and in daily use, signed with a
development profile good until September 2027. It also runs on desktop and in
the simulator. There is no App Store listing yet. The database schema is at
v7, holding nine months of imported history: 225 days, 10,200 taps, 98,400
reps across push-ups, pull-ups and squats.

7,200 lines of Rust, 87 tests, 2,800 lines of pure domain.

## What works

- One screen for everything: the mark and today's date sharing the hero's top
  row, then today across every counter, then a quote, then the counters
  themselves. Home and the counter list used to be separate; reaching the
  counters cost a tap on the thing the app is for.
- Cross-counter figures on that hero: logged today, goals met of those with a
  goal, lifetime across everything, and a streak counting days anything at all
  was logged rather than the best per-counter run.
- Counter cards with plus and minus by each counter's step, an Edit sheet, and
  a goal tag that counts down in the error colour and turns green when the
  day's share is in. Tapping the name or numbers opens the counter. A counter
  can carry a second, larger step, which puts a wider pair outside the first.
- Something plays when a counter's day is finished: a sheen, confetti,
  poppers, a typed line, a stamp, a scanline, or a pulse at the screen edge,
  chosen per counter or app-wide, or turned off. It fires from the tap that
  crosses the goal and nowhere else, so no state is stored to remember that a
  day was already celebrated.
- Counter screen: odometer, today, this week, this month, trends (six views),
  goal pace, bests, habit figures, by-year table. Delete behind a dialog.
- Goals per day, week or year, all reducing to what today has to clear.
- A quote that turns over on the clock hour, 41 of them, each traced to a
  book, newsletter or interview.
- Config: theme, mark (the C or the owner's S), goal animation, dark mode,
  date format, when a day and week start, rest days, counter order,
  confirm-minus, CSV export.
  Every row explains itself behind a question mark, and every screen and sheet
  has one in its header.
- OS back gesture on iOS and Android, closing open sheets first.

## Shipping

`scripts/ios/deploy.sh` is the daily loop: back up the phone's database,
build, install. A reinstall keeps the data container, so deploying never
touches the counts, and the backup runs first regardless because the phone
holds taps that exist nowhere else. Backups live outside the repo at
`~/Developer/cairn-data/backups/<phone>/`, one directory per device.

`operations/ios/dev_deploy.md` has the whole thing, `testers.md` covers
getting it onto someone else's phone.

## Known gaps

- No App Store listing. TestFlight and review are next.
- The database push only works on a development-signed build, which carries
  `get-task-allow`. A TestFlight build cannot be seeded this way; that needs
  the in-app import on the backlog.
- Past days are read-only. Only today can be adjusted.
- The UI layer has 2,570 lines and two tests. The domain boundary is what
  makes things testable, so logic belongs on the other side of it.
