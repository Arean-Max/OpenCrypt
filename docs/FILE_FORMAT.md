# OpenCrypt File Format (.ocrypt)

Version: 1 (spec revision 2026-07-31)

This document describes the on-disk format of OpenCrypt encrypted files.
The format is not backward compatible: files are written with a single
version byte, and future revisions may change the layout. Decrypting a
file with a version byte this build does not know is rejected with
`CRYPT_ERR_INVALID_FORMAT`.

## File layout

```
+--------+---------+--------------+--------+--------+------------------+
| magic  | version | plaintext_len| salt   | nonce  | ciphertext chunks|
| 4 bytes| 1 byte  | 8 bytes (LE) | 16 bytes|12 bytes| ...              |
+--------+---------+--------------+--------+--------+------------------+
```

Total header size: **41 bytes**.

| Field | Size | Description |
|-------|------|-------------|
| `magic` | 4 | Constant `b"ORP\0"` (0x4F 0x52 0x50 0x00). |
| `version` | 1 | Format version, currently `1`. |
| `plaintext_len` | 8 | Length of the original plaintext in bytes, little-endian `u64`. |
| `salt` | 16 | Random KDF salt, freshly generated per file. |
| `nonce` | 12 | Random base nonce, freshly generated per file. |

## Key derivation (password mode)

- Salt: 16 random bytes from the file header.
- KDF: Argon2id with the library defaults (`argon2` crate v0.5,
  `Argon2::default()`: m = 19 MiB, t = 2, p = 1, Argon2id).
- Output: 32 bytes used directly as the AES-256-GCM key.
- **Important:** KDF parameters are NOT stored in the file. If the
  library defaults change in a future release, old files will no longer
  decrypt. Parameter migration is planned but not yet implemented.

## Key mode

- Keys are 32 random bytes encoded as base64url (no padding), which
  always yields a 43-character ASCII string.
- The raw 32 bytes are used directly as the AES-256-GCM key, no KDF.
- The `salt` field is still written (random) and ignored on decrypt.

## Chunking and encryption

- Plaintext is processed in chunks of **1 MiB** (`CHUNK_SIZE = 1024 * 1024`).
- Each chunk is encrypted with **AES-256-GCM** and written as
  ciphertext + 16-byte authentication tag. Chunk overhead: 16 bytes.
- Empty file: header only, no chunks.

### Per-chunk nonce

Each chunk uses a 12-byte nonce derived from the header base nonce:

```
nonce[0..8]  = base_nonce[0..8]        (random, from header)
nonce[8..12] = chunk_counter as u32    (big-endian)
```

- `chunk_counter` starts at 0 and increments per chunk.
- A 32-bit counter permits up to 2^32 chunks = 4 PiB per file.
- The 64-bit random base makes nonce collisions across files
  improbable until ~2^32 files share the same key (birthday bound).

## Authentication and integrity

- GCM authenticates each chunk individually; a tampered chunk fails
  with `CRYPT_ERR_INVALID_PASSWORD` on decrypt.
- The overall plaintext length is NOT covered by GCM. Truncation or
  appending at exact chunk boundaries is detected by comparing the
  number of decrypted bytes against `plaintext_len`; mismatch fails
  with `CRYPT_ERR_INVALID_FORMAT`. Note: truncation mid-chunk is
  already caught by the chunk tag.
- On any decrypt failure the partial output file is removed.
- `CRYPT_ERR_INVALID_PASSWORD` is returned both for a wrong password
  and for any GCM tag failure (tampering, corruption), so callers
  cannot distinguish the two.

## Error codes

| Code | Constant | Meaning |
|------|----------|---------|
| 0 | `CRYPT_SUCCESS` | Success |
| 1 | `CRYPT_ERR_IO` | I/O error |
| 2 | `CRYPT_ERR_INVALID_FORMAT` | Not an OpenCrypt file, unknown version, or length mismatch |
| 3 | `CRYPT_ERR_INVALID_PASSWORD` | Wrong password/key or corrupted ciphertext |
| 4 | `CRYPT_ERR_MEMORY` | KDF/RNG/encryption failure |
| 5 | `CRYPT_ERR_INVALID_PARAM` | Null/invalid pointer, bad key string, undersized buffer |
| 6 | `CRYPT_ERR_PANIC` | Panic inside the Rust core was caught at the FFI boundary |

## Versioning policy

- `version` byte starts at 1.
- Changing any field size or meaning requires a new version byte.
- This build only reads version `1`.
