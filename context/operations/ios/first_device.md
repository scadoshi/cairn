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
     `~/certs/Count_Development.mobileprovision`.
   - *App Store*, for TestFlight and review. Save as
     `~/certs/Count_App_Store.mobileprovision`.
4. Install both by double-clicking, or copy them into
   `~/Library/Developer/Xcode/UserData/Provisioning Profiles/`, which is
   where dx looks.

Then update the team prefix in `Entitlements.plist` and
`Entitlements-Release.plist` if it differs from zwipe's `VV74WQ89GD`.

## Route A: straight onto the phone, no TestFlight

Fastest way to start using it daily. Plug the phone in, trust the Mac, then
one command does build, icons, signing and install:

```bash
scripts/ios/install_device.sh --db ~/Developer/crow-data/count.db
```

It stops with a clear message if the phone is not tethered or the
development profile is missing. What it does, in order:

1. `dx build --release --platform ios --device true`, an arm64 binary in
   `target/dx/crow/release/ios/Crow.app` (the folder keeps the crate name;
   the label inside is Count).
2. `plutil -convert xml1` on Info.plist. dx writes the bundle name from the
   crate and Dioxus.toml writes it again as Count, so the plist holds each
   name key twice; re-serializing keeps the last, which is Count.
3. `scripts/ios/icons.sh`, which runs `actool` over the PNGs in
   `assets/favicon` and adds the `CFBundleIcons` keys. dx never does this,
   and without it the phone shows a blank icon.
4. Embeds the profile, signs with the development identity and
   `Entitlements.plist`, verifies the signature.
5. `xcrun devicectl device install app`.
6. With `--db`, pushes that database into the app container (see Data).

A development-signed app runs for a year on registered devices only, which
is exactly the situation here. `xcrun devicectl list devices` shows the
phone once it is trusted.

## Route B: TestFlight

Follow `submission.md` through packaging, upload with Transporter, then in
App Store Connect add yourself as an internal tester on the build. Internal
TestFlight needs no review. Use this once the app should survive Xcode
updates and phone swaps without a rebuild.

## Data

The phone starts empty unless a database is pushed to it. The real one, the
simulator's after the tap-log import, is kept outside the repo at
`~/Developer/crow-data/count.db` (the repo gitignores `*.db`). Refresh that
copy from the simulator with:

```bash
SIM=$(xcrun simctl get_app_container booted com.scadoshi.count data)
cp "$SIM/Library/Application Support/scadoshi-count/count.db" \
   ~/Developer/crow-data/count.db
```

`install_device.sh --db <path>` pushes it with `devicectl device copy to`
into `Library/Application Support/scadoshi-count/count.db` inside the app
container. That works because a development-signed build carries
`get-task-allow`, which is what lets devicectl reach the container at all.
Push it while the app is not running, then open Count.

`--db-only` skips the build and pushes just the database, for when the app
is already installed.

Two things this route cannot do, both of which need the in-app import that
is on the backlog:

- A TestFlight or App Store build is not development-signed, so devicectl
  cannot write into its container. Only Documents is reachable there, over
  Finder file sharing.
- Nothing merges. The push replaces the database wholesale, so counts
  logged on the phone since the last push are lost. Treat it as a one-time
  seed, not a sync.
