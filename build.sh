#!/usr/bin/env bash
set -euo pipefail
bash "$(dirname "${BASH_SOURCE[0]}")/scripts/build-ui.sh"
cargo build --release --target x86_64-pc-windows-gnu "$@"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(wslpath -w "$(dirname "${BASH_SOURCE[0]}")/scripts/sign-windows.ps1")" \
  -BinaryPath "$(wslpath -w target/x86_64-pc-windows-gnu/release/AION2-Damage-Calculator.exe)" \
  -CertificatePath "$(wslpath -w target/x86_64-pc-windows-gnu/release/AION2-Damage-Calculator.crt)"
