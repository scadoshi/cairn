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

**2026-09-21: the app is Count, the repo stays Crow.** Crow said nothing to a stranger reading a store listing, so everything that reaches a device now says count: the bundle id `com.scadoshi.count`, the home-screen label Count, the data folder `scadoshi-count/count.db`, the CSV names. The repo, crate and internal event names keep the bird. `architecture/decisions.md` entry 11 has the reasoning.

The phone route is one command, `scripts/ios/install_device.sh --db ~/Developer/crow-data/count.db`: build, icon catalog, signing, install, then the existing database pushed into the app container and read back to check it landed. It stops with a clear message until the portal paperwork exists.

**Next:** one profile stands between the repo and the phone. The device is already registered from zwipe and the certificate runs to 2027, so it is just an App ID plus an iOS App Development profile for `com.scadoshi.count`. Then `dx build --platform ios --device true && ios-deploy --bundle ...`, spelled out in [`operations/ios/dev_deploy.md`](operations/ios/dev_deploy.md).

For the store-signed build, in the developer portal register the App ID `com.scadoshi.count`, the phone's UDID, and the two profiles, saved as `~/certs/Count_Development.mobileprovision` and `Count_App_Store`. Then run the install script. Every step is in [`operations/ios/first_device.md`](operations/ios/first_device.md). Then TestFlight and review via [`operations/ios/submission.md`](operations/ios/submission.md).

See [`progress/todo.md`](progress/todo.md) for the ordered list.
