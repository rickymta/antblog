"""Lý do một hồ sơ vào hàng chờ duyệt. Mã ổn định để máy đọc, câu tiếng Việt để người đọc."""

from enum import StrEnum


class ReviewReason(StrEnum):
    NO_FACE_CONSENT = "no_face_consent"  # tài xế chưa đồng ý so khớp khuôn mặt
    OCR_LOW_CONFIDENCE = "ocr_low_confidence"  # một giấy tờ có độ tin cậy OCR dưới ngưỡng
    FACE_LOW_SCORE = "face_low_score"  # điểm khuôn mặt dưới ngưỡng
    RISK_HIGH = "risk_high"  # điểm rủi ro từ ngưỡng trở lên
    ROUTINE = "routine"  # không ngưỡng nào bị vượt; quy trình vẫn cần người duyệt


REASON_TEXT: dict[ReviewReason, str] = {
    ReviewReason.NO_FACE_CONSENT: "chưa có đồng ý so khớp khuôn mặt",
    ReviewReason.OCR_LOW_CONFIDENCE: "độ tin cậy OCR dưới ngưỡng",
    ReviewReason.FACE_LOW_SCORE: "điểm khuôn mặt dưới ngưỡng",
    ReviewReason.RISK_HIGH: "điểm rủi ro đạt ngưỡng xem xét",
    ReviewReason.ROUTINE: "kết quả kiểm tra tự động là đầu vào cho người duyệt",
}
