"""Che số giấy tờ và số điện thoại trước khi ghi vào hàng đợi hay nhật ký.

Quy ước loạt bài (bài 47): giữ ba số đầu và hai số cuối, che phần giữa bằng dấu sao: `099123456789` -> `099*******89`.

Cách làm: (1) cất đi các chuỗi được biết là KHÔNG phải số định danh (ngày giờ như `2026-03-02 10:00`, giờ `10:00`, tiền có dấu chấm
ngăn nghìn như `125.000.000`; số bắt đầu bằng 0 không bao giờ là tiền); (2) với phần còn lại, tìm mọi dãy chữ số có thể xen MỘT
dấu cách, chấm, gạch ngang hoặc gạch chéo giữa hai chữ số (`099 123-456 789`, `09 91 23 45 67 89`, `099/123/456/789`); (3) dãy có từ 9 chữ số
trở lên thì che theo quy ước trên và bỏ các dấu phân cách; (4) trả các chuỗi đã cất về chỗ cũ.

Giới hạn (nêu trong bài): số viết liền từ 9 chữ số trở lên luôn bị che, kể cả khi đó là tiền viết liền (`125000000`); số tách bằng
chữ, xuống dòng hoặc ký tự lạ lọt qua; đây là lớp chặn cuối cho văn bản tự do, không thay cho việc không đưa số giấy tờ vào lý do.
"""

import re
from typing import Any

_PROTECT = re.compile(
    r"\d{4}-\d{2}-\d{2}(?:[ T]\d{1,2}:\d{2}(?::\d{2})?)?"  # ngày, ngày giờ
    r"|(?<![\d:])\d{1,2}:\d{2}(?::\d{2})?(?![\d:])"  # giờ
    r"|(?<![\d.])[1-9]\d{0,2}(?:\.\d{3})+(?![\d.])"  # tiền có dấu chấm ngăn nghìn (không bắt đầu bằng 0)
)
_SPAN = re.compile(r"(?<!\d)\d(?:[ ./\-]?\d)*(?!\d)")
_SENTINEL = "\x00{}\x00"


def _mask_span(match: re.Match[str]) -> str:
    digits = re.sub(r"\D", "", match.group())
    if len(digits) < 9:
        return match.group()
    return digits[:3] + "*" * (len(digits) - 5) + digits[-2:]


def scrub(text: str) -> str:
    kept: list[str] = []

    def keep(m: re.Match[str]) -> str:
        kept.append(m.group())
        return _SENTINEL.format(len(kept) - 1)

    out = _SPAN.sub(_mask_span, _PROTECT.sub(keep, text))
    return re.sub(r"\x00(\d+)\x00", lambda m: kept[int(m.group(1))], out)


def scrub_json(obj: Any) -> Any:
    """Che đệ quy mọi chuỗi trong dict/list (reasons ở dạng JSON)."""
    if isinstance(obj, str):
        return scrub(obj)
    if isinstance(obj, dict):
        return {k: scrub_json(v) for k, v in obj.items()}
    if isinstance(obj, list):
        return [scrub_json(v) for v in obj]
    return obj
