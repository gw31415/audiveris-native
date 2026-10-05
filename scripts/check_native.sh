#!/bin/sh
# SPDX-License-Identifier: AGPL-3.0-or-later
# Native-only verification. No Java, oracle regeneration, publication or global setup.
set -eu
cd "$(dirname "$0")/.."
cargo fmt --all --check
cargo clippy --workspace --locked --all-targets -j 2 -- -D warnings
cargo test --workspace --locked -j 2
cargo test --workspace --locked --release -j 2
cargo build --workspace --locked --release -j 2
