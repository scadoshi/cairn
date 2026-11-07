#!/usr/bin/env bash
# Every patch the dx-generated Android project needs, in one call.
#
# dx regenerates the android/ tree on every build, MainActivity.kt and
# WryActivity.kt included, so each patch runs after `dx build` or
# `dx bundle` and before the Gradle repackage:
#
#   back_handler.sh    missing -> OS back exits the app instead of navigating
#   wry_activity.sh    missing -> UnsatisfiedLinkError in onCreate on every
#                                 device: the stock activity calls tao 0.34's
#                                 native names and the vendored tao is 0.37
#
# APP_DIR is the generated Gradle project; it defaults to the release one that
# `dx bundle --release` writes. debug_apk.sh passes the debug one.
#
# Usage: scripts/android/patch_bundle.sh [APP_DIR]
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../.." && pwd)"
APP_DIR="${1:-$REPO_ROOT/target/dx/cairn/release/android/app}"
KOTLIN="$APP_DIR/app/src/main/kotlin/dev/dioxus/main"

[ -d "$KOTLIN" ] || { echo "generated Android project not found: $APP_DIR" >&2; exit 1; }

echo "== back navigation =="
"$HERE/back_handler.sh" "$KOTLIN/MainActivity.kt"

echo "== WryActivity (tao 0.37 entry points) =="
"$HERE/wry_activity.sh" "$KOTLIN/WryActivity.kt"
