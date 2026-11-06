"""Lưu embedding thay ảnh, đồng ý, retention, quyền xóa, và cổng quyết định của người.

Đây là mô hình thu nhỏ để thấy cơ chế, dùng SQLite trong bộ nhớ. Số ngày lưu, tên mục đích, nội dung
đồng ý đều là GIÁ TRỊ MINH HỌA của lab: chúng do nghiệp vụ và pháp chế của Tới Nơi quyết định,
không phải khuyến nghị pháp lý.
"""

import hashlib
import sqlite3
from dataclasses import dataclass
from datetime import datetime, timedelta
from enum import StrEnum

import numpy as np

PURPOSE = "kyc_face_match"
CONSENT_TEXT_VERSION = "consent-kyc-face-v1"
RETENTION_DAYS = 90
# Muối của biệt danh trong nhật ký kiểm toán. Biệt danh KHÔNG phải ẩn danh: mã tài xế có ít khả năng
# nên ai biết muối vẫn dò ngược được; nó chỉ tránh để mã rõ trong nhật ký.
AUDIT_SALT = "lab53-audit-salt"


class ConsentRequired(Exception):
    """Chưa có đồng ý còn hiệu lực cho mục đích này: không được lưu hay xử lý khuôn mặt."""


class ModelVersionMismatch(Exception):
    """Vector sinh từ hai phiên bản mô hình khác nhau không so sánh được."""


def pseudonym(driver_id: str) -> str:
    return hashlib.sha256((AUDIT_SALT + driver_id).encode()).hexdigest()[:12]


