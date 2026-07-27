import time, ctypes, logging
from pathlib import Path

from PyQt6.QtWidgets import (
    QDialog, QVBoxLayout, QHBoxLayout, QPushButton,
    QLineEdit, QLabel, QFileDialog, QApplication, QWidget,
)
from PyQt6.QtCore import Qt, QPropertyAnimation, QEasingCurve, pyqtSignal, QThread, QUrl
from PyQt6.QtGui import QDesktopServices

from ..core.rust_bridge import get_bridge
from ..core.file_ops import secure_delete
from ..i18n import _t
from .styles import *

log = logging.getLogger(__name__)


def _animations_enabled() -> bool:
    try:
        SPI_GETCLIENTAREAANIMATION = 0x2001
        v = ctypes.c_int(0)
        ctypes.windll.user32.SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, ctypes.byref(v), 0)
        return bool(v.value)
    except Exception:
        return True


class CryptWorker(QThread):
    finished = pyqtSignal(bool, str, float)

    def __init__(self, fn):
        super().__init__()
        self.fn = fn

    def run(self):
        start = time.time()
        try:
            self.fn()
            self.finished.emit(True, "", time.time() - start)
        except Exception as e:
            self.finished.emit(False, str(e), time.time() - start)


class EncryptDialog(QDialog):
    def __init__(self, file_path: str):
        super().__init__()
        self.file_path = Path(file_path)
        self.setWindowFlags(Qt.WindowType.FramelessWindowHint)
        self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground, False)
        self._drag_pos = None
        self._working = False
        self._setup_ui()
        self._apply_stylesheet()
        self._center()
        self._do_animation()
        self._generate_key()

    def _center(self):
        w, h = 300, 140
        screen = QApplication.primaryScreen()
        if screen is not None:
            geo = screen.availableGeometry()
            sw, sh = geo.width(), geo.height()
        else:
            sw, sh = 1920, 1080
        self.setFixedSize(w, h)
        self.move(sw // 2 - w // 2, sh // 2 - h // 2)

    def _apply_stylesheet(self):
        self.setStyleSheet(f"""
            QDialog {{
                background-color: {BG_DARK};
            }}
            QLabel {{
                color: {FG_WHITE};
                font-size: 11pt;
            }}
            QLineEdit {{
                background-color: {BG_MID};
                color: {FG_WHITE};
                border: 1px solid {BG_LIGHT};
                border-radius: 4px;
                padding: 6px 10px;
                font-size: 11pt;
                font-family: Consolas;
            }}
            QPushButton#close {{
                background-color: transparent;
                color: {FG_MUTED};
                font-size: 14pt;
                padding: 2px 8px;
            }}
            QPushButton#close:hover {{
                color: {FG_WHITE};
                background-color: #c00;
            }}
            QPushButton#icon {{
                background-color: transparent;
                color: {FG_WHITE};
                border: none;
                border-radius: 4px;
                padding: 6px;
                font-size: 16pt;
            }}
            QPushButton#icon:hover {{
                background-color: {BG_LIGHT};
            }}
            QPushButton#icon:disabled {{
                color: {FG_MUTED};
            }}
        """)

    def _setup_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(16, 8, 16, 16)
        layout.setSpacing(10)

        title_bar = QWidget()
        title_bar.setFixedHeight(24)
        tl = QHBoxLayout(title_bar)
        tl.setContentsMargins(0, 0, 0, 0)
        lbl = QLabel("OpenCrypt")
        lbl.setStyleSheet(f"color: {FG_MUTED}; font-size: 10pt;")
        tl.addWidget(lbl)
        tl.addStretch()
        close_btn = QPushButton("✕")
        close_btn.setObjectName("close")
        close_btn.setFixedSize(24, 24)
        close_btn.clicked.connect(self.reject)
        tl.addWidget(close_btn)
        layout.addWidget(title_bar)

        self.key_display = QLineEdit()
        self.key_display.setReadOnly(True)
        self.key_display.setContextMenuPolicy(Qt.ContextMenuPolicy.NoContextMenu)
        self.key_display.setFocusPolicy(Qt.FocusPolicy.NoFocus)
        layout.addWidget(self.key_display)

        icon_row = QHBoxLayout()
        icon_row.setSpacing(8)
        icon_row.addStretch()

        self.gen_btn = QPushButton("⟳")
        self.gen_btn.setObjectName("icon")
        self.gen_btn.setToolTip(_t("tooltip_generate_key"))
        self.gen_btn.setFixedSize(36, 36)
        self.gen_btn.clicked.connect(self._generate_key)
        icon_row.addWidget(self.gen_btn)

        self.copy_btn = QPushButton("⧉")
        self.copy_btn.setObjectName("icon")
        self.copy_btn.setToolTip(_t("tooltip_copy_key"))
        self.copy_btn.setFixedSize(36, 36)
        self.copy_btn.clicked.connect(self._on_copy)
        icon_row.addWidget(self.copy_btn)

        self.save_btn = QPushButton("↓")
        self.save_btn.setObjectName("icon")
        self.save_btn.setToolTip(_t("tooltip_save_key"))
        self.save_btn.setFixedSize(36, 36)
        self.save_btn.clicked.connect(self._on_save)
        icon_row.addWidget(self.save_btn)

        icon_row.addStretch()
        layout.addLayout(icon_row)

        layout.addStretch()
        issue = QLabel(f'<a href="{REPO_URL}" style="color:{FG_DIM};text-decoration:none;font-size:8pt;">{_t("report_issue")}</a>')
        issue.setAlignment(Qt.AlignmentFlag.AlignLeft)
        issue.linkActivated.connect(lambda url: QDesktopServices.openUrl(QUrl(url)))
        layout.addWidget(issue)

    def _do_animation(self):
        if _animations_enabled():
            self.setWindowOpacity(0.0)
            anim = QPropertyAnimation(self, b"windowOpacity")
            anim.setDuration(200)
            anim.setStartValue(0.0)
            anim.setEndValue(1.0)
            anim.setEasingCurve(QEasingCurve.Type.OutCubic)
            anim.start()

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self._drag_pos = event.globalPosition().toPoint()
            event.accept()

    def mouseMoveEvent(self, event):
        if event.buttons() == Qt.MouseButton.LeftButton and self._drag_pos is not None:
            delta = event.globalPosition().toPoint() - self._drag_pos
            self.move(self.pos() + delta)
            self._drag_pos = event.globalPosition().toPoint()
            event.accept()

    def mouseReleaseEvent(self, event):
        self._drag_pos = None
        event.accept()

    def keyPressEvent(self, event):
        if event.key() == Qt.Key.Key_Escape:
            self.reject()
        super().keyPressEvent(event)

    def _generate_key(self):
        try:
            key = get_bridge().generate_random_key()
            self.key_display.setText(key)
        except Exception as e:
            log.error(f"keygen: {e}")

    def _on_copy(self):
        key = self.key_display.text()
        if not key:
            return
        QApplication.clipboard().setText(key)
        log.info("key copied")
        self._start_encrypt()

    def _on_save(self):
        key = self.key_display.text()
        if not key:
            return
        f, _ = QFileDialog.getSaveFileName(
            self, _t("save_key_title"),
            str(self.file_path.stem + "_key.key"),
            _t("key_filter")
        )
        if f:
            Path(f).write_text(key)
            log.info(f"key saved: {f}")
            self._start_encrypt()

    def _start_encrypt(self):
        if self._working:
            return
        key = self.key_display.text()
        if not key:
            return
        self._working = True
        self.gen_btn.setEnabled(False)
        self.copy_btn.setEnabled(False)
        self.save_btn.setEnabled(False)
        b = get_bridge()
        out = self.file_path.with_suffix(self.file_path.suffix + ".ocrypt")

        def work():
            b.encrypt_file_with_key(str(self.file_path), str(out), key)

        def done(success, msg, duration):
            self._working = False
            if success:
                log.info(f"encrypted {self.file_path.name} -> {out.name} ({duration:.1f}s)")
                try:
                    secure_delete(self.file_path)
                    log.info(f"original deleted: {self.file_path.name}")
                except Exception as e:
                    log.warning(f"delete original: {e}")
                self.accept()
                QApplication.instance().quit()
            else:
                log.error(f"encrypt: {msg}")
                self.gen_btn.setEnabled(True)
                self.copy_btn.setEnabled(True)
                self.save_btn.setEnabled(True)

        self._worker = CryptWorker(work)
        self._worker.finished.connect(done)
        self._worker.start()
