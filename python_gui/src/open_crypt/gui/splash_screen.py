import math, ctypes
from PyQt6.QtWidgets import QWidget
from PyQt6.QtCore import Qt, QTimer, QRectF, QPointF, QPropertyAnimation, QEasingCurve
from PyQt6.QtGui import QPainter, QColor, QFont, QFontDatabase, QBrush, QPen, QPainterPath

from open_crypt.gui.styles import *

SPLASH_W, SPLASH_H = 380, 280
SPLASH_DURATION_MS = 2800

def _animations_enabled() -> bool:
    try:
        v = ctypes.c_int(0)
        ctypes.windll.user32.SystemParametersInfoW(0x2001, 0, ctypes.byref(v), 0)
        return bool(v.value)
    except Exception:
        return True

class SplashScreen(QWidget):
    def __init__(self):
        super().__init__()
        self.setWindowFlags(Qt.WindowType.FramelessWindowHint | Qt.WindowType.WindowStaysOnTopHint)
        self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground)
        self.setFixedSize(SPLASH_W, SPLASH_H)
        self._center()
        self._progress = 0.0
        self._status = ""
        self._closed = False
        qb = QFontDatabase
        self._font = QFont(qb.systemFont(qb.SystemFont.GeneralFont))
        self._font.setPixelSize(13)
        if _animations_enabled():
            self.setWindowOpacity(0.0)
            self._fade = QPropertyAnimation(self, b"windowOpacity")
            self._fade.setDuration(300)
            self._fade.setStartValue(0.0)
            self._fade.setEndValue(1.0)
            self._fade.setEasingCurve(QEasingCurve.Type.OutCubic)
            self._fade.start()
        self._timer = QTimer(self)
        self._timer.timeout.connect(self._tick)
        self._timer.start(16)

    def _center(self):
        screen = self.screen()
        if screen:
            geo = screen.availableGeometry()
            sw, sh = geo.width(), geo.height()
        else:
            sw, sh = 1920, 1080
        self.move(sw // 2 - SPLASH_W // 2, sh // 2 - SPLASH_H // 2)

    def set_status(self, text: str):
        self._status = text

    def set_progress(self, pct: float):
        self._progress = pct

    def close_splash(self):
        if self._closed:
            return
        self._closed = True
        self._timer.stop()
        if _animations_enabled():
            self._fade = QPropertyAnimation(self, b"windowOpacity")
            self._fade.setDuration(200)
            self._fade.setStartValue(1.0)
            self._fade.setEndValue(0.0)
            self._fade.finished.connect(self.close)
            self._fade.start()
        else:
            self.close()

    def paintEvent(self, event):
        p = QPainter(self)
        p.setRenderHint(QPainter.RenderHint.Antialiasing)
        rect = self.rect()
        path = QPainterPath()
        path.addRoundedRect(QRectF(rect).adjusted(1, 1, -1, -1), 16, 16)
        p.setClipPath(path)
        p.fillPath(path, QColor(BG_DARK))
        cx, cy = rect.width() // 2, rect.height() // 2
        self._draw_icon(p, cx, cy - 50)
        self._draw_title(p, cx, cy - 8)
        self._draw_spinner(p, cx, cy + 36)
        self._draw_status(p, cx, cy + 65)
        p.end()

    def _draw_icon(self, p: QPainter, cx: int, cy: int):
        s = 40
        p.setPen(QPen(QColor(ACCENT), 3))
        p.setBrush(QBrush(QColor(ACCENT), Qt.BrushStyle.Dense4Pattern))
        rect = QRectF(cx - s // 2, cy - s // 2, s, s * 0.75)
        p.drawRoundedRect(rect, 6, 6)
        body = QRectF(cx - s // 4, cy - s // 6, s // 2, s // 2 + 2)
        p.setPen(QPen(QColor(ACCENT), 2))
        p.setBrush(QColor(ACCENT_HOVER))
        p.drawRect(body)

    def _draw_title(self, p: QPainter, cx: int, cy: int):
        f = QFont(self._font)
        f.setPixelSize(24)
        f.setBold(True)
        p.setFont(f)
        p.setPen(QColor(FG_WHITE))
        p.drawText(QRectF(cx - 100, cy - 14, 200, 28), Qt.AlignmentFlag.AlignCenter, "OpenCrypt")

    def _draw_spinner(self, p: QPainter, cx: int, cy: int):
        r = 14
        p.setPen(QPen(QColor(BG_LIGHT), 2))
        p.setBrush(Qt.BrushStyle.NoBrush)
        p.drawArc(QRectF(cx - r, cy - r, r * 2, r * 2), 0, 360 * 16)
        p.setPen(QPen(QColor(ACCENT), 2))
        span = int(min(self._progress * 360.0, 359.99) * 16)
        start = int((self._progress * 360.0 * 3) % (360 * 16))
        p.drawArc(QRectF(cx - r, cy - r, r * 2, r * 2), start, span)

    def _draw_status(self, p: QPainter, cx: int, cy: int):
        p.setFont(self._font)
        p.setPen(QColor(FG_MUTED))
        p.drawText(QRectF(cx - 140, cy - 10, 280, 20), Qt.AlignmentFlag.AlignCenter, self._status)

    def _tick(self):
        self._progress = min(self._progress + 0.008, 1.0)
        self.update()
