param([string]$WslSource = '/home/avatar/gui-dps-aion2/')
$ErrorActionPreference = 'Stop'
# shortcut: paths are fixed to this WSL/Windows setup; change them if the project moves.
$source = $WslSource.TrimEnd('/') + '/'
$destination = 'C:\dev\gui-dps-aion2'
$destinationWsl = '/mnt/c/dev/gui-dps-aion2/'

New-Item -ItemType Directory -Path $destination -Force | Out-Null
wsl.exe -d CachyOS --exec bash -l "${source}scripts/build-ui.sh"
if ($LASTEXITCODE -ne 0) { throw 'Could not build the WebAssembly interface in WSL.' }
# shortcut: no --delete preserves caches and prior builds; remove stale source files manually if needed.
wsl.exe -d CachyOS --exec rsync -rt --checksum `
    --exclude=.git/ --exclude=.claude/ --exclude=target/ --exclude=node_modules/ `
    --exclude=/dist/ --exclude=docs/ --exclude=certs/ `
    --exclude='*.onnx' --exclude='*.onnx_data' --exclude='*.download' `
    -- $source $destinationWsl
if ($LASTEXITCODE -ne 0) { throw 'Could not copy the project from WSL.' }

Push-Location $destination
try {
    cargo build --release -j 1
    if ($LASTEXITCODE -ne 0) { throw 'cargo build failed.' }

    $binary = Join-Path $destination 'target\release\AION2-Damage-Calculator.exe'

    $version = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).packages[0].version
    if ($LASTEXITCODE -ne 0) { throw 'Could not read the Cargo version.' }
    $output = Join-Path $destination "dist\v$version"
    New-Item -ItemType Directory -Path $output -Force | Out-Null
    & (Join-Path $destination 'scripts\sign-windows.ps1') -BinaryPath $binary `
        -CertificatePath (Join-Path $output 'AION2-Damage-Calculator.crt')
    Copy-Item -LiteralPath (Join-Path $destination 'target\release\AION2-Damage-Calculator.exe') -Destination $output -Force
    Write-Host "Built: $(Join-Path $output 'AION2-Damage-Calculator.exe')"
} finally {
    Pop-Location
}
