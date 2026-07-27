import sys
import winreg
from pathlib import Path

from open_crypt.i18n import _t


APP_NAME = "OpenCrypt"
ENCRYPT_CMD = _t("encrypt_context")
DECRYPT_CMD = _t("decrypt_context")
ENCRYPT_GUID = "{8A2C5B1E-3D4F-4A6B-9C8D-1E2F3A4B5C6D}"
DECRYPT_GUID = "{9B3D6C2F-4E5A-5B7C-0D9E-2F3A4B5C6D7E}"


def get_app_exe() -> str:
    if getattr(sys, 'frozen', False):
        return sys.executable
    return str(Path(sys.executable).parent / "pythonw.exe")


def get_script_path() -> str:
    return str(Path(__file__).parent.parent / "__main__.py")


def _check_exists() -> bool:
    if getattr(sys, 'frozen', False):
        return Path(sys.executable).exists()
    return Path(get_script_path()).exists()


def register() -> bool:
    if not _check_exists():
        return False
    exe = get_app_exe()
    script = get_script_path()

    if getattr(sys, 'frozen', False):
        encrypt_cmd = f'"{exe}" --encrypt "%1"'
        decrypt_cmd = f'"{exe}" --decrypt "%1"'
    else:
        encrypt_cmd = f'"{exe}" "{script}" --encrypt "%1"'
        decrypt_cmd = f'"{exe}" "{script}" --decrypt "%1"'

    ok = True

    base = winreg.HKEY_CURRENT_USER
    try:
        key = winreg.CreateKey(base, f"Software\\Classes\\*\\shell\\{ENCRYPT_GUID}")
        winreg.SetValueEx(key, "", 0, winreg.REG_SZ, ENCRYPT_CMD)
        winreg.SetValueEx(key, "Icon", 0, winreg.REG_SZ, exe)
        winreg.SetValueEx(key, "Exclude", 0, winreg.REG_SZ, ".ocrypt")
        cmd = winreg.CreateKey(key, "command")
        winreg.SetValueEx(cmd, "", 0, winreg.REG_SZ, encrypt_cmd)
        winreg.CloseKey(cmd)
        winreg.CloseKey(key)
    except PermissionError:
        ok = False

    try:
        key = winreg.CreateKey(base, f"Software\\Classes\\.ocrypt\\shell\\{DECRYPT_GUID}")
        winreg.SetValueEx(key, "", 0, winreg.REG_SZ, DECRYPT_CMD)
        winreg.SetValueEx(key, "Icon", 0, winreg.REG_SZ, exe)
        cmd = winreg.CreateKey(key, "command")
        winreg.SetValueEx(cmd, "", 0, winreg.REG_SZ, decrypt_cmd)
        winreg.CloseKey(cmd)
        winreg.CloseKey(key)
    except PermissionError:
        ok = False

    return ok


def unregister() -> None:
    base = winreg.HKEY_CURRENT_USER
    for guid in [ENCRYPT_GUID, DECRYPT_GUID]:
        try:
            winreg.DeleteKey(base, f"Software\\Classes\\*\\shell\\{guid}\\command")
            winreg.DeleteKey(base, f"Software\\Classes\\*\\shell\\{guid}")
        except FileNotFoundError:
            pass
        try:
            winreg.DeleteKey(base, f"Software\\Classes\\.ocrypt\\shell\\{guid}\\command")
            winreg.DeleteKey(base, f"Software\\Classes\\.ocrypt\\shell\\{guid}")
        except FileNotFoundError:
            pass


def is_registered() -> bool:
    try:
        key = winreg.OpenKey(winreg.HKEY_CURRENT_USER, f"Software\\Classes\\*\\shell\\{ENCRYPT_GUID}\\command")
        winreg.CloseKey(key)
        return True
    except FileNotFoundError:
        return False
