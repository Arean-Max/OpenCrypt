import ctypes
import threading
from pathlib import Path

from ..utils.paths import get_rust_lib_path
from .exceptions import error_from_code


class RustBridge:
    _instance = None
    _lock = threading.Lock()

    def __new__(cls):
        if cls._instance is None:
            with cls._lock:
                if cls._instance is None:
                    cls._instance = super().__new__(cls)
                    cls._instance._init()
        return cls._instance

    def _init(self):
        lib_path = get_rust_lib_path()
        if not lib_path.exists():
            raise RuntimeError(
                f"Rust library not found: {lib_path}\n"
                f"Build: cd rust_core && cargo build --release"
            )
        self._lib = ctypes.CDLL(str(lib_path))

        self._lib.crypt_encrypt_file.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
        self._lib.crypt_encrypt_file.restype = ctypes.c_int

        self._lib.crypt_decrypt_file.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
        self._lib.crypt_decrypt_file.restype = ctypes.c_int

        self._lib.crypt_verify_password.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
        self._lib.crypt_verify_password.restype = ctypes.c_int

        self._lib.crypt_encrypt_file_with_key.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
        self._lib.crypt_encrypt_file_with_key.restype = ctypes.c_int

        self._lib.crypt_decrypt_file_with_key.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
        self._lib.crypt_decrypt_file_with_key.restype = ctypes.c_int

        self._lib.crypt_generate_random_key.argtypes = [ctypes.c_char_p, ctypes.c_int]
        self._lib.crypt_generate_random_key.restype = ctypes.c_int

        self._lib.crypt_secure_delete.argtypes = [ctypes.c_char_p]
        self._lib.crypt_secure_delete.restype = ctypes.c_int

        self._lib.crypt_validate_paths.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_int]
        self._lib.crypt_validate_paths.restype = ctypes.c_int

        self._lib.crypt_register_context_menu.argtypes = [
            ctypes.c_char_p,
            ctypes.c_char_p,
            ctypes.c_char_p,
            ctypes.c_char_p,
        ]
        self._lib.crypt_register_context_menu.restype = ctypes.c_int

        self._lib.crypt_unregister_context_menu.argtypes = []
        self._lib.crypt_unregister_context_menu.restype = ctypes.c_int

        self._lib.crypt_context_menu_registered.argtypes = [ctypes.POINTER(ctypes.c_int)]
        self._lib.crypt_context_menu_registered.restype = ctypes.c_int

    def validate_paths(self, input_path: str, output_path: str, is_decrypt: bool) -> None:
        r = self._lib.crypt_validate_paths(
            input_path.encode(), output_path.encode(), int(is_decrypt)
        )
        if r != 0:
            raise error_from_code(r)

    def register_context_menu(
        self, exe_path: str, script_path: str, encrypt_label: str, decrypt_label: str
    ) -> None:
        r = self._lib.crypt_register_context_menu(
            exe_path.encode(), script_path.encode(), encrypt_label.encode(), decrypt_label.encode()
        )
        if r != 0:
            raise error_from_code(r)

    def unregister_context_menu(self) -> None:
        r = self._lib.crypt_unregister_context_menu()
        if r != 0:
            raise error_from_code(r)

    def context_menu_registered(self) -> bool:
        out = ctypes.c_int(0)
        r = self._lib.crypt_context_menu_registered(ctypes.byref(out))
        if r != 0:
            raise error_from_code(r)
        return bool(out.value)

    def encrypt_file_with_key(self, input_path: str, output_path: str, key_b64: str) -> None:
        r = self._lib.crypt_encrypt_file_with_key(
            input_path.encode(), output_path.encode(), key_b64.encode()
        )
        if r != 0:
            raise error_from_code(r)

    def decrypt_file_with_key(self, input_path: str, output_path: str, key_b64: str) -> None:
        r = self._lib.crypt_decrypt_file_with_key(
            input_path.encode(), output_path.encode(), key_b64.encode()
        )
        if r != 0:
            raise error_from_code(r)

    def encrypt_file(self, input_path: str, output_path: str, password: str) -> None:
        r = self._lib.crypt_encrypt_file(
            input_path.encode(), output_path.encode(), password.encode()
        )
        if r != 0:
            raise error_from_code(r)

    def decrypt_file(self, input_path: str, output_path: str, password: str) -> None:
        r = self._lib.crypt_decrypt_file(
            input_path.encode(), output_path.encode(), password.encode()
        )
        if r != 0:
            raise error_from_code(r)

    def verify_password(self, input_path: str, password: str) -> bool:
        r = self._lib.crypt_verify_password(input_path.encode(), password.encode())
        if r == 0:
            return True
        if r == 3:
            return False
        raise error_from_code(r)

    def generate_random_key(self) -> str:
        # 44 = 32 bytes base64url-encoded, no padding. Magic number,
        # yes — could ask the DLL for the size, but then we'd need yet
        # another export just for that.
        buf = ctypes.create_string_buffer(44)
        r = self._lib.crypt_generate_random_key(buf, 44)
        if r != 0:
            raise error_from_code(r)
        return buf.value.decode()

    def secure_delete(self, path: str) -> None:
        r = self._lib.crypt_secure_delete(path.encode())
        if r != 0:
            raise error_from_code(r)


def get_bridge() -> RustBridge:
    return RustBridge()
