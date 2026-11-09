//! The one screen you land on: the mark and today's date together at the
//! top, a quote, then the counters.
//!
//! It is one screen rather than a landing page and a list. Tapping through to
//! reach the counters would cost a tap on the thing the app exists for, and a
//! landing page has nothing on it you can act on.
//!
//! Both sheets hang off the screen rather than off a card, because a fixed
//! overlay inside a card is clipped by the card's rounded corners.

use std::collections::VecDeque;
use std::time::Duration;

use chrono::Datelike;
use dioxus::prelude::*;
use dioxus::router::Navigator;
use dioxus_primitives::toast::{Toasts, use_toast};
use orangui::icons::{self, DockIcon, EditIcon};
use orangui::{
    ActionDock, BottomSheet, Decode, DockAction, DockPlace, HintBullet, HintBullets, HintChip,
    HintDialog, HintKey, HintLine, use_on_resume, use_screen_hint,
};

use crate::domain::counter::{Counter, CounterId, format};
use crate::domain::preferences::Logo;
use crate::inbound::ui::components::celebration::{self, CelebrationHost};
use crate::inbound::ui::components::counter_form::{
    CounterForm, CounterFormState, EditSheet, save_form,
};
use crate::inbound::ui::components::counter_list::CounterList;
use crate::inbound::ui::components::quote_card::QuoteCard;
use crate::inbound::ui::components::reveal::Reveal;
use crate::inbound::ui::components::tile::{Num, Tile};
use crate::inbound::ui::router::Route;
use crate::inbound::ui::{self, SharedStore, use_date_format, use_prefs, use_store};
use crate::screens::counter_form::Mode;
use crate::screens::home::{Action, Effect, Row, State};
use crate::screens::tap::Outcome;

/// The C, the app's own mark.
pub(crate) const LOGO_CAIRN: &str = include_str!("../../../../../assets/c.txt");
/// The S, the owner's dev mark. This started as a personal app and the
/// wordmark is his name, so it stays available behind a setting.
const LOGO_SCADOSHI: &str = include_str!("../../../../../assets/s.txt");

/// Performs what Home's model asks: the store's reads and writes, the
/// toasts and celebrations, the navigation and the sheets.
#[derive(Clone, Copy)]
struct Driver {
    state: Signal<State>,
    store: CopyValue<SharedStore>,
    nav: Navigator,
    toasts: Toasts,
    host: CelebrationHost,
    create_open: Signal<bool>,
    edit_open: Signal<bool>,
    editing: Signal<Option<Counter>>,
}

impl Driver {
    /// Runs `action` and every action its effects answer with.
    fn dispatch(self, action: Action) {
        let mut state = self.state;
        let mut queue = VecDeque::from([action]);
        while let Some(action) = queue.pop_front() {
            let effects = state.write().update(action);
            for effect in effects {
                if let Some(answer) = self.perform(effect) {
                    queue.push_back(answer);
                }
            }
        }
    }

    /// Performs one effect, returning the store's answer when it has one.
    fn perform(self, effect: Effect) -> Option<Action> {
        let Self {
            mut create_open,
            mut edit_open,
            mut editing,
            ..
        } = self;
        match effect {
            Effect::LoadCounters => Some(Action::Loaded(load(&self.store.read()))),
            Effect::Adjust { id, day, delta } => {
                let written = self.store.read().adjust(id, ui::now(), day, delta);
                Some(Action::Adjusted(
                    written.map(|_| ()).map_err(|e| e.to_string()),
                ))
            }
            Effect::Toast(toast) => {
                ui::raise(self.toasts, &toast);
                None
            }
            Effect::Celebrate { how, saying } => {
                if let Some(toast) =
                    celebration::play(self.host, Outcome::Celebrate { how, saying })
                {
                    ui::raise(self.toasts, &toast);
                }
                None
            }
            Effect::Open(id) => {
                self.nav.push(Route::CounterScreen { id: id.0 });
                None
            }
            Effect::Edit(counter) => {
                // Mount the sheet closed, then open it a beat later. One
                // that mounts already open has no off-screen state to slide
                // up from, so it appears in place instead of rising. The
                // wait has to outlast BottomSheet's own premount guard,
                // which drops `transition: none` after WebKit's first
                // post-insert paint.
                editing.set(Some(counter));
                spawn(async move {
                    tokio::time::sleep(Duration::from_millis(70)).await;
                    edit_open.set(true);
                });
                None
            }
            Effect::Create => {
                create_open.set(true);
                None
            }
        }
    }
}

/// Every counter with its entries, or why the list could not be read. A
/// counter whose entries fail to read shows as empty.
fn load(store: &SharedStore) -> Result<Vec<Row>, String> {
    let counters = store.list_counters().map_err(|e| e.to_string())?;
    Ok(counters
        .into_iter()
        .map(|c| {
            let entries = store.entries(c.id).unwrap_or_default();
            Row::new(c, entries)
        })
        .collect())
}

