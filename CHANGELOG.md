# Changelog

## [0.2.1] — 2026-08-01

### Fixed

- **Installer: PATH refresh reliability.** The environment broadcast was
  sent manually via `SendMessageTimeout`; replaced with Inno Setup's
  native `ChangesEnvironment=yes`, which reliably notifies running
  applications after install and uninstall.
- **Installer: PATH deduplication.** `EnvAddPath` now checks the full
  path list before appending, so re-installs don't grow the PATH
  variable with duplicates.

### Changed

- README: documented the headless `opc` CLI (usage, key auto-detection,
  exit codes, dev entry point in the venv).

## [0.2.0] — 2026-08-01

First release.

### Added

- **Rust crypto core** (`rust_core`)
  - AES-256-GCM encryption with per-file random nonces
  - Argon2id key derivation
  - Header magic + truncated-file detection — corrupted or cut-off
    files are rejected, never silently decrypted as garbage
  - Key material zeroized after use
  - C FFI boundary with panic guards, tested against the real DLL
- **PyQt6 GUI** (`python_gui`)
  - Right-click context menu: "Encrypt with OpenCrypt" /
    "Decrypt with OpenCrypt"
  - One-time key display with copy / save-as-`.key` dialog
  - File picker fallback; Russian and English UI
  - `--encrypt` / `--decrypt` / `--unregister` CLI flags
- **Headless CLI** (`opc`)
  - Ships with the installer and can be added to PATH
  - `opc FILE` encrypts, `opc -d FILE` decrypts, `opc -d -k KEY FILE`
    decrypts with an explicit key
  - Key auto-detection (`<name>_key.key` next to the file)
  - Never overwrites existing files; removes the `.ocrypt` file after
    a successful decrypt
- **Installer** (Inno Setup)
  - No admin required, installs to `%LOCALAPPDATA%\Programs\OpenCrypt`
  - Optional "Add OpenCrypt to PATH" task (checked by default)
  - PATH entry removed on uninstall
- **Portable single-file build** of the GUI (`OpenCrypt.exe`)

### Changed

- 54 automated tests: unit, integration, and FFI-boundary tests
  (wrong password, null pointers, buffer sizes, truncation, nonce
  uniqueness, empty files)
- `cargo fmt --check` and `clippy -D warnings` clean

### Known caveats

- No third-party security audit yet — only internal review
- Binaries are not code-signed; Windows SmartScreen may warn
