import argparse
import sys
from pathlib import Path

from open_crypt.core.rust_bridge import get_bridge
from open_crypt.core.file_ops import generate_random_key, secure_delete
from open_crypt.core.exceptions import OutputExistsError


def _find_key_file(path: Path) -> Path | None:
    # TODO: fall back to scanning for any *.key in the same folder —
    # people rename the key file, then get confused why -d can't find it.
    if path.name.endswith(".ocrypt"):
        base = Path(path.name[: -len(".ocrypt")]).stem
    else:
        base = path.stem
    key_file = path.with_name(base + "_key.key")
    if key_file.exists():
        return key_file
    return None


def _encrypt(path: Path) -> int:
    out = path.with_suffix(path.suffix + ".ocrypt")
    if out.exists():
        print(f"Error: output already exists: {out}", file=sys.stderr)
        return 1
    key = generate_random_key()
    try:
        get_bridge().encrypt_file_with_key(str(path), str(out), key)
    except OutputExistsError:
        print(f"Error: output already exists: {out}", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1
    key_file = path.with_name(path.stem + "_key.key")
    key_file.write_text(key)
    print(f"Encrypted: {out}")
    print(f"Key: {key}")
    print(f"Saved key to: {key_file}")
    print()
    print("Keep this key! Without it the file is unrecoverable.")
    return 0


def _decrypt(path: Path, key: str | None) -> int:
    if key is None:
        key_file = _find_key_file(path)
        if key_file is None:
            print(
                f"Error: no key given. Use --key <KEY> or place {path.stem}_key.key next to the file",
                file=sys.stderr,
            )
            return 1
        key = key_file.read_text().strip()
        print(f"Using key from: {key_file}")
    out = path.with_suffix("")
    if out.exists():
        print(f"Error: output already exists: {out}", file=sys.stderr)
        return 1
    try:
        get_bridge().decrypt_file_with_key(str(path), str(out), key)
    except OutputExistsError:
        print(f"Error: output already exists: {out}", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1
    try:
        secure_delete(path)
    except Exception as e:
        print(f"Warning: could not delete encrypted file: {e}", file=sys.stderr)
    print(f"Decrypted: {out}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        prog="opc",
        description="Encrypt or decrypt a file. Default: encrypt.",
    )
    parser.add_argument("file", metavar="FILE", help="file to encrypt (or decrypt with --decrypt)")
    parser.add_argument("-d", "--decrypt", action="store_true", help="decrypt instead of encrypt")
    parser.add_argument("-k", "--key", metavar="KEY", help="key for decryption; if omitted, looks for <name>_key.key next to the file")
    args = parser.parse_args()

    if not args.file:
        parser.print_usage()
        return 2

    path = Path(args.file)
    if not path.exists():
        print(f"Error: file not found: {path}", file=sys.stderr)
        return 1

    if args.decrypt:
        return _decrypt(path, args.key)
    return _encrypt(path)


if __name__ == "__main__":
    sys.exit(main())
