import sys, argparse, ctypes, logging, os
from pathlib import Path

logging.basicConfig(
    filename=Path(os.environ.get("TEMP", ".")) / "opencrypt.log",
    level=logging.INFO, format="%(asctime)s %(levelname)s: %(message)s",
)

from PyQt6.QtWidgets import QApplication
from PyQt6.QtCore import QTimer

from open_crypt.i18n import _t
from open_crypt.gui.splash_screen import SplashScreen


def _hide_console():
    try:
        ctypes.windll.user32.ShowWindow(ctypes.windll.kernel32.GetConsoleWindow(), 0)
    except Exception:
        pass


def _run_setup(splash: SplashScreen):
    from open_crypt.shell.context_menu import register

    def step0():
        splash.set_status(_t("splash_init"))
        splash.set_progress(0.3)
        QTimer.singleShot(600, step1)

    def step1():
        splash.set_status(_t("splash_register"))
        splash.set_progress(0.6)
        QTimer.singleShot(50, lambda: _do_register())

    def _do_register():
        register()
        step2()

    def step2():
        splash.set_status(_t("splash_done"))
        splash.set_progress(1.0)
        QTimer.singleShot(800, splash.close_splash)

    QTimer.singleShot(400, step0)


def main():
    _hide_console()
    parser = argparse.ArgumentParser(description=_t("app_description"))
    parser.add_argument("--encrypt", metavar="FILE")
    parser.add_argument("--decrypt", metavar="FILE")
    parser.add_argument("--unregister", action="store_true")
    args, _ = parser.parse_known_args()

    if args.unregister:
        from open_crypt.shell.context_menu import unregister
        unregister()
        print(_t("cli_menu_removed"))
        return

    app = QApplication(sys.argv)
    app.setStyle("Fusion")

    if args.encrypt:
        fp = Path(args.encrypt)
        if not fp.exists():
            print(f"{_t('cli_file_not_found')}: {fp}")
            return
        from open_crypt.gui.encrypt_dialog import EncryptDialog
        dlg = EncryptDialog(str(fp))
        dlg.exec()
        return

    if args.decrypt:
        fp = Path(args.decrypt)
        if not fp.exists():
            print(f"{_t('cli_file_not_found')}: {fp}")
            return
        from open_crypt.gui.decrypt_dialog import DecryptDialog
        dlg = DecryptDialog(str(fp))
        dlg.exec()
        return

    splash = SplashScreen()
    splash.show()
    _run_setup(splash)
    app.exec()


if __name__ == "__main__":
    main()