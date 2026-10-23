<p align="center">
  <a href="https://github.com/Arean-Max/OpenCrypt">
    <picture>
      <source srcset="docs/logo-dark.png" media="(prefers-color-scheme: dark)">
      <source srcset="docs/logo-light.png" media="(prefers-color-scheme: light)">
      <img src="docs/logo.png" alt="OpenCrypt logo" width="560">
    </picture>
  </a>
</p>

<p align="center">A lightweight Windows file encryption utility powered by AES-256-GCM, Argon2id, and a native Rust core.</p>

<p align="center">
  <a href="https://github.com/Arean-Max/OpenCrypt/releases"><img alt="Releases" src="https://img.shields.io/github/v/release/Arean-Max/OpenCrypt?style=flat-square&color=emerald" /></a>
  <a href="LICENSE.txt"><img alt="License" src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" /></a>
  <a href="https://microsoft.com/windows"><img alt="Platform" src="https://img.shields.io/badge/Platform-Windows%2010%20%2F%2011%20(x64)-informational?style=flat-square" /></a>
  <a href="https://deepwiki.com/Arean-Max/OpenCrypt"><img alt="Docs" src="https://img.shields.io/badge/Docs-DeepWiki-blueviolet?style=flat-square" /></a>
  <a href="https://gitdiagram.com/Arean-Max/OpenCrypt"><img alt="Architecture" src="https://img.shields.io/badge/Diagram-GitDiagram-orange?style=flat-square" /></a>
</p>

<p align="center">
  <a href="README.md">English</a> |
  <a href="README.zh.md">简体中文</a> |
  <a href="README.zht.md">繁體中文</a> |
  <a href="README.ko.md">한국어</a> |
  <a href="README.de.md">Deutsch</a> |
  <a href="README.es.md">Español</a> |
  <a href="README.fr.md">Français</a> |
  <a href="README.it.md">Italiano</a> |
  <a href="README.da.md">Dansk</a> |
  <a href="README.ja.md">日本語</a> |
  <a href="README.pl.md">Polski</a> |
  <a href="README.ru.md">Русский</a> |
  <a href="README.bs.md">Bosanski</a> |
  <a href="README.ar.md">العربية</a> |
  <a href="README.no.md">Norsk</a> |
  <a href="README.br.md">Português (Brasil)</a> |
  <a href="README.th.md">ไทย</a> |
  <a href="README.tr.md">Türkçe</a> |
  <a href="README.uk.md">Українська</a> |
  <a href="README.bn.md">বাংলা</a> |
  <a href="README.gr.md">Ελληνικά</a> |
  <a href="README.vi.md">Tiếng Việt</a>
</p>

<p align="center">
  <a href="https://github.com/Arean-Max/OpenCrypt/releases">
    <img src="docs/screenshots/preview.png" alt="OpenCrypt Preview" />
  </a>
</p>

---

### Installation

Grab the latest build from [**Releases**](https://github.com/Arean-Max/OpenCrypt/releases):

- **Portable (`OpenCrypt.exe`)** — Standalone executable. Run once without installation, context menu registers automatically. Operates self-contained from any directory or USB drive.
- **Installer (`OpenCrypt_Setup_*.exe`)** — Standard Windows setup. Installs to `%LOCALAPPDATA%\Programs\OpenCrypt` without requiring administrator privileges, configures Explorer shell context menu, and adds the headless `opc` CLI tool to PATH.

---

### How It Works

1. **Right-click** any file in Windows Explorer &rarr; **Encrypt with OpenCrypt**.
2. OpenCrypt generates a cryptographically secure 256-bit key and displays it once. Save it to a `.key` file or copy it to your password manager.
3. The file is sealed into a `.ocrypt` container. Without the key, the contents are indistinguishable from random noise.
4. **Decryption** uses the same menu — paste your key or let OpenCrypt detect the adjacent `.key` file to restore the original plaintext.

---

### CLI Usage

OpenCrypt includes a standalone headless CLI (`opc`) alongside GUI automation flags:

#### Headless CLI (`opc`)

```cmd
:: Encrypt a file (key is printed to console and saved next to the file)
opc "C:\Docs\report.docx"

:: Decrypt (auto-detects adjacent key file or pass --key explicitly)
opc -d "C:\Docs\report.docx.ocrypt"
opc -d -k "C:\Docs\report_key.key" "C:\Docs\report.docx.ocrypt"
```

#### GUI Automation

```cmd
OpenCrypt.exe --encrypt "C:\Docs\report.docx"
OpenCrypt.exe --decrypt "C:\Docs\report.docx.ocrypt"
OpenCrypt.exe --unregister
```

---

### FAQ

#### How strong is the encryption?
OpenCrypt utilizes industry-standard **AES-256-GCM** authenticated encryption with keys derived via **Argon2id**. Every file encryption pass generates a unique random nonce, ensuring identical plaintexts never yield identical ciphertext outputs.

#### Can I recover my files if I lose the key?
No. OpenCrypt does not contain backdoors, master recovery keys, or telemetry endpoints. If a key is lost, recovery is mathematically infeasible.

#### Does OpenCrypt transmit my data to the cloud?
No. OpenCrypt operates completely offline with zero telemetry, zero analytics, and zero external network requests. Everything executes locally on your machine.

#### What happens if an encrypted file gets damaged or truncated?
Thanks to AEAD verification tags, OpenCrypt immediately detects truncated or tampered data during decryption and fails safely rather than writing corrupt plaintext.

---

### Architecture & Format

- **Format Specification**: [docs/FILE_FORMAT.md](docs/FILE_FORMAT.md)
- **Interactive Diagram**: [GitDiagram](https://gitdiagram.com/Arean-Max/OpenCrypt)
- **Engine**: Memory-safe Rust core (`rust_core`) exposing a panic-free C FFI layer consumed by a lightweight PyQt6 interface.

```
OpenCrypt/
├── assets/              # Application icons and visuals
├── docs/
│   ├── FILE_FORMAT.md   # .ocrypt container specification
│   └── screenshots/     # Interface previews and assets
├── installer/           # Inno Setup packaging scripts
├── python_gui/          # PyQt6 desktop client and CLI entry points
├── rust_core/           # Memory-safe AES-256-GCM + Argon2id engine
└── build.bat            # Automated build script
```

---

### Build from source

Prerequisites: Rust (Cargo), Python 3.11+, and optional Inno Setup 6.

```cmd
git clone https://github.com/Arean-Max/OpenCrypt.git
cd OpenCrypt
build.bat
```

---

### License

[MIT](LICENSE.txt)
