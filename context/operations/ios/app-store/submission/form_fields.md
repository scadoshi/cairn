# App Store Connect Copy

Text to paste into App Store Connect fields, kept here so what is live is in the repo. Same sections as zwipe's `form_fields.md`. Update a field here the day it changes in the console.

Release-note rule: keep every "What's New" within 500 characters. Cairn is iOS only today, but the same text would go to Play unchanged if an Android build ever ships, and the rule keeps the notes short regardless.

---

## Name (max 30 characters)

Cairn Counter

"Cairn" alone was taken when the app record was created on 2026-10-18. The home-screen name stays Cairn; that comes from the bundle's display name, not this field.

---

## Subtitle (max 30 characters)

A lifetime counter

---

## Promotional Text

(170 character limit. Can be changed without a new submission; refresh it between releases.)

Count the things you do, for life. One tap a day, and the total breaks down by year so it means something. Everything stays on your phone.

---

## Description

Cairn counts the things you do, for life. Name a counter, log how many you did today, and the number goes up and stays up.

A lifetime total on its own is trivia. So every counter also shows this year's total, the average per day, what day of the year it is, and, if you set a goal, whether you're ahead or behind and what today needs to be to catch up.

Built to track pull-ups and push-ups, and it counts anything that happens in whole numbers: pages read, miles run, days without.

- Today across every counter on one screen, then a quote
- Sixty days of history and a heatmap for every counter
- Goals with pace: ahead, behind, and the number today needs
- Rest days that don't count against you
- A celebration when you hit a goal, your pick of several
- 31 color themes, dark and light, including colorblind-accessible ones
- Export is a CSV of two columns, day and count, because that's all the raw data is

Everything lives on your phone. No account, no server, nothing leaves the device.

---

## What's New in This Version

Lives in [`../../../store-submissions/`](../../../store-submissions/README.md), one directory per version. Paste from `store-submissions/<version>/whats_new.md`.

---

## Keywords (max 100 characters, commas included)

The field counts the separators, so a comma costs as much as a letter. This is 80. Do not pad it to the limit: a term added later has to fit.

counter,tally,pull-ups,push-ups,habit,streak,lifetime,tracker,goal,daily,workout

---

## Other Fields

| Field | Value |
|-------|-------|
| Support URL | https://scottyfermo.com/side-quests/cairn |
| Marketing URL | https://scottyfermo.com/side-quests/cairn |
| Privacy Policy URL | https://scottyfermo.com/projects/cairn/privacy (portfolio `src/pages/privacy.rs`; Apple requires one for every app, collecting or not) |
| Version | 1.0.0 (must match the build's `CFBundleShortVersionString`; the console creates the version as "1.0", change it) |
| Copyright | 2026 Scotty Fermo |
| Routing App Coverage File | (leave empty) |

---

## App Review

### Sign-In Information

No account. Leave sign-in off.

### Notes

A personal counter with no account and no network. Open the app, tap Add to make a counter, then + on it to log today's count. Config has the settings, each with a ? that explains it. Nothing in the app needs a connection.

---

## App Information

| Field | Value |
|-------|-------|
| Category | Health & Fitness |
| Secondary Category | Productivity |
| Age Rating | None for every question: 4+ |
| Content Rights | Yes, necessary rights held: the 72 one-line quotes, each attributed to its speaker and source |
| Pricing | Free |
| App Store Version Release | Automatically release |

---

## App Privacy

Data Not Collected. Nothing leaves the device; there is no account, no analytics and no network call.

---

## Screenshots

The four in the README (`docs/screenshots/`: home, counters, counter, config) were taken on the phone. The listing wants the 6.5" size (1242×2688), so retake them on the iPhone 11 Pro Max simulator, the same device zwipe uses (`zwipe/context/operations/ios/simulator.md`), and keep them in `screenshots/` here. From April 2027 Apple also wants iPhone Duo screenshots for apps on the iOS 27.1 SDK or later.
