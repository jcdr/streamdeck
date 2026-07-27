#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UNIT_SOURCE="${PROJECT_ROOT}/systemd/streamdeck-starship.service"
UNIT_DESTINATION_DIRECTORY="${HOME}/.config/systemd/user"
UNIT_DESTINATION="${UNIT_DESTINATION_DIRECTORY}/streamdeck-starship.service"

cd "${PROJECT_ROOT}"
cargo build --release

mkdir -p "${UNIT_DESTINATION_DIRECTORY}"
cp "${UNIT_SOURCE}" "${UNIT_DESTINATION}"

systemctl --user daemon-reload
systemctl --user enable --now streamdeck-starship.service
systemctl --user status streamdeck-starship.service --no-pager
