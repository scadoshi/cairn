# Versioning

The rule, borrowed from zwipe:

- **Minor bump (0.1 to 0.2): the release carries any feature.** A new screen,
  a new metric, a new behavior you can see, however small.
- **Patch bump (0.2.0 to 0.2.1): fixes, UI tweaks, copy, polish only.** A
  patch number tells you "nothing new, just better".
- **Major** is for a product-shape change. 1.0 is the first build that goes
  on a real phone and stays there.

Mechanics: the version lives in `Cargo.toml`. It bumps at cut time, in the
same commit that moves the changelog's Unreleased entries under a dated
version heading in `context/progress/changelog.md`. iOS build numbers
increment on every upload, independently of the version, and are recorded in
`context/operations/ios/history.md`.
