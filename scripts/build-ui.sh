#!/usr/bin/env bash
# Builds the Rust/WebAssembly interface into ui/dist. Requires the wasm32-unknown-unknown target
# and wasm-bindgen-cli at the version pinned in ui/Cargo.toml (cargo install wasm-bindgen-cli --version 0.2.130 --locked).
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

cargo build --release --target wasm32-unknown-unknown -p aion2-ui
rm -rf ui/dist
wasm-bindgen --target web --out-dir ui/dist target/wasm32-unknown-unknown/release/aion2_ui.wasm
cp ui/index.html ui/dist/index.html
cp style.css ui/dist/style.css
echo "built ui/dist"
