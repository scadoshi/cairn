# Getting Count onto someone else's phone

Two routes. Which one depends on whether the person is standing next to you.

## TestFlight, for anyone not in the room

The right answer for a tester who will keep using it and sending back ideas. They need an email address and nothing else: no UDID, no cable, no Mac, and no trip back to you when the build changes. Apple pushes updates to them the way any app updates.

External testers, up to 10,000 of them, never touch the Apple Developer account. The cost is a Beta App Review on the first build of each version, usually under a day and much lighter than a full App Store review. Internal testers skip review entirely, but they have to be added as users on the App Store Connect team, which hands a friend access to the account. Not worth it for one person.

What has to exist first, none of which does yet:

1. An **Apple Distribution certificate**. Already on this Mac, `Apple Distribution: SCOTTY RAY FERMO (VV74WQ89GD)`.
2. An **App Store provisioning profile** for `com.scadoshi.count`. Profiles, +, Distribution, App Store Connect. Save it as `~/certs/Count_App_Store.mobileprovision`.
3. An **app record** in App Store Connect. Apps, +, iOS. Name "Scadoshi Count", primary language, the bundle id, any SKU. The name is reserved once this exists.
4. An **app icon**. Apple rejects uploads without one. `scripts/ios/icons.sh` handles the mechanics and the placeholder mark passes; the artwork can change later.

Then follow [`submission.md`](submission.md) end to end: build, patch the plist, compile the icon catalog, sign with the distribution identity and `Entitlements-Release.plist`, package the `.ipa`, upload with Transporter.

In App Store Connect, TestFlight tab, external group, add their email. They get a link, install Apple's TestFlight app, and Count appears in it.

Their phone starts empty, which is what you want from someone looking for rough edges with fresh eyes. The database push in [`dev_deploy.md`](dev_deploy.md) will not work for them: a TestFlight build is signed without `get-task-allow`, so `devicectl` cannot reach its container. Moving data onto a TestFlight build needs the in-app import on the backlog.

## Direct install, if the phone is in your hand

Faster when they are physically present. No review wait, no App Store Connect at all. The catch is that it only works with their phone plugged into this Mac, so every future build means another visit.

1. Get the UDID. Plug their phone in, then `xcrun devicectl list devices`, or Xcode's Window, Devices and Simulators.
2. developer.apple.com, Devices, +, iOS, a recognizable name, paste the UDID.
3. Profiles, edit "Count Development", tick their device as well as yours, regenerate, download.
4. Reinstall it over the old one:

```bash
cp ~/Downloads/Count_Development.mobileprovision ~/Library/Developer/Xcode/UserData/Provisioning\ Profiles/
cp ~/Downloads/Count_Development.mobileprovision ~/certs/
```

5. With their phone tethered, name it:

```bash
scripts/ios/deploy.sh --device matthew --no-backup
```

The selector matches any part of the phone's name or its UDID. Both phones can stay plugged in; without `--device` the script refuses rather than guessing. Use `--no-backup` for someone else's phone unless you actually want a copy of their counts, and note that backups are filed per phone under `~/Developer/crow-data/backups/<phone>/` so a tester's data can never be restored over yours.

6. On their phone: Settings, Privacy & Security, Developer Mode, on, then restart. If iOS says "Untrusted Developer", Settings, VPN & Device Management, the Apple ID, Trust.

Done once already, for Matthew's iPhone 17 Pro on 22 September 2026.

A development profile covers 100 devices per membership year. Removing a device does not free its slot until the membership renews, so do not burn them casually.

Regenerating the profile does not invalidate your own install. The rebuilt profile still lists your phone, and the app on it keeps running on the profile already embedded in the installed bundle.
