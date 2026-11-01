//! Opens a link inside the app where the platform allows it.
//!
//! On iOS, `http` and `https` URLs open in an `SFSafariViewController` sheet
//! over Cairn, with a Done button that returns to the same screen. Any other
//! scheme, or a sheet that can't be presented, goes to the OS with
//! `UIApplication.openURL`. The desktop build hands every URL to the default
//! browser.

/// Opens `url` in the in-app browser, or with the OS when it can't.
///
/// # Errors
///
/// A message saying why the URL couldn't be opened at all.
pub fn open(url: &str) -> Result<(), String> {
    imp::open(url)
}

/// Whether `url` belongs in the in-app browser, which only loads web pages:
/// its scheme is `http` or `https`, in any case.
#[cfg_attr(
    not(any(target_os = "ios", test)),
    expect(dead_code, reason = "desktop opens every URL in the browser")
)]
fn opens_in_app(url: &str) -> bool {
    url.split_once(':').is_some_and(|(scheme, _)| {
        scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
    })
}

#[cfg(target_os = "ios")]
mod imp {
    use block2::Block;
    use objc2::rc::{Allocated, Retained};
    use objc2::runtime::{AnyClass, AnyObject, Bool};
    use objc2::{MainThreadMarker, msg_send};
    use objc2_foundation::{NSDictionary, NSString, NSURL};

    // Links SafariServices, which holds SFSafariViewController. The class is
    // looked up by name at runtime, so no symbols are declared.
    #[link(name = "SafariServices", kind = "framework")]
    unsafe extern "C" {}

    pub(super) fn open(url: &str) -> Result<(), String> {
        // UIKit runs on the main thread, where a tap handler already is.
        MainThreadMarker::new().ok_or("Links open only from the main thread")?;
        let app = app().ok_or("no UIApplication")?;
        let url_object = NSURL::URLWithString(&NSString::from_str(url))
            .ok_or_else(|| format!("Not a link: {url}"))?;

        if super::opens_in_app(url) && present_safari(app, &url_object).is_ok() {
            return Ok(());
        }
        open_external(app, &url_object);
        Ok(())
    }

    fn app() -> Option<&'static AnyObject> {
        let class = AnyClass::get(c"UIApplication")?;
        // SAFETY: `sharedApplication` takes no arguments and returns the
        // process's UIApplication, which lives as long as the process.
        let app: *mut AnyObject = unsafe { msg_send![class, sharedApplication] };
        // SAFETY: the pointer is either null or the live shared application.
        unsafe { app.as_ref() }
    }

    /// Presents an `SFSafariViewController` for `url` over the topmost view
    /// controller of the key window. `url` must be `http` or `https`; the
    /// controller raises an exception for any other scheme.
    fn present_safari(app: &AnyObject, url: &NSURL) -> Result<(), &'static str> {
        // SAFETY: `keyWindow` is a nullable UIWindow getter, read on the main
        // thread.
        let window: Option<Retained<AnyObject>> = unsafe { msg_send![app, keyWindow] };
        let window = window.ok_or("no key window")?;

        // SAFETY: `rootViewController` is a nullable UIViewController getter.
        let root: Option<Retained<AnyObject>> = unsafe { msg_send![&*window, rootViewController] };
        let mut presenter = root.ok_or("no root view controller")?;

        // A controller already presenting something can't present again, so
        // walk up to the one on top.
        loop {
            // SAFETY: `presentedViewController` is a nullable getter.
            let on_top: Option<Retained<AnyObject>> =
                unsafe { msg_send![&*presenter, presentedViewController] };
            match on_top {
                Some(next) => presenter = next,
                None => break,
            }
        }

        let class = AnyClass::get(c"SFSafariViewController").ok_or("no SFSafariViewController")?;
        // SAFETY: `alloc` on a class object returns an uninitialized instance.
        let allocated: Allocated<AnyObject> = unsafe { msg_send![class, alloc] };
        // SAFETY: `initWithURL:` takes an NSURL, and `opens_in_app` limited it
        // to the http(s) schemes the initializer accepts.
        let safari: Option<Retained<AnyObject>> = unsafe { msg_send![allocated, initWithURL: url] };
        let safari = safari.ok_or("SFSafariViewController init returned nil")?;

        let completion: Option<&Block<dyn Fn()>> = None;
        // SAFETY: a nil completion block is allowed, and this runs on the
        // main thread.
        unsafe {
            let () = msg_send![
                &*presenter,
                presentViewController: &*safari,
                animated: Bool::YES,
                completion: completion
            ];
        }
        Ok(())
    }

    /// Hands `url` to the OS with `UIApplication.openURL`.
    fn open_external(app: &AnyObject, url: &NSURL) {
        let options: Retained<NSDictionary<AnyObject, AnyObject>> = NSDictionary::new();
        let handler: Option<&Block<dyn Fn(Bool)>> = None;
        // SAFETY: an empty options dictionary and a nil completion handler
        // are both allowed.
        let () =
            unsafe { msg_send![app, openURL: url, options: &*options, completionHandler: handler] };
    }
}

#[cfg(not(target_os = "ios"))]
mod imp {
    pub(super) fn open(url: &str) -> Result<(), String> {
        webbrowser::open(url).map_err(|e| format!("Couldn't open {url}: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_schemes_open_in_app() {
        for url in [
            "https://scottyfermo.com/projects/cairn/changelog/",
            "http://example.com",
            "HTTPS://SCOTTYFERMO.COM",
        ] {
            assert!(opens_in_app(url), "input {url:?}");
        }
    }

    #[test]
    fn other_schemes_go_to_the_os() {
        for url in [
            "mailto:scottyfermo@hotmail.com",
            "tel:+15555550100",
            "httpx://example.com",
            "https",
            "",
            "scottyfermo.com/https:",
        ] {
            assert!(!opens_in_app(url), "input {url:?}");
        }
    }
}
