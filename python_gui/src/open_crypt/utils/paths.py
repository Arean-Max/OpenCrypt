import sys
from pathlib import Path


def get_app_dir() -> Path:
    if getattr(sys, 'frozen', False):
        return Path(sys.executable).parent
    return Path(__file__).parent.parent.parent.parent


def get_rust_lib_path() -> Path:
    app_dir = get_app_dir()
    candidates = []
    meipass = getattr(sys, '_MEIPASS', None)
    if meipass:
        candidates.append(Path(meipass) / "open_crypt" / "core" / "rust_core.dll")
    candidates += [
        app_dir / "rust_core.dll",
        app_dir / ".." / "rust_core" / "target" / "release" / "rust_core.dll",
    ]
    for c in candidates:
        c = c.resolve()
        if c.exists():
            return c
    return app_dir / "rust_core.dll"
