import locale

_LANG = None

_RU = {
    "encrypt_file": "Зашифровать файл",
    "decrypt_ocrypt": "Расшифровать .ocrypt",
    "context_menu": "Контекстное меню",
    "choose_file_encrypt": "Выберите файл для шифрования",
    "choose_file_decrypt": "Выберите .ocrypt файл",
    "ocrypt_filter": "OpenCrypt файлы (*.ocrypt);;Все файлы (*.*)",
    "menu_registered_remove": "Контекстное меню уже зарегистрировано.\nУдалить?",
    "menu_removed": "Контекстное меню удалено",
    "menu_registered": "Контекстное меню зарегистрировано!\nПКМ по любому файлу.",
    "error_admin_required": "Ошибка: запустите от имени администратора",
    "report_issue": "Сообщить о проблеме",
    "app_description": "OpenCrypt — шифрование файлов",
    "cli_menu_registered": "Контекстное меню зарегистрировано",
    "cli_error_admin": "Ошибка: запустите от имени администратора",
    "cli_menu_removed": "Контекстное меню удалено",
    "cli_file_not_found": "Файл не найден",
    "tooltip_generate_key": "Сгенерировать ключ",
    "tooltip_copy_key": "Копировать ключ",
    "tooltip_save_key": "Сохранить ключ в файл",
    "save_key_title": "Сохранить ключ",
    "key_filter": "Key файлы (*.key);;Все файлы (*.*)",
    "tooltip_paste_key": "Вставить из буфера",
    "encrypt_context": "Зашифровать данные с помощью OpenCrypt",
    "decrypt_context": "Расшифровать данные с помощью OpenCrypt",
    "splash_init": "Инициализация...",
    "splash_register": "Регистрация контекстного меню...",
    "splash_done": "Готово!",
    "key_warning": "Сохраните ключ! Без него файл невозможно расшифровать.",
    "key_warning_close_title": "Ключ не сохранён",
    "key_warning_close_text": "Ключ не был скопирован или сохранён.\nБез него расшифровать файл будет невозможно.\nЗакрыть окно?",
}

_EN = {
    "encrypt_file": "Encrypt file",
    "decrypt_ocrypt": "Decrypt .ocrypt",
    "context_menu": "Context menu",
    "choose_file_encrypt": "Choose file to encrypt",
    "choose_file_decrypt": "Choose .ocrypt file",
    "ocrypt_filter": "OpenCrypt files (*.ocrypt);;All files (*.*)",
    "menu_registered_remove": "Context menu already registered.\nRemove?",
    "menu_removed": "Context menu removed",
    "menu_registered": "Context menu registered!\nRight-click any file.",
    "error_admin_required": "Error: run as administrator",
    "report_issue": "Report issue",
    "app_description": "OpenCrypt — file encryption",
    "cli_menu_registered": "Context menu registered",
    "cli_error_admin": "Error: run as administrator",
    "cli_menu_removed": "Context menu removed",
    "cli_file_not_found": "File not found",
    "tooltip_generate_key": "Generate key",
    "tooltip_copy_key": "Copy key",
    "tooltip_save_key": "Save key to file",
    "save_key_title": "Save key",
    "key_filter": "Key files (*.key);;All files (*.*)",
    "tooltip_paste_key": "Paste from clipboard",
    "encrypt_context": "Encrypt with OpenCrypt",
    "decrypt_context": "Decrypt with OpenCrypt",
    "splash_init": "Initializing...",
    "splash_register": "Registering context menu...",
    "splash_done": "Done!",
    "key_warning": "Save this key! Without it, the file cannot be decrypted.",
    "key_warning_close_title": "Key not saved",
    "key_warning_close_text": "The key was not copied or saved.\nWithout it, the file cannot be decrypted.\nClose anyway?",
}

_TR = {"ru": _RU, "en": _EN}
_DEFAULT = _EN


def detect_language() -> str:
    try:
        import ctypes
        lcid = ctypes.windll.kernel32.GetUserDefaultUILanguage()
        if lcid & 0x3FF == 0x19:
            return "ru"
    except Exception:
        pass
    try:
        loc = locale.getdefaultlocale()[0]
        if loc and loc.startswith("ru"):
            return "ru"
    except Exception:
        pass
    return "en"


def _t(key: str) -> str:
    global _LANG
    if _LANG is None:
        _LANG = detect_language()
    return _TR.get(_LANG, _DEFAULT).get(key, key)