class TemplateStore:
    """Mẫu khuôn mặt (embedding) của tài xế. Không có cột nào chứa ảnh."""

    def __init__(self, path: str = ":memory:") -> None:
        self._db = sqlite3.connect(path)
        self._db.executescript(
            """
            CREATE TABLE consent (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                driver_id TEXT NOT NULL, purpose TEXT NOT NULL, text_version TEXT NOT NULL,
                granted_at TEXT NOT NULL, withdrawn_at TEXT);
            CREATE TABLE face_template (
                driver_id TEXT PRIMARY KEY, embedding BLOB NOT NULL, model_version TEXT NOT NULL,
                created_at TEXT NOT NULL, expires_at TEXT NOT NULL);
            CREATE TABLE audit (
                event TEXT NOT NULL, subject TEXT NOT NULL, at TEXT NOT NULL);
            """
        )

    # ---- đồng ý
    def grant_consent(self, driver_id: str, now: datetime) -> None:
        """Mỗi lần cấp là một dòng mới: lịch sử cấp và rút được giữ, không bị ghi đè."""
        self._db.execute(
            "INSERT INTO consent (driver_id, purpose, text_version, granted_at) VALUES (?, ?, ?, ?)",
            (driver_id, PURPOSE, CONSENT_TEXT_VERSION, now.isoformat()))
        self._audit("consent_granted", driver_id, now)

    def withdraw_consent(self, driver_id: str, now: datetime) -> int:
        """Rút đồng ý: đóng các dòng còn hiệu lực VÀ xóa mẫu của tài xế. Hết căn cứ xử lý thì không giữ
        vector, kể cả khi chưa tới hạn lưu. Trả số mẫu đã xóa."""
        self._db.execute(
            "UPDATE consent SET withdrawn_at = ? WHERE driver_id = ? AND purpose = ? AND withdrawn_at IS NULL",
            (now.isoformat(), driver_id, PURPOSE))
        removed = self._db.execute("DELETE FROM face_template WHERE driver_id = ?", (driver_id,)).rowcount
        self._audit("consent_withdrawn", driver_id, now)
        if removed:
            self._audit("template_deleted_on_withdrawal", driver_id, now)
        return removed

    def has_consent(self, driver_id: str) -> bool:
        row = self._db.execute(
            "SELECT 1 FROM consent WHERE driver_id = ? AND purpose = ? AND withdrawn_at IS NULL",
            (driver_id, PURPOSE)).fetchone()
        return row is not None

    def consent_history(self, driver_id: str) -> list[tuple[str, str | None]]:
        """[(granted_at, withdrawn_at)] theo thứ tự cấp."""
        return list(self._db.execute(
            "SELECT granted_at, withdrawn_at FROM consent WHERE driver_id = ? ORDER BY id", (driver_id,)))

    # ---- lưu và so khớp
    def enroll(self, driver_id: str, embedding: np.ndarray, model_version: str, now: datetime) -> None:
        if not self.has_consent(driver_id):
            raise ConsentRequired(f"{driver_id}: chưa có đồng ý cho mục đích {PURPOSE}")
        expires = now + timedelta(days=RETENTION_DAYS)
        self._db.execute(
            "INSERT OR REPLACE INTO face_template VALUES (?, ?, ?, ?, ?)",
            (driver_id, embedding.astype(np.float32).tobytes(), model_version,
             now.isoformat(), expires.isoformat()))
        self._audit("template_enrolled", driver_id, now)

    def get(self, driver_id: str, model_version: str) -> np.ndarray | None:
        row = self._db.execute(
            "SELECT embedding, model_version FROM face_template WHERE driver_id = ?",
            (driver_id,)).fetchone()
        if row is None:
            return None
        if row[1] != model_version:
            raise ModelVersionMismatch(f"mẫu lưu bằng {row[1]}, mô hình hiện tại {model_version}")
        return np.frombuffer(row[0], dtype=np.float32)

    def all_templates(self, model_version: str) -> dict[str, np.ndarray]:
        rows = self._db.execute(
            "SELECT driver_id, embedding FROM face_template WHERE model_version = ?",
            (model_version,)).fetchall()
        return {d: np.frombuffer(b, dtype=np.float32) for d, b in rows}

    # ---- retention và quyền xóa
    def purge_expired(self, now: datetime) -> int:
        """Xóa mẫu quá hạn lưu; trả số mẫu đã xóa. Chạy định kỳ (cron/worker như bài 40).

        Chỉ xóa MẪU: bản ghi đồng ý còn hiệu lực vẫn ở đó (muốn lưu lại thì chụp lại), và các bản sao dẫn xuất
        như `FaceIndex` chỉ hết mẫu khi được dựng lại.
        """
        rows = self._db.execute(
            "SELECT driver_id FROM face_template WHERE expires_at <= ?", (now.isoformat(),)).fetchall()
        for (driver_id,) in rows:
            self._db.execute("DELETE FROM face_template WHERE driver_id = ?", (driver_id,))
            self._audit("template_expired", driver_id, now)
        return len(rows)

    def erase(self, driver_id: str, now: datetime) -> int:
        """Quyền xóa: rút đồng ý (kéo theo xóa mẫu) và ghi một dòng kiểm toán chỉ có biệt danh, không có vector."""
        removed = self.withdraw_consent(driver_id, now)
        self._audit("template_erased", driver_id, now)
        return removed

    # ---- kiểm toán
    def _audit(self, event: str, driver_id: str, now: datetime) -> None:
        self._db.execute("INSERT INTO audit VALUES (?, ?, ?)", (event, pseudonym(driver_id), now.isoformat()))

    def audit_events(self) -> list[tuple[str, str]]:
        return [(e, s) for e, s, _ in self._db.execute("SELECT * FROM audit ORDER BY rowid")]

    def columns(self, table: str) -> list[str]:
        return [row[1] for row in self._db.execute(f"PRAGMA table_info({table})")]


class FaceIndex:
    """Bản sao dẫn xuất của mẫu để tìm kiếm nhanh (như chỉ mục ở bài 51). Bản sao nằm NGOÀI bảng gốc:
    xóa ở bảng gốc mà quên bản sao thì vẫn lộ."""

    def __init__(self) -> None:
        self.vectors: dict[str, np.ndarray] = {}

    def rebuild(self, store: TemplateStore, model_version: str) -> None:
        self.vectors = store.all_templates(model_version)

    def search(self, probe: np.ndarray, k: int = 1) -> list[tuple[str, float]]:
        scored = [(driver, float(np.dot(vec, probe))) for driver, vec in self.vectors.items()]
        return sorted(scored, key=lambda item: -item[1])[:k]


