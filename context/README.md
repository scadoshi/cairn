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

**2026-09-20: named Crow, real data loaded, ready for a phone.** The app is
feature-complete for daily use: counters with per-day entries and tap
events, goals per day, week, or year with pace, trend charts, tiles, a
config screen with hints, OS back gesture, toasts, CSV export, and the
owner's nine months of real pull-up, push-up, and squat taps imported. The
release device build compiles and stops only at code signing.

**Next, 2026-09-21:** register the App ID `com.scadoshi.crow`, the phone,
and the two profiles, then install straight onto the phone. Every step is
in [`operations/ios/first_device.md`](operations/ios/first_device.md).
Push the 46 local commits first. Then TestFlight and review via
[`operations/ios/submission.md`](operations/ios/submission.md).

See [`progress/todo.md`](progress/todo.md) for the ordered list.
