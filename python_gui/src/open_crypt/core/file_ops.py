from pathlib import Path
from .rust_bridge import get_bridge
from .exceptions import OutputExistsError

def encrypt_file(input_path: str, output_path: str, password: str) -> None:
    bridge = get_bridge()
    bridge.encrypt_file(input_path, output_path, password)

def decrypt_file(input_path: str, output_path: str, password: str) -> None:
    bridge = get_bridge()
    bridge.decrypt_file(input_path, output_path, password)

def generate_random_key() -> str:
    bridge = get_bridge()
    return bridge.generate_random_key()
