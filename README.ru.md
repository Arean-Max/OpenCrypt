<p align="center">
  <a href="https://github.com/Arean-Max/OpenCrypt">
    <picture>
      <source srcset="docs/logo-dark.png" media="(prefers-color-scheme: dark)">
      <source srcset="docs/logo-light.png" media="(prefers-color-scheme: light)">
      <img src="docs/logo.png" alt="OpenCrypt logo" width="560">
    </picture>
  </a>
</p>

<p align="center">Легковесная утилита для шифрования файлов на Windows на базе AES-256-GCM, Argon2id и нативного Rust-ядра.</p>

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

### Установка

Загрузите актуальную версию со страницы [**Releases**](https://github.com/Arean-Max/OpenCrypt/releases):

- **Портативная версия (`OpenCrypt.exe`)** — Автономный исполняемый файл. Запустите один раз без установки: пункт контекстного меню пропишется автоматически. Работает напрямую с любого каталога или USB-накопителя.
- **Инсталлятор (`OpenCrypt_Setup_*.exe`)** — Стандартный установщик Windows. Устанавливается в `%LOCALAPPDATA%\Programs\OpenCrypt` без требования прав администратора, регистрирует пункт контекстного меню Проводника и добавляет консольную утилиту `opc` в системный PATH.

---

### Принцип работы

1. **Клик правой кнопкой мыши** по любому файлу &rarr; **Encrypt with OpenCrypt**.
2. OpenCrypt генерирует криптографически стойкий 256-битный ключ и отображает его один раз. Сохраните ключ в файл `.key` или в менеджер паролей.
3. Файл запечатывается в контейнер `.ocrypt`. Без ключа его содержимое неотличимо от случайного шума.
4. **Расшифровка** запускается через то же меню — вставьте ключ либо позвольте программе считать соседний `.key` файл для восстановления исходных данных.

---

### Использование CLI

В состав OpenCrypt входит автономный консольный интерфейс (`opc`), а также ключи запуска GUI:

#### Headless CLI (`opc`)

```cmd
:: Зашифровать файл (ключ выводится в консоль и сохраняется рядом)
opc "C:\Docs\report.docx"

:: Расшифровать (автопоиск ключа рядом либо явный ключ через -k)
opc -d "C:\Docs\report.docx.ocrypt"
opc -d -k "C:\Docs\report_key.key" "C:\Docs\report.docx.ocrypt"
```

#### Автоматизация через GUI

```cmd
OpenCrypt.exe --encrypt "C:\Docs\report.docx"
OpenCrypt.exe --decrypt "C:\Docs\report.docx.ocrypt"
OpenCrypt.exe --unregister
```

---

### FAQ

#### Насколько надежно шифрование?
OpenCrypt использует аутентифицированное шифрование **AES-256-GCM** с деривацией ключа через алгоритм **Argon2id**. Каждый акт шифрования генерирует уникальный случайный nonce, поэтому одинаковые файлы дают абсолютно разный шифротекст.

#### Можно ли восстановить файл при утере ключа?
Нет. В программе принципиально отсутствуют мастер-пароли, бэкдоры и механизмы удаленного сброса. Без ключа дешифрование математически невозможно.

#### Передаются ли данные по сети?
Нет. OpenCrypt работает на 100% локально в изолированном режиме без телеметрии, аналитики и сетевых запросов.

#### Что происходит при повреждении или обрезке зашифрованного файла?
Благодаря механизму аутентификации AEAD контейнер немедленно прерывает процесс расшифровки при обнаружении несовпадения тега подлинности, предотвращая запись битых данных.

---

### Архитектура и спецификация формата

- **Спецификация контейнера**: [docs/FILE_FORMAT.md](docs/FILE_FORMAT.md)
- **Интерактивная диаграмма**: [GitDiagram](https://gitdiagram.com/Arean-Max/OpenCrypt)
- **Ядро**: Безопасное Rust-ядро (`rust_core`), предоставляющее C FFI слой для графического интерфейса PyQt6.

```
OpenCrypt/
├── assets/              # Иконки приложения
├── docs/
│   ├── FILE_FORMAT.md   # Спецификация формата .ocrypt
│   └── screenshots/     # Скриншоты и графические ассеты
├── installer/           # Скрипты упаковки Inno Setup
├── python_gui/          # Десктоп-клиент PyQt6 и точка входа CLI
├── rust_core/           # Криптографическое ядро на Rust
└── build.bat            # Скрипт полной сборки
```

---

### Сборка из исходников

Требования: компилятор Rust (Cargo), Python 3.11+ и Inno Setup 6 (опционально для инсталлятора).

```cmd
git clone https://github.com/Arean-Max/OpenCrypt.git
cd OpenCrypt
build.bat
```

---

### Лицензия

[MIT](LICENSE.txt)
