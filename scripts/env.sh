#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PKG_CONFIG_PATH="${PROJECT_ROOT}/.deps/pkgconfig${PKG_CONFIG_PATH:+:${PKG_CONFIG_PATH}}"
export C_INCLUDE_PATH="${PROJECT_ROOT}/.deps/include${C_INCLUDE_PATH:+:${C_INCLUDE_PATH}}"
export CPATH="${PROJECT_ROOT}/.deps/include${CPATH:+:${CPATH}}"
export LIBRARY_PATH="${PROJECT_ROOT}/.deps/lib${LIBRARY_PATH:+:${LIBRARY_PATH}}"
export RUSTFLAGS="-L native=${PROJECT_ROOT}/.deps/lib ${RUSTFLAGS:-}"
