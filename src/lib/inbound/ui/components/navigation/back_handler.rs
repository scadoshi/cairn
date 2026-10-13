//! The OS back intent (iOS edge swipe, Android gesture and button), bridged
//! into the router through orangui's `use_os_back`, which closes any open
//! overlay first.
//!
//! On Android the generated `MainActivity` is patched post-bundle
//! (`scripts/android/back_handler.sh`) to dispatch a `cairn:back` DOM event
//! for the OS back intent; this layout names that event. On iOS it hands over
//! the WKWebView the edge swipe attaches to. The desktop build is untouched.

use std::{ffi::c_void, ptr::NonNull};

use crate::inbound::ui::router::Route;
use dioxus::prelude::*;

/// Layout wrapping all routes: installs the OS-back bridge, then renders the
/// active route.
#[component]
pub fn BackHandlerLayout() -> Element {
    let nav = use_navigator();
    let go_back = use_callback(move |()| {
        let can = nav.can_go_back();
        if can {
            nav.go_back();
        }
        can
    });
    orangui::use_os_back("cairn:back", webview, go_back);

    rsx! {
        Outlet::<Route> {}
    }
}

/// The app's WKWebView, for the edge-swipe recognizer.
#[cfg(all(target_os = "ios", feature = "mobile"))]
fn webview() -> Option<NonNull<c_void>> {
    use dioxus::mobile::wry::WebViewExtIOS;
    let webview = dioxus::mobile::window().webview.webview();
    NonNull::new((&raw const *webview).cast_mut().cast())
}

/// No WKWebView off iOS.
#[cfg(not(all(target_os = "ios", feature = "mobile")))]
fn webview() -> Option<NonNull<c_void>> {
    None
}
