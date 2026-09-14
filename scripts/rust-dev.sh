#!/usr/bin/env bash
# Bootstrap this project's Rust dev environment (idempotent).
#
#   source scripts/rust-dev.sh   # export CARGO_TARGET_DIR (+ sccache) into this shell
#   ./scripts/rust-dev.sh        # print the activation commands instead
#
# Locates nixpt-common via $NIXPT_COMMON, or a sibling `nixpt-common/` directory.
set -euo pipefail

find_common() {
    if [[ -n "${NIXPT_COMMON:-}" && -f "$NIXPT_COMMON/shell/dev.sh" ]]; then
        printf '%s\n' "$NIXPT_COMMON"
        return 0
    fi
    local d
    d="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    local _
    for _ in 1 2 3 4; do
        if [[ -f "$d/nixpt-common/shell/dev.sh" ]]; then
            printf '%s\n' "$d/nixpt-common"
            return 0
        fi
        d="$(dirname "$d")"
    done
    return 1
}

COMMON="$(find_common)" || {
    echo "rust-dev: nixpt-common not found; set NIXPT_COMMON to its path" >&2
    exit 1
}

# shellcheck source=/dev/null
source "$COMMON/shell/dev.sh"

if [[ "${BASH_SOURCE[0]}" != "${0}" ]]; then
    # Sourced: run in the caller's shell so the exports persist.
    nixpt_rust_dev_up >/dev/null
    echo "rust env active: CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-unset}"
else
    nixpt_rust_dev_up >/dev/null
    nixpt_dev_activate
fi
