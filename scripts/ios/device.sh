#!/usr/bin/env bash
# Shared helpers for the iOS scripts. Source it, don't run it.

BUNDLE_ID="com.scadoshi.count"
# Inside the app's data container, matching outbound/paths.rs.
DB_DEST="Library/Application Support/scadoshi-count/count.db"

# Sets UDID and DEVICE_NAME for the one tethered phone, or exits.
# devicectl reports hardware "reality" as physical or simulated, the same
# thing the plain `list devices` table shows in its Reality column.
find_device() {
  local found count
  found="$(xcrun devicectl list devices --quiet --json-output /dev/stdout 2>/dev/null \
    | python3 -c '
import json, sys
for d in json.load(sys.stdin)["result"]["devices"]:
    hw = d.get("hardwareProperties", {})
    if hw.get("reality") == "physical" and hw.get("deviceType") == "iPhone":
        print(hw.get("udid", ""), d.get("deviceProperties", {}).get("name", "?"))
')"
  count="$(printf '%s' "$found" | grep -c . || true)"
  if [ "$count" -eq 0 ]; then
    echo "No iPhone found. Plug it in, unlock it, trust the Mac, and turn on" >&2
    echo "Settings > Privacy & Security > Developer Mode. Then check:" >&2
    echo "  xcrun devicectl list devices" >&2
    return 1
  fi
  if [ "$count" -gt 1 ]; then
    echo "More than one iPhone attached; unplug the others:" >&2
    echo "$found" >&2
    return 1
  fi
  UDID="$(printf '%s' "$found" | awk '{print $1}')"
  DEVICE_NAME="$(printf '%s' "$found" | cut -d' ' -f2-)"
}

# Stops the app so its SQLite connection is closed before the file is read
# or replaced. A live or suspended app holds the old file open and would
# write its stale copy back over a restore.
stop_app() {
  local launch pid
  launch="$(mktemp)"
  if xcrun devicectl device process launch --device "$UDID" \
      --terminate-existing --json-output "$launch" "$BUNDLE_ID" > /dev/null 2>&1; then
    pid="$(python3 -c '
import json, sys
print(json.load(open(sys.argv[1]))["result"]["process"]["processIdentifier"])
' "$launch" 2>/dev/null || true)"
    sleep 3
    [ -n "$pid" ] && xcrun devicectl device process terminate --device "$UDID" \
      --pid "$pid" > /dev/null 2>&1 || true
    sleep 1
  fi
  rm -f "$launch"
}
