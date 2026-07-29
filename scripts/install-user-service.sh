#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UNIT_TEMPLATE="${PROJECT_ROOT}/systemd/streamdeck-starship.service.in"
UNIT_DESTINATION_DIRECTORY="${HOME}/.config/systemd/user"
UNIT_DESTINATION="${UNIT_DESTINATION_DIRECTORY}/streamdeck-starship.service"

cd "${PROJECT_ROOT}"
cargo build --release

mkdir -p "${UNIT_DESTINATION_DIRECTORY}"
sed "s|@PROJECT_ROOT@|${PROJECT_ROOT}|g" "${UNIT_TEMPLATE}" > "${UNIT_DESTINATION}"

systemctl --user daemon-reload
systemctl --user enable --now streamdeck-starship.service
systemctl --user status streamdeck-starship.service --no-pager

echo
echo "Installed user unit: ${UNIT_DESTINATION}"
echo "Project root baked into unit: ${PROJECT_ROOT}"
