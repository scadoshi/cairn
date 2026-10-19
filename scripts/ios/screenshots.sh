#!/usr/bin/env bash
# Store screenshots: the booted iPhone 16 Pro Max simulator (the 6.9" set the
# App Store wants, 1320x2868) tapped through Home, a counter, its trends, the
# counter list and Config, in each theme listed below. Needs the app installed
# on the simulator, a spoof database pushed in (scripts/ios/spoof_db.py) and
# idb on PATH (brew install facebook/fb/idb-companion; pip install fb-idb) to
# do the tapping, since simctl cannot. Coordinates below are points measured
# on the 11 Pro Max's 414x896 screen; the 16 Pro Max is 440x956, so re-measure
# the card and action bar taps before the next screenshot run.
#
# Usage: scripts/ios/screenshots.sh [OUT_DIR]   # default context/.../screenshots
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="${1:-$REPO_ROOT/context/operations/ios/app-store/submission/screenshots}"
APPID=com.scadoshi.count
UDID="$(xcrun simctl list devices booted -j | python3 -c '
import json, sys
for devs in json.load(sys.stdin)["devices"].values():
    for d in devs:
        if d["name"] == "iPhone 16 Pro Max": print(d["udid"])')"
[ -n "$UDID" ] || { echo "boot the iPhone 16 Pro Max simulator first" >&2; exit 1; }
C="$(xcrun simctl get_app_container "$UDID" $APPID data)"
DB="$C/Library/Application Support/scadoshi-count/count.db"

xcrun simctl status_bar "$UDID" override --time 9:41 --batteryState charged \
  --batteryLevel 100 --wifiBars 3 --cellularBars 4 --operatorName ""
tap() { idb ui tap --udid "$UDID" "$1" "$2" > /dev/null 2>&1; sleep "${3:-1.5}"; }
swipe_up() { idb ui swipe --udid "$UDID" 207 700 207 200 --duration 0.4 > /dev/null 2>&1; sleep 2; }
shot() { xcrun simctl io "$UDID" screenshot "$1" > /dev/null 2>&1; }

for spec in "gruvbox:true" "catppuccin:false" "nord:true" "rose-pine:false" "tokyo-night:true"; do
  theme=${spec%%:*}; dark=${spec##*:}; mode=$([ "$dark" = true ] && echo dark || echo light)
  D="$OUT/$theme-$mode"; rm -rf "$D"; mkdir -p "$D"
  xcrun simctl terminate "$UDID" $APPID 2>/dev/null; sleep 1
  sqlite3 "$DB" "update settings set value = '{\"name\":\"$theme\",\"is_dark\":$dark}' where key = 'theme';"
  xcrun simctl launch "$UDID" $APPID > /dev/null; sleep 5
  shot "$D/01-home.png"
  tap 62 636 2;   shot "$D/02-counter.png"          # the first counter card
  swipe_up;       shot "$D/03-counter-trends.png"
  tap 132 829 2                                      # Back, left of the bar
  swipe_up;       shot "$D/04-counters.png"
  tap 248 829 2;  shot "$D/05-config.png"
  tap 207 829 1                                      # Back, centered on Config
  echo "$theme $mode"
done
