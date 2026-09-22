#!/usr/bin/env bash
# The daily loop: back up the phone's database, build, install.
#
# Count is a live app being iterated on, so the phone holds taps that exist
# nowhere else. The backup runs first and a failed backup stops the deploy,
# because a reinstall that goes wrong is exactly when the copy is wanted.
#
# Reinstalling over an existing app keeps its data container, so an ordinary
# deploy does not touch the counts. Restoring is a separate, deliberate step:
#   scripts/ios/install_device.sh --db-only ~/Developer/crow-data/backups/<file>
#
# Usage:
#   scripts/ios/deploy.sh              # backup, debug build, install
#   scripts/ios/deploy.sh --release    # same with a release build
#   scripts/ios/deploy.sh --no-backup  # skip the backup (rarely what you want)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=device.sh
. "$REPO_ROOT/scripts/ios/device.sh"

PROFILE=debug
BACKUP=1
while [ $# -gt 0 ]; do
  case "$1" in
    --release) PROFILE=release; shift ;;
    --no-backup) BACKUP=0; shift ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

find_device
echo "device: $DEVICE_NAME"

if [ "$BACKUP" -eq 1 ]; then
  "$REPO_ROOT/scripts/ios/backup_db.sh"
fi

if [ "$PROFILE" = release ]; then
  dx build --release --platform ios --device true
else
  dx build --platform ios --device true
fi

APP="$REPO_ROOT/target/dx/crow/$PROFILE/ios/Crow.app"
[ -d "$APP" ] || { echo "no app bundle at $APP" >&2; exit 1; }

ios-deploy --bundle "$APP"
echo "installed $PROFILE build on $DEVICE_NAME"
