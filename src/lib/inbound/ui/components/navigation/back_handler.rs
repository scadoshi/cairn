//! Edge back-swipe / hardware-back navigation.
//!
//! The router runs on an in-memory history stack, so the OS back intent (iOS
//! left-edge swipe, Android hardware or gesture back) does not reach it on its
//! own. This layout wraps every route and routes that intent into the router's
//! `go_back()`, closing any open overlay first.
//!
//! **iOS.** A `UIScreenEdgePanGestureRecognizer` on the WKWebView drives
//! `go_back()` directly over a channel, on gesture start, so it feels as
//! immediate as tapping Back. (WKWebView's own back-forward gesture fed by a
//! `history.pushState` trap works but takes seconds to settle.)
//!
//! **Android.** The generated `MainActivity` is patched post-bundle
//! (`scripts/android/back_handler.sh`) to dispatch an `crow:back` DOM event
//! for the OS back intent; this layout listens for it, and finishes the
//! Activity from a root screen.
//!
//! All native-gated, so the desktop build is untouched. Ported from zwiper.

use crate::inbound::ui::router::Route;
use dioxus::prelude::*;

/// Layout wrapping all routes: installs the OS-back bridge, then renders the
/// active route.
#[component]
pub fn BackHandlerLayout() -> Element {
    #[cfg(all(target_os = "ios", feature = "mobile"))]
    {
        let nav = use_navigator();
        let mut overlays: super::overlay_stack::OverlayBackStack = use_context();
        use_effect(move || {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
            spawn(async move {
                while rx.recv().await.is_some() {
                    if !overlays.close_top() && nav.can_go_back() {
                        nav.go_back();
                    }
                }
            });
            ios::install_edge_back(tx);
        });
    }

    #[cfg(all(target_os = "android", feature = "mobile"))]
    {
        let nav = use_navigator();
        let mut overlays: super::overlay_stack::OverlayBackStack = use_context();
        use_future(move || async move {
            let mut eval =
                document::eval("window.addEventListener('crow:back', () => dioxus.send(1));");
            while eval.recv::<i32>().await.is_ok() {
                if !overlays.close_top() {
                    if nav.can_go_back() {
                        nav.go_back();
                    } else {
                        android::finish_activity();
                    }
                }
            }
        });
    }

    rsx! {
        Outlet::<Route> {}
    }
}

/// iOS left-edge back gesture: a `UIScreenEdgePanGestureRecognizer` on the
/// WKWebView whose target forwards `go_back` requests over a channel.
#[cfg(all(target_os = "ios", feature = "mobile"))]
mod ios {
    use dioxus::mobile::wry::WebViewExtIOS;
    use objc2::{
        AnyThread, DefinedClass, MainThreadMarker, class, define_class, msg_send,
        rc::{Allocated, Retained},
        runtime::{AnyObject, NSObject},
        sel,
    };
    use tokio::sync::mpsc::UnboundedSender;

    /// `UIGestureRecognizerStateBegan`: fire once as the pan starts.
    const STATE_BEGAN: isize = 1;
    /// `UIRectEdgeLeft` (1 << 1): recognize only pans from the left edge.
    const EDGE_LEFT: usize = 2;

    struct Ivars {
        tx: UnboundedSender<()>,
    }

    define_class!(
        // SAFETY: NSObject has no subclassing requirements; no Drop impl.
        #[unsafe(super(NSObject))]
        #[name = "OdoEdgeBackTarget"]
        #[ivars = Ivars]
        struct EdgeBackTarget;

        impl EdgeBackTarget {
            #[unsafe(method(handleEdgePan:))]
            fn handle_edge_pan(&self, sender: &AnyObject) {
                // SAFETY: `state` is a valid selector on UIGestureRecognizer.
                let state: isize = unsafe { msg_send![sender, state] };
                if state == STATE_BEGAN {
                    let _ = self.ivars().tx.send(());
                }
            }
        }
    );

    impl EdgeBackTarget {
        fn new(tx: UnboundedSender<()>) -> Retained<Self> {
            let this = Self::alloc().set_ivars(Ivars { tx });
            // SAFETY: designated initializer for our NSObject subclass.
            unsafe { msg_send![super(this), init] }
        }
    }

    /// Attach the edge-back recognizer to the app's WKWebView. Must run on
    /// the main thread (UI effects do).
    pub fn install_edge_back(tx: UnboundedSender<()>) {
        if MainThreadMarker::new().is_none() {
            return;
        }
        let webview = dioxus::mobile::window().webview.webview();
        let target = EdgeBackTarget::new(tx);
        // SAFETY: standard UIKit gesture setup with correct selectors and
        // types, on the main thread.
        unsafe {
            let alloc: Allocated<AnyObject> =
                msg_send![class!(UIScreenEdgePanGestureRecognizer), alloc];
            let recognizer: Retained<AnyObject> =
                msg_send![alloc, initWithTarget: &*target, action: sel!(handleEdgePan:)];
            let _: () = msg_send![&*recognizer, setEdges: EDGE_LEFT];
            let _: () = msg_send![&*webview, addGestureRecognizer: &*recognizer];
        }
        // A recognizer keeps only a weak reference to its target; leak ours
        // so it outlives this call (one target per app launch).
        std::mem::forget(target);
    }
}

/// Android app exit: finish the Activity when back is pressed at a root
/// screen. JNI through ndk-context.
#[cfg(all(target_os = "android", feature = "mobile"))]
mod android {
    use jni::objects::JObject;

    /// Finish the current Activity (exit the app); failures are ignored, the
    /// worst case being that back does nothing at the root.
    pub fn finish_activity() {
        let _ = try_finish();
    }

    fn try_finish() -> Result<(), Box<dyn std::error::Error>> {
        let ctx = ndk_context::android_context();
        // SAFETY: ndk-context guarantees a valid JavaVM and Activity jobject.
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;
        // SAFETY: as above, the context pointer is the Activity.
        let activity = unsafe { JObject::from_raw(ctx.context().cast()) };
        env.call_method(&activity, "finish", "()V", &[])?;
        Ok(())
    }
}
