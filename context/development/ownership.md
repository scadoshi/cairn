# Ownership & Responsibility

The distilled mindset inherited from zwipe and zynergy: every module has
exactly one owner and one job. When adding code, the first question is "who
owns this?" If the answer is two places, the design is wrong.

## Who owns what in this repo

| Concern | Sole owner |
|---|---|
| What a counter and a day's entry ARE | `domain/counter/models.rs` |
| A valid counter name, a valid goal | `CounterName`, `Goal` newtypes, validated at construction |
| Totals, averages, day of year, pace | `domain/counter/stats.rs`, pure over `&[DayCount]` + `today` |
| The CSV shape | `domain/counter/csv.rs` |
| What the app needs from storage | `domain/counter/ports.rs` (traits, written by the consumer) |
| SQL, schema, migrations | `outbound/sqlite/` and nowhere else |
| Where files live on this platform | `outbound/paths.rs` |
| Screens, routing, theme signal | `inbound/ui/` |
| Wiring the store to the UI | `src/bin/notch.rs`, and only wiring |

## The rules behind the table

- **Domain owns meaning; adapters own translation.** The domain never imports
  from `inbound/` or `outbound/`. SQLite rows are mapped to domain types in
  exactly one file.
- **Newtypes carry their own validity.** If a `CounterName` exists, it is
  valid. No re-checking downstream.
- **Time is an input.** Every stat takes `today` as a parameter. The clock is
  read once, at the UI edge, so the math is testable with fixed dates.
- **Ports are written from the consumer's side.** `CounterStore` says what a
  screen needs, not what rusqlite offers. The watch build gets a new adapter,
  not a refactor.
- **Consolidate before adding.** One crate, one binary. A workspace or a shared
  crate needs a reason the current layout can't absorb.
