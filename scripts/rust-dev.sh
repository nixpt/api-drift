#!/usr/bin/env bash
# Bootstrap this project's Rust dev environment (idempotent, self-contained).
#
#   source scripts/rust-dev.sh   # export CARGO_TARGET_DIR (+ sccache) into this shell
#   ./scripts/rust-dev.sh        # print the activation commands instead
#
# Works in two modes:
#   fleet   — nixpt-common is available ($NIXPT_COMMON, or a sibling
#             `nixpt-common/` directory up to four levels up): its shell/dev.sh
#             is sourced and drives everything, exactly as before.
#   generic — nixpt-common is absent (public clone, CI, contributor laptop):
#             the same defaults are applied inline. Nothing is required; plain
#             `cargo test` works without this script at all.
#
# Environment (both modes):
#   NIXPT_RUST_TARGET_DIR  explicit cargo target dir (highest priority)
#   CARGO_TARGET_DIR       honoured if already set
#   NIXPT_BUILD_BASE       per-agent target root; when set the target dir is
#                          $NIXPT_BUILD_BASE/${AGENT_NAME:-<user>-<project>}.
#                          When unset in generic mode, cargo's default ./target
#                          is used — no path outside the checkout is imposed.
#   RUSTC_WRAPPER          set to sccache when sccache is on PATH (opt out by
#                          exporting RUSTC_WRAPPER= empty before sourcing)
set -uo pipefail

_rd_info() { printf '%s\n' "$*" >&2; }
_rd_warn() { printf 'warn: %s\n' "$*" >&2; }

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

# Generic-mode equivalents of nixpt_rust_check / nixpt_rust_target_dir /
# nixpt_rust_dev_up / nixpt_dev_activate. Kept in step with
# nixpt-common/shell/dev.sh; change both when changing one.
generic_rust_check() {
    if ! command -v cargo >/dev/null 2>&1; then
        _rd_warn "cargo not found; install rustup from https://rustup.rs"
        return 1
    fi
    _rd_info "$(cargo --version 2>&1)"
    if command -v rustup >/dev/null 2>&1; then
        local missing=()
        rustup component list --installed 2>/dev/null | grep -q '^clippy' || missing+=(clippy)
        rustup component list --installed 2>/dev/null | grep -q '^rustfmt' || missing+=(rustfmt)
        if (( ${#missing[@]} > 0 )); then
            _rd_warn "missing rustup components: ${missing[*]} (rustup component add ${missing[*]})"
        fi
    fi
}

generic_rust_target_dir() {
    if [[ -n "${NIXPT_RUST_TARGET_DIR:-}" ]]; then
        printf '%s\n' "$NIXPT_RUST_TARGET_DIR"
    elif [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
        printf '%s\n' "$CARGO_TARGET_DIR"
    elif [[ -n "${NIXPT_BUILD_BASE:-}" ]]; then
        local project
        project="$(basename "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)")"
        printf '%s/%s\n' "$NIXPT_BUILD_BASE" "${AGENT_NAME:-$(id -un)-$project}"
    fi
    # Nothing set: print nothing; cargo's default ./target applies.
}

generic_rust_dev_up() {
    generic_rust_check || return 1
    local target
    target="$(generic_rust_target_dir)"
    if [[ -n "$target" ]]; then
        mkdir -p "$target"
        export CARGO_TARGET_DIR="$target"
    fi
    if [[ -z "${RUSTC_WRAPPER+x}" ]] && command -v sccache >/dev/null 2>&1; then
        export RUSTC_WRAPPER=sccache
    fi
    _rd_info "rust env ready: CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-<cargo default ./target>}${RUSTC_WRAPPER:+ RUSTC_WRAPPER=$RUSTC_WRAPPER}"
}

generic_dev_activate() {
    local target
    target="$(generic_rust_target_dir)"
    [[ -n "$target" ]] && printf 'export CARGO_TARGET_DIR="%s"\n' "$target"
    if command -v sccache >/dev/null 2>&1; then
        printf 'export RUSTC_WRAPPER=sccache\n'
    fi
    return 0
}

if COMMON="$(find_common)"; then
    # shellcheck source=/dev/null
    source "$COMMON/shell/dev.sh"
    _up() { nixpt_rust_dev_up >/dev/null; }
    _activate() { nixpt_dev_activate; }
else
    _rd_info "rust-dev: nixpt-common not found; using built-in defaults (set NIXPT_COMMON to use the fleet script)"
    _up() { generic_rust_dev_up; }
    _activate() { generic_dev_activate; }
fi

if [[ "${BASH_SOURCE[0]}" != "${0}" ]]; then
    # Sourced: run in the caller's shell so the exports persist. Never abort
    # the caller's shell on a missing toolchain — warn and return.
    _up || true
    echo "rust env active: CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-<cargo default ./target>}"
else
    set -e
    _up
    _activate
fi
