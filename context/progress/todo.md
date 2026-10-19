# Todo

Ordered. Top is next.

1. On the iPhone 16 Pro simulator (iOS 18.6) the page stops about three quarters down: the action bar sits at y=711 of 874 points and the rest is empty background. The iPhone 11 Pro Max fills its screen. The phone (an iPhone 16, iOS 26) fills its screen, checked 2026-10-18, so this is the iOS 18.6 simulator runtime, not the app. Low priority: only matters for taking screenshots at that size, and no iOS 26 Dynamic Island simulator is installed to try instead.
2. Confirm phone deploys ship the app icon. Builds from `scripts/ios/deploy.sh` showed a blank icon through 2026-10-18 because dx never runs actool. Since `016685b` the script runs `scripts/ios/icons.sh` and signs the bundle again, so the next `scripts/ios/deploy_debug.sh --device scotland-mobile` from a pulled main should carry the gruvbox icon and the 61 alternates. If it is still blank, check the re-sign with `codesign -dvv` on the installed bundle.
3. Device check of the first build from main with orangui, on the simulator and the phone. The first macOS build is also the first compile of the iOS `webview()` helper, so watch that build before anything else.
    - iOS edge swipe while a list is still gliding goes back at once; it closes only a sheet that is open, and does nothing on home.
    - Dialogs (take N off, delete, hints, about) have the kit's padding and a tertiary-accent title, and fade out on close.
    - The heat tip glides between cells.
    - The counter chart draws a 1.6 line, and its overlay starts after a short delay.
    - Config lists, pref notes and stat chips look right.
4. Draw the Cairn mark properly. `assets/c.txt` is the home-screen logo and the source `scripts/icon.py` renders the app icon from; `assets/s.txt` is the scadoshi mark kept behind the Mark setting. The pipeline is verified, only the artwork is pending.
5. Edit a past day's count (today's +/- exists; history is read-only).
6. Export: pick a destination instead of always writing to Downloads.
7. A yearly heat strip per counter.
