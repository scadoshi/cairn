# Apple Watch

Far horizon; nothing here is started.

The point of keeping `domain/` free of Dioxus and SQLite is that the watch
build reuses it verbatim. Open questions to answer when the time comes:

- UI toolkit. Dioxus does not target watchOS. Likely SwiftUI over a Rust
  domain via `uniffi` or a C ABI, with the domain crate built as a static lib.
- Storage. SQLite works on watchOS; the alternative is syncing entries from
  the phone via WatchConnectivity and keeping the watch read-mostly.
- Scope. A watch complication showing today's count and pace, and a +1 button.
  Stats screens stay on the phone.
