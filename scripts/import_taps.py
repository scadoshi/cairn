#!/usr/bin/env python3
"""Import a tap log export into an Crow database.

    scripts/import_taps.py EXPORT.csv PATH_TO_odo.db [--replace]

The export is one file with a totals block at the top, then a section per
counter headed `N-<name>-export.csv` with rows of
`Time Stamp, Date, Time, Counter Value, Increment`. Each row is one tap;
Increment is signed (the -10s are undos). That is Crow's events table
exactly, so every row becomes an event with its timestamp, and the daily
entries are rebuilt as the sum of each day's increments.

A counter is created if missing, named as in the export (trimmed), with no
goal, its step set to the most common increment, and created_on the first
tap's date. A section named Days is skipped: it is a day-of-year tracker,
which the app derives. With --replace, every counter in the database is
removed first so the app shows only this data.

Each counter's total is checked against the totals block and reported.
"""

import collections
import csv
import datetime as dt
import sqlite3
import sys


def parse(path: str):
    """Yields (name, taps) per section, plus the totals block."""
    totals = {}
    sections = []
    current = None
    with open(path, newline="") as f:
        for raw in csv.reader(f):
            if not raw or not any(c.strip() for c in raw):
                continue
            row = [c.strip() for c in raw]
            if row[0].endswith("-export.csv"):
                name = row[0].split("-", 1)[1].rsplit("-export.csv", 1)[0].strip()
                current = (name, [])
                sections.append(current)
            elif row[0] in ("Counter Name", "Time Stamp"):
                continue
            elif current is None and len(row) == 2:
                totals[row[0]] = int(row[1])
            elif current is not None and len(row) == 5:
                stamp = dt.datetime.strptime(row[0][:19], "%Y-%m-%d %H:%M:%S")
                current[1].append((stamp, int(row[4])))
    return totals, sections


def main(csv_path: str, db_path: str, replace: bool) -> None:
    totals, sections = parse(csv_path)
    db = sqlite3.connect(db_path)
    db.execute("PRAGMA foreign_keys = ON")
    if replace:
        # Explicit as well as the cascade: SQLite reuses rowids after a
        # delete, so any orphaned row would collide with a new counter's id.
        db.execute("DELETE FROM events")
        db.execute("DELETE FROM entries")
        db.execute("DELETE FROM counters")

    for name, taps in sections:
        if name == "Days" or not taps:
            continue
        taps.sort()
        step = collections.Counter(abs(d) for _, d in taps if d > 0).most_common(1)[0][0]
        if step not in (1, 5, 10, 25, 50, 100):
            step = 1
        row = db.execute("SELECT id FROM counters WHERE name = ?", (name,)).fetchone()
        if row is None:
            db.execute(
                "INSERT INTO counters (name, created_on, step) VALUES (?, ?, ?)",
                (name, taps[0][0].date().isoformat(), step),
            )
            counter_id = db.execute("SELECT last_insert_rowid()").fetchone()[0]
        else:
            counter_id = row[0]
            db.execute("UPDATE counters SET step = ? WHERE id = ?", (step, counter_id))
            db.execute("DELETE FROM events WHERE counter_id = ?", (counter_id,))
            db.execute("DELETE FROM entries WHERE counter_id = ?", (counter_id,))

        db.executemany(
            "INSERT INTO events (counter_id, at, delta) VALUES (?, ?, ?)",
            [(counter_id, at.strftime("%Y-%m-%dT%H:%M:%S"), delta) for at, delta in taps],
        )
        per_day = collections.defaultdict(int)
        for at, delta in taps:
            per_day[at.date()] += delta
        db.executemany(
            "INSERT INTO entries (counter_id, day, count) VALUES (?, ?, ?)",
            [(counter_id, d.isoformat(), max(c, 0)) for d, c in per_day.items() if c > 0],
        )
        total = sum(delta for _, delta in taps)
        expected = totals.get(name)
        check = "matches" if expected == total else f"file says {expected}"
        print(f"{name}: {total} across {len(per_day)} days, {len(taps)} taps, step {step} ({check})")

    db.commit()


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if a != "--replace"]
    if len(args) != 2:
        sys.exit(__doc__)
    main(args[0], args[1], replace="--replace" in sys.argv)
