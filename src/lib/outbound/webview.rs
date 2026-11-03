//! The WKWebView's layout on iOS.
//!
//! UIKit adds the safe-area insets to the webview's scroll view as content
//! insets, so WebKit lays the page out in the window less the status bar
//! strip and the home indicator strip together, and the webview's own
//! background shows in the band left under the page. [`fill_window`] turns
//! that adjustment off: the page then covers the whole window, as its
//! `viewport-fit=cover` asks, and `env(safe-area-inset-*)` still reports the
//! insets for the bands that want them.

#[cfg(target_os = "ios")]
mod imp {
    use std::ffi::c_void;
    use std::ptr::NonNull;

    use objc2::msg_send;
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;

    /// `UIScrollViewContentInsetAdjustmentNever`, from UIKit's
    /// `UIScrollViewContentInsetAdjustmentBehavior`.
    const ADJUSTMENT_NEVER: isize = 2;

    pub(super) fn fill_window(webview: NonNull<c_void>) {
        // SAFETY: the pointer is the app's live WKWebView, which outlives
        // every call made from the page it hosts.
        let webview: &AnyObject = unsafe { webview.cast::<AnyObject>().as_ref() };
        // SAFETY: `scrollView` is a WKWebView getter returning its UIScrollView.
        let scroll: Retained<AnyObject> = unsafe { msg_send![webview, scrollView] };
        // SAFETY: the setter takes the enum as an NSInteger, and the two
        // layout calls take nothing. Called from the UI thread, which is
        // the main thread on iOS.
        unsafe {
            let () = msg_send![&*scroll, setContentInsetAdjustmentBehavior: ADJUSTMENT_NEVER];
            // WebKit reads the insets back on its next layout pass.
            let () = msg_send![webview, setNeedsLayout];
            let () = msg_send![webview, layoutIfNeeded];
        }
    }
}

/// Lays the page out over the whole window instead of the safe area.
#[cfg(target_os = "ios")]
pub fn fill_window(webview: std::ptr::NonNull<std::ffi::c_void>) {
    imp::fill_window(webview);
}

/// No WKWebView off iOS.
#[cfg(not(target_os = "ios"))]
pub fn fill_window(_webview: std::ptr::NonNull<std::ffi::c_void>) {}
