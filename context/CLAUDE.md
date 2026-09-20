# CLAUDE.md

Guidance for Claude Code and any other AI assistant working in this repository.

## Project Overview

Crow is a lifetime counter, named for the bird that can count. You name a thing you do (pull-ups,
push-ups, pages read), log how many you did each day, and Crow keeps the running
total plus the rates and averages that make a total mean something: this year's
total, average per day, where you stand against a yearly goal, and what today
needs to be to stay on pace.

- Single Rust crate, Dioxus 0.7 desktop app first, iOS later, Apple Watch
  eventually.
- Data lives on the device in SQLite. There is no server and no account.
- The UI is built from `zwipe-components` (Panel, NavBar, Button, ThemePicker
  and the theme palettes) so it looks like zwipe, zite, and the portfolio.
- Exports are CSV with two columns, `day` and `count`, one file per counter.

## Layout

Hexagonal, the same shape as zwipe and zynergy, sized for one crate:

```
src/
├── bin/crow.rs            # entrypoint: opens the store, launches the app
└── lib/
    ├── domain/counter/   # pure: models, stats, goals, csv, ports (no dioxus, no sqlite)
    ├── inbound/ui/       # Dioxus: router, screens/, components/
    └── outbound/sqlite/  # rusqlite adapter implementing the ports
```

`architecture/structure.md` walks the tree; `architecture/decisions.md` says
why. `development/ownership.md` is the one-page mindset: every module has one
owner and one job.

## Domain Purity Rules

`domain/` is the part that will one day run on a watch, so it stays portable:

- No `dioxus`, no `rusqlite`, no `dirs`, no platform crates. Allowed: `chrono`,
  `serde`, `thiserror`.
- No `#[cfg(feature = ...)]` anywhere in `domain/`.
- All stats and goal math is a pure function over `&[DayCount]` plus a `today`
  date passed in. Never read the clock inside the domain.
- Newtypes validate at construction (`CounterName`, `Goal`). Consumers trust
  them and never re-check.
- Ports (`CounterStore`, `SettingsStore`) are written from the domain's side:
  what the app needs, not what SQLite offers. SQL lives in `outbound/sqlite/`
  and nowhere else.

## Common Commands

```bash
dx serve                                  # desktop app with hot reload
dx serve --platform ios                   # iOS simulator (needs Xcode)
cargo test                                # domain tests, offline
cargo +nightly fmt                        # rustfmt.toml uses an unstable option; stable fmt skips it
cargo clippy --all-targets -- -D warnings # what CI runs
```

The database lives at the platform data dir, `~/Library/Application
Support/crow/crow.db` on macOS. Delete it to start over.

## Linting

Pedantic clippy with the panic family denied outside tests (`unwrap`, `expect`,
`panic`, indexing, slicing), same lint set as chickadee. `clippy.toml` allows
them inside tests. CI treats warnings as errors.

## Commit Guidelines

See `development/commit_guidelines.md`. The short version: one-line messages,
no emojis, never any AI-agent signature or Co-Authored-By trailer. Never push
without being asked.

## Context Directory

```
context/
├── README.md         — start-here index + current focus
├── CLAUDE.md         — this file
├── architecture/     — structure.md (layout), decisions.md (why)
├── development/      — commit_guidelines, versioning, documentation, ownership, dioxus cheatsheet
├── operations/       — ios/ (first_device, submission runbook, build history)
├── plans/            — specs for in-flight work (watch.md is the far horizon)
└── progress/         — overview.md (live), todo.md (next), backlog.md (someday), changelog.md
```
