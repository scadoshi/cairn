//! The home-screen icon: which theme's colors it is drawn in, dark or light.
//!
//! iOS keeps the choice itself, so nothing is stored here: every read asks
//! the OS for its alternate icon name. Every other platform has one fixed
//! icon, and the pick is held for the session so the sheet behaves the same
//! there. Whether the icon follows the theme is a preference, stored with
//! the rest.
//!
//! `scripts/icon.py` renders an icon per theme and mode into
//! `assets/app-icons/`, and `scripts/ios/icons.sh` bundles each as an
//! alternate icon named `AppIcon-<theme>-<mode>`.

use orangui::ThemeConfig;

/// The theme the bundled primary icon is drawn in, in dark mode.
pub const PRIMARY: &str = "gruvbox";

/// The prefix `scripts/ios/icons.sh` gives each alternate icon.
const PREFIX: &str = "AppIcon-";

/// The bundled primary icon.
pub fn primary() -> ThemeConfig {
    ThemeConfig {
        name: PRIMARY.to_string(),
        is_dark: true,
    }
}

/// The theme and mode the icon is drawn in now. A retired theme's icon reads
/// as its replacement's, and a name this build cannot read as the primary.
pub fn current() -> ThemeConfig {
    imp::current()
        .and_then(|name| parse(&name))
        .and_then(ThemeConfig::resolve)
        .unwrap_or_else(primary)
}

/// Moves an icon this build no longer bundles (a retired theme's) onto the one
/// [`current`] reads it as. A no-op once they match; on iOS the OS shows its
/// own alert when it switches.
pub fn replace_retired() -> Result<(), String> {
    match imp::current().and_then(|name| parse(&name)) {
        Some(shown) if shown != current() => set(&current()),
        _ => Ok(()),
    }
}

/// Switches the icon to `icon`'s colors. On iOS the OS shows its own alert
/// saying the icon changed.
pub fn set(icon: &ThemeConfig) -> Result<(), String> {
    imp::set(alternate_name(icon).as_deref())
}

/// Switches the icon to `icon`'s colors unless it already shows them, so the
/// OS alert comes only with a change.
pub fn show(icon: &ThemeConfig) -> Result<(), String> {
    if current() == *icon {
        Ok(())
    } else {
        set(icon)
    }
}

/// "dark" or "light", as the icon files and bundle names spell it.
pub fn mode(is_dark: bool) -> &'static str {
    if is_dark { "dark" } else { "light" }
}

/// The alternate icon's bundle name, or `None` for the primary icon.
fn alternate_name(icon: &ThemeConfig) -> Option<String> {
    (*icon != primary()).then(|| format!("{PREFIX}{}-{}", icon.name, mode(icon.is_dark)))
}

/// The icon an alternate's bundle name stands for.
fn parse(name: &str) -> Option<ThemeConfig> {
    let (theme, mode) = name.strip_prefix(PREFIX)?.rsplit_once('-')?;
    let is_dark = match mode {
        "dark" => true,
        "light" => false,
        _ => return None,
    };
    Some(ThemeConfig {
        name: theme.to_string(),
        is_dark,
    })
}

#[cfg(target_os = "ios")]
mod imp {
    use block2::Block;
    use objc2::msg_send;
    use objc2::rc::Retained;
    use objc2::runtime::{AnyClass, AnyObject};
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
        // False only when Info.plist lists no alternates, which is a build
        // that skipped scripts/ios/icons.sh.
        // SAFETY: `supportsAlternateIcons` is a BOOL getter.
        let supported: bool = unsafe { msg_send![app, supportsAlternateIcons] };
        if !supported {
            return Err("No alternate icons in this build".to_string());
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

    pub(super) fn current() -> Option<String> {
        PICKED.lock().ok()?.clone()
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "the same signature as the iOS set, which can fail"
    )]
    pub(super) fn set(name: Option<&str>) -> Result<(), String> {
        if let Ok(mut picked) = PICKED.lock() {
            *picked = name.map(str::to_string);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icon(name: &str, is_dark: bool) -> ThemeConfig {
        ThemeConfig {
            name: name.to_string(),
            is_dark,
        }
    }

    #[test]
    fn only_the_primary_icon_has_no_alternate_name() {
        assert_eq!(alternate_name(&primary()), None);
        assert_eq!(
            alternate_name(&icon("vantablack", true)).as_deref(),
            Some("AppIcon-vantablack-dark")
        );
        assert_eq!(
            alternate_name(&icon(PRIMARY, false)).as_deref(),
            Some("AppIcon-gruvbox-light")
        );
    }

    #[test]
    fn a_bundle_name_parses_back_keeping_the_themes_hyphens() {
        assert!(parse("AppIcon-tokyo-night-light") == Some(icon("tokyo-night", false)));
        assert!(parse("AppIcon-nord-dark") == Some(icon("nord", true)));
        assert!(parse("AppIcon-nord").is_none());
    }

    #[test]
    fn every_other_theme_and_mode_has_a_rendered_icon() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/app-icons");
        for theme in orangui::ALLOWED_THEMES {
            for is_dark in [true, false] {
                let png = dir.join(format!("{theme}-{}.png", mode(is_dark)));
                if icon(theme, is_dark) == primary() {
                    assert!(
                        !png.exists(),
                        "the primary icon is assets/favicon, not an alternate"
                    );
                } else {
                    assert!(
                        png.exists(),
                        "{theme} {} has no icon; run scripts/icon.py",
                        mode(is_dark)
                    );
                }
            }
        }
    }

    // One test, since the pick is process-wide.
    #[test]
    fn a_pick_reads_back_as_its_theme_and_mode() {
        assert!(set(&icon("nord", false)).is_ok());
        assert!(current() == icon("nord", false));
        assert!(show(&icon("nord", false)).is_ok());
        assert!(show(&icon("dracula", true)).is_ok());
        assert!(current() == icon("dracula", true));
        assert!(set(&primary()).is_ok());
        assert!(current() == primary());

        assert!(imp::set(Some("AppIcon-visual-studio-light")).is_ok());
        assert!(current() == icon("vscode", false));
        assert!(replace_retired().is_ok());
        assert_eq!(imp::current().as_deref(), Some("AppIcon-vscode-light"));
        assert!(imp::set(Some("AppIcon-no-such-theme-dark")).is_ok());
        assert!(current() == primary());
        assert!(set(&primary()).is_ok());
    }
}
