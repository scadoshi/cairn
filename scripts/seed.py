#!/usr/bin/env python3
"""Fill an Odo database with plausible test data across several years.

    scripts/seed.py PATH_TO_odo.db

Adds three counters if they are not there (pull-ups at 15 a day, push-ups at
10000 a year, pages read with no goal) and writes one row per day from
January 1 two years back through today, plus the tap events behind each day
(one to three sessions at realistic hours) so the hourly chart has shape.
The pattern is deterministic: a weekly rhythm with rest days, a slow upward
trend across the years, and the odd big day. Existing rows for those days
are replaced.

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
    ("pull-ups", None, 15, 10, 5),
    ("push-ups", 10000, None, 24, 10),
    ("pages read", None, None, 16, 12),
]

# Hours a session tends to start, weighted: an early block and an evening one.
SESSION_HOURS = [6, 6, 7, 7, 7, 8, 12, 17, 18, 18, 19, 19, 20, 21]


def main(path: str) -> None:
    db = sqlite3.connect(path)
    today = dt.date.today()
    start = dt.date(today.year - 2, 1, 1)
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
        db.execute("DELETE FROM events WHERE counter_id = ?", (counter_id,))

        day = start
        total_days = (today - start).days or 1
        while day <= today:
            weekday = day.weekday()  # 0 = Monday
            progress = (day - start).days / total_days  # 0 at the start, 1 today
            if weekday == 6 or rng.random() < 0.12:
                count = 0  # Sundays off, plus the odd missed day
            else:
                count = base + int(base * 1.2 * progress) + rng.randint(-noise, noise)
                if weekday == 5:
                    count = int(count * 1.4)  # Saturday sessions run long
                if rng.random() < 0.04:
                    count *= 2  # a big day now and then
                count = max(count, 1)
            iso = day.isoformat()
            if count == 0:
                db.execute(
                    "DELETE FROM entries WHERE counter_id = ? AND day = ?", (counter_id, iso)
                )
            else:
                db.execute(
                    "INSERT INTO entries (counter_id, day, count) VALUES (?, ?, ?)"
                    " ON CONFLICT (counter_id, day) DO UPDATE SET count = excluded.count",
                    (counter_id, iso, count),
                )
                # Split the day's reps across one to three sessions.
                sessions = rng.randint(1, 3)
                cuts = sorted(rng.randint(1, count) for _ in range(sessions - 1))
                parts = [b - a for a, b in zip([0] + cuts, cuts + [count])]
                for part in parts:
                    if part <= 0:
                        continue
                    hour = rng.choice(SESSION_HOURS)
                    at = dt.datetime.combine(day, dt.time(hour, rng.randint(0, 59), 0))
                    db.execute(
                        "INSERT INTO events (counter_id, at, delta) VALUES (?, ?, ?)",
                        (counter_id, at.strftime("%Y-%m-%dT%H:%M:%S"), part),
                    )
            day += dt.timedelta(days=1)

    db.commit()
    for name, *_ in COUNTERS:
        total, days = db.execute(
            "SELECT COALESCE(SUM(count), 0), COUNT(*) FROM entries"
            " WHERE counter_id = (SELECT id FROM counters WHERE name = ?)",
            (name,),
        ).fetchone()
        events = db.execute(
            "SELECT COUNT(*) FROM events WHERE counter_id = (SELECT id FROM counters WHERE name = ?)",
            (name,),
        ).fetchone()[0]
        print(f"{name}: {total} across {days} days, {events} taps")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
