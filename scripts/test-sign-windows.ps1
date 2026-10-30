param([Parameter(Mandatory)][string]$BinaryPath)
$ErrorActionPreference = 'Stop'
$directory = Join-Path ([IO.Path]::GetTempPath()) ([Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $directory | Out-Null
$binary = Join-Path $directory 'signed.exe'
$certificate = Join-Path $directory 'signer.crt'
$before = @(Get-ChildItem Cert:\CurrentUser\My | Where-Object Subject -eq 'CN=AION 2 Damage Calculator').Count
try {
    Copy-Item -LiteralPath $BinaryPath -Destination $binary
    & (Join-Path $PSScriptRoot 'sign-windows.ps1') -BinaryPath $binary -CertificatePath $certificate
    $after = @(Get-ChildItem Cert:\CurrentUser\My | Where-Object Subject -eq 'CN=AION 2 Damage Calculator').Count
    if ($before -ne $after) { throw 'Signing left a personal-store certificate behind.' }
    $public = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2($certificate)
    if ($public.HasPrivateKey) { throw 'The exported certificate contains a private key.' }
    $signature = Get-AuthenticodeSignature -LiteralPath $binary
    if ($signature.SignerCertificate.Thumbprint -ne $public.Thumbprint) { throw 'Wrong exported certificate.' }
    $untouched = [AionSignatureVerifier]::Verify($binary)
    if ($untouched -ne 0 -and $untouched -ne -2146762487) { throw 'Signed executable failed integrity verification.' }
    $bytes = [IO.File]::ReadAllBytes($binary)
    if ($bytes.Length -le 4096) { throw 'Executable is too small for the tamper probe.' }
    $bytes[4096] = $bytes[4096] -bxor 1
    [IO.File]::WriteAllBytes($binary, $bytes)
    $tampered = [AionSignatureVerifier]::Verify($binary)
    if ($tampered -eq 0 -or $tampered -eq -2146762487) { throw 'Tampered executable was incorrectly accepted.' }
    Write-Host ('PASS: signer, timestamp, public-only certificate, key cleanup; tampered EXE rejected (0x{0:X8}).' -f $tampered)
} finally {
    Remove-Item -LiteralPath $directory -Recurse -Force
}
