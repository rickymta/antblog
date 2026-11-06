"""Luồng hồ sơ KYC của tài xế: trạng thái hợp lệ và service nào đẩy hồ sơ sang trạng thái nào.

Luồng: nộp hồ sơ -> OCR giấy tờ (kyc-ocr) -> so khớp khuôn mặt (kyc-face) -> chấm điểm rủi ro
(fraud-scoring) -> người duyệt. Bài 41 chỉ dựng bảng chuyển trạng thái; use case nối ba service
là bài 70.
"""

from enum import StrEnum


class KycStatus(StrEnum):
    SUBMITTED = "submitted"  # tài xế đã nộp ảnh, chưa service nào xử lý
    OCR_DONE = "ocr_done"  # kyc-ocr đã đọc xong các trường
    FACE_CHECKED = "face_checked"  # kyc-face đã so ảnh chân dung với selfie
    RISK_SCORED = "risk_scored"  # fraud-scoring đã chấm điểm rủi ro
    NEEDS_REVIEW = "needs_review"  # đang chờ người duyệt xem
    APPROVED = "approved"
    REJECTED = "rejected"


S = KycStatus

# Trạng thái -> các trạng thái được đi tiếp. Hai quy tắc đáng chú ý:
# 1. Bất kỳ bước nào không đủ tin cậy đều có đường sang NEEDS_REVIEW (ảnh quá mờ, điểm thấp...).
# 2. REJECTED chỉ đến từ NEEDS_REVIEW: máy không tự từ chối một người. Cạnh RISK_SCORED -> APPROVED
#    (duyệt tự động) không có trong bảng: chỉ đi được khi nghiệp vụ bật chính sách và truyền
#    `auto_approve=True`; mặc định tắt, lab không bật.
ALLOWED: dict[KycStatus, frozenset[KycStatus]] = {
    S.SUBMITTED: frozenset({S.OCR_DONE, S.NEEDS_REVIEW}),
    S.OCR_DONE: frozenset({S.FACE_CHECKED, S.NEEDS_REVIEW}),
    S.FACE_CHECKED: frozenset({S.RISK_SCORED, S.NEEDS_REVIEW}),
    S.RISK_SCORED: frozenset({S.NEEDS_REVIEW}),
    S.NEEDS_REVIEW: frozenset({S.APPROVED, S.REJECTED}),
    S.APPROVED: frozenset(),
    S.REJECTED: frozenset(),
}

# Service tạo ra kết quả khiến hồ sơ vào trạng thái này.
PRODUCED_BY: dict[KycStatus, str] = {
    S.OCR_DONE: "kyc-ocr",
    S.FACE_CHECKED: "kyc-face",
    S.RISK_SCORED: "fraud-scoring",
}


def advance(current: KycStatus, target: KycStatus, *, auto_approve: bool = False) -> KycStatus:
    """Trả về `target` nếu chuyển hợp lệ, không thì ném ValueError (không đổi trạng thái im lặng).

    `auto_approve=True` mở thêm đúng một cạnh: RISK_SCORED -> APPROVED. Mặc định tắt.
    """
    if auto_approve and (current, target) == (S.RISK_SCORED, S.APPROVED):
        return target
    if target not in ALLOWED[current]:
        raise ValueError(f"không được chuyển hồ sơ KYC từ {current.value} sang {target.value}")
    return target
