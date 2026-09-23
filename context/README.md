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

**2026-09-23: the app is Cairn.** A cairn is the pile of stones where everyone who passes adds one, which is what a lifetime counter is. The crate, the repo and the home-screen header are Cairn. The mark is the C in `assets/c.txt`, with the owner's S behind the Mark setting in Config. `architecture/decisions.md` entry 12 covers the reasoning and the names that lost.

Two things deliberately did not change, because both are permanent once real data exists: the bundle id `com.scadoshi.count`, which App Store Connect never lets you edit, and the on-device data folder `scadoshi-count/count.db`, which holds nine months of real counts.

It is installed on scotland-mobile and in daily use, signed with a development profile good until September 2027. The daily loop is `scripts/ios/deploy.sh`: back up the phone's database, build, install. A reinstall keeps the data container, so deploying never touches the counts, and the backup runs first regardless because the phone holds taps that exist nowhere else. `operations/ios/dev_deploy.md` has the whole thing.

There is a write-up at [scottyfermo.com/side-quests/cairn](https://scottyfermo.com/side-quests/cairn) with a demo and screenshots.

**Next:** draw the Cairn mark properly, then TestFlight and review via [`operations/ios/submission.md`](operations/ios/submission.md). The database push does not work on a TestFlight build, which is signed without `get-task-allow`; moving data there needs the in-app import on the backlog.

See [`progress/todo.md`](progress/todo.md) for the ordered list.
