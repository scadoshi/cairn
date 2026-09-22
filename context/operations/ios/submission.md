# iOS: build, sign, upload

Crow's version of zwipe's runbook, with the parts a local-only app never needs cut out. One-time setup (Apple Developer account, the App ID for `com.scadoshi.count`, an Apple Distribution certificate, an App Store provisioning profile saved as `~/certs/Count_App_Store.mobileprovision`) is the same as zwipe's `context/operations/ios/setup.md`; do that once and don't repeat it here.

Entitlements are checked in: `Entitlements.plist` for debug and `Entitlements-Release.plist` for the store, identical except `get-task-allow`. The team prefix is zwipe's.

## Always the latest Xcode

Apple's submission allowlist rejects binaries linked against anything but the current Xcode and SDK, with a misleading "beta Xcode" message. Before a release build, update Xcode from the App Store, check `xcodebuild -version`, then wipe the cached iOS objects so cargo relinks:

```bash
rm -rf target/aarch64-apple-ios target/dx/crow/release/ios
```

The plist keys in `Dioxus.toml` are re-patched below anyway.

## 1. Build

```bash
dx build --release --platform ios --device true
APP=target/dx/crow/release/ios/Crow.app
```

## 2. Patch Info.plist

Dioxus writes these from a stale template.

```bash
# dx names the bundle after the crate (Crow), so Dioxus.toml's [ios.plist]
# adds CFBundleDisplayName and CFBundleName = "Count" a second time. iOS
# takes the last value, but the store bundle should not carry duplicate
# keys: re-serializing through plutil keeps one copy of each, the "Count" one.
plutil -convert xml1 $APP/Info.plist
plutil -extract CFBundleDisplayName raw $APP/Info.plist   # must print Count

# iPhone only: Apple wants exactly one platform and crow has no iPad layout.
/usr/libexec/PlistBuddy \
  -c "Delete :CFBundleSupportedPlatforms" \
  -c "Add :CFBundleSupportedPlatforms array" \
  -c "Add :CFBundleSupportedPlatforms:0 string iPhoneOS" \
  -c "Delete :UIDeviceFamily" \
  -c "Add :UIDeviceFamily array" \
  -c "Add :UIDeviceFamily:0 integer 1" \
  $APP/Info.plist

# Marketing version = Cargo.toml's; build number increments every upload and
# is never reused. Record both in history.md.
/usr/libexec/PlistBuddy \
  -c "Set :CFBundleShortVersionString <VERSION>" \
  -c "Set :CFBundleVersion <BUILD>" \
  $APP/Info.plist

# SDK and Xcode keys from the live toolchain.
XCODE_BUILD=$(xcodebuild -version | awk '/Build version/ {print $3}')
XCODE_VERSION=$(xcodebuild -version | awk 'NR==1 {gsub(/\./,"",$2); printf "%s0", $2}')
SDK_VERSION=$(xcrun --sdk iphoneos --show-sdk-version)
SDK_BUILD=$(xcrun --sdk iphoneos --show-sdk-build-version)
OS_BUILD=$(sw_vers -buildVersion)
for kv in \
  "DTXcode:string:$XCODE_VERSION" \
  "DTXcodeBuild:string:$XCODE_BUILD" \
  "DTSDKName:string:iphoneos$SDK_VERSION" \
  "DTSDKBuild:string:$SDK_BUILD" \
  "DTPlatformName:string:iphoneos" \
  "DTPlatformVersion:string:$SDK_VERSION" \
  "DTPlatformBuild:string:$SDK_BUILD" \
  "BuildMachineOSBuild:string:$OS_BUILD"; do
  KEY="${kv%%:*}"; REST="${kv#*:}"; TYPE="${REST%%:*}"; VAL="${REST#*:}"
  /usr/libexec/PlistBuddy -c "Delete :$KEY" $APP/Info.plist 2>/dev/null
  /usr/libexec/PlistBuddy -c "Add :$KEY $TYPE $VAL" $APP/Info.plist
done
```

## 3. Compile the icon catalog

dx never runs `actool`, so without this the bundle has no `Assets.car` and Apple rejects the upload with "Missing required icon file ... 120x120". `scripts/icon.py` writes every size this needs into `assets/favicon/`, and the catalog build is a script so the device route and this one stay identical:

```bash
scripts/ios/icons.sh $APP
```

It compiles the catalog, adds the `CFBundleIcons` keys, and fails loudly if no `Assets.car` came out. Safe to run twice on the same bundle.

If `actool` can't produce `Assets.car`, the iOS platform isn't installed: `xcodebuild -downloadPlatform iOS`.

## 4. Sign

```bash
security find-identity -v -p codesigning     # the "Apple Distribution" entry
cp ~/certs/Count_App_Store.mobileprovision $APP/embedded.mobileprovision
codesign --force --sign "<HASH-OR-NAME>" --entitlements Entitlements-Release.plist $APP
```

## 5. Package

```bash
rm -f Crow.ipa && mkdir -p Payload && cp -r $APP Payload/ && zip -r Crow.ipa Payload && rm -rf Payload
```

## 6. Upload and submit

Use Transporter from the Mac App Store: sign in, drag `Crow.ipa` in, Deliver. Not `xcrun altool` (deprecated, and its metadata errors trigger false "beta Xcode" rejections) and not `iTMSTransporter` (wants `.itmsp`, not `.ipa`).

The build appears in App Store Connect after five to ten minutes. Create the version if needed, select the build, answer No to export compliance (no encryption beyond the OS), submit. TestFlight first for anything you want on your own phone before review.

Then add a row to `history.md`.
