#!/usr/bin/env bash
set -euo pipefail
bash "$(dirname "${BASH_SOURCE[0]}")/scripts/build-ui.sh"
cargo build --release --target x86_64-pc-windows-gnu "$@"
