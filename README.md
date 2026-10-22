<p align="center">
  <a href="https://github.com/Arean-Max/OpenCrypt">
    <img src="docs/logo.png" alt="OpenCrypt logo" width="560">
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

# OpenCrypt

A lightweight Windows file encryption utility powered by AES-256-GCM, Argon2id, and a native Rust core.

### Quick Start
Right-click any file in Windows Explorer -> Encrypt with OpenCrypt.

### CLI Usage

```cmd
opc "report.docx"
opc -d "report.docx.ocrypt"
```

### FAQ

#### How strong is the encryption?
OpenCrypt utilizes industry-standard AES-256-GCM authenticated encryption with keys derived via Argon2id.

<p align="center">
  <img src="docs/screenshots/preview.png" alt="OpenCrypt Preview" />
</p>
