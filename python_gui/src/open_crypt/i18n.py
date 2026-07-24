STRINGS = {
    "en": {
        "title": "OpenCrypt",
        "encrypt": "Encrypt",
        "decrypt": "Decrypt",
    },
    "ru": {
        "title": "OpenCrypt",
        "encrypt": "Зашифровать",
        "decrypt": "Расшифровать",
    }
}

def get_string(key: str, lang: str = "en") -> str:
    return STRINGS.get(lang, STRINGS["en"]).get(key, key)
