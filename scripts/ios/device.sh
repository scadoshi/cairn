#!/usr/bin/env bash
# Shared helpers for the iOS scripts. Source it, don't run it.

BUNDLE_ID="com.scadoshi.count"
# Inside the app's data container, matching outbound/paths.rs.
DB_DEST="Library/Application Support/scadoshi-count/count.db"

# Sets UDID and DEVICE_NAME. With a selector, matches that name or UDID;
# without one, requires exactly one attached iPhone.
#
# devicectl reports hardware "reality" as physical or simulated, the same
# thing the plain `list devices` table shows in its Reality column. A phone
# listed as "connected (no DDI)" has not had Developer Mode turned on yet,
# and nothing can be installed on it until it has.
find_device() {
  local want="${1:-}" found count
  found="$(xcrun devicectl list devices --quiet --json-output /dev/stdout 2>/dev/null \
    | python3 -c '
import json, sys
want = sys.argv[1] if len(sys.argv) > 1 else ""
for d in json.load(sys.stdin)["result"]["devices"]:
    hw = d.get("hardwareProperties", {})
    if hw.get("reality") != "physical" or hw.get("deviceType") != "iPhone":
        continue
    udid = hw.get("udid", "")
    name = d.get("deviceProperties", {}).get("name", "?")
    if want and want.lower() not in name.lower() and want.lower() != udid.lower():
        continue
    print(udid, name)
' "$want")"
  count="$(printf '%s' "$found" | grep -c . || true)"
  if [ "$count" -eq 0 ]; then
    if [ -n "$want" ]; then
      echo "No attached iPhone matches \"$want\"." >&2
    else
      echo "No iPhone found. Plug it in, unlock it, and trust the Mac." >&2
    fi
    echo "Attached now:" >&2
    xcrun devicectl list devices 2>/dev/null | grep -i physical >&2 || true
    return 1
  fi
  if [ "$count" -gt 1 ]; then
    echo "More than one iPhone attached. Name one with --device:" >&2
    printf '%s\n' "$found" | sed 's/^/  /' >&2
    return 1
  fi
  UDID="$(printf '%s' "$found" | awk '{print $1}')"
  DEVICE_NAME="$(printf '%s' "$found" | cut -d' ' -f2-)"
}

# A filename-safe version of DEVICE_NAME, so one Mac can hold backups from
# more than one phone without mixing them.
device_slug() {
  printf '%s' "$DEVICE_NAME" | tr '[:upper:]' '[:lower:]' \
    | sed "s/[^a-z0-9]\{1,\}/-/g; s/^-//; s/-$//"
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
