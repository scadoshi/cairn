# Screen models

Built 2026-11-08, one PR per step into main: #11 `tap`, #12 `home`, #13 `counter`, #15 `draft`, #18 `counter_form`, #19 `config`. This brings zwipe's screen model pattern (`zwipe/zwipe-ui/context/screen-models.md`, where every screen of the app now runs on one) into Cairn, so both apps put screen logic in the same shape and the same lessons apply to both.

## Why

- **The tap flow is written twice.** `screens/counter.rs` (the `adjust` callback) and `components/counter_list.rs` (the `bump` callback) each work out the applied delta, the "-0" toast, the goal crossing, the year mark, the all-done check, the saying and the celebration or toast. The comments in both say they must match. One tested rule replaces both.
- **The store is read during render.** Home reads every counter's entries in its body to build the tiles, each `CounterCard` reads its own entries again, and `ui::all_goals_met` reloads every counter and every entry on each goal-crossing tap. A model loads once and its views compute from what it holds.
- **Screen logic has no tests.** The domain is well tested; what a screen does with it (which toast, which kind, when Save is off, what Back says on an unchanged draft) is only checked on the phone.
- **The drafted sheets repeat one flow.** Mark, Goal animation, Rest days and App icon each snapshot on open, toast "unchanged" on a Back with nothing changed and save on Save. One draft model, as zwipe did for its pickers.
- **Fewer runtime borrow panics.** Handlers that only dispatch an action can't hold a read guard across a write (`context/development/dioxus.md`, Runtime Borrows), which is the class of crash zwipe hit in its Config screen.
- **One shape across both apps.** Anything learned in one (the `Rc` mounted flag in `use_on_bump`, binding a value before dispatching it) carries straight over.

## Where it lives

A new top-level module, `src/lib/screens/`, beside `domain/`. Pure like the domain: no Dioxus, no SQLite, no clock. Cairn stays one crate (`architecture/decisions.md` §2); a module is enough to keep the boundary, and a test can check it imports neither `dioxus` nor `rusqlite`.

```
bin ──> inbound/ui ──> screens ──> domain <── outbound/sqlite
```

`inbound/ui` draws a model and performs its effects; `screens` decides; `domain` holds the rules both lean on.

## The shape

The same as zwipe's: one module per screen with `State`, `Action` and `Effect`, and `State::update(Action) -> Vec<Effect>`. State fields are private and views compute on demand. Copy is `pub const` at the top of the module, verbatim from today's app, pinned by a `the_copy_matches_the_app` test.

Where Cairn differs from zwipe:

- **The store is synchronous.** A read is an effect (`Effect::LoadCounters`) that the driver performs and answers in the same dispatch (`Action::Loaded(..)`). So there are no `Ticket`s and no `Generation`: a reply can't arrive late. If a store goes async (the CloudKit sync in §8), the waiting list and generations come in then, the way zwipe has them.
- **Writes are effects too.** `Effect::Adjust { id, at, day, delta }` comes back as `Action::Adjusted(Result<u32, StoreError>)`, so the model decides the toast for both outcomes.
- **No server, no session, no telemetry.** None of zwipe's request runner, refresh or error mapping. A `StoreError` is already a message.
- **The clock comes in on actions.** `today` and `now` ride on the action that needs them (`Tap { delta, now, today }`), as zwipe's `CheckFlavor { now }` does, so tests pin the day.
- **Celebrations are an effect.** `Effect::Celebrate { how, saying }`; the client plays it through `CelebrationHost` and toasts the line it gets back. `Saying` moves from the component into `screens` since it is plain data.

## Toasts

A model emits `Toast { kind, text, duration }` with orangui-core's bands, the same as `zwipe_ui::toast::Toast`. Cairn's stylesheet ties a toast's fade to its kind (`architecture/structure.md`, "One coupling"), so info must be `QUICK` and the rest `NORMAL`; a test runs every toast a model can emit and checks that pairing.

Cairn defines its own `Toast` at first. The tidy follow-up is lifting `Toast` and `ToastKind` from zwipe-ui into orangui-core, plus a `raise(toasts, toast)` in orangui, so both apps share the type and the adapter. That is an orangui branch and the owner's call; nothing here waits on it.

## What stays in the client

The same list as zwipe's: drawing, layout, focus, scroll, gestures, entrances and the celebration layers themselves. Overlays that are pure presentation (whether the hint dialog or a confirm dialog is up) stay in signals. The odometer `Decode`, the logo tap count and the quote's countdown are presentation too.

## Wiring

One `Copy` `Driver` per screen holding the model signal and the handles that perform effects (store, navigator, toasts, celebration host, `StoreVersion`), with `dispatch` and `perform` as in zwipe. Rules carried over:

- Handlers bind what they read before dispatching: `let shown = *sig.peek(); driver.dispatch(Action::X { shown });`, never a guard inside the call.
- Render reads the model once and drops the guard before `rsx!`.
- A first-run-skipping effect shares its flag through `Rc<Cell<_>>`, never a plain `Cell` from `use_hook` (zwipe's `use_on_bump`).

## Order

Each step is its own branch off main, merged before the next starts, with the app switched in the same branch.

1. **`screens::tap`.** The tap rule as a pure function: from the counter, today's count, the year's total before the tap, the delta, the day, the app's celebration setting and whether every other goal is met, return `Nothing(toast)` for a "-0", or `Apply(delta)` and then the outcome (`Celebrate { how, saying }` or a toast). Both call sites switch. Tests: "-0" on an empty day, a plain +1, a crossing, a year mark winning over all-done, all-done winning over a plain crossing, a counter's own celebration over the app's, Off. S.
2. **`screens::home`.** Home's model: counters with their entries loaded once, the four tiles (`across_counters`), the empty line, open, edit and create, and the card taps through `screens::tap`, which also answers all-done from what Home already holds. `CounterList` takes rows and handlers instead of reading the store. S to M.
3. **`screens::counter_form`.** The Create and Edit sheet's draft and validation (`CounterFormState` today), one model in two modes as zwipe's `deck_form` is, with the "Saved {name}" toasts. S.
4. **`screens::counter`.** The counter screen: load, the trend and best pickers, taps through `screens::tap`, rename, delete and its toast, Back. M.
5. **`screens::draft`.** One draft model for Mark, Goal animation, Rest days and App icon: snapshot on open, Save off when unchanged, Back toasts what was left unchanged. S.
6. **`screens::config`.** Settings (`screens/config.rs`) itself: the preference toggles, the date format, export and its toasts, the hint topics. M.

Then stop and look at it on the phone before anything else moves.

## As built

Where the build differs from the text above:

- `Toast` is built only through `info`, `success` and `error`, which tie the duration to the kind, so the pairing holds by construction and a unit test replaces the run-every-toast test.
- Home answers all-done from the rows it holds with the tap applied. The counter screen still asks the store, through an `AskAllMet` effect sent only when the tap would ask (`Tap::needs_all_met`).
- The counter form keeps its typed fields in signals, since orangui's `TextInput` binds a `Signal<String>`; the client reads them into `Fields` before checking.
- Settings keeps the App icon's dark toggle and the date format button in the client.
- Building the counter screen turned up a bug: moving between counters from the menu re-rendered the screen without reloading it. #14 fixed it with `Action::Open`, run on each new id.

## Out of scope

- A second client. A web or terminal Cairn would draw these same models, but whether Cairn has one is a product decision (§8), not part of this plan.
- The watch. Its logic is a small Swift port (`plans/watch.md`), not these models.
- Moving the domain or the models into their own crates.
