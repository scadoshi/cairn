#!/usr/bin/env python3
"""Write a cairn database full of made-up counters, for screenshots.

    scripts/ios/spoof_db.py SCHEMA_DB OUT_DB

SCHEMA_DB is any real count.db (a backup from scripts/ios/backup_db.sh will
do): its tables and settings are kept and its counters, entries and events are
replaced. Four counters with a year of plausible days, seeded, so every run
draws the same numbers. The result goes into the simulator with
scripts/ios/screenshots.sh.
"""

import datetime
import random
import shutil
import sqlite3
import sys

random.seed(7)
TODAY = datetime.date.today()
# name, goal per day, per week, per year, step, big step, daily range, created
COUNTERS = [
    ("Pull-ups", 60, None, None, 5, 20, (40, 90), datetime.date(TODAY.year, 1, 1)),
    ("Push-ups", 120, None, None, 10, 50, (80, 180), datetime.date(TODAY.year, 1, 1)),
    ("Pages read", None, 200, None, 5, 25, (10, 60), datetime.date(TODAY.year, 2, 14)),
    ("Miles", None, None, 1200, 1, 5, (0, 6), datetime.date(TODAY.year, 3, 2)),
]


def main() -> None:
    schema, out = sys.argv[1], sys.argv[2]
    shutil.copy(schema, out)
    db = sqlite3.connect(out)
    db.executescript("delete from events; delete from entries; delete from counters;")
    for cid, (name, gd, gw, gy, step, big, (lo, hi), created) in enumerate(COUNTERS, start=1):
        db.execute(
            "insert into counters (id, name, goal_per_day, goal_per_week, goal_per_year,"
            " step, big_step, created_on) values (?,?,?,?,?,?,?,?)",
            (cid, name, gd, gw, gy, step, big, created.isoformat()),
        )
        day = created
        while day <= TODAY:
            # A rest day now and then, a few big days, and today part way through.
            total = 0 if random.random() < 0.08 else random.randint(lo, hi)
            if total and random.random() < 0.1:
                total = int(total * 1.6)
            if day == TODAY:
                total = max(step, int(total * 0.4) // step * step)
            total = total // step * step
            if total:
                db.execute(
                    "insert into entries (counter_id, day, count) values (?,?,?)",
                    (cid, day.isoformat(), total),
                )
                left, hour = total, 6
                while left > 0:
                    delta = min(left, random.choice([step, step * 2, big]))
                    hour = min(21, hour + random.randint(0, 3))
                    at = f"{day.isoformat()}T{hour:02d}:{random.randint(0, 59):02d}:{random.randint(0, 59):02d}"
                    db.execute(
                        "insert into events (counter_id, at, delta) values (?,?,?)",
                        (cid, at, delta),
                    )
                    left -= delta
            day += datetime.timedelta(days=1)
    db.commit()
    for name, total, days in db.execute(
        "select c.name, sum(e.count), count(e.day) from counters c"
        " join entries e on e.counter_id = c.id group by c.id"
    ):
        print(f"{name}: {total} over {days} days")


if __name__ == "__main__":
    main()
