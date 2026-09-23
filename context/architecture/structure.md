# Structure

```
cairn/
├── Cargo.toml              # single crate; lib + one bin
├── Dioxus.toml             # app name, bundle id, iOS plist
├── assets/                 # main.css, toast.css, fonts, the c and s marks
├── scripts/                # icon.py, seed.py, import_taps.py, ios/, android/
├── context/                # this documentation
└── src/
    ├── bin/cairn.rs        # open the store, launch Dioxus with it in context
    └── lib/
        ├── lib.rs
        ├── domain/                 # pure: no dioxus, no rusqlite, no clock
        │   ├── date_format.rs      # MM/DD/YY or DD/MM/YY
        │   ├── preferences.rs      # week start, rollover, rest days, Logo
        │   ├── quote.rs            # the hourly quote and its rotation
        │   └── counter/
        │       ├── models.rs       # Counter, CounterName, Goal, Step, DayCount, Event
        │       ├── stats.rs        # summarize, pace, streaks, merge_days, applied_delta
        │       ├── series.rs       # daily, weekly, monthly, weekday, hourly
        │       ├── format.rs       # thousands, compact, rate
        │       ├── csv.rs          # entries -> "day,count"
        │       └── ports.rs        # CounterStore, SettingsStore, StoreError
        ├── inbound/ui/
        │   ├── mod.rs              # App root, Shell, contexts, today()/now()
        │   ├── router.rs           # Route enum + layouts
        │   ├── screens/            # home.rs, counter.rs, config.rs
        │   └── components/
        │       ├── counter_list.rs # the cards, the main body of home
        │       ├── counter_form.rs # the shared form and the edit sheet
        │       ├── quote_card.rs   # the quote and its countdown
        │       ├── dialog_host.rs  # one slot at the root so overlays escape
        │       ├── alert_dialog.rs, bottom_sheet.rs, hint.rs
        │       ├── line_chart.rs, tile.rs
        │       └── navigation/     # back_handler.rs, overlay_stack.rs
        └── outbound/
            ├── paths.rs            # data dir, exports dir
            └── sqlite/mod.rs       # SqliteStore: migrations + both port impls
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
