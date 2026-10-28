# Context — Start Here

Orientation for AI assistants and returning contributors. This `context/` tree is the project's living documentation; each subdirectory owns one concern. The outline mirrors zwipe's so navigation carries over between repos.

Read [`CLAUDE.md`](CLAUDE.md) first for the rules.

## Directory map

| Directory | What's in it |
|-----------|--------------|
| [`architecture/`](architecture/) | `structure.md` (the tree and who owns what), `decisions.md` (the numbered why) |
| [`development/`](development/) | How to write code here: ownership (read first), commit and doc standards, the Dioxus 0.7 cheatsheet |
| [`operations/`](operations/) | How to ship: `ios/app-store/submission/` (build, publish, form fields, history) and `store-submissions/` (What's New per version) |
| [`plans/`](plans/) | Specs for upcoming work. `watch.md` is the Apple Watch horizon |
| [`progress/`](progress/) | `overview.md` (live), `todo.md` (next), `backlog.md` (someday), `changelog.md` |

## Current focus

**2026-09-23: the app is Cairn.** A cairn is the waymarker you build one stone at a time, walking the same path and adding one every time you pass. The same small act, repeated, and the pile is the record.

The crate, the repo and the home-screen header are Cairn. The mark is the C in `assets/c.txt`, with the owner's S behind the Mark setting in Config. `architecture/decisions.md` entry 12 covers the reasoning and the names that lost.

Two things deliberately did not change, because both are permanent once real data exists: the bundle id `com.scadoshi.count`, which App Store Connect never lets you edit, and the on-device data folder `scadoshi-count/count.db`, which holds nine months of real counts.

It is installed on scotland-mobile and in daily use, signed with a development profile good until September 2027. The daily loop is `scripts/ios/deploy.sh`: back up the phone's database, build, install. A reinstall keeps the data container, so deploying never touches the counts, and the backup runs first regardless because the phone holds taps that exist nowhere else. `operations/ios/dev_deploy.md` has the whole thing.

There is a write-up at [scottyfermo.com/side-quests/cairn](https://scottyfermo.com/side-quests/cairn) with a demo and screenshots.

**2026-10-18: 1.0.0 submitted to the App Store** as Cairn Counter ("Cairn" was taken), iOS build 1, straight to review with no TestFlight round. The same day the repo got zwipe's submission layout under `operations/ios/app-store/submission/` (build, publish, form fields, history), `operations/store-submissions/` for the per-version What's New, a Version row in Config, a privacy policy page on the portfolio, and `scripts/ios/spoof_db.py` plus `scripts/ios/screenshots.sh`, which fill a simulator with invented counters and tap through the app for the store screenshots, so no real numbers go on the listing. Build 1 is the first with the App icon picker and the light-mode icons in each theme's accent color.

**2026-10-20: 1.0.0 live on the App Store.** 1.1.0 took everything since and went to review the same night as iOS build 7, which replaces build 6 (uploaded, never submitted). It was signed on scotland-halo, which needed the Count App Store profile downloaded again to `~/certs`. The runbook is `operations/ios/app-store/submission/overview.md`.

**2026-10-27: 1.1.2 uploaded** through Transporter as iOS build 9, built on scotland-halo. It brings a quote that types out in full when it turns while the app stays open, the heatmap's totals as tags under the grid, and the lifetime number in a box; 1.1.1 (build 8, Flexoki and Synthwave '84) went to review on 2026-10-21. Notes are at [`operations/store-submissions/1.1.2/whats_new.md`](operations/store-submissions/1.1.2/whats_new.md), and portfolio's changelog keeps 1.1.2 under UPCOMING until it goes live, when it moves to RELEASES with that day; move this repo's changelog heading to the same day if it differs from 2026-10-27.

**Next:** submit 1.1.2 for review once the build finishes processing, and draw the Cairn mark properly.

Two features are scoped but not started: [`plans/celebrations.md`](plans/celebrations.md) and [`plans/big_step.md`](plans/big_step.md).

See [`progress/todo.md`](progress/todo.md) for the ordered list.
