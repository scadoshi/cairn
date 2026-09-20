# First install on your own iPhone

The release device build already compiles (`dx build --release --platform
ios --device true` produces an 11 MB arm64 binary) and stops at code
signing: there is no provisioning profile for the bundle id yet. Everything
below is the paperwork between that build and the phone. Do the name first:
the bundle id in `Dioxus.toml` is permanent once Apple has it.

Signing identities already on this Mac, from zwipe:

- `Apple Development: SCOTTY RAY FERMO (NVSWB62C54)`
- `Apple Distribution: SCOTTY RAY FERMO (VV74WQ89GD)`

## One-time, in the developer portal

1. **App ID.** Certificates, Identifiers & Profiles, Identifiers, +. Explicit
   bundle id matching `[bundle] identifier` in `Dioxus.toml`. No capabilities
   needed yet; iCloud comes with CloudKit later.
2. **Register the phone.** Devices, +, the phone's UDID (Finder, click the
   device name under the model until the UDID shows). Only needed for the
   direct-install route below; TestFlight skips it.
3. **Profiles.** Two, both against the new App ID:
   - *Development*, with the phone selected, for direct installs. Save as
     `~/certs/Odo_Development.mobileprovision`.
   - *App Store*, for TestFlight and review. Save as
     `~/certs/Odo_App_Store.mobileprovision`.
4. Install both by double-clicking, or copy them into
   `~/Library/Developer/Xcode/UserData/Provisioning Profiles/`, which is
   where dx looks.

Then update the team prefix in `Entitlements.plist` and
`Entitlements-Release.plist` if it differs from zwipe's `VV74WQ89GD`.

## Route A: straight onto the phone, no TestFlight

Fastest way to start using it daily. Plug the phone in, trust the Mac, then:

```bash
dx build --release --platform ios --device true
APP=target/dx/notch/release/ios/Notch.app
cp ~/certs/Odo_Development.mobileprovision $APP/embedded.mobileprovision
codesign --force --sign "Apple Development: SCOTTY RAY FERMO (NVSWB62C54)" \
  --entitlements Entitlements.plist $APP
xcrun devicectl device install app --device <UDID> $APP
```

`xcrun devicectl list devices` shows the UDID once the phone is trusted. A
development-signed app runs for a year on registered devices only, which is
exactly the situation here.

## Route B: TestFlight

Follow `submission.md` through packaging, upload with Transporter, then in
App Store Connect add yourself as an internal tester on the build. Internal
TestFlight needs no review. Use this once the app should survive Xcode
updates and phone swaps without a rebuild.

## Data

The phone starts empty. To bring the real counts over, export the tap log
from the old app, run `scripts/import_taps.py` against a copy of the
database, and put that database in place through Finder's file sharing
(the app's Documents folder is visible there) or wait for the CSV import
that's on the backlog. Until then the Files app route only carries CSVs
out, not the database in.
