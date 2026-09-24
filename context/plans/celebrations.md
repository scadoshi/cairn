# Goal-met celebrations

Built 2026-09-24.

A counter crossing its daily goal should say so: a sheen running up the screen, or confetti in the theme's colors. One or the other, or off. Plus a toast with a rotating success line.

## The firing rule, which is the whole trick

The naive version watches render state and asks "is the goal met", which fires on every launch and every redraw of a day already finished. Avoiding that would need a celebrated-today flag per counter, cleared on rollover, living in the database.

None of that is necessary. At the tap site the screen already holds the count before the tap, the delta that actually applied, and the goal. The crossing is a pure function of those three:

```rust
/// Whether this tap is the one that finished the day.
pub fn crosses_goal(goal: Goal, before: u32, applied: i64, days_in_year: u32) -> bool
```

True only when `remaining_today` was above zero before and is zero after. It fires once, on the tap that crosses, and never on a launch, a redraw, a restore or an import. No stored state and no schema change.

Lives in `domain/counter/stats.rs` beside `remaining_today` and `applied_delta`, with tests for: crossing up, landing exactly on the target, overshooting, a tap that stays short, a tap on an already-finished day, and a subtraction.

Both tap sites call it: `components/counter_list.rs` and `screens/counter.rs`.

## Where it renders

At the app root, beside the router, the same as `DialogHostView`. Not inside a card and not inside the scroll column.

This is not a style preference. A fixed overlay inside a card is clipped by the card's `overflow: hidden`, and one inside `.profile-sections` was trapped by the content-enter transform. Both have already shipped as bugs. A `CelebrationHost` context carrying `Option<Celebration>` plus a `CelebrationHostView` mirrors the dialog host exactly, including clearing on unmount.

## The two animations

Sheen: one full-height gradient bar translating bottom to top, roughly 600ms, `transform` and `opacity` only. Nothing that triggers layout, or it will jank in the WebView.

Confetti: a fixed number of absolutely positioned pieces, each with a precomputed offset, delay and duration from a const table so nothing needs a random number generator. Colors come from `--accent-primary`, `--accent-secondary`, `--accent-tertiary` and `--color-success`, so they follow the theme.

Both respect `prefers-reduced-motion`, which is a media query around the keyframes and costs two lines.

## Config

A `Celebration` enum in `domain/preferences.rs`, cycled by a button, the same shape as `Logo` and `CounterOrder`:

- `Off`
- `Sheen`
- `Confetti`

Deliberately no "both". One or the other.

The row goes under Counter behavior beside Confirm minus, since it is feedback on counting rather than appearance. It gets a hint like every other row.

`Preferences` already carries `#[serde(default)]` and a test proving settings written before a field existed still load, so adding the field is safe for the database on the phone.

## The toast

A const list of short success lines, picked by a stride coprime with the list length, the way `domain/quote.rs` rotates. That gets variety without storing a cursor and without repeating twice running.

## Open question

A counter crossing its goal and the hero's "goals met" tile ticking up are the same event, so they share one celebration.

The separate event is the **last** counter of the day crossing, when goals met reaches its denominator and the day is finished. Whether that deserves its own treatment is undecided. Ship the per-counter case first and see whether the day-complete case feels missing.
