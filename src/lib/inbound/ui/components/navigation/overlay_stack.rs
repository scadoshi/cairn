//! Back-aware overlay stack.
//!
//! The OS back intent is bridged by [`BackHandlerLayout`](super::back_handler),
//! which normally routes it to the router's `go_back`. Overlays, anything shown
//! on top of the current screen without a route change (the bottom sheets, the
//! delete dialog), are not routes, so a raw `go_back` would blow past them and
//! leave the screen. This stack lets the back intent close the top-most open
//! overlay first, falling through to the router only when none are open.
//!
//! Each overlay registers a close action while it is open. Because closing
//! flips the overlay's open state, the registration effect then deregisters
//! it: the stack stays truthful with no manual bookkeeping.

use dioxus::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// Monotonic source of per-overlay-instance ids, stable across an instance's
/// re-renders and unique across instances.
static NEXT_OVERLAY_ID: AtomicU64 = AtomicU64::new(0);

fn next_overlay_id() -> u64 {
    NEXT_OVERLAY_ID.fetch_add(1, Ordering::Relaxed)
}

/// App-level LIFO of the currently open overlays, provided once above the
/// router. A `Copy` handle over a signal.
#[derive(Clone, Copy)]
pub struct OverlayBackStack {
    entries: Signal<Vec<(u64, Callback<()>)>>,
}

/// Creates the stack. Provide it via `use_context_provider` above the router
/// so both the overlays and the back handler can reach it.
pub fn use_overlay_back_stack() -> OverlayBackStack {
    OverlayBackStack {
        entries: use_signal(Vec::new),
    }
}

impl OverlayBackStack {
    fn push(&mut self, id: u64, close: Callback<()>) {
        let mut entries = self.entries.write();
        entries.retain(|(i, _)| *i != id);
        entries.push((id, close));
    }

    fn remove(&mut self, id: u64) {
        self.entries.write().retain(|(i, _)| *i != id);
    }

    /// Closes the top-most open overlay if there is one. Returns `true` when
    /// an overlay was closed, so the back handler skips the router.
    #[cfg_attr(
        not(all(any(target_os = "ios", target_os = "android"), feature = "mobile")),
        allow(dead_code)
    )]
    pub fn close_top(&mut self) -> bool {
        let popped = self.entries.write().pop();
        if let Some((_, close)) = popped {
            close.call(());
            true
        } else {
            false
        }
    }
}

/// Keeps `close` on the stack while `is_open` is true.
pub fn use_overlay_back_action(is_open: ReadSignal<bool>, close: Callback<()>) {
    let stack: OverlayBackStack = use_context();
    let id = use_hook(next_overlay_id);
    use_effect(move || {
        let mut stack = stack;
        if is_open() {
            stack.push(id, close);
        } else {
            stack.remove(id);
        }
    });
    use_drop(move || {
        let mut stack = stack;
        stack.remove(id);
    });
}

/// Registers a signal-toggled overlay so the back intent closes it by setting
/// `open` false.
pub fn use_overlay_back(open: Signal<bool>) {
    let close = use_callback(move |()| {
        let mut open = open;
        open.set(false);
    });
    use_overlay_back_action(open.into(), close);
}
