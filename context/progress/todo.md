# Todo

Ordered. Top is next.

1. Device check of the first build from main with orangui, on the simulator and the phone. The first macOS build is also the first compile of the iOS `webview()` helper, so watch that build before anything else.
    - iOS edge swipe while a list is still gliding goes back at once; it closes only a sheet that is open, and does nothing on home.
    - Dialogs (take N off, delete, hints, about) have the kit's padding and a tertiary-accent title, and fade out on close.
    - The heat tip glides between cells.
    - The counter chart draws a 1.6 line, and its overlay starts after a short delay.
    - Config lists, pref notes and stat chips look right.
2. Draw the Cairn mark properly. `assets/c.txt` is the home-screen logo and the source `scripts/icon.py` renders the app icon from; `assets/s.txt` is the scadoshi mark kept behind the Mark setting. The pipeline is verified, only the artwork is pending.
3. TestFlight and App Store review. `operations/ios/submission.md`.
4. Edit a past day's count (today's +/- exists; history is read-only).
5. Export: pick a destination instead of always writing to Downloads.
6. A yearly heat strip per counter.
