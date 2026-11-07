"""Dữ liệu tổng hợp cho adapter giả: ba tài xế, CCCD giả bắt đầu 099, GPLX giả bắt đầu 99."""

from kyc_service.domain.kyc import DocumentKind

# image_ref -> (loại giấy tờ, các trường «đọc được», độ tin cậy)
OCR_FIXTURES: dict[str, tuple[DocumentKind, dict[str, str], float]] = {
    "synthetic/cccd/DR-001.png": (
        DocumentKind.CCCD,
        {
            "full_name": "Nguyễn Văn A",
            "id_number": "099090000183",
            "date_of_birth": "15/03/1990",
            "sex": "Nam",
            "hometown_city": "Hà Nội",
        },
        0.97,
    ),
    "synthetic/gplx/DR-001.png": (
        DocumentKind.GPLX,
        {
            "full_name": "Nguyễn Văn A",
            "license_number": "990000000001",
            "date_of_birth": "15/03/1990",
        },
        0.93,
    ),
    # DR-002: ảnh CCCD mờ nên độ tin cậy thấp.
    "synthetic/cccd/DR-002.png": (
        DocumentKind.CCCD,
        {
            "full_name": "Trần Thị B",
            "id_number": "099190000271",
            "date_of_birth": "02/11/1990",
            "sex": "Nữ",
            "hometown_city": "TP.HCM",
        },
        0.58,
    ),
    "synthetic/gplx/DR-002.png": (
        DocumentKind.GPLX,
        {
            "full_name": "Trần Thị B",
            "license_number": "990000000002",
            "date_of_birth": "02/11/1990",
        },
        0.90,
    ),
    "synthetic/cccd/DR-003.png": (
        DocumentKind.CCCD,
        {
            "full_name": "Lê Văn C",
            "id_number": "099095000345",
            "date_of_birth": "21/07/1995",
            "sex": "Nam",
            "hometown_city": "Đà Nẵng",
        },
        0.95,
    ),
    "synthetic/gplx/DR-003.png": (
        DocumentKind.GPLX,
        {"full_name": "Lê Văn C", "license_number": "990000000003", "date_of_birth": "21/07/1995"},
        0.92,
    ),
}

# selfie_ref -> điểm tương đồng khuôn mặt
FACE_FIXTURES: dict[str, float] = {
    "synthetic/selfie/DR-001.png": 0.88,
    "synthetic/selfie/DR-002.png": 0.71,
    "synthetic/selfie/DR-003.png": 0.90,
}
