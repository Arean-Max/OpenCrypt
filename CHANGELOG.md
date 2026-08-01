# Changelog

## [0.2.0] — 2026-08-01

First release.

### Added

- **Rust crypto core** (`rust_core`)
  - AES-256-GCM encryption with per-file random nonces
  - Argon2id key derivation
  - Header magic + truncated-file detection
  - Key material zeroized after use
  - C FFI boundary with panic guards
- **PyQt6 GUI** (`python_gui`)
  - Context menu: "Encrypt with OpenCrypt" / "Decrypt with OpenCrypt"
  - One-time key display with copy / save-as-`.key` dialog
  - Russian and English UI
- **Headless CLI** (`opc`)
- **Installer** (Inno Setup)
