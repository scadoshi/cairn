# Apple Watch

Parked 2026-10-25. Nothing is built. This is what a first look at the difficulty found, so the next pass starts from here instead of from scratch.

## Scope

A complication showing today's count and pace, and a +1 button. Stats screens, charts, the heatmap and settings stay on the phone.

## Decided

- Storage and sync: CloudKit's private database syncs events between phone and watch (`architecture/decisions.md` §8). The watch keeps its own SQLite and applies the synced events; no WatchConnectivity relay.
- UI toolkit: SwiftUI. Dioxus does not target watchOS.

## The visual translation is the small part

The watch does not port orangui, only its tokens. With the scope above, almost none of the kit applies: no bottom sheets, theme picker, hint dialogs or charts.

- Palette: orangui has 100 themes at about 37 variables each, but the watch needs around six (background, primary and muted text, accent, success and warning for pace). Generate a Swift `Theme` file from `orangui/assets/themes.css` with a script rather than translating by hand, so the CSS stays the one source of truth and an orangui change reaches the watch by rerunning it.
- Type: bundle JetBrains Mono (already in `assets/fonts`) for the big count so it keeps the odometer feel; system font for small text, which reads better at watch sizes.
- Mark: `assets/c.txt` renders in monospace, or export it once as a vector asset.
- Motion: the entrances, theme wipe and celebrations are CSS and JS and do not carry over. Rebuild one moment natively, the +1 tick, and leave the rest.

## The hard parts, hardest first

1. Packaging. A watch app ships inside the iOS app's bundle as an Xcode target, and `dx` builds the iOS bundle itself. There is no known supported way to embed a watch target in a `dx` bundle. Options are post-processing the `.app` before signing, or moving iOS packaging into an Xcode project that wraps the Dioxus binary. This is the biggest unknown.
2. Rust on watchOS. `aarch64-apple-watchos` and `arm64_32-apple-watchos` (needed for older watches) were tier 3 when last checked: nightly, `-Zbuild-std`, no prebuilt std. It works but costs build setup forever. The alternative is porting only what the watch needs to Swift: today's count, `stats::pace`, `remaining_today` and the goal crossing, roughly 200 of the domain's ~3k lines. Shared JSON fixtures that both the Rust and Swift test suites run keep the two from drifting. Leaning Swift port for this scope; measure the Rust route in the spike before deciding.
3. CloudKit on the phone. The watch side is easy since Swift calls CloudKit directly. The phone side is Dioxus, so it needs an objc2 bridge to CloudKit's callback-based async API, which is much heavier than `os_back.rs`. The phone must also start writing events to iCloud before the watch has anything to read.
4. Complication. WidgetKit refreshes on a timeline budget, so it shows the last known count, not a live one.

## Order when this resumes

1. Packaging spike: a hello-world watch target inside a `dx`-built iOS app, installed on a real watch.
2. Settle Rust or Swift for the domain: try building `domain/` for watchOS; if it fights back, port the small piece to Swift with shared fixtures.
3. CloudKit event sync on the phone, shipped in the existing app before the watch exists.
4. Watch UI with the generated theme and the +1 button.
5. Complication.
