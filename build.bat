@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

set ROOT=%~dp0
set ROOT=%ROOT:~0,-1%

set VENV=%ROOT%\python_gui\.venv
set PYTHON=%VENV%\Scripts\python.exe
set PIP=%VENV%\Scripts\pip.exe
set PYINSTALLER=%VENV%\Scripts\pyinstaller.exe

echo === OpenCrypt Build ===

REM Step 1: Build Rust core
echo [1/5] Building Rust core...
cd /d "%ROOT%\rust_core"
cargo build --release
if %ERRORLEVEL% neq 0 ( echo Rust build FAILED & exit /b 1 )

set DLL=%ROOT%\rust_core\target\release\rust_core.dll
if not exist "%DLL%" (
    echo DLL not found: %DLL% & exit /b 1
)

REM Step 2: Build PyInstaller executable
echo [2/5] Building executable...
cd /d "%ROOT%\python_gui"
%PIP% install -e . 1>nul
%PYINSTALLER% --onefile --windowed --name "OpenCrypt" ^
    --icon "%ROOT%\assets\shield.ico" ^
    --add-data "%DLL%;open_crypt\core" ^
    --hidden-import "PyQt6" --hidden-import "PyQt6.QtWidgets" ^
    --hidden-import "PyQt6.QtCore" --hidden-import "PyQt6.QtGui" ^
    --hidden-import "PyQt6.sip" ^
    --collect-submodules "PyQt6" ^
    --distpath "%ROOT%\Release" --workpath "%ROOT%\build_tmp" ^
    --specpath "%ROOT%" ^
    "src\open_crypt\__main__.py"
if %ERRORLEVEL% neq 0 ( echo PyInstaller build FAILED & exit /b 1 )

REM Step 3: Build opc CLI executable
echo [3/5] Building opc CLI...
%PYINSTALLER% --onefile --console --name "opc" ^
    --icon "%ROOT%\assets\shield.ico" ^
    --add-data "%DLL%;open_crypt\core" ^
    --distpath "%ROOT%\Release" --workpath "%ROOT%\build_tmp" ^
    --specpath "%ROOT%" ^
    "src\open_crypt\cli.py"
if %ERRORLEVEL% neq 0 ( echo opc build FAILED & exit /b 1 )

REM Step 4: Build InnoSetup installer
echo [4/5] Building installer...
if exist "%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe" (
    cd /d "%ROOT%"
    "%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe" installer\installer.iss >nul
    if exist "%ROOT%\Release\OpenCrypt_Setup_v0.2.1.exe" (
        echo Installer: Release\OpenCrypt_Setup_v0.2.1.exe
    )
) else (
    echo InnoSetup not found. Installer SKIPPED.
)

REM Step 5: Cleanup
echo [5/5] Cleaning up...
if exist "%ROOT%\build_tmp" rmdir /s /q "%ROOT%\build_tmp" 2>nul

echo === Build complete! ===
echo Executable: %ROOT%\Release\OpenCrypt.exe
echo CLI:        %ROOT%\Release\opc.exe
echo Installer:  %ROOT%\Release\OpenCrypt_Setup_v0.2.1.exe
