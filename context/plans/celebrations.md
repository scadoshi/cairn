# Goal-met celebrations

Built 2026-09-24. Seven animations rather than two, a Random option, and a
per-counter override on top of the app-wide setting, which schema v7 carries.
The firing rule below is the part that survived unchanged, and it is the part
that mattered.

A counter crossing its daily goal should say so. Seven ways it can, one at a time, or none. Plus a toast with a rotating success line.

## The firing rule, which is the whole trick

Watching render state and asking "is the goal met" fires on every launch and every redraw of a day already finished. Avoiding that needs a celebrated-today flag per counter, cleared on rollover, in the database.

None of which is necessary. The tap site already holds the count before, the delta that applied, and the goal, and the crossing is a pure function of those three:

```rust
/// Whether this tap is the one that finished the day.
pub fn crosses_goal(goal: Goal, before: u32, applied: i64, days_in_year: u32) -> bool
```

True only when `remaining_today` was above zero before and is zero after, so it fires once and never on a launch, a redraw, a restore or an import. It lives in `domain/counter/stats.rs` beside `remaining_today`, and both tap sites call it.

## Where it renders

At the app root, beside the router, the same as `DialogHostView`. Not inside a card and not inside the scroll column.

This is not a style preference. A fixed overlay inside a card is clipped by the card's `overflow: hidden`, and one inside `.profile-sections` was trapped by the content-enter transform. Both have already shipped as bugs. A `CelebrationHost` context carrying `Option<Celebration>` plus a `CelebrationHostView` mirrors the dialog host exactly, including clearing on unmount.

## The animations

All of them are `transform` and `opacity` only. Anything that triggers layout janks in the WebView.

Particle counts, offsets, delays and spins come from const tables, so nothing needs a random number generator. Poppers stack two elements per piece, because one transform travels in a straight line and an arc needs horizontal and vertical on separate layers, each with its own curve.

Every one of them sits behind `prefers-reduced-motion`.

## Config

A `Celebration` enum in `domain/preferences.rs`, cycled like `Logo` and `CounterOrder`. `Random` rotates rather than rolling dice: chance repeats, and the same animation twice running is what choosing Random is meant to avoid.

Typewriter and stamp draw the line themselves, so `celebrate` returns the line to toast or nothing when the animation is already saying it. The two cannot then disagree about which line this is.

Single-colour animations take one of the theme's three accents in rotation. All of them in the success green read as variations of one effect.

## The toast

A const list of short success lines, picked by a stride coprime with the list length, the way `domain/quote.rs` rotates. That gets variety without storing a cursor and without repeating twice running.

## Still open

The **last** counter of the day crossing, when goals met reaches its denominator, is a different event from any single counter finishing. Whether it deserves its own treatment is undecided.

## What the build taught

The overlay has to be keyed on the crossing and its cleanup timer has to belong to the host. Spawning that timer from the tap handler tied it to whichever card was tapped, and Dioxus cancels a task when its scope goes away, so a re-render could kill the animation halfway through. Without a key, a re-render builds a fresh node and the animation starts over, which is what incrementing mid-typewriter did.

Poppers launched from the bottom corners have almost no screen beneath them. Anything that tries to fall off the bottom edge gets cut mid-flight however long the animation runs, so the pieces fade in mid-air instead. Two attempts went into lengthening a fall that had nowhere to go before that was obvious.
