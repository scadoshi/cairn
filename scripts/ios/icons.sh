#!/usr/bin/env bash
# Compile the app icons into an asset catalog and reference them from
# Info.plist. dx does not run actool, so without this the phone shows a blank
# icon and Apple rejects the upload ("Missing required icon file ... 120x120").
#
# Run after every release build and before signing. scripts/ios/install_device.sh
# does it automatically; operations/ios/submission.md calls it by hand.
#
# The PNGs come from scripts/icon.py, which renders the mark from assets/c.txt.
#
# Usage: scripts/ios/icons.sh [APP_BUNDLE]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP="${1:-$REPO_ROOT/target/dx/crow/release/ios/Crow.app}"
ICONS="$REPO_ROOT/assets/favicon"
[ -d "$APP" ] || { echo "no app bundle at $APP" >&2; exit 1; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
SET="$WORK/Assets.xcassets/AppIcon.appiconset"
mkdir -p "$SET"

echo '{"info":{"version":1,"author":"xcode"}}' > "$WORK/Assets.xcassets/Contents.json"
cat > "$SET/Contents.json" <<'JSON'
{
  "images": [
    {"size":"20x20","idiom":"iphone","scale":"2x","filename":"icon-40.png"},
    {"size":"20x20","idiom":"iphone","scale":"3x","filename":"icon-60.png"},
    {"size":"29x29","idiom":"iphone","scale":"2x","filename":"icon-60.png"},
    {"size":"29x29","idiom":"iphone","scale":"3x","filename":"icon-87.png"},
    {"size":"40x40","idiom":"iphone","scale":"2x","filename":"icon-80.png"},
    {"size":"40x40","idiom":"iphone","scale":"3x","filename":"icon-120.png"},
    {"size":"60x60","idiom":"iphone","scale":"2x","filename":"icon-120.png"},
    {"size":"60x60","idiom":"iphone","scale":"3x","filename":"icon-180.png"},
    {"size":"1024x1024","idiom":"ios-marketing","scale":"1x","filename":"icon-1024.png"}
  ],
  "info":{"version":1,"author":"xcode"}
}
JSON

for px in 40 60 80 87 120 180 1024; do
  cp "$ICONS/icon-$px.png" "$SET/"
done

actool --compile "$APP" --platform iphoneos --minimum-deployment-target 16.0 \
  --app-icon AppIcon --output-partial-info-plist "$WORK/partial.plist" \
  "$WORK/Assets.xcassets" > /dev/null

PLIST="$APP/Info.plist"
# Rewrite rather than Add so a rebuilt bundle can be patched twice.
/usr/libexec/PlistBuddy -c "Delete :CFBundleIcons" "$PLIST" 2>/dev/null || true
/usr/libexec/PlistBuddy \
  -c "Add :CFBundleIcons dict" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon dict" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon:CFBundleIconFiles array" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon:CFBundleIconFiles:0 string AppIcon60x60" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon:CFBundleIconName string AppIcon" \
  "$PLIST"

[ -f "$APP/Assets.car" ] || { echo "actool produced no Assets.car" >&2; exit 1; }
echo "icons: Assets.car + $(ls "$APP" | grep -c '^AppIcon') AppIcon png(s)"
