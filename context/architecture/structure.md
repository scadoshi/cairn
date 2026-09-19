# Structure

```
odo/
├── Cargo.toml              # single crate; lib + one bin
├── Dioxus.toml             # app name, bundle id, iOS plist
├── assets/                 # main.css, fonts, ASCII logo
├── context/                # this documentation
└── src/
    ├── bin/odo.rs          # open the store, launch Dioxus with it in context
    └── lib/
        ├── lib.rs
        ├── domain/
        │   └── counter/
        │       ├── mod.rs
        │       ├── models.rs   # Counter, CounterId, CounterName, DayCount, Goal
        │       ├── stats.rs    # summarize(): lifetime, this year, per-day, pace
        │       ├── csv.rs      # entries -> "day,count" text
        │       └── ports.rs    # CounterStore, SettingsStore, StoreError
        ├── inbound/
        │   └── ui/
        │       ├── mod.rs      # App root: theme signal, CSS, router
        │       ├── router.rs   # Route enum + layout
        │       ├── screens/    # home.rs, counter.rs
        │       └── components/ # navbar.rs, stat_tile.rs
        └── outbound/
            ├── paths.rs        # data dir, download dir
            └── sqlite/
                └── mod.rs      # SqliteStore: schema + both port impls
```

## Data model

- `counters(id, name, goal_per_year, created_at)`
- `entries(counter_id, day, count)` with `PRIMARY KEY (counter_id, day)`; one
  row per counter per day, which is also the export shape.
- `settings(key, value)` for the theme.

## Dependency direction

```
bin ──> inbound/ui ──> domain <── outbound/sqlite
```

Nothing points into `inbound/` or `outbound/` from `domain/`.
