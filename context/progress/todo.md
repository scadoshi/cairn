# Todo

Ordered. Top is next.

1. Register the App ID `com.scadoshi.count`, the phone's UDID, and the two
   profiles in the developer portal, then run
   `scripts/ios/install_device.sh --db ~/Developer/crow-data/count.db`.
   `operations/ios/first_device.md`.
2. Redraw the wordmark as "scadoshi" (`assets/crow.txt`) and regenerate the
   icon from it with `scripts/icon.py`. The icon pipeline is verified; only
   the artwork is pending.
3. TestFlight and App Store review. `operations/ios/submission.md`.
4. Edit a past day's count (today's +/- exists; history is read-only).
5. Export: pick a destination instead of always writing to Downloads.
6. A yearly heat strip per counter.
