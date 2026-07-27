class CryptError(Exception):
    pass

class InvalidPasswordError(CryptError):
    pass

class CorruptedFileError(CryptError):
    pass

class InvalidFormatError(CryptError):
    pass

ERROR_CODES = {
    1: ("I/O error", CryptError),
    2: ("Invalid file format", InvalidFormatError),
    3: ("Invalid password", InvalidPasswordError),
    4: ("Memory error", CryptError),
    5: ("Invalid parameter", CryptError),
}

def error_from_code(code: int) -> CryptError:
    msg, cls = ERROR_CODES.get(code, ("Unknown error", CryptError))
    return cls(f"{msg} (code {code})")
