param(
    [Parameter(Mandatory)][string]$BinaryPath,
    [Parameter(Mandatory)][string]$CertificatePath
)
$ErrorActionPreference = 'Stop'
$binary = (Resolve-Path -LiteralPath $BinaryPath).ProviderPath
$signtool = Get-Command signtool.exe -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source -First 1
if (!$signtool) {
    $signtool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" |
        Sort-Object { [version]$_.Directory.Parent.Name } -Descending |
        Select-Object -ExpandProperty FullName -First 1
}
if (!$signtool) { throw 'Install the Windows SDK Signing Tools (signtool.exe).' }

# WinVerifyTrust checks the PE digest; only the fresh self-signed root may be untrusted.
if (!('AionSignatureVerifier' -as [type])) {
    Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class AionSignatureVerifier {
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    struct FileInfo {
        public uint Size;
        [MarshalAs(UnmanagedType.LPWStr)] public string Path;
        public IntPtr Handle, Subject;
    }
    [StructLayout(LayoutKind.Sequential)]
    struct TrustData {
        public uint Size;
        public IntPtr Policy, Sip;
        public uint UI, Revocation, Choice;
        public IntPtr File;
        public uint StateAction;
        public IntPtr State, Url;
        public uint Flags, Context;
        public IntPtr Settings;
    }
    [DllImport("wintrust.dll", ExactSpelling = true)]
    static extern int WinVerifyTrust(IntPtr window, ref Guid action, ref TrustData data);
    public static int Verify(string path) {
        var file = new FileInfo { Size = (uint)Marshal.SizeOf(typeof(FileInfo)), Path = path };
        var pointer = Marshal.AllocHGlobal(Marshal.SizeOf(typeof(FileInfo)));
        try {
            Marshal.StructureToPtr(file, pointer, false);
            var data = new TrustData {
                Size = (uint)Marshal.SizeOf(typeof(TrustData)), UI = 2, Choice = 1,
                File = pointer, Flags = 0x1000
            };
            var action = new Guid("00AAC56B-CD44-11d0-8CC2-00C04FC295EE");
            return WinVerifyTrust(new IntPtr(-1), ref action, ref data);
        } finally {
            Marshal.DestroyStructure(pointer, typeof(FileInfo));
            Marshal.FreeHGlobal(pointer);
        }
    }
}
'@
}

$cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject 'CN=AION 2 Damage Calculator' `
    -CertStoreLocation 'Cert:\CurrentUser\My' -KeyAlgorithm RSA -KeyLength 3072 `
    -HashAlgorithm SHA256 -KeyExportPolicy NonExportable -NotAfter (Get-Date).AddYears(1)
try {
    & $signtool sign /s My /sha1 $cert.Thumbprint /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $binary
    if ($LASTEXITCODE -ne 0) { throw 'Authenticode signing or RFC3161 timestamping failed.' }
    $signature = Get-AuthenticodeSignature -LiteralPath $binary
    if ($signature.SignerCertificate.Thumbprint -ne $cert.Thumbprint -or !$signature.TimeStamperCertificate) {
        throw 'The executable does not contain the expected signer and timestamp.'
    }
    $result = [AionSignatureVerifier]::Verify($binary)
    # CERT_E_UNTRUSTEDROOT is expected without changing Windows trust stores.
    if ($result -ne 0 -and $result -ne -2146762487) {
        throw ('Authenticode integrity verification failed: 0x{0:X8}' -f $result)
    }
    $parent = Split-Path -Parent ([IO.Path]::GetFullPath($CertificatePath))
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
    Export-Certificate -Cert $cert -FilePath $CertificatePath -Type CERT -Force | Out-Null
    Write-Host "Verified Authenticode integrity and SHA-256 timestamp: $binary"
    Write-Host "Public certificate: $CertificatePath (self-signed; publisher trust is not established)."
} finally {
    Remove-Item -LiteralPath "Cert:\CurrentUser\My\$($cert.Thumbprint)" -DeleteKey
}
