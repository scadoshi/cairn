#!/usr/bin/env bash
# Deploy a release build: optimized, and what the App Store will run.
#
# Worth its own script because the profile picks the output directory as well
# as the compiler flags. Building with --release and then installing
# target/dx/cairn/debug/ios/Cairn.app by hand rebuilds everything and puts
# the previous debug bundle on the phone, which looks like a build that
# silently ignored your changes.
#
# Usage: scripts/ios/deploy_release.sh [--device NAME] [--no-backup]
set -euo pipefail
exec "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/deploy.sh" --release "$@"
