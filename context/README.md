# Context — Start Here

Orientation for AI assistants and returning contributors. This `context/` tree
is the project's living documentation; each subdirectory owns one concern. The
outline mirrors zwipe's so navigation carries over between repos.

Read [`CLAUDE.md`](CLAUDE.md) first for the rules.

## Directory map

| Directory | What's in it |
|-----------|--------------|
| [`architecture/`](architecture/) | `structure.md` (the tree and who owns what), `decisions.md` (the numbered why) |
| [`development/`](development/) | How to write code here: ownership (read first), commit and doc standards, the Dioxus 0.7 cheatsheet |
| [`operations/`](operations/) | How to ship: `ios/submission.md` and the build history |
| [`plans/`](plans/) | Specs for upcoming work. `watch.md` is the Apple Watch horizon |
| [`progress/`](progress/) | `overview.md` (live), `todo.md` (next), `backlog.md` (someday), `changelog.md` |

## Current focus

**2026-09-19: scaffold.** The crate builds, the domain (counters, daily
entries, stats, yearly goals, CSV) has tests, SQLite persists everything
including the picked theme, and the desktop UI has a home screen and a counter
screen using the shared zwipe components. Nothing is polished yet.

**Next:** run it daily, feel out the counter screen, then the iOS build. See
[`progress/todo.md`](progress/todo.md).
