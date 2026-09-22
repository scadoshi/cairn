#!/usr/bin/env bash
# Build, sign and install the release app on a tethered iPhone, then
# optionally seed it with an existing database.
#
# This is Route A of context/operations/ios/first_device.md in one command:
# a development-signed build that runs for a year on registered devices. The
# portal paperwork (App ID, the phone's UDID, the development profile) has to
# exist first; this script checks for the profile and stops if it is missing.
#
# The bundle folder is named after the crate (Crow.app) while the app's own
# label is Count. dx writes the name keys twice, once from the crate and once
# from Dioxus.toml's [ios.plist]; re-serializing the plist keeps the last of
# each, which is the Count one. See operations/ios/submission.md step 2.
#
# Usage:
#   scripts/ios/install_device.sh                  # build, sign, install
#   scripts/ios/install_device.sh --db path/to.db  # ... then push a database
#   scripts/ios/install_device.sh --db-only p.db   # skip the build, push only
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=device.sh
. "$REPO_ROOT/scripts/ios/device.sh"

APP="$REPO_ROOT/target/dx/crow/release/ios/Crow.app"
PROFILE="$HOME/certs/Count_Development.mobileprovision"
IDENTITY="Apple Development: SCOTTY RAY FERMO (NVSWB62C54)"

DB=""
BUILD=1
while [ $# -gt 0 ]; do
  case "$1" in
    --db) DB="${2:?--db needs a path}"; shift 2 ;;
    --db-only) DB="${2:?--db-only needs a path}"; BUILD=0; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

find_device
echo "device: $DEVICE_NAME ($UDID)"

if [ "$BUILD" -eq 1 ]; then
  [ -f "$PROFILE" ] || { echo "missing $PROFILE (see first_device.md)" >&2; exit 1; }

  dx build --release --platform ios --device true
  [ -d "$APP" ] || { echo "no app bundle at $APP" >&2; exit 1; }

  plutil -convert xml1 "$APP/Info.plist"
  NAME="$(plutil -extract CFBundleDisplayName raw "$APP/Info.plist")"
  [ "$NAME" = "Count" ] || { echo "display name is '$NAME', expected Count" >&2; exit 1; }

  "$REPO_ROOT/scripts/ios/icons.sh" "$APP"

  cp "$PROFILE" "$APP/embedded.mobileprovision"
  codesign --force --sign "$IDENTITY" \
    --entitlements "$REPO_ROOT/Entitlements.plist" "$APP"
  codesign --verify --verbose "$APP"

  xcrun devicectl device install app --device "$UDID" "$APP"
fi

if [ -n "$DB" ]; then
  [ -f "$DB" ] || { echo "no database at $DB" >&2; exit 1; }
  # Fail on a file that is not a usable database before it reaches the phone.
  sqlite3 "$DB" 'select count(*) from counters' >/dev/null

  # This overwrites whatever is on the phone, so keep a copy of that first.
  # Skipped when restoring a backup onto a phone that has nothing yet.
  "$REPO_ROOT/scripts/ios/backup_db.sh"

  # The app creates Library/Application Support/scadoshi-count on first run,
  # and the copy needs that directory to exist. Stopping it also closes its
  # SQLite connection: a live app holds the old file open and would write
  # its stale copy back over the restore.
  stop_app

  echo "pushing $(basename "$DB")"
  xcrun devicectl device copy to --device "$UDID" \
    --domain-type appDataContainer --domain-identifier "$BUNDLE_ID" \
    --source "$DB" --destination "$DB_DEST"

  # Read it back and compare. Both copy directions want a full file path as
  # the destination; handing copy-from a directory fails with "Is a directory".
  BACK="$(mktemp -d)"
  if xcrun devicectl device copy from --device "$UDID" \
      --domain-type appDataContainer --domain-identifier "$BUNDLE_ID" \
      --source "$DB_DEST" --destination "$BACK/count.db" > /dev/null 2>&1 \
     && [ -f "$BACK/count.db" ] \
     && [ "$(shasum -a 256 < "$DB" | cut -d' ' -f1)" \
        = "$(shasum -a 256 < "$BACK/count.db" | cut -d' ' -f1)" ]; then
    echo "verified: the database on the phone matches the one pushed"
  else
    echo "WARNING: could not read the database back and match it." >&2
    echo "Check the app container before trusting the counts on the phone." >&2
  fi
  rm -rf "$BACK"

  echo
  echo "Force-quit Count from the app switcher and reopen it."
fi
