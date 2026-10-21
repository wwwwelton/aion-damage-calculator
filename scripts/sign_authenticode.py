#!/usr/bin/env python3
"""Authenticode-sign a PE executable with a local self-signed code-signing certificate.

Creates (once) a self-signed RSA-2048 code-signing certificate under
scripts/certs/, then appends an Authenticode WIN_CERTIFICATE to the PE:
a CMS SignedData whose eContent is the SpcIndirectDataContent holding
the PE's Authenticode digest (SHA-256), signed with PKCS#1 v1.5 RSA.

Usage:
  python3 scripts/sign_authenticode.py <path-to-exe>
  python3 scripts/sign_authenticode.py <path-to-exe> --export-cer out.cer

To make the signature trusted on Windows (no admin required):
  powershell -NoProfile -Command "Import-Certificate -FilePath 'cert.cer' -CertStoreLocation Cert:\\CurrentUser\\Root"
"""
import datetime
import hashlib
import sys
from pathlib import Path

from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding, rsa
from cryptography.x509.oid import ExtendedKeyUsageOID, NameOID

CERT_DIR = Path(__file__).resolve().parent / "certs"
CN = "AION 2 Damage Calculator"

# --- minimal DER encoder -------------------------------------------------

def _der_len(n):
    if n < 0x80:
        return bytes([n])
    b = n.to_bytes((n.bit_length() + 7) // 8, "big")
    return bytes([0x80 | len(b)]) + b

def _t(tag, *parts):
    body = b"".join(parts)
    return bytes([tag]) + _der_len(len(body)) + body

def der_seq(*parts):
    return _t(0x30, *parts)

def der_set(*parts):
    return _t(0x31, *parts)

def der_oid(oid):
    arcs = [int(a) for a in oid.split(".")]
    body = bytearray([arcs[0] * 40 + arcs[1]])
    for arc in arcs[2:]:
        chunks = []
        while arc:
            chunks.append(arc & 0x7F)
            arc >>= 7
        for i, c in enumerate(reversed(chunks)):
            body.append(c | (0x80 if i < len(chunks) - 1 else 0))
    return _t(0x06, bytes(body))

def der_int(v):
    b = v.to_bytes(max(1, (v.bit_length() + 7) // 8), "big")
    if b[0] & 0x80:
        b = b"\x00" + b
    return _t(0x02, b)

def der_octet(data):
    return _t(0x04, data)

def der_bit(data):
    return _t(0x03, b"\x00" + data)

def der_utctime(now):
    s = now.strftime("%y%m%d%H%M%SZ").encode()
    return _t(0x17, s)

def _tlv_value(der):
    """Content bytes of the first TLV (definite length)."""
    l, off = der[1], 2
    if l & 0x80:
        n = l & 0x7F
        l = int.from_bytes(der[2:2 + n], "big")
        off = 2 + n
    return der[off:off + l]

# OID constants
SHA256 = "2.16.840.1.101.3.4.2.1"
SHA256_WITH_RSA = "1.2.840.113549.1.1.5"
ATTR_MESSAGE_DIGEST = "1.2.840.113549.1.9.4"
CT_SIGNED_DATA = "1.2.840.113549.1.9.2"
ATTR_CONTENT_TYPE = "1.2.840.113549.1.9.3"
ATTR_SIGNING_TIME = "1.2.840.113549.1.9.5"
SPC_INDIRECT_DATA = "1.3.6.1.4.1.311.2.1.4"
SPC_PE_IMAGE_DATA = "1.3.6.1.4.1.311.2.1.15"
SPC_STATEMENT_TYPE = "1.3.6.1.4.1.311.2.1.11"
SPC_INDIV_SP_KEY = "1.3.6.1.4.1.311.2.1.21"
def make_cert():
    """Create or load a self-signed RSA-2048 code-signing certificate."""
    key_path = CERT_DIR / "key.pem"
    cert_path = CERT_DIR / "cert.pem"
    if key_path.exists() and cert_path.exists():
        key = serialization.load_pem_private_key(key_path.read_bytes(), None)
        cert = x509.load_pem_x509_certificate(cert_path.read_bytes())
        return key, cert
    CERT_DIR.mkdir(parents=True, exist_ok=True)
    key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    name = x509.Name([
        x509.NameAttribute(NameOID.ORGANIZATION_NAME, "AION 2 Tools"),
        x509.NameAttribute(NameOID.COMMON_NAME, CN),
    ])
    now = datetime.datetime.now(datetime.timezone.utc)
    cert = (
        x509.CertificateBuilder()
        .subject_name(name)
        .issuer_name(name)
        .public_key(key.public_key())
        .serial_number(x509.random_serial_number())
        .not_valid_before(now - datetime.timedelta(days=1))
        .not_valid_after(now + datetime.timedelta(days=3650))
        .add_extension(x509.BasicConstraints(ca=True, path_length=None), critical=True)
        .add_extension(
            x509.KeyUsage(
                digital_signature=True, content_commitment=False, key_encipherment=False,
                data_encipherment=False, key_agreement=False, key_cert_sign=True,
                crl_sign=True, encipher_only=False, decipher_only=False,
            ),
            critical=True,
        )
        .add_extension(x509.ExtendedKeyUsage([ExtendedKeyUsageOID.CODE_SIGNING]), critical=False)
        .sign(key, hashes.SHA256())
    )
    key_path.write_bytes(key.private_bytes(
        serialization.Encoding.PEM, serialization.PrivateFormat.PKCS8, serialization.NoEncryption()))
    cert_path.write_bytes(cert.public_bytes(serialization.Encoding.PEM))
    return key, cert

def der_name(name):
    rdns = []
    for rdn in name.rdns:
        attrs = []
        for at in rdn:
            val = at.value.encode() if isinstance(at.value, str) else at.value
            attrs.append(der_seq(der_oid(at.oid.dotted_string), _t(0x0c, val)))
        rdns.append(der_set(*attrs))
    return der_set(*rdns)

def build_spc_pe_image_data():
    """SpcPeImageData: empty BIT STRING + [0] EXPLICIT [2] EXPLICIT '<<<Obsolete>>>'."""
    bmp = "<<<Obsolete>>>".encode("utf-16-be")
    return der_seq(
        b"\x03\x01\x00",  # BIT STRING: 0 unused bits, no content
        _t(0xa0, _t(0xa2, _t(0x80, bmp))),
    )

def build_cms(cert, key, digest):
    """Authenticode CMS SignedData: eContent is a direct SpcIndirectDataContent
    (PE digest + spcPeImageData); signed with SHA-256 / PKCS#1 v1.5."""
    now = datetime.datetime.now(datetime.timezone.utc)
    spc = der_seq(
        der_seq(der_oid(SPC_PE_IMAGE_DATA), build_spc_pe_image_data()),
        der_seq(der_seq(der_oid(SHA256), b"\x05\x00"), der_octet(digest)),
    )
    # messageDigest covers the SpcIndirectDataContent value bytes (contents,
    # not the outer 30/len) -- the Authenticode quirk for a direct-SEQUENCE
    # eContent; osslsigncode does the same ASN1_get_object peel.
    md = hashlib.sha256(_tlv_value(spc)).digest()
    attrs = [
        der_seq(der_oid(ATTR_CONTENT_TYPE), der_set(der_oid(SPC_INDIRECT_DATA))),
        der_seq(der_oid(ATTR_MESSAGE_DIGEST), der_set(der_octet(md))),
        der_seq(der_oid(ATTR_SIGNING_TIME), der_set(der_utctime(now))),
        der_seq(der_oid(SPC_STATEMENT_TYPE), der_set(der_seq(der_oid(SPC_INDIV_SP_KEY)))),
    ]
    attrs.sort()  # DER: SET OF sorted by encoded bytes
    body = b"".join(attrs)
    sig = key.sign(_t(0x31, body), padding.PKCS1v15(), hashes.SHA256())
    signer_info = der_seq(
        der_int(1),
        der_seq(der_name(cert.issuer), der_int(cert.serial_number)),
        der_seq(der_oid(SHA256), b"\x05\x00"),
        _t(0xa0, body),
        der_seq(der_oid(SHA256_WITH_RSA), b"\x05\x00"),
        der_bit(sig),
    )
    signed_data = der_seq(
        der_int(1),
        der_set(der_seq(der_oid(SHA256), b"\x05\x00")),
        der_seq(der_oid(SPC_INDIRECT_DATA), _t(0xa0, spc)),
        _t(0xa0, cert.public_bytes(serialization.Encoding.DER)),
        der_set(signer_info),
    )
    return der_seq(der_oid(CT_SIGNED_DATA), _t(0xa0, signed_data))

def pe_checksum(data, checksum_off):
    buf = bytearray(data)
    buf[checksum_off:checksum_off + 4] = b"\x00\x00\x00\x00"
    total = 0
    for i in range(0, len(buf) - 1, 2):
        total = (total + buf[i] + (buf[i + 1] << 8)) & 0xFFFFFFFF
    if len(buf) & 1:
        total = (total + buf[-1]) & 0xFFFFFFFF
    total = (total & 0xFFFF) + (total >> 16)
    total = (total & 0xFFFF) + (total >> 16)
    return (total + len(buf)) & 0xFFFFFFFF

def parse_pe(data):
    e_lfanew = int.from_bytes(data[0x3C:0x40], "little")
    if data[e_lfanew:e_lfanew + 4] != b"PE\x00\x00":
        raise SystemExit("not a PE file")
    off = e_lfanew
    nsections = int.from_bytes(data[off + 6:off + 8], "little")
    opt_off = off + 24
    magic = int.from_bytes(data[opt_off:opt_off + 2], "little")
    pe32plus = magic == 0x20B
    checksum_off = opt_off + 0x40
    dd_off = opt_off + (0x70 if pe32plus else 0x60)
    cert_rva = int.from_bytes(data[dd_off + 32:dd_off + 36], "little")
    cert_size = int.from_bytes(data[dd_off + 36:dd_off + 40], "little")
    sections = []
    soff = opt_off + int.from_bytes(data[off + 20:off + 22], "little")
    for i in range(nsections):
        s = soff + 40 * i
        sections.append(dict(
            rva=int.from_bytes(data[s + 12:s + 16], "little"),
            vsize=int.from_bytes(data[s + 8:s + 12], "little"),
            raw=int.from_bytes(data[s + 20:s + 24], "little"),
            rsize=int.from_bytes(data[s + 24:s + 28], "little"),
        ))
    return dict(opt_off=opt_off, checksum_off=checksum_off, dd_off=dd_off,
                dd_cert=dd_off + 32, cert_rva=cert_rva, cert_size=cert_size,
                sections=sections)

def rva_to_off(pe, rva):
    for s in pe["sections"]:
        if s["rva"] <= rva < s["rva"] + max(s["vsize"], s["rsize"]):
            return s["raw"] + (rva - s["rva"])
    raise SystemExit("RVA outside sections")

def off_to_rva(pe, off):
    for s in sorted(pe["sections"], key=lambda s: s["raw"]):
        if s["raw"] <= off < s["raw"] + s["rsize"]:
            return s["rva"] + (off - s["raw"])
    last = max(pe["sections"], key=lambda s: s["raw"])
    return last["rva"] + (off - last["raw"])

def sign_pe(path, key, cert):
    data = bytearray(Path(path).read_bytes())
    pe = parse_pe(data)
    if pe["cert_size"] and pe["cert_rva"]:
        table_off = rva_to_off(pe, pe["cert_rva"])
        if table_off + pe["cert_size"] != len(data):
            raise SystemExit("existing certificate table not at file end; refusing to modify")
        del data[table_off:]
    else:
        table_off = (len(data) + 7) & ~7
        data += b"\x00" * (table_off - len(data))
    # PE checksum: field and cert data-directory entry zeroed.
    data[pe["dd_cert"]:pe["dd_cert"] + 8] = b"\x00" * 8
    checksum = pe_checksum(data, pe["checksum_off"])
    # CMS length is fixed for a given cert (digest is 32 bytes).
    cms_len = len(build_cms(cert, key, b"\x00" * 32))
    table_size = 8 + cms_len
    rva = off_to_rva(pe, table_off)
    data[pe["dd_cert"]:pe["dd_cert"] + 8] = (
        rva.to_bytes(4, "little") + table_size.to_bytes(4, "little"))
    data[pe["checksum_off"]:pe["checksum_off"] + 4] = checksum.to_bytes(4, "little")
    # Authenticode eContent: hash the file with the Certificate Table
    # Data Directory entry and the CheckSum field zeroed.
    hbuf = bytearray(data)
    hbuf[pe["dd_cert"]:pe["dd_cert"] + 8] = b"\x00" * 8
    hbuf[pe["checksum_off"]:pe["checksum_off"] + 4] = b"\x00" * 4
    digest = hashlib.new("sha256", bytes(hbuf[:table_off]), usedforsecurity=False).digest()
    cms = build_cms(cert, key, digest)
    if len(cms) != cms_len:
        raise SystemExit(f"CMS length {len(cms)} != expected {cms_len}")
    table = (table_size.to_bytes(4, "little")
             + b"\x01\x00"  # wRevision = 1
             + b"\x02\x00"  # wCertificateType = WIN_CERT_TYPE_PKCS_SIGNED_DATA
             + cms)
    data[table_off:table_off + table_size] = table
    Path(path).write_bytes(bytes(data))
    return table_off, table_size, checksum

def main():
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    target = Path(sys.argv[1]).resolve()
    export = sys.argv[3] if len(sys.argv) > 3 and sys.argv[2] == "--export-cer" else None
    key, cert = make_cert()
    table_off, table_size, checksum = sign_pe(target, key, cert)
    print(f"signed {target} (table at {table_off}, {table_size} bytes, PE checksum 0x{checksum:08x})")
    print(f"cert: {CN} (self-signed; key at {CERT_DIR / 'key.pem'})")
    if export:
        cer = cert.public_bytes(serialization.Encoding.DER)
        Path(export).write_bytes(cer)
        print(f"cert DER exported to {export}")
        print("install for local trust (no admin):")
        print(f"  powershell -NoProfile -Command \"Import-Certificate -FilePath '{export}' -CertStoreLocation Cert:\\CurrentUser\\Root\"")

if __name__ == "__main__":
    main()
