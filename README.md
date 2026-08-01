<div align="center">
  <img src="assets/shield.ico" width="90" alt="OpenCrypt logo">

  # OpenCrypt

  <b>Encrypt a file with one right-click. That's the whole idea.</b>

  <p>
    <a href="https://github.com/Arean-Max/OpenCrypt/blob/main/LICENSE.md">
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

OpenCrypt is a small Windows tool that encrypts files. Not a platform,
not a service — a right-click menu item. The crypto lives in a Rust
core; the PyQt6 part is just a thin wrapper around it. No accounts, no
cloud, no telemetry, and nothing that phones home.

## How it works

1. Right-click a file → **Encrypt with OpenCrypt**.
2. OpenCrypt generates a random key and shows it to you once. Save it
   (a `.key` file or just copy it somewhere).
3. The file becomes `name.ocrypt`. Without the key it's noise.

Decryption is the same menu item on the `.ocrypt` file — paste the key,
get your file back. The key is wiped from memory after use.

## The key is everything

OpenCrypt uses AES-256-GCM with an Argon2id key. What that means in
practice: there is no backdoor, no "forgot your key" button, no recovery
service. Losing the key loses the data. That's not a bug, that's the
point — save the key before you encrypt anything you care about.

## Install

**Portable** — grab [`OpenCrypt.exe`](https://github.com/Arean-Max/OpenCrypt/releases),
run it once (splash screen, context menu registers itself), done. One
file, works from a USB stick.

**Installer** — run `OpenCrypt_Setup_v0.2.0.exe`. No admin needed,
installs to `%LOCALAPPDATA%\Programs\OpenCrypt`, uninstalls through
Settings → Apps.

Both versions speak Russian or English depending on your system language.

## CLI

```cmd
OpenCrypt.exe --encrypt "C:\Docs\report.docx"
OpenCrypt.exe --decrypt "C:\Docs\report.docx.ocrypt"
OpenCrypt.exe --unregister
```

## Building from source

You need Rust, Python 3.11+, and optionally Inno Setup 6.

```cmd
git clone https://github.com/Arean-Max/OpenCrypt.git
cd OpenCrypt

:: 1. Rust core
cd rust_core
cargo build --release
cargo test        :: optional
cd ..

:: 2. Python environment
cd python_gui
python -m venv .venv
.venv\Scripts\pip install -e .
cd ..

:: 3. Portable EXE
python_gui\.venv\Scripts\pyinstaller --onefile --windowed ^
    --icon assets\shield.ico ^
    --add-data "rust_core\target\release\rust_core.dll;open_crypt\core" ^
    --hidden-import PyQt6 --hidden-import PyQt6.QtWidgets ^
    --hidden-import PyQt6.QtCore --hidden-import PyQt6.QtGui ^
    --hidden-import PyQt6.sip ^
    --collect-submodules PyQt6 ^
    --distpath Release --workpath build_tmp ^
    python_gui\src\open_crypt\__main__.py

:: (Optional) 4. Installer
"%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe" installer\installer.iss
```

Or just run `build.bat` once the venv exists.

## Design notes

- **Memory-safe Rust core** — no buffer overflows or use-after-free in
  the crypto path, and every C export catches panics at the boundary.
- **Truncated files are detected**, never silently decrypted as garbage.
- **Per-file random nonces** — the same key never produces the same
  key stream twice.
- **54 automated tests**: unit + integration, plus FFI-boundary tests
  that load the real DLL (wrong password, null pointers, buffer sizes,
  truncation, nonce uniqueness, empty files).
- `cargo fmt --check` and `clippy -D warnings` are clean.
- The `.ocrypt` format is documented in
  [docs/FILE_FORMAT.md](docs/FILE_FORMAT.md).

One honest caveat: the core has been reviewed internally but there is
no third-party audit. Use it for your own judgment of what it's worth.

## Layout

```
OpenCrypt/
├── assets/              # Icons
├── docs/
│   └── FILE_FORMAT.md   # .ocrypt format specification
├── installer/
│   └── installer.iss    # Inno Setup script
├── python_gui/          # PyQt6 frontend (FFI bridge, dialogs, context menu)
├── rust_core/           # Encryption engine (crypto.rs, ffi.rs, error.rs)
├── build.bat            # One-command build
├── LICENSE.md
└── README.md
```

## License

MIT — see [LICENSE.md](LICENSE.md).

---

<div align="center">
  <sub>Just encrypt.</sub>
</div>
