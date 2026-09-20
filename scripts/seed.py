#!/usr/bin/env python3
"""Fill an Odo database with plausible test data across several years.

    scripts/seed.py PATH_TO_odo.db

Adds five counters if they are not there (pull-ups at 15 a day, push-ups at
10000 a year, pages read at 150 a week, squats at 40 a day, sit-ups at
12000 a year) and writes one row per day from
January 1 two years back through today, plus the tap events behind each day
(one to three sessions at realistic hours) so the hourly chart has shape.
The pattern is deterministic: a weekly rhythm with rest days, a slow upward
trend across the years, and the odd big day. Existing rows for those days
are replaced.

Add --stress for the UI torture set: a hundred counters, thirty of them
with data, a few of those in the millions (steps, ml of water, calories)
so tiles, pills, and the odometer meet numbers that need shortening.

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
    # name, goal_per_year, goal_per_day, goal_per_week, base reps, per-day noise, step
    ("pull-ups", None, 15, None, 10, 5, 1),
    ("push-ups", 10000, None, None, 24, 10, 5),
    ("pages read", None, None, 150, 16, 12, 10),
    ("squats", None, 40, None, 30, 12, 10),
    ("sit-ups", 12000, None, None, 28, 10, 5),
]

# Hours a session tends to start, weighted: an early block and an evening one.
SESSION_HOURS = [6, 6, 7, 7, 7, 8, 12, 17, 18, 18, 19, 19, 20, 21]


STRESS = [
    # name, goal_per_year, goal_per_day, goal_per_week, base, noise, step
    ("steps", None, 10000, None, 9000, 3000, 100),
    ("ml water", None, 2500, None, 2200, 600, 50),
    ("calories", 800000, None, None, 2100, 400, 25),
    ("seconds planked", None, None, 900, 120, 60, 10),
    ("words written", 200000, None, None, 500, 300, 100),
]


def main(path: str, stress: bool = False) -> None:
    db = sqlite3.connect(path)
    db.execute("PRAGMA foreign_keys = ON")
    today = dt.date.today()
    start = dt.date(today.year - 2, 1, 1)
    rng = random.Random(today.year)  # same data every run for a given year

    counters = list(COUNTERS)
    if stress:
        counters += STRESS
        # Twenty more with modest data, then enough empty ones to reach a
        # hundred, so the list scrolls a long way and the home box counts high.
        for i in range(1, 21):
            counters.append((f"habit {i:02d}", None, None, None, 5 + i, 3, 1))
        empties = 100 - len(counters)
    else:
        empties = 0

    for name, per_year, per_day, per_week, base, noise, step in counters:
        row = db.execute("SELECT id FROM counters WHERE name = ?", (name,)).fetchone()
        if row is None:
            db.execute(
                "INSERT INTO counters"
                " (name, goal_per_year, goal_per_day, goal_per_week, created_on, step)"
                " VALUES (?, ?, ?, ?, ?, ?)",
                (name, per_year, per_day, per_week, start.isoformat(), step),
            )
            counter_id = db.execute("SELECT last_insert_rowid()").fetchone()[0]
        else:
            counter_id = row[0]
            db.execute("UPDATE counters SET step = ? WHERE id = ?", (step, counter_id))
        db.execute("DELETE FROM events WHERE counter_id = ?", (counter_id,))

        day = start
        total_days = (today - start).days or 1
        while day <= today:
            weekday = day.weekday()  # 0 = Monday
            progress = (day - start).days / total_days  # 0 at the start, 1 today
            daily = any(name == n for n, *_ in STRESS)  # steps and water happen every day
            if (weekday == 6 and not daily) or rng.random() < (0.02 if daily else 0.12):
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

    for i in range(empties):
        name = f"counter {i + 1:03d}"
        if db.execute("SELECT 1 FROM counters WHERE name = ?", (name,)).fetchone() is None:
            db.execute(
                "INSERT INTO counters (name, created_on, step) VALUES (?, ?, 1)",
                (name, today.isoformat()),
            )

    db.commit()
    for name, *_ in counters:
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
    args = [a for a in sys.argv[1:] if a != "--stress"]
    if len(args) != 1:
        sys.exit(__doc__)
    main(args[0], stress="--stress" in sys.argv)
