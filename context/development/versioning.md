# Versioning

The rule, borrowed from zwipe:

- Minor bump (0.1 to 0.2): the release carries any feature. A new screen, a new metric, a new behavior you can see, however small.
- Patch bump (0.2.0 to 0.2.1): fixes, UI tweaks, copy, polish only. A patch number tells you "nothing new, just better".
- **Major** is for a product-shape change. 1.0 is the first build that goes on a real phone and stays there.

Mechanics: the version lives in `Cargo.toml`. It bumps at cut time, in the same commit that moves the changelog's Unreleased entries under a dated version heading in `context/progress/changelog.md`. iOS build numbers increment on every upload, independently of the version, and are recorded in `context/operations/ios/app-store/submission/history.md`.

The public changelog is portfolio's `src/changelog.rs`, rendered at scottyfermo.com/projects/cairn/changelog; the app makes no network requests, so the site is the only place it shows. At cut time the version goes there too, under `UPCOMING` until the App Store releases it, then to the top of `RELEASES` with the "Ready for Distribution" date. Entries there follow zwipe's changelog rules: phrases with no period, no examples, no em dashes, and its tests hold them.
