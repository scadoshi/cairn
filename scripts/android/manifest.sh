#!/usr/bin/env bash
# Patch the dx-generated AndroidManifest.xml's MainActivity tag. Ported from
# zwipe's zcripts/android/manifest.sh.
#
# 1. launchMode="singleTask": without it MainActivity is `standard`, so an
#    explicit start of the component while an instance lives (another app,
#    a shortcut, Play's "Open" after an update) creates a second Activity in
#    the live process. That re-runs wry/tao's native init, and
#    ndk_context::initialize_android_context panics on
#    `assert!(previous.is_none())`. singleTask reuses the instance through
#    onNewIntent instead.
#
# 2. uiMode|locale|density|fontScale added to configChanges: dx emits only
#    orientation|screenLayout|screenSize|keyboardHidden, so a system dark or
#    light switch (or a locale, display-size or font-size change) recreates
#    the Activity, which reaches onDestroy, where back_handler.sh's process
#    kill fires, and the app vanishes mid-session. Auto dark mode at sunset
#    does this on a schedule.
#
# 3. screenOrientation="portrait": dx emits none, so the app turns with the
#    phone; the layout is portrait only, as Dioxus.toml's
#    UISupportedInterfaceOrientations says for iOS.
#
# 4. windowSoftInputMode="adjustResize": dx emits none, so the keyboard
#    slides over the WebView without resizing it, and a sheet's checkmark,
#    pinned to the bottom, sits under the keyboard. Resized, the page's
#    bottom is the keyboard's top. This holds at dx's targetSdk 34; from 35
#    on, edge-to-edge apps get the keyboard as an inset instead, so a
#    release built at 35 or above needs the sheet checked with the keyboard
#    up.
#
# zwipe's allowBackup="false" is left out: it keeps zwipe's session token out
# of Auto Backup, and Cairn holds no secret, only counters worth restoring.
#
# dx regenerates AndroidManifest.xml on every build, so run this after
# `dx build` or `dx bundle` and before the Gradle repackage; patch_bundle.sh
# does.
#
# Usage: scripts/android/manifest.sh [ANDROID_MANIFEST_XML]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MANIFEST="${1:-$REPO_ROOT/target/dx/cairn/release/android/app/app/src/main/AndroidManifest.xml}"

[ -f "$MANIFEST" ] || { echo "AndroidManifest.xml not found: $MANIFEST" >&2; exit 1; }

CONFIG_CHANGES="orientation|screenLayout|screenSize|keyboardHidden|uiMode|locale|density|fontScale"

python3 - "$MANIFEST" "$CONFIG_CHANGES" <<'PY'
import re, sys

path, config_changes = sys.argv[1], sys.argv[2]
src = open(path).read()

# The MainActivity open tag, however dx has wrapped its attributes.
m = re.search(r'<activity\b[^>]*android:name="dev\.dioxus\.main\.MainActivity"[^>]*>', src)
if not m:
    sys.exit("MainActivity <activity> tag not found — did dx change its manifest template?")
tag = m.group(0)
patched = tag

# 1. configChanges: replace whatever dx emitted with our fuller set.
if 'android:configChanges="' in patched:
    patched = re.sub(r'android:configChanges="[^"]*"',
                     f'android:configChanges="{config_changes}"', patched)
else:
    patched = patched.replace("<activity", f'<activity android:configChanges="{config_changes}"', 1)

# 2. launchMode: add it if absent, correct it if dx ever starts emitting one.
if 'android:launchMode="' in patched:
    patched = re.sub(r'android:launchMode="[^"]*"', 'android:launchMode="singleTask"', patched)
else:
    patched = patched.replace("<activity", '<activity android:launchMode="singleTask"', 1)

# 3. screenOrientation: portrait only, the same rule.
if 'android:screenOrientation="' in patched:
    patched = re.sub(r'android:screenOrientation="[^"]*"', 'android:screenOrientation="portrait"', patched)
else:
    patched = patched.replace("<activity", '<activity android:screenOrientation="portrait"', 1)

# 4. windowSoftInputMode: the page resizes for the keyboard.
if 'android:windowSoftInputMode="' in patched:
    patched = re.sub(r'android:windowSoftInputMode="[^"]*"', 'android:windowSoftInputMode="adjustResize"', patched)
else:
    patched = patched.replace("<activity", '<activity android:windowSoftInputMode="adjustResize"', 1)

if patched != tag:
    src = src.replace(tag, patched, 1)

open(path, "w").write(src)

print("launchMode=singleTask, screenOrientation=portrait, windowSoftInputMode=adjustResize, configChanges=" + config_changes)
PY

echo "Patched MainActivity attributes into $MANIFEST"
