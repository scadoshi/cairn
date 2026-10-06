# Why tao is vendored

This is tao 0.34.8 as published on crates.io, the version `dioxus-desktop` already resolves to, with one edit: the iOS event loop keeps running while a scroll is moving. Nothing else differs from upstream.

## The iOS event loop runs mid-scroll

`src/platform_impl/ios/event_loop.rs` registers its three run-loop observers (`begin_observer`, `main_end_observer`, `end_observer`) in `kCFRunLoopCommonModes`, where upstream uses `kCFRunLoopDefaultMode`, and its begin handler ignores `kCFRunLoopEntry` instead of hitting `unimplemented!()`. Those observers are what hand the app its queued events: the proxy's run-loop source only wakes the loop. While a finger drags or a list glides, UIKit runs the main loop in `UITrackingRunLoopMode`, so upstream's observers never fire and every queued event waits for the scroll to end. On Dioxus that held everything Rust sends the page (DOM edits, `eval`), so a tap mid-scroll did its work only once momentum stopped. Common modes include the tracking mode, so the loop now drains during a scroll; entering that mode is what fires `kCFRunLoopEntry`. tao's own macOS backend already registers the same observers in common modes.

The same edit ships in zwipe (`zwipe/vendor/tao/VENDOR.md`), where it was measured on an iPhone 16 (iOS 26.6), 2026-10-06: a mid-glide tap's DOM update went from landing about 1.2s late, when the scroll stopped, to landing while the list was still moving. Page-side work such as `assets/entrance.js` never needed it, because it runs in the page.

## When to delete this

When the tao that `dioxus-desktop` resolves to registers the iOS observers in common modes (upstream tao 0.37.0 and winit 0.30 both still use the default mode). Until then, a `dioxus-desktop` bump that moves tao means vendoring the new version with the same edit. `cargo tree -i tao` shows which tao is actually in the graph; a patch that does not apply is a warning, not an error, so check rather than assume.
