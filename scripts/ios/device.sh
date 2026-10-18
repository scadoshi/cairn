#!/usr/bin/env bash
# Shared helpers for the iOS scripts. Source it, don't run it.

BUNDLE_ID="com.scadoshi.count"
# Inside the app's data container, matching outbound/paths.rs.
DB_DEST="Library/Application Support/scadoshi-count/count.db"

# Sets UDID and DEVICE_NAME. With a selector, matches that name or UDID;
# without one, requires exactly one attached iPhone.
#
# devicectl marks simulators with hardware "reality" = "simulated". Since
# Xcode 27 (devicectl JSON v5) real hardware has no "reality" at all and the
# data lives under "properties", so anything not simulated is a phone; the old
# top-level fields are a fallback for older Xcodes. A phone listed as
# "connected (no DDI)" has not had Developer Mode turned on yet, and nothing
# can be installed on it until it has.
find_device() {
  local want="${1:-}" found count json
  json="$(mktemp)"
  if ! xcrun devicectl list devices --quiet --json-output "$json" > /dev/null 2>&1 \
      || [ ! -s "$json" ]; then
    rm -f "$json"
    echo "devicectl could not list devices. Run: xcrun devicectl list devices" >&2
    return 1
  fi
  found="$(python3 -c '
import json, sys
want = sys.argv[2] if len(sys.argv) > 2 else ""
for d in json.load(open(sys.argv[1]))["result"]["devices"]:
    props = d.get("properties", {})
    hw = props.get("hardware") or d.get("hardwareProperties", {})
    if hw.get("reality") == "simulated" or hw.get("deviceType") != "iPhone":
        continue
    udid = hw.get("udid", "")
    name = (props.get("state") or {}).get("name") or d.get("deviceProperties", {}).get("name", "?")
    if want and want.lower() not in name.lower() and want.lower() != udid.lower():
        continue
    print(udid, name)
' "$json" "$want")"
  rm -f "$json"
  count="$(printf '%s' "$found" | grep -c . || true)"
  if [ "$count" -eq 0 ]; then
    if [ -n "$want" ]; then
      echo "No attached iPhone matches \"$want\"." >&2
    else
      echo "No iPhone found. Plug it in, unlock it, and trust the Mac." >&2
    fi
    echo "Attached now:" >&2
    xcrun devicectl list devices 2>/dev/null | grep -iv simulated >&2 || true
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
