# A second, larger increment

Built 2026-09-24. The bar wrap question is still open; look at it on the phone.

A counter gets an optional big step alongside its step, so the bar reads `[-20] [-10] [+10] [+20] [Edit]`. Usually sets of ten, sometimes twenty.

## Optional, deliberately

The column is nullable and the model carries `Option<Step>`. A counter without a big step keeps the three-button bar it has today and nothing about it changes. That is also what keeps the layout problem below off most counters.

## Schema v6

```sql
ALTER TABLE counters ADD COLUMN big_step INTEGER;
```

Nullable, so no default and no backfill. This is the first migration since `opens_a_database_left_at_every_older_version` existed, so it is the first one whose upgrade path is actually covered. Adding it means:

- a `SCHEMA_V6` constant and a rung in `migrate`
- `SCHEMA_VERSION` to 6
- `SCHEMA_V6` appended to the `ladder` array in that test, which then exercises v1 through v6

`COUNTER_COLS` and the row mapping in `outbound/sqlite/mod.rs` both gain the column, and `update_counter` gains the parameter.

## Validation

`Step::ALLOWED` stays as it is. The big step must be a different allowed value and larger than the step, otherwise the bar shows two buttons that do nearly the same thing. That is a `ValidationError` variant and a test.

## The bar wraps, and that is the thing to decide

`.util-bar` is `flex-wrap: wrap`, so five buttons will not overflow. They will go to two rows on a phone once the labels get wide, and the card height jumps when they do. `-100` and `+100` are the cases that push it.

Three ways out, undecided:

- accept the wrap
- cap the big step below 100
- shorten the labels on the card

Worth looking at on the phone with a real counter before choosing.

## Naming

**Step** and **Big step**. Not small and large: the existing one is called step throughout the code, the store, the form and the hints, and renaming it buys nothing.

Both fields need a hint saying which button is which, since the bar itself has no labels.

## Elsewhere

Confirm minus applies to both minus buttons.

The goal tag, `remaining_today` and every statistic are untouched. A step is only how much one tap moves the count.
