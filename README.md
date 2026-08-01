# OpenCrypt

[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.md)
[![platform](https://img.shields.io/badge/platform-Windows-0078D4.svg?logo=windows)]()

Small Windows tool to encrypt files. Right-click a file, get a key, done.
Rust does the crypto, PyQt6 is just a wrapper around it. No accounts, no
cloud, no telemetry.

## Why I made this

I wanted a one-click file encryptor that doesn't require installing
something big, doesn't phone home, and doesn't lock me into a format I
don't understand. So I wrote the format myself, kept it simple, and put
everything in one directory.

## How it works

1. Right-click a file → **Encrypt with OpenCrypt**.
2. OpenCrypt generates a random key and shows it to you once. Save it
   (a `.key` file or just copy it somewhere).
3. The file becomes `name.ocrypt`. Without the key it's noise.

Decryption is the same menu item on the `.ocrypt` file — paste the key,
get your file back.

## The key is everything

AES-256-GCM with an Argon2id-derived key. There is no backdoor, no
"forgot your key" button, no recovery service. Losing the key loses the
data. Save the key before you encrypt anything you care about.

## Install

**Portable** — grab [`OpenCrypt.exe`](https://github.com/Arean-Max/OpenCrypt/releases),
run it once (splash screen, context menu registers itself), done. One
file, works from a USB stick.

**Installer** — run `OpenCrypt_Setup_v0.2.1.exe`. No admin needed,
installs to `%LOCALAPPDATA%\Programs\OpenCrypt`, uninstalls through
Settings → Apps. Also installs the `opc` CLI (see below).

Both versions speak Russian or English depending on your system language.

## CLI

**GUI** (also usable from the command line):

```cmd
OpenCrypt.exe --encrypt "C:\Docs\report.docx"
OpenCrypt.exe --decrypt "C:\Docs\report.docx.ocrypt"
OpenCrypt.exe --unregister
```

**Headless CLI** — `opc` ships with the installer and is added to PATH
if you keep the "Add OpenCrypt to PATH" checkbox (checked by default).
Encryption is the default action; the key is printed to the console and
saved next to the file:

```cmd
opc "C:\Docs\report.docx"
:: Encrypted: C:\Docs\report.docx.ocrypt
:: Key: e9Jc2Llh6h_DvzjaxrHbOdY5j9XB7Ne98W1H4OBadHw
:: Saved key to: C:\Docs\report_key.key

opc -d "C:\Docs\report.docx.ocrypt"      :: key auto-detected
opc -d -k "C:\Docs\report_key.key" "C:\Docs\report.docx.ocrypt"
```

`opc` never overwrites an existing file, and removes the `.ocrypt`
file after a successful decrypt. Exit codes: 0 success, 1 error, 2
usage. While developing, the same entry point lives in
`python_gui\.venv\Scripts\opc.exe`.

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

## Limitations / known issues

- **No third-party security audit.** The core has been reviewed
  internally only. Use it knowing that.
- **Binaries are not code-signed.** Windows SmartScreen will warn on
  first run.
- **Whole-folder encryption is not implemented yet** — only single
  files. (tracked in the issues)
- **No key import UI** — you can save a key to a `.key` file, but
  there's no import dialog; paste it manually when decrypting.
- The `.ocrypt` format is still young. It is documented and versioned,
  but treat it as subject to change before 1.0.

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
