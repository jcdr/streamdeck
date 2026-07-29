#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HELPERS_DIRECTORY="${PROJECT_ROOT}/helpers"
INSTALL_DIRECTORY="${DESTDIR:-}/usr/local/bin"

if [[ -z "${DESTDIR:-}" && "${EUID}" -ne 0 ]]; then
  exec sudo -- "${BASH_SOURCE[0]}" "$@"
fi

install -d -m 755 "${INSTALL_DIRECTORY}"
install -m 755 \
  "${HELPERS_DIRECTORY}/screenshoot-full" \
  "${HELPERS_DIRECTORY}/screenshoot-window" \
  "${HELPERS_DIRECTORY}/screenshoot-region" \
  "${INSTALL_DIRECTORY}/"

echo "Installed screenshot helpers to ${INSTALL_DIRECTORY}:"
ls -la \
  "${INSTALL_DIRECTORY}/screenshoot-full" \
  "${INSTALL_DIRECTORY}/screenshoot-window" \
  "${INSTALL_DIRECTORY}/screenshoot-region"
