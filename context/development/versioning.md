# Versioning

Version numbers read the way most people read them:

- **Minor** bump (1.1 to 1.2): the release adds something you can see. Any feature counts, however small: a new screen, a new metric, a new theme, a new behavior.
- **Patch** bump (1.2.0 to 1.2.1): fixes and polish only. A patch number means nothing new, just better.
- **Major** bump: a change to what the product is. That one is the owner's call. 1.0 was the first build that went on a real phone and stayed there.

History: this plain rule was set on 2026-10-23. Versions already shipped keep their numbers, and nothing is renumbered. The rule applies from the next release on.

Mechanics: the version lives in `Cargo.toml`. It bumps at cut time, in the same commit that moves the changelog's Unreleased entries under a dated version heading in `context/progress/changelog.md`. The date on that heading is the day App Store Connect shows the version "Ready for Distribution", when it went live, and the portfolio's `RELEASES` entry uses the same date. iOS build numbers increment on every upload, independently of the version, and are recorded in `context/operations/ios/app-store/submission/history.md`.

The public changelog is portfolio's `src/changelog.rs`, rendered at scottyfermo.com/projects/cairn/changelog; the app makes no network requests, so the site is the only place it shows. Every entry added under Unreleased here goes there in the same sitting, in `UPCOMING` under the next version, with `date: None`, so the site never lags the repo. At cut time, in the same sitting as the dated heading here, the version moves to the top of `RELEASES` with the same day written as a typed date, `date: Some(Date::new(2026, 10, 21))`, and reads as released. The page groups releases into months by that date, and a test fails if `RELEASES` is not newest first. Entries there follow zwipe's changelog rules: phrases with no period, no examples, no em dashes, and its tests hold them.
