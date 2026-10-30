$ErrorActionPreference = 'Stop'
# shortcut: paths are fixed to this WSL/Windows setup; change them if the project moves.
$source = '/home/avatar/gui-dps-aion2/'
$destination = 'C:\dev\gui-dps-aion2'
$destinationWsl = '/mnt/c/dev/gui-dps-aion2/'

New-Item -ItemType Directory -Path $destination -Force | Out-Null
wsl.exe -d CachyOS --exec bash -l "${source}scripts/build-ui.sh"
if ($LASTEXITCODE -ne 0) { throw 'Could not build the WebAssembly interface in WSL.' }
# shortcut: no --delete preserves caches and prior builds; remove stale source files manually if needed.
wsl.exe -d CachyOS --exec rsync -rt --checksum `
    --exclude=.git/ --exclude=.claude/ --exclude=target/ --exclude=node_modules/ `
    --exclude=/dist/ --exclude=scripts/ --exclude=docs/ `
    --exclude='*.onnx' --exclude='*.onnx_data' --exclude='*.download' `
    -- $source $destinationWsl
if ($LASTEXITCODE -ne 0) { throw 'Could not copy the project from WSL.' }

Push-Location $destination
try {
    cargo build --release -j 1
    if ($LASTEXITCODE -ne 0) { throw 'cargo build failed.' }

    # Authenticode SHA-256 signing with DigiCert timestamp
    $signtool = 'C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe'
    $cert = Join-Path $destination 'certs\authenticode.pfx'
    $binary = Join-Path $destination 'target\release\AION2-Damage-Calculator.exe'
    if ((Test-Path $signtool) -and (Test-Path $cert)) {
        Write-Host "Signing executable with Authenticode SHA-256 and DigiCert timestamp..."
        & $signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /f $cert /p 'aion2' $binary
        if ($LASTEXITCODE -ne 0) { throw 'signtool signing failed.' }
    }

    $version = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).packages[0].version
    if ($LASTEXITCODE -ne 0) { throw 'Could not read the Cargo version.' }
    $output = Join-Path $destination "dist\v$version"
    New-Item -ItemType Directory -Path $output -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $destination 'target\release\AION2-Damage-Calculator.exe') -Destination $output -Force
    Write-Host "Built: $(Join-Path $output 'AION2-Damage-Calculator.exe')"
} finally {
    Pop-Location
}
