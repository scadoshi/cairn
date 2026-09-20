# Decisions

Numbered so other docs can cite them.

## 1. Local SQLite, no server

The data is one person's counts. A server would add accounts, sync, and
hosting for no gain. `rusqlite` with the `bundled` feature compiles SQLite in,
so desktop, iOS, and later the watch ship the same engine.

## 2. Single crate, hexagonal inside

zwipe needs a workspace because it has a server, an app, and a site. Notch has
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

The portfolio keeps the theme in `localStorage` because it is a website. Notch
has a database already, so the theme goes in a `settings` table and there is
one persistence path.

## 8. No server. Sync, when it comes, is CloudKit

Decided 2026-09-20. The worry was losing a phone and losing the counts; the
answer is not a hosted backend. iOS device backup already carries the app's
data folder, and CSV export covers the rest. What a server would add (more
than one device, a web view, Android, anything social) is a product change,
and it brings accounts, auth, Postgres, hosting, a privacy policy, and a
service to run for as long as anyone uses it. zwipe has all of that; Notch
does not want it.

When Notch needs data on more than one device, which the Apple Watch will
force, the answer is CloudKit's private database: the user's own records,
synced by Apple, no accounts, no server, no cost. It needs a native bridge
in the shape of the back gesture's objc2 module. The README's promise holds:
nothing leaves the phone except into the user's own iCloud.

The sync unit is the events table. Every tap is an append-only record with
a timestamp, and entries are derivable from it, so events merge cleanly
across devices where daily totals would conflict. Nothing goes into the
schema that is not derivable from events.

## 9. Renamed from Odo to Notch

2026-09-20. An "odo" step-counter with odometer digits already sits in the
App Store, same concept, same name. Notch is free there, one syllable, and
means the thing: one notch per rep, notching up. The bundle id is
`com.scadoshi.notch`; the data folder and database moved with it. The big
lifetime readout is still called the odometer in the code, since that is
what it is.

## 10. Renamed again, from Notch to Crow

2026-09-20, the same day. Notch was fine but anonymous. Crows count: a
2024 study had carrion crows caw a set number of times on cue, and corvid
numeracy is the best studied among birds. It also sits in the bird
lineage with chickadee and steller. The store name will be "scadoshi
count" with Crow under the icon; the bundle id `com.scadoshi.crow` is the
part that lasts.
