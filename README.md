<div align="center">
  <img src="assets/shield.ico" width="80" alt="OpenCrypt logo">

  # OpenCrypt

  <b>File encryption tool for Windows — right-click, encrypt, done.</b>

  <p>
    <a href="https://github.com/Arean-Max/OpenCrypt/blob/main/LICENSE">
      <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License">
    </a>
    <a href="https://www.rust-lang.org/">
      <img src="https://img.shields.io/badge/core-Rust-orange.svg" alt="Rust">
    </a>
    <a href="https://www.python.org/">
      <img src="https://img.shields.io/badge/gui-Python-3776AB.svg?logo=python" alt="Python">
    </a>
    <a href="https://pypi.org/project/PyQt6/">
      <img src="https://img.shields.io/badge/framework-PyQt6-41CD52.svg?logo=qt" alt="PyQt6">
    </a>
    <img src="https://img.shields.io/badge/platform-Windows-0078D4.svg?logo=windows" alt="Windows">
    <img src="https://img.shields.io/badge/arch-x64-00599C.svg" alt="x64">
  </p>
</div>

---

OpenCrypt encrypts and decrypts files **with one right-click**. Built on Rust for speed and safety, wrapped in a PyQt6 GUI for comfort. No admin required, no telemetry, no cloud.

## Features

- **Context menu integration** — right-click any file → Encrypt/Decrypt
- **Auto‑register on first launch** — splash screen, register, done
- **AES‑256‑GCM** — authenticated encryption, tamper‑proof
- **Argon2id key derivation** — memory‑hard, brute‑force resistant
- **Key file export / import** (`.key`) — share or back up your keys
- **Portable EXE** — single file, zero dependencies, no install needed
- **Installer available** — per‑user install, clean uninstall
- **Russian & English** — auto‑detects system language

> **Warning:** your key is the only way to decrypt your files. If you
> lose the key, your data is **unrecoverable** — no backdoor exists.
> Always save the key when encrypting (copy or download button).

## Quick start

### Portable (one‑shot)

1. Download [`OpenCrypt.exe`](https://github.com/Arean-Max/OpenCrypt/releases)
2. Double‑click — splash → register → done
3. Right‑click any file → **Encrypt with OpenCrypt**

### Installer

1. Run `OpenCrypt_Setup_v0.2.0.exe` (no admin required)
2. Installs to `%LOCALAPPDATA%\Programs\OpenCrypt`
3. Uninstall via Settings → Apps

### CLI

```cmd
OpenCrypt.exe --encrypt "C:\Docs\report.docx"
OpenCrypt.exe --decrypt "C:\Docs\report.docx.ocrypt"
OpenCrypt.exe --unregister
```

## How it works

| Layer | Technology |
|---|---|
| Encryption | AES‑256‑GCM (authenticated) via `aes‑gcm` crate |
| Key derivation | Argon2id (memory‑hard KDF) |
| Key material | Zeroized in memory after use (`zeroize` crate) |
| FFI bridge | `extern "C"` from Rust → Python `ctypes` |
| GUI | PyQt6 (Fusion style) |
| Context menu | Windows registry → `HKCU\Software\Classes` |
| Language | Auto‑detected (RU / EN) |

## Build from source

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- Python 3.11+
- [InnoSetup 6](https://jrsoftware.org/isdl.php) *(optional — for the installer)*

### Steps

```cmd
git clone https://github.com/Arean-Max/OpenCrypt.git
cd OpenCrypt

:: 1. Build Rust core
cd rust_core
cargo build --release
cargo test            :: optional: run the test suite
cd ..

:: 2. Setup Python environment
cd python_gui
python -m venv .venv
.venv\Scripts\pip install -e .
cd ..

:: 3. Build portable EXE
python_gui\.venv\Scripts\pyinstaller --onefile --windowed ^
    --icon assets\shield.ico ^
    --add-data "rust_core\target\release\rust_core.dll;open_crypt\core" ^
    --hidden-import PyQt6 --hidden-import PyQt6.QtWidgets ^
    --hidden-import PyQt6.QtCore --hidden-import PyQt6.QtGui ^
    --hidden-import PyQt6.sip ^
    --collect-submodules PyQt6 ^
    --distpath Release --workpath build_tmp ^
    python_gui\src\open_crypt\__main__.py

:: (Optional) 4. Build installer
"%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe" installer\installer.iss
```

> Or simply run `build.bat` after setting up `python_gui\.venv`.

## Security

- **No network access** — OpenCrypt never makes outbound connections
- **No telemetry** — zero tracking, zero analytics
- **No dependencies at runtime** — everything is bundled into one EXE
- **Encryption key is never logged or persisted** — only the key file you explicitly save
- **Memory‑safe Rust core** — no buffer overflows, no use‑after‑free in the crypto path
- **Panic‑safe FFI** — every C export catches panics at the boundary
- **Length‑verified decrypt** — truncated files are detected, not silently returned
- **Unique per‑file nonces** — 64‑bit random base + per‑chunk counter, no key stream reuse

### File format

The `.ocrypt` format (header layout, chunking, nonce scheme, KDF,
error codes) is specified in [docs/FILE_FORMAT.md](docs/FILE_FORMAT.md).
It is versioned and not backward compatible.

### Testing

- **36 automated tests**: unit + integration tests in `rust_core/tests`,
  including FFI‑boundary tests that load the real DLL (wrong password,
  null pointers, buffer sizes, truncation, nonce uniqueness, empty files).
- **CI**: GitHub Actions runs `cargo fmt --check`, `clippy -D warnings`,
  and the full test suite on every push.

### Audit status

The core has been reviewed internally (2026‑07). It is **not** a
public, third‑party security audit — no claims of formal certification
are made. Use at your own risk.

### Antivirus note

Portable executables built with PyInstaller are occasionally flagged by
some antivirus engines (unsigned one‑file bundles). This is a known
false‑positive pattern. Build from source or use the installer if you
prefer, and report false positives to your AV vendor.

## Project structure

```
OpenCrypt/
├── .github/workflows/    # CI (fmt, clippy, tests, release build)
├── assets/              # Icons and resources
│   └── shield.ico
├── docs/
│   └── FILE_FORMAT.md   # .ocrypt format specification
├── installer/
│   └── installer.iss    # InnoSetup script
├── python_gui/          # Python + PyQt6 frontend
│   └── src/open_crypt/
│       ├── core/        # Rust FFI bridge
│       ├── gui/         # Dialogs + splash screen
│       ├── shell/       # Context menu registration
│       └── utils/       # Path utilities
├── rust_core/           # Rust encryption engine
│   ├── src/
│   │   ├── crypto.rs    # AES‑256‑GCM + Argon2id
│   │   ├── ffi.rs       # C‑compatible exports
│   │   └── error.rs     # Error types
│   └── tests/           # Integration tests (incl. FFI boundary)
├── build.bat            # Automated build script
├── LICENSE
└── README.md
```

## License

MIT — see [LICENSE](LICENSE).

---

<div align="center">
  <sub>Just encrypt.</sub>
</div>