/// The landing screen, which is also the counter list.
#[component]
pub fn Home() -> Element {
    let store = use_store();
    let create_open = use_signal(|| false);
    let edit_open = use_signal(|| false);
    let editing = use_signal(|| None::<Counter>);
    let hint_open = use_signal(|| false);
    let mut logo_taps = use_signal(|| 0_u32);
    use_screen_hint(hint_open);

    let driver = Driver {
        state: use_signal(State::default),
        store: use_hook(|| CopyValue::new(store)),
        nav: use_navigator(),
        toasts: use_toast(),
        host: use_context::<CelebrationHost>(),
        create_open,
        edit_open,
        editing,
    };

    // Hooks, read once. `use_prefs` and friends are context hooks, so calling
    // them inside the loop below would run a different number of hooks on the
    // render before the counters load than on the one after, which Dioxus
    // treats as a fatal hook-order change.
    let prefs = use_prefs();
    let date_format = use_date_format();

    use_effect(move || driver.dispatch(Action::Load));
    // Back from the background, the list reads the store again: another
    // day's figures, or a change made while away.
    use_on_resume(move || driver.dispatch(Action::Load));
    let reload = move |()| driver.dispatch(Action::Load);

    let (logo, logo_label) = match prefs().logo {
        Logo::Cairn => (LOGO_CAIRN, "Cairn"),
        Logo::Scadoshi => (LOGO_SCADOSHI, "scadoshi"),
    };

    let now = ui::shown_day();

    // Read once, the guard dropped before drawing.
    let (across, empty, error, rows) = {
        let model = driver.state.read();
        let rows: Vec<Row> = model
            .rows(prefs().counter_order, now)
            .into_iter()
            .cloned()
            .collect();
        (
            model.across(now, &prefs()),
            model.is_empty(),
            model.error().map(str::to_owned),
            rows,
        )
    };

    let date = format!("{} {}", now.format("%a"), date_format().date(now));
    let day = now.ordinal();
    let week = prefs().week_number(now);

    rsx! {
        div { class: "screen-content",
            div { class: "settings-sections content-enter",
                Reveal { class: "home-hero",
                    div { class: "settings-list-header home-hero-head",
                        // The mark resolves from static as the screen opens.
                        // Keyed on the mark and the tap count, so a change in
                        // Config or a tap on it decodes anew.
                        for mark in [logo_label] {
                            pre {
                                key: "{mark}-{logo_taps}",
                                class: "logo",
                                "aria-label": "{logo_label}",
                                onclick: move |_| logo_taps += 1,
                                Decode { text: logo }
                            }
                        }
                        div { class: "home-hero-when",
                            span { class: "settings-list-title", "{date}" }
                            div { class: "chip-tags",
                                span { class: "stat-chip stat-chip-p4", "day " Num { text: day.to_string() } }
                                span { class: "stat-chip", "week " Num { text: week.to_string() } }
                            }
                        }
                    }
                    if empty {
                        p { class: "pref-note", style: "padding: 1rem;",
                            "Create a counter with "
                            HintKey { DockIcon { path: icons::PLUS } }
                        }
                    } else {
                        div { class: "tile-grid tile-grid-2",
                            Tile { label: "logged today", value: format::compact(across.logged_today) }
                            Tile {
                                label: "goals met",
                                value: across.goals_met.to_string(),
                                hint: if across.with_goals == 0 { "no goals set".to_string() } else { format!("of {}", across.with_goals) },
                            }
                            Tile { label: "lifetime", value: format::compact(across.lifetime) }
                            Tile { label: "streak", value: format::compact(across.streak), hint: "days".to_string() }
                        }
                    }
                }
                QuoteCard {}
                if let Some(e) = error {
                    p { class: "form-error", "{e}" }
                }
                CounterList {
                    rows,
                    on_tap: move |(id, delta): (CounterId, i64)| {
                        let prefs = *prefs.peek();
                        driver.dispatch(Action::Tap { id, delta, day: ui::today(&prefs), prefs });
                    },
                    on_open: move |id: i64| driver.dispatch(Action::Open(CounterId(id))),
                    on_edit: move |id: CounterId| driver.dispatch(Action::Edit(id)),
                }
            }
        }
        // Create as the action dock's plus, pinned bottom right.
        ActionDock {
            DockAction { label: "Create", icon: icons::PLUS, onclick: move |()| driver.dispatch(Action::Create) }
        }
        HintDialog { open: hint_open, title: "Counters",
            HintLine { "Tap a counter's name or numbers to open it" }
            HintBullets {
                HintBullet {
                    HintChip { class: "stat-chip-error", "40 to go" }
                    " is what today still owes, and turns into "
                    HintChip { class: "stat-chip-success", "goal met" }
                }
                HintBullet { HintKey { EditIcon {} } " on a card edits that counter" }
                HintBullet { "Up top is every counter added together" }
                HintBullet { HintKey { DockIcon { path: icons::PLUS } } " at the bottom right starts a counter" }
            }
        }
        CreateSheet { open: create_open, on_created: reload }
        if let Some(c) = editing() {
            EditSheet {
                key: "{c.id.0}",
                open: edit_open,
                id: c.id,
                current_name: c.name.to_string(),
                current_goal: c.goal,
                current_step: c.step.get(),
                current_big_step: c.big_step.map(crate::domain::counter::Step::get),
                current_celebration: c.celebration,
                on_saved: reload,
            }
        }
    }
}

/// Create a counter in a sheet, the same form the edit sheet uses.
#[component]
fn CreateSheet(open: Signal<bool>, on_created: EventHandler<()>) -> Element {
    let mut open = open;
    let store = use_store();
    let prefs = use_prefs();
    let toast = use_toast();
    let mut form = use_hook(CounterFormState::default);

    // Every open starts blank.
    use_effect(move || {
        if open() {
            form.clear();
        }
    });

    let save = move |()| {
        let today = ui::today(&prefs.peek());
        if save_form(&mut form, Mode::Create, &store, toast, today) {
            on_created.call(());
            open.set(false);
        }
    };

    rsx! {
        BottomSheet {
            open,
            title: "Create counter",
            closable: true,
            CounterForm { state: form }
            // Save as the action dock's checkmark, pinned bottom right.
            ActionDock { place: DockPlace::Sheet,
                DockAction { label: "Save", icon: icons::CHECK, onclick: save }
            }
        }
    }
}
