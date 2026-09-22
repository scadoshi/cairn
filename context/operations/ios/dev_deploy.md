# Deploy a Build to Phone

Build and install Count on a connected iPhone. Same shape as zwipe's `operations/ios/dev_deploy.md`, because it is the same Mac, the same team, and the same already-registered phone.

**Prerequisite:** a development provisioning profile for `com.scadoshi.count`. Without it `dx build` stops at "No provisioning profile found matching bundle identifier". Make it once, below.

---

## One-time: the App ID and the profile

The phone is already registered. zwipe's development profile lists UDID `00008140-00166D6C3482801C`, and the Apple Development certificate on this Mac is good until August 2027, so Devices and Certificates both need nothing.

1. developer.apple.com, Certificates, Identifiers & Profiles, Identifiers, **+**. Explicit App ID, description "Count", bundle id `com.scadoshi.count`. No capabilities. iCloud comes later if CloudKit sync happens.
2. Profiles, **+**, under Development pick **iOS App Development**. App ID `com.scadoshi.count`, the Apple Development certificate, the phone. Name it "Count Dev" and download it.
3. Install it where dx looks:

```bash
cp ~/Downloads/Count_Dev.mobileprovision ~/Library/Developer/Xcode/UserData/Provisioning\ Profiles/
```

Keep a copy at `~/certs/Count_Development.mobileprovision` alongside zwipe's, which is where `scripts/ios/install_device.sh` expects it.

---

## Build and deploy

One line. Paste it as one line: zsh mangles `\` continuations on paste and splits the arguments into their own commands.

```bash
cd ~/Developer/crow && dx build --platform ios --device true && ios-deploy --bundle ~/Developer/crow/target/dx/crow/debug/ios/Crow.app
```

dx signs the bundle itself from the profile in the directory above. There is no manual `codesign` step for dev builds.

The bundle folder is `Crow.app` after the crate. The app's own label is Count, which is what shows on the home screen.

For the store-parity build instead, `scripts/ios/install_device.sh` does a release build with the icon catalog and explicit signing. Use that before a TestFlight upload, not for daily work.

---

## First launch

Developer Mode, on iOS 16 and up: Settings, Privacy & Security, Developer Mode, on, then restart the phone.

If iOS says "Untrusted Developer": Settings, VPN & Device Management, your Apple ID, Trust.

---

## Bringing the real counts over

The phone starts empty. The database with nine months of imported taps sits outside the repo at `~/Developer/crow-data/count.db`, since the repo gitignores `*.db`.

```bash
scripts/ios/install_device.sh --db-only ~/Developer/crow-data/count.db
```

That opens Count once so it creates its data directory, stops it again, pushes the file into the app container, and reads it back to check the checksums match. Stopping it first matters: a live app holds the old file open and would write its stale copy back over the restore. Force-quit and reopen Count afterwards. It works because a development-signed build carries `get-task-allow`, which is what lets `devicectl` reach the container at all. A TestFlight build is signed differently and this will not work there.

The push replaces the database wholesale. Anything logged on the phone since the last push is gone, so treat it as a one-time seed rather than a sync. To refresh the staged copy from the simulator first:

```bash
SIM=$(xcrun simctl get_app_container booted com.scadoshi.count data)
cp "$SIM/Library/Application Support/scadoshi-count/count.db" ~/Developer/crow-data/count.db
```

---

## Why `--device true`

Without it, `dx build --platform ios` targets the simulator, and a simulator binary dies on real hardware with "wrong platform to load into process". Check the metadata if a build behaves oddly:

```bash
vtool -show ~/Developer/crow/target/dx/crow/debug/ios/Crow.app/crow
# want LC_VERSION_MIN_IPHONEOS, not platform 7 or MACOS
```

A release build lands in `release/ios/`, not `debug/ios/`. Deploying the debug path after a release build installs the stale bundle and fails with `Error 0xe8008014: The executable contains an invalid signature`.
