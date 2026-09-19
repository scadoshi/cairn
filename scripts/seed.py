#!/usr/bin/env python3
"""Fill an Odo database with a year of plausible test data.

    scripts/seed.py PATH_TO_odo.db

Adds three counters if they are not there (pull-ups at 15 a day, push-ups at
10000 a year, pages read with no goal) and writes one row per day from
January 1 through today. The pattern is deterministic: a weekly rhythm with
rest days, a slow upward trend, and the odd big day, so the year view has
shape rather than noise. Existing rows for those days are replaced.

Only stdlib, so it runs anywhere python3 does. Point it at the simulator's
copy with:

    xcrun simctl get_app_container booted com.scadoshi.odo data
    # then append /Library/Application Support/odo/odo.db
"""

import datetime as dt
import random
import sqlite3
import sys

COUNTERS = [
    # name, goal_per_year, goal_per_day, base reps, per-day noise
    ("pull-ups", None, 15, 14, 6),
    ("push-ups", 10000, None, 30, 12),
    ("pages read", None, None, 20, 15),
]


def main(path: str) -> None:
    db = sqlite3.connect(path)
    today = dt.date.today()
    start = dt.date(today.year, 1, 1)
    rng = random.Random(today.year)  # same data every run for a given year

    for name, per_year, per_day, base, noise in COUNTERS:
        row = db.execute("SELECT id FROM counters WHERE name = ?", (name,)).fetchone()
        if row is None:
            db.execute(
                "INSERT INTO counters (name, goal_per_year, goal_per_day, created_on)"
                " VALUES (?, ?, ?, ?)",
                (name, per_year, per_day, start.isoformat()),
            )
            counter_id = db.execute("SELECT last_insert_rowid()").fetchone()[0]
        else:
            counter_id = row[0]

        day = start
        while day <= today:
            weekday = day.weekday()  # 0 = Monday
            progress = (day - start).days / 365  # 0 at new year, ~1 at year end
            if weekday == 6 or rng.random() < 0.12:
                count = 0  # Sundays off, plus the odd missed day
            else:
                count = base + int(base * 0.6 * progress) + rng.randint(-noise, noise)
                if weekday == 5:
                    count = int(count * 1.4)  # Saturday sessions run long
                if rng.random() < 0.04:
                    count *= 2  # a big day now and then
                count = max(count, 1)
            if count == 0:
                db.execute(
                    "DELETE FROM entries WHERE counter_id = ? AND day = ?",
                    (counter_id, day.isoformat()),
                )
            else:
                db.execute(
                    "INSERT INTO entries (counter_id, day, count) VALUES (?, ?, ?)"
                    " ON CONFLICT (counter_id, day) DO UPDATE SET count = excluded.count",
                    (counter_id, day.isoformat(), count),
                )
            day += dt.timedelta(days=1)

    db.commit()
    for name, *_ in COUNTERS:
        total, days = db.execute(
            "SELECT COALESCE(SUM(count), 0), COUNT(*) FROM entries"
            " WHERE counter_id = (SELECT id FROM counters WHERE name = ?)",
            (name,),
        ).fetchone()
        print(f"{name}: {total} across {days} days")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
