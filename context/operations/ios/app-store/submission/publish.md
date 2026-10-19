# Publish to the App Store (upload + submit)

Takes the signed `Cairn.ipa` from [build.md](build.md) and gets it into review.

## 1. Upload via Transporter


Use Transporter from the Mac App Store: sign in, drag `Cairn.ipa` in, Deliver. Not `xcrun altool` (deprecated, and its metadata errors trigger false "beta Xcode" rejections) and not `iTMSTransporter` (wants `.itmsp`, not `.ipa`).

The build appears in App Store Connect after five to ten minutes. Create the version if needed, select the build, answer No to export compliance (no encryption beyond the OS), submit. TestFlight first for anything you want on your own phone before review.

```bash
open -a Transporter ~/Developer/cairn/Cairn.ipa
```

## 2. Submit

1. App Store Connect, Cairn Counter, the build shows up after five to ten minutes
2. Create the version if it is not there yet (the **+** next to iOS App)
3. Select the build
4. Export compliance: **No**
5. Paste the version's "What's New" from [`../../../store-submissions/<version>/whats_new.md`](../../../store-submissions/README.md)
6. **Submit for Review**

## 3. Afterwards

Add the row to [history.md](history.md): version, build number, what changed, the date it was submitted. Listing copy (subtitle, description, keywords) lives in [form_fields.md](form_fields.md); keep it in step with what is live.
