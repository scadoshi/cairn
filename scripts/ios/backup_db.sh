#!/usr/bin/env bash
# Pull the live database off the phone into a timestamped backup.
#
# Count is in daily use while it is also being developed, so the phone holds
# the only copy of the newest taps. Every deploy runs this first.
#
# Backups land in ~/Developer/cairn-data/backups/<phone>/count-YYYYMMDD-HHMMSS.db,
# outside the repo, which gitignores *.db anyway. One directory per phone,
# because a tester's counts are not yours and must never be restored over
# them. A pull that matches the newest backup byte for byte is dropped
# instead of stored twice, so running this repeatedly costs nothing.
#
# Restore one with:
#   scripts/ios/install_device.sh --db-only <backup file>
#
# Usage: scripts/ios/backup_db.sh [--device NAME] [--keep N]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=device.sh
. "$REPO_ROOT/scripts/ios/device.sh"

KEEP=30
WANT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --device) WANT="${2:?--device needs a name}"; shift 2 ;;
    --keep) KEEP="${2:?--keep needs a number}"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

find_device "$WANT"
DIR="$HOME/Developer/cairn-data/backups/$(device_slug)"
mkdir -p "$DIR"

stop_app

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
if ! xcrun devicectl device copy from --device "$UDID" \
    --domain-type appDataContainer --domain-identifier "$BUNDLE_ID" \
    --source "$DB_DEST" --destination "$TMP/count.db" > /dev/null 2>&1; then
  echo "No database on the phone yet; nothing to back up." >&2
  exit 0
fi

# A truncated or half-written pull is worse than no backup, because it looks
# like one. Make SQLite read the whole file before it is kept.
if ! sqlite3 "$TMP/count.db" 'pragma integrity_check' | grep -q '^ok$'; then
  echo "The database pulled off the phone did not pass integrity_check." >&2
  echo "Nothing was saved. Close Count on the phone and try again." >&2
  exit 1
fi

NEWEST="$(ls -1t "$DIR"/count-*.db 2>/dev/null | head -1 || true)"
if [ -n "$NEWEST" ] && \
   [ "$(shasum -a 256 < "$TMP/count.db" | cut -d' ' -f1)" \
   = "$(shasum -a 256 < "$NEWEST" | cut -d' ' -f1)" ]; then
  echo "backup: unchanged since $(basename "$NEWEST")"
else
  OUT="$DIR/count-$(date +%Y%m%d-%H%M%S).db"
  cp "$TMP/count.db" "$OUT"
  TOTAL="$(sqlite3 "$OUT" 'select coalesce(sum(count), 0) from entries')"
  echo "backup: $(device_slug)/$(basename "$OUT") ($(du -h "$OUT" | cut -f1), $TOTAL logged)"
fi

# Keep the newest N. These are a few megabytes each and the phone is the
# only place the newest taps exist, so err on the side of keeping too many.
ls -1t "$DIR"/count-*.db 2>/dev/null | tail -n +"$((KEEP + 1))" | while read -r old; do
  rm -f "$old"
  echo "pruned $(basename "$old")"
done
