#!/usr/bin/env bash
# Deploy a debug build. The daily one: fast to build, and the only kind the
# database scripts can reach, since a debug build carries get-task-allow.
#
# Usage: scripts/ios/deploy_debug.sh [--device NAME] [--no-backup]
set -euo pipefail
exec "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/deploy.sh" "$@"
