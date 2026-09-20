# Overview

What is live, as of 2026-09-20. `todo.md` is what's next; `backlog.md` is
someday.

## State

Notch runs on desktop and in the iOS simulator. Nothing has shipped to a real
phone yet; there is no App Store listing. The database schema is at v4.

## What works

- Home landing with the wordmark and a box for the day: date, day of the year,
  ISO week, reps logged, counters touched, best streak.
- Counters list with plus and minus by each counter's step.
- Counter screen: odometer, today, streak, this month, trends (six views),
  numbers, this year with goal pace, by-year table. Edit sheet for name, goal,
  and step. Delete behind a dialog.
- New counter screen with per-day or per-year goal and step.
- Profile: theme sheet with live preview, dark mode, CSV export.
- OS back gesture on iOS (edge swipe) and Android (gesture and button, via
  the post-bundle patch).

## What it does not do yet

- Edit a past day's count. Today is the only day the buttons touch.
- Import from CSV.
- Anything on the Apple Watch.
