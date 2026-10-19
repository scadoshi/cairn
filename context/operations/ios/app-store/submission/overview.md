# iOS App Store submission

The parts of shipping a Cairn build to the App Store, in order. Same layout as zwipe's `operations/ios/app-store/submission/`, so the two runbooks read the same.

| Part | What it covers |
|------|----------------|
| [build.md](build.md) | Build the release `.ipa`: dx build, Info.plist patches, icons, sign, package. **The recurring runbook.** |
| [publish.md](publish.md) | Upload via Transporter, select the build, paste What's New, submit for review. |
| [form_fields.md](form_fields.md) | Store listing copy: name, subtitle, description, keywords, URLs. |
| [history.md](history.md) | Per-release build log. |

One-time setup (the App ID, the distribution certificate, the App Store profile) is zwipe's `setup.md`; the account and certificate are shared. The store notes per version live in [`../../../store-submissions/`](../../../store-submissions/README.md). Dev-device work is not submission: [`../../dev_deploy.md`](../../dev_deploy.md), [`../../testers.md`](../../testers.md), [`../../first_device.md`](../../first_device.md).

Recurring update = [build.md](build.md) then [publish.md](publish.md), then a row in [history.md](history.md).
