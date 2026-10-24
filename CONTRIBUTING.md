# Contributing

Thanks for looking. This is a small personal project, so things are
kept simple.

## What I actually need

- Bug reports with a repro (file size, Windows version, steps).
- Real-world feedback on the key workflow — is saving the key as a
  `.key` file next to the data sane, or does it surprise you?
- Pull requests for small, contained fixes. I'd rather merge 3 small
  PRs than one big rewrite.

## Setting up a dev environment

1. `rust_core`: `cargo build --release` (needs Rust toolchain).
2. `python_gui`: `python -m venv .venv && .venv\Scripts\pip install -e .`
3. Run the GUI: `python -m open_crypt`.
4. CLI during development: `.venv\Scripts\opc.exe`.

The Rust DLL must exist before the Python side loads — see
`python_gui/src/open_crypt/utils/paths.py` for where it is looked up.

## Tests

```cmd
cd rust_core
cargo test
```

Python side has no test suite yet — it is mostly a thin wrapper, but
that is not a great excuse. Help welcome.

## Style

- Rust: `cargo fmt` + `cargo clippy -- -D warnings`. Keep it clean.
- Python: readable over clever. The project's Python code is not
  meant to be a showpiece — it is a bridge to the Rust core.

## Committing

- Messy history is fine. Clear message is not negotiable: say what
  changed and why, one short paragraph if needed.
- Don't commit build artifacts (`target/`, `Release/`, `*.spec`).

## Code of conduct

Be civil. That's the whole thing.
