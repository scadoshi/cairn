//! The theme sheet and the wiring every host shares: Settings' theme row
//! and the menu's footer both raise it. A kept theme carries the home-screen
//! icon along when the icon follows the theme, and a flipped mode saves at
//! once through the app's theme signal.

use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use orangui::{ThemeConfig, ThemeSheet};

use crate::domain::preferences::{AppIcon, Preferences};
use crate::inbound::ui::use_prefs;
use crate::outbound::app_icon;

/// Whether the app-wide theme sheet is up. Provided at the root so the menu,
/// which folds as it raises the sheet, can reach it.
#[derive(Debug, Clone, Copy)]
pub struct ThemeSheetOpen(pub Signal<bool>);

/// Provides [`ThemeSheetOpen`] to the tree below and returns its signal, the
/// one the root hands [`AppThemeSheet`]. The signal is made once, so the
/// sheet and every footer that reaches for the context share it across the
/// root's re-renders.
pub fn use_theme_sheet_open() -> Signal<bool> {
    use_context_provider(|| ThemeSheetOpen(Signal::new(false))).0
}

/// Redraws the home-screen icon in `kept` when the icon follows the theme.
/// The icon now shown, or `None` when the icon keeps its own look.
///
/// # Errors
///
/// The platform's message when the icon could not be switched.
pub fn follow_icon(prefs: &Preferences, kept: &ThemeConfig) -> Result<Option<ThemeConfig>, String> {
    if prefs.app_icon != AppIcon::MatchesTheme {
        return Ok(None);
    }
    app_icon::show(kept).map(|()| Some(kept.clone()))
}

/// A callback that flips the app's theme between its dark and light
/// palettes and carries the icon along. The root
/// effect persists the change. `on_kept` hears the icon after it followed,
/// for a host that shows the icon.
pub fn use_toggle_dark(on_kept: Option<EventHandler<ThemeConfig>>) -> Callback<()> {
    let mut theme = use_context::<Signal<ThemeConfig>>();
    let toast = use_toast();
    let prefs = use_prefs();
    use_callback(move |()| {
        let prev = theme.read().clone();
        let next = ThemeConfig {
            name: prev.name,
            is_dark: !prev.is_dark,
        };
        theme.set(next.clone());
        match follow_icon(&prefs.peek(), &next) {
            Ok(Some(icon)) => {
                if let Some(on_kept) = on_kept {
                    on_kept.call(icon);
                }
            }
            Ok(None) => {}
            Err(e) => toast.error(e, ToastOptions::default()),
        }
    })
}

/// The kit's theme sheet over the app's theme: live preview, the checkmark
/// keeps the pick and the x restores. `on_kept` hears a kept theme after
/// the icon followed it, for a host that shows the icon.
#[component]
pub fn AppThemeSheet(
    open: Signal<bool>,
    #[props(default)] hint: Option<Signal<bool>>,
    #[props(default)] on_kept: Option<EventHandler<ThemeConfig>>,
) -> Element {
    let theme = use_context::<Signal<ThemeConfig>>();
    let prefs = use_prefs();
    let toast = use_toast();
    rsx! {
        ThemeSheet {
            open,
            theme,
            hint,
            on_save: move |kept: ThemeConfig| {
                match follow_icon(&prefs.peek(), &kept) {
                    Ok(Some(icon)) => {
                        if let Some(on_kept) = on_kept {
                            on_kept.call(icon);
                        }
                    }
                    Ok(None) => {}
                    Err(e) => toast.error(e, ToastOptions::default()),
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use dioxus::core::NoOpMutations;

    use super::*;

    thread_local! {
        /// What each render of `Sheet` saw: the signal handed down as a prop
        /// beside the one read from the context.
        static SEEN: RefCell<Vec<(Signal<bool>, Signal<bool>)>> = const { RefCell::new(Vec::new()) };
    }

    /// Stands in for the root: provides the open state, hands it down as the
    /// sheet's prop, and re-renders when `tick` moves, as the root does when
    /// the shown theme changes.
    #[component]
    fn Root() -> Element {
        let open = use_theme_sheet_open();
        let tick = use_context_provider(|| Signal::new(0u32));
        rsx! {
            Sheet { open, tick: tick() }
        }
    }

    /// Stands in for the footer and the sheet together: the footer writes
    /// the context's signal, the sheet reads its prop. `tick` differs per
    /// root render so this re-renders with it.
    #[component]
    fn Sheet(open: Signal<bool>, tick: u32) -> Element {
        let _ = tick;
        let from_context = use_context::<ThemeSheetOpen>().0;
        SEEN.with(|seen| seen.borrow_mut().push((open, from_context)));
        rsx! {}
    }

    #[test]
    fn the_sheet_prop_and_the_context_stay_one_signal_across_root_rerenders() {
        let mut dom = VirtualDom::new(Root);
        dom.rebuild_in_place();
        for _ in 0..2 {
            dom.in_scope(ScopeId::APP, || {
                let mut tick = consume_context::<Signal<u32>>();
                tick.with_mut(|n| *n = n.wrapping_add(1));
            });
            dom.render_immediate(&mut NoOpMutations);
        }

        let seen = SEEN.with(|seen| seen.borrow().clone());
        assert!(seen.len() >= 3, "renders seen: {}", seen.len());
        let first = seen[0].1;
        for (i, (prop, from_context)) in seen.iter().enumerate() {
            assert_eq!(
                *prop, first,
                "render {i}: the sheet's prop is a fresh signal"
            );
            assert_eq!(*from_context, first, "render {i}: the context moved");
        }
    }
}
