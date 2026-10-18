//! The home-screen icon: which theme's colors it is drawn in.
//!
//! iOS keeps the choice itself, so nothing is stored here: every read asks
//! the OS for its alternate icon name. Every other platform has one fixed
//! icon, and the pick is held for the session so the sheet behaves the same
//! there.
//!
//! `scripts/icon.py` renders an icon per theme into `assets/app-icons/`, and
//! `scripts/ios/icons.sh` bundles each as an alternate icon named
//! `AppIcon-<theme>`.

/// The theme the bundled primary icon is drawn in.
pub const PRIMARY: &str = "gruvbox";

/// The prefix `scripts/ios/icons.sh` gives each alternate icon.
const PREFIX: &str = "AppIcon-";

/// The theme the icon is drawn in now.
pub fn current() -> String {
    imp::current()
        .and_then(|name| name.strip_prefix(PREFIX).map(str::to_string))
        .unwrap_or_else(|| PRIMARY.to_string())
}

/// Switches the icon to `theme`'s. On iOS the OS shows its own alert saying
/// the icon changed.
pub fn set(theme: &str) -> Result<(), String> {
    imp::set(alternate_name(theme).as_deref())
}

/// The alternate icon's bundle name, or `None` for the primary icon.
fn alternate_name(theme: &str) -> Option<String> {
    (theme != PRIMARY).then(|| format!("{PREFIX}{theme}"))
}

#[cfg(target_os = "ios")]
mod imp {
    use block2::Block;
    use objc2::{
        msg_send,
        rc::Retained,
        runtime::{AnyClass, AnyObject},
    };
    use objc2_foundation::NSString;

    fn app() -> Option<&'static AnyObject> {
        let class = AnyClass::get(c"UIApplication")?;
        // SAFETY: `sharedApplication` takes no arguments and returns the
        // process's UIApplication, which lives as long as the process.
        let app: *mut AnyObject = unsafe { msg_send![class, sharedApplication] };
        // SAFETY: the pointer is either null or the live shared application.
        unsafe { app.as_ref() }
    }

    pub fn current() -> Option<String> {
        let app = app()?;
        // SAFETY: `alternateIconName` is a nullable NSString getter.
        let name: Option<Retained<NSString>> = unsafe { msg_send![app, alternateIconName] };
        name.map(|n| n.to_string())
    }

    pub fn set(name: Option<&str>) -> Result<(), String> {
        let app = app().ok_or("no UIApplication")?;
        // SAFETY: `supportsAlternateIcons` is a BOOL getter.
        let supported: bool = unsafe { msg_send![app, supportsAlternateIcons] };
        if !supported {
            return Err("this device can't change its app icon".to_string());
        }
        let name = name.map(NSString::from_str);
        let name: Option<&NSString> = name.as_deref();
        let handler: Option<&Block<dyn Fn(*mut AnyObject)>> = None;
        // SAFETY: both arguments are nullable: nil restores the primary icon,
        // and a nil completion handler is allowed. Called from the UI thread,
        // which is the main thread on iOS.
        let () = unsafe { msg_send![app, setAlternateIconName: name, completionHandler: handler] };
        Ok(())
    }
}

#[cfg(not(target_os = "ios"))]
mod imp {
    use std::sync::Mutex;

    static PICKED: Mutex<Option<String>> = Mutex::new(None);

    pub fn current() -> Option<String> {
        PICKED.lock().ok()?.clone()
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn set(name: Option<&str>) -> Result<(), String> {
        if let Ok(mut picked) = PICKED.lock() {
            *picked = name.map(str::to_string);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_primary_theme_restores_the_bundled_icon() {
        assert_eq!(alternate_name(PRIMARY), None);
        assert_eq!(
            alternate_name("vantablack").as_deref(),
            Some("AppIcon-vantablack")
        );
    }

    #[test]
    fn every_other_theme_has_a_rendered_icon() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/app-icons");
        for theme in orangui::ALLOWED_THEMES {
            let png = dir.join(format!("{theme}.png"));
            if *theme == PRIMARY {
                assert!(
                    !png.exists(),
                    "the primary icon is assets/favicon, not an alternate"
                );
            } else {
                assert!(png.exists(), "{theme} has no icon; run scripts/icon.py");
            }
        }
    }

    #[test]
    fn a_pick_reads_back_as_its_theme() {
        set("nord").unwrap();
        assert_eq!(current(), "nord");
        set(PRIMARY).unwrap();
        assert_eq!(current(), PRIMARY);
    }
}
