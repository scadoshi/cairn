# Context — Start Here

Orientation for AI assistants and returning contributors. This `context/` tree is the project's living documentation; each subdirectory owns one concern. The outline mirrors zwipe's so navigation carries over between repos.

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

**2026-09-22: it is on the phone and in daily use.** Count is installed on scotland-mobile with the imported counts, signed with a development profile good until September 2027. The daily loop is `scripts/ios/deploy.sh`: back up the phone's database, build, install. A reinstall keeps the data container, so deploying does not touch the counts, and the backup runs first anyway because the phone holds taps that exist nowhere else. `operations/ios/dev_deploy.md` has the whole thing.

The app on a device says Count; the repo stays crow. Everything that reaches a phone uses the new name: bundle id `com.scadoshi.count`, home-screen label Count, data folder `scadoshi-count/count.db`, CSV names. The repo, crate and internal event names keep the bird. `architecture/decisions.md` entry 11 has the reasoning.

**Next:** the wordmark and icon artwork, then TestFlight and review via [`operations/ios/submission.md`](operations/ios/submission.md). Note that the database push does not work on a TestFlight build, which is signed without `get-task-allow`; moving data there needs the in-app import on the backlog.

See [`progress/todo.md`](progress/todo.md) for the ordered list.
