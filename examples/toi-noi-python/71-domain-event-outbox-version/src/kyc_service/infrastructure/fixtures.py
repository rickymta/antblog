"""Dữ liệu tổng hợp cho adapter giả: năm tài xế, CCCD giả bắt đầu 099, GPLX giả bắt đầu 99."""

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
    # DR-004: giấy tờ rõ nhưng ảnh selfie khớp kém.
    "synthetic/cccd/DR-004.png": (
        DocumentKind.CCCD,
        {
            "full_name": "Phạm Thị D",
            "id_number": "099196000412",
            "date_of_birth": "09/09/1996",
            "sex": "Nữ",
            "hometown_city": "Hải Phòng",
        },
        0.95,
    ),
    "synthetic/gplx/DR-004.png": (
        DocumentKind.GPLX,
        {
            "full_name": "Phạm Thị D",
            "license_number": "990000000004",
            "date_of_birth": "09/09/1996",
        },
        0.92,
    ),
    # DR-005: khuôn mặt khớp tốt nhưng ảnh CCCD hơi mờ.
    "synthetic/cccd/DR-005.png": (
        DocumentKind.CCCD,
        {
            "full_name": "Võ Văn E",
            "id_number": "099097000518",
            "date_of_birth": "30/01/1997",
            "sex": "Nam",
            "hometown_city": "Cần Thơ",
        },
        0.72,
    ),
    "synthetic/gplx/DR-005.png": (
        DocumentKind.GPLX,
        {"full_name": "Võ Văn E", "license_number": "990000000005", "date_of_birth": "30/01/1997"},
        0.90,
    ),
}

# selfie_ref -> điểm tương đồng khuôn mặt
FACE_FIXTURES: dict[str, float] = {
    "synthetic/selfie/DR-001.png": 0.88,
    "synthetic/selfie/DR-002.png": 0.71,
    "synthetic/selfie/DR-003.png": 0.90,
    "synthetic/selfie/DR-004.png": 0.62,
    "synthetic/selfie/DR-005.png": 0.90,
}
