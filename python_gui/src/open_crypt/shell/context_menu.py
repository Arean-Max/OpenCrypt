import sys
from pathlib import Path

from open_crypt.core.exceptions import CryptError
from open_crypt.core.rust_bridge import get_bridge
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


def _script_for_ffi() -> str:
    if getattr(sys, 'frozen', False):
        return ""
    return get_script_path()


def register() -> bool:
    if not _check_exists():
        return False
    try:
        get_bridge().register_context_menu(
            get_app_exe(), _script_for_ffi(), ENCRYPT_CMD, DECRYPT_CMD
        )
        return True
    except CryptError:
        return False


def unregister() -> bool:
    try:
        get_bridge().unregister_context_menu()
        return True
    except CryptError:
        return False


def is_registered() -> bool:
    try:
        return get_bridge().context_menu_registered()
    except CryptError:
        return False
