# Deploy a Build to Phone

Build and install Cairn on a connected iPhone. Same shape as zwipe's `operations/ios/dev_deploy.md`, because it is the same Mac, the same team, and the same already-registered phone.

Done on 22 September 2026: App ID, profile, and the first install with the imported counts. The one-time section below is kept for the next machine or the next time a profile expires.

---

## One-time: the App ID and the profile

Already done, and `Count_Development.mobileprovision` is installed in both `~/Library/Developer/Xcode/UserData/Provisioning Profiles/` and `~/certs/`. It expires 22 September 2027.

The phone was already registered from zwipe, UDID `00008140-00166D6C3482801C`, and the Apple Development certificate on this Mac is good until August 2027, so Devices and Certificates needed nothing. Do not create a second certificate: the profile carries both of the team's development certificates, and only one has its private key on this Mac.

1. developer.apple.com, Certificates, Identifiers & Profiles, Identifiers, **+**. Explicit App ID, description "Count", bundle id `com.scadoshi.count`. No capabilities. iCloud comes later if CloudKit sync happens.
2. Profiles, **+**, under Development pick **iOS App Development**. App ID `com.scadoshi.count`, the Apple Development certificate, the phone. Name it "Count Dev" and download it.
3. Install it where dx looks:

```bash
cp ~/Downloads/Count_Dev.mobileprovision ~/Library/Developer/Xcode/UserData/Provisioning\ Profiles/
```

Keep a copy at `~/certs/Count_Development.mobileprovision` alongside zwipe's, which is where `scripts/ios/install_device.sh` expects it.

---

## Build and deploy

```bash
scripts/ios/deploy.sh
```

That backs up the phone's database, builds, and installs. A failed backup stops the deploy, because a reinstall going wrong is exactly when the copy is wanted. Add `--release` for the store-parity build, or `--no-backup` to skip the copy, which is rarely what you want.

Reinstalling over an existing app keeps its data container, so an ordinary deploy does not touch the counts. Verified: a deploy on 22 September left 53,900 push-ups, 27,400 pull-ups and 16,110 squats exactly where they were.

The long way, if you want to see the pieces, pasted as one line because zsh mangles `\` continuations on paste:

```bash
cd ~/Developer/cairn && dx build --release --platform ios --device true && ios-deploy --bundle ~/Developer/cairn/target/dx/cairn/debug/ios/Cairn.app
```

dx signs the bundle itself from `~/Library/Developer/Xcode/UserData/Provisioning Profiles/Count_Development.mobileprovision`. There is no manual `codesign` step for dev builds.

The bundle folder, the crate and the home-screen label are all Cairn. They used to differ, which is why older notes mention re-serializing the plist to drop duplicate name keys; that is no longer needed.

For a release build with the icon catalog and explicit signing, `scripts/ios/install_device.sh`. Use that before a TestFlight upload, not for daily work.

## First launch

Developer Mode, on iOS 16 and up: Settings, Privacy & Security, Developer Mode, on, then restart the phone.

If iOS says "Untrusted Developer": Settings, VPN & Device Management, your Apple ID, Trust.

---

## Someone else's phone

[`testers.md`](testers.md). TestFlight for anyone not in the room, a second device on the development profile if they are.

## Backups

Cairn is in daily use while it is also being developed, so the phone holds taps that exist nowhere else. `scripts/ios/backup_db.sh` pulls the live database off and keeps it at `~/Developer/cairn-data/backups/count-YYYYMMDD-HHMMSS.db`, outside the repo, which gitignores `*.db` anyway.

`deploy.sh` runs it on every deploy. Run it on its own any time:

```bash
scripts/ios/backup_db.sh
```

It stops the app first so SQLite has closed the file, runs `pragma integrity_check` on what came off the phone, and refuses to keep a copy that fails. A pull matching the newest backup byte for byte is dropped rather than stored twice, so running it repeatedly costs nothing. It keeps the last 30 and prunes older ones, `--keep N` to change that.

## Restoring

```bash
scripts/ios/install_device.sh --db-only ~/Developer/cairn-data/backups/<file>
```

This replaces the phone's database wholesale, so anything logged since that backup is gone. It takes its own backup first, before overwriting. Force-quit and reopen Cairn afterwards.

It works because a development-signed build carries `get-task-allow`, which is what lets `devicectl` reach the app container. A TestFlight build is signed differently and this will not work there.

`~/Developer/cairn-data/original-import-20260921.db` is the one-time seed from the tap-log import, kept for the record. Restore from a backup instead, since the seed is older than the phone by whatever has been logged since.

