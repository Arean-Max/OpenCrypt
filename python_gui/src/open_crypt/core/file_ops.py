import threading
import time
from pathlib import Path
from typing import Callable

from .rust_bridge import get_bridge


def _encrypt(input_path: Path, output_path: Path, password: str) -> None:
    get_bridge().encrypt_file(str(input_path), str(output_path), password)


def _decrypt(input_path: Path, output_path: Path, password: str) -> None:
    get_bridge().decrypt_file(str(input_path), str(output_path), password)


def _encrypt_with_key(input_path: Path, output_path: Path, key_b64: str) -> None:
    get_bridge().encrypt_file_with_key(str(input_path), str(output_path), key_b64)


def _decrypt_with_key(input_path: Path, output_path: Path, key_b64: str) -> None:
    get_bridge().decrypt_file_with_key(str(input_path), str(output_path), key_b64)


def encrypt_async(
    input_path: Path,
    password: str,
    on_done: Callable[[bool, str, float], None],
) -> threading.Thread:
    out = input_path.with_suffix(input_path.suffix + ".ocrypt")
    start = time.time()

    def work():
        try:
            _encrypt(input_path, out, password)
            on_done(True, str(out), time.time() - start)
        except Exception as e:
            on_done(False, str(e), time.time() - start)

    t = threading.Thread(target=work, daemon=True)
    t.start()
    return t


def decrypt_async(
    input_path: Path,
    password: str,
    on_done: Callable[[bool, str, float], None],
) -> threading.Thread:
    out = input_path.with_suffix("")
    start = time.time()

    def work():
        try:
            _decrypt(input_path, out, password)
            on_done(True, str(out), time.time() - start)
        except Exception as e:
            on_done(False, str(e), time.time() - start)

    t = threading.Thread(target=work, daemon=True)
    t.start()
    return t


def encrypt_async_with_key(
    input_path: Path,
    key_b64: str,
    on_done: Callable[[bool, str, float], None],
) -> threading.Thread:
    out = input_path.with_suffix(input_path.suffix + ".ocrypt")
    start = time.time()

    def work():
        try:
            _encrypt_with_key(input_path, out, key_b64)
            on_done(True, str(out), time.time() - start)
        except Exception as e:
            on_done(False, str(e), time.time() - start)

    t = threading.Thread(target=work, daemon=True)
    t.start()
    return t


def decrypt_async_with_key(
    input_path: Path,
    key_b64: str,
    on_done: Callable[[bool, str, float], None],
) -> threading.Thread:
    out = input_path.with_suffix("")
    start = time.time()

    def work():
        try:
            _decrypt_with_key(input_path, out, key_b64)
            on_done(True, str(out), time.time() - start)
        except Exception as e:
            on_done(False, str(e), time.time() - start)

    t = threading.Thread(target=work, daemon=True)
    t.start()
    return t


def verify_password(input_path: Path, password: str) -> bool:
    return get_bridge().verify_password(str(input_path), password)


def generate_random_key() -> str:
    return get_bridge().generate_random_key()


def secure_delete(path: Path) -> None:
    get_bridge().secure_delete(str(path))


def format_size(n: int) -> str:
    for unit in ['B', 'KB', 'MB', 'GB']:
        if n < 1024:
            return f"{n:.1f} {unit}"
        n /= 1024
    return f"{n:.1f} TB"
