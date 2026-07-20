import argparse
import sys
from pathlib import Path

from open_crypt.core.rust_bridge import get_bridge
from open_crypt.core.file_ops import generate_random_key

def main():
    parser = argparse.ArgumentParser(description="OpenCrypt CLI")
    parser.add_argument("file", help="File to encrypt/decrypt")
    parser.add_argument("-d", "--decrypt", action="store_true", help="Decrypt mode")
    parser.add_argument("-k", "--key", help="Decryption key")
    args = parser.parse_args()

    p = Path(args.file)
    if not p.exists():
        print(f"Error: file not found: {p}", file=sys.stderr)
        return 1

    if args.decrypt:
        key = args.key
        if key is None:
            cand = p.with_name(p.stem + "_key.key")
            if cand.exists():
                key = cand.read_text().strip()
                print(f"Using key from: {cand}")
            else:
                print("Error: --key required or place keyfile next to file", file=sys.stderr)
                return 1
        out = p.with_suffix("")
        get_bridge().decrypt_file_with_key(str(p), str(out), args.key)
        print(f"Decrypted: {out}")
    else:
        out = p.with_suffix(p.suffix + ".ocrypt")
        key = generate_random_key()
        get_bridge().encrypt_file_with_key(str(p), str(out), key)
        print(f"Encrypted: {out}")
        print(f"Key: {key}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
