# Decisions

Numbered so other docs can cite them.

## 1. Local SQLite, no server

The data is one person's counts. A server would add accounts, sync, and
hosting for no gain. `rusqlite` with the `bundled` feature compiles SQLite in,
so desktop, iOS, and later the watch ship the same engine.

## 2. Single crate, hexagonal inside

zwipe needs a workspace because it has a server, an app, and a site. Odo has
one binary. The hexagonal split (domain / inbound / outbound) is kept as
modules inside one crate, which is enough to keep the domain portable to a
watch target without a workspace's overhead.

## 3. Stats take `today` as a parameter

Every summary and pace calculation is a pure function of the entry list and a
date. The UI reads the clock once. This makes the math unit-testable with
fixed dates and keeps the domain free of platform time APIs.

## 4. One row per counter per day

Entries are `(counter_id, day, count)`. Tapping "+1" upserts today's row rather
than appending an event. Simpler queries, and the table is already the CSV.

## 5. Goals are per year

The use case is "how many pull-ups this year." A yearly goal derives today's
target (goal x day_of_year / days_in_year), whether you are ahead or behind,
and the per-day rate needed from here. Other goal periods can come later if a
real need shows up.

## 6. zwipe-components as a git dependency

Same as the portfolio: the shared UI crate is pulled from the zwipe repo and
its CSS is inlined through `THEMES_CSS` and `COMPONENTS_CSS`. `Cargo.lock` pins
the commit. Upgrading is a deliberate `cargo update -p zwipe-components`.

## 7. Theme stored in SQLite

The portfolio keeps the theme in `localStorage` because it is a website. Odo
has a database already, so the theme goes in a `settings` table and there is
one persistence path.
