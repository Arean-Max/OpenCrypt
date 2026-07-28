@echo off
set ROOT=%~dp0
set ROOT=%ROOT:~0,-1%
set VENV=%ROOT%\python_gui\.venv
echo Building OpenCrypt...
cd /d "%ROOT%\rust_core"
cargo build --release
