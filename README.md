# Odo

A lifetime counter, as in odometer. Name a thing you do (pull-ups, push-ups,
pages), log how many each day, and Odo keeps the running total plus the numbers
that make a total mean something: this year, average per day, day of the year,
and where you stand against a yearly goal.

Local only. SQLite on the device, no account, no server. Exports to CSV with
two columns, `day` and `count`.

## Stack

- Rust, [Dioxus](https://dioxuslabs.com/) 0.7, desktop first, iOS next
- UI from [zwipe-components](https://github.com/scadoshi/zwipe), so it looks
  like zwipe, zite, and scottyfermo.com, with the same theme picker
- `rusqlite` (bundled)

## Build

```
cargo install dioxus-cli --locked
dx serve                      # desktop
dx serve --platform ios       # simulator
cargo test
```

Rules and layout live in `context/`; start at `context/README.md`.