# ---------------------------------------------------------------- quyết định cuối do người


class KycStatus(StrEnum):
    SUBMITTED = "submitted"
    OCR_DONE = "ocr_done"
    FACE_CHECKED = "face_checked"
    RISK_SCORED = "risk_scored"
    NEEDS_REVIEW = "needs_review"
    APPROVED = "approved"
    REJECTED = "rejected"


class Band(StrEnum):
    STRONG = "strong"
    REVIEW = "review"
    WEAK = "weak"
    NO_FACE = "no_face"  # YuNet không lấy được mặt: không có điểm để so, không có phương án lùi âm thầm


@dataclass(frozen=True)
class Thresholds:
    version: str
    review: float   # dưới mức này: điểm thấp, gợi ý không khớp
    strong: float   # từ mức này: điểm cao, gợi ý khớp


@dataclass(frozen=True)
class MatchSuggestion:
    """Kết quả của hệ thống: gợi ý có kèm bằng chứng, KHÔNG phải quyết định."""

    kyc_id: str
    score: float | None
    band: Band
    model_version: str
    threshold_version: str


def suggest(kyc_id: str, score: float, model_version: str, th: Thresholds) -> MatchSuggestion:
    band = Band.STRONG if score >= th.strong else Band.REVIEW if score >= th.review else Band.WEAK
    return MatchSuggestion(kyc_id, score, band, model_version, th.version)


def suggest_no_face(kyc_id: str, model_version: str, th: Thresholds) -> MatchSuggestion:
    """Ảnh không lấy được mặt: báo riêng, không có điểm, hồ sơ đi tới người duyệt."""
    return MatchSuggestion(kyc_id, None, Band.NO_FACE, model_version, th.version)


def system_next_status(suggestion: MatchSuggestion) -> KycStatus:
    """Hệ thống chỉ được đẩy hồ sơ tới `face_checked` (dải strong, rồi hồ sơ đi tiếp tới `risk_scored` như
    bài 41) hoặc `needs_review`, không bao giờ tới `approved`/`rejected`: hai trạng thái đó chỉ đến từ một
    người, và theo bài 41 chỉ đi tới từ `needs_review`."""
    return KycStatus.FACE_CHECKED if suggestion.band is Band.STRONG else KycStatus.NEEDS_REVIEW


@dataclass(frozen=True)
class HumanDecision:
    kyc_id: str
    outcome: KycStatus
    reviewer_id: str
    reason: str
    decided_at: datetime


class DecisionRejected(Exception):
    pass


def apply_decision(current: KycStatus, decision: HumanDecision) -> KycStatus:
    """Chỉ chấp nhận quyết định có người ký tên và có lý do, và chỉ từ `needs_review` (bài 41).

    Danh tính người duyệt ở đây chỉ kiểm tiền tố `RV-` để lab chạy được; production lấy danh tính từ xác
    thực (token, SSO), không tin một chuỗi do bên gọi tự khai.
    """
    if decision.outcome not in (KycStatus.APPROVED, KycStatus.REJECTED):
        raise DecisionRejected(f"kết cục {decision.outcome} không phải quyết định cuối")
    if current is not KycStatus.NEEDS_REVIEW:
        raise DecisionRejected(f"hồ sơ ở trạng thái {current}: chỉ needs_review mới nhận quyết định của người")
    if not decision.reviewer_id.startswith("RV-"):
        raise DecisionRejected(f"người quyết định {decision.reviewer_id!r} không phải người duyệt (RV-...)")
    if not decision.reason.strip():
        raise DecisionRejected("thiếu lý do quyết định")
    return decision.outcome
