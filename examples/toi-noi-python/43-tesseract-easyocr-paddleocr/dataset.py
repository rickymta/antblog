"""Bộ ảnh dùng chung cho cả ba engine: 6 thẻ căn cước giả x 4 biến thể = 24 ảnh.

Mỗi thẻ do `make_id_card(seed)` vẽ nên đáp án là `make_card_fields(seed)`, không gán nhãn tay.
Bốn biến thể mô phỏng điều hay gặp khi tài xế chụp bằng điện thoại: ảnh sạch, mờ, nghiêng, tối.
Bộ này nhỏ hơn bộ dữ liệu KYC đầy đủ của loạt bài (để PaddleOCR chạy hết trong vài phút);
mọi ảnh sinh vào out/images/ (gitignore), seed cố định nên lần nào cũng ra đúng ảnh cũ.
"""

import hashlib
from dataclasses import dataclass
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

from ocr_common import nfc_chars, write_json
from synthetic_card import make_card_fields, make_id_card

# Sáu seed chọn để sáu tên mẫu và sáu thành phố của synthetic_card mỗi thứ xuất hiện đúng một lần
# (seed liền nhau cho ra tên lặp lại, làm số đo nghiêng về vài tên). Thẻ seed 50 sinh năm 2001.
SEEDS = (1, 2, 4, 7, 21, 50)
VARIANTS = ("clean", "blur", "rotate", "dark")
BLUR_RADIUS = 2.0  # bán kính Gauss, điểm ảnh
ROTATE_DEGREES = 5.0  # xoay ngược chiều kim đồng hồ, đổi dấu theo seed chẵn lẻ
DARK_FACTOR = 0.45  # nhân độ sáng
PAPER = (205, 205, 205)  # màu lấp góc trống sau khi xoay


@dataclass(frozen=True)
class Sample:
    sample_id: str
    seed: int
    variant: str
    path: Path
    fields: dict[str, str]


def make_variant(card: Image.Image, variant: str, seed: int) -> Image.Image:
    """Biến thể của một thẻ sạch. `seed` chỉ dùng cho nhiễu và chiều xoay."""
    if variant == "clean":
        return card.copy()
    if variant == "blur":
        return card.filter(ImageFilter.GaussianBlur(BLUR_RADIUS))
    if variant == "rotate":
        angle = ROTATE_DEGREES if seed % 2 == 0 else -ROTATE_DEGREES
        return card.rotate(angle, resample=Image.Resampling.BICUBIC, expand=True, fillcolor=PAPER)
    if variant == "dark":
        rng = np.random.default_rng(seed)
        pixels = np.asarray(card, dtype=np.float32) * DARK_FACTOR
        pixels += rng.normal(0.0, 4.0, size=pixels.shape)  # thêm nhiễu cảm biến
        return Image.fromarray(np.clip(pixels, 0, 255).astype(np.uint8), mode="RGB")
    raise ValueError(f"biến thể lạ: {variant}")


def build_dataset(out_dir: Path) -> list[Sample]:
    """Sinh (hoặc sinh lại) 24 ảnh PNG và manifest.json trong out_dir."""
    images = out_dir / "images"
    images.mkdir(parents=True, exist_ok=True)
    samples: list[Sample] = []
    for seed in SEEDS:
        card = make_id_card(seed)
        for variant in VARIANTS:
            sample_id = f"s{seed}-{variant}"
            path = images / f"{sample_id}.png"
            make_variant(card, variant, seed).save(path)
            samples.append(Sample(sample_id, seed, variant, path, make_card_fields(seed)))
    write_json(out_dir / "manifest.json", [
        {"id": s.sample_id, "seed": s.seed, "variant": s.variant, "path": s.path.name, "fields": s.fields}
        for s in samples
    ])
    return samples


def non_ascii_chars(samples: list[Sample]) -> str:
    """Các ký tự ngoài ASCII (chữ Việt có dấu) có mặt trong đáp án của bộ ảnh, không trùng, đã sắp."""
    return "".join(sorted({c for s in samples for v in s.fields.values() for c in nfc_chars(v) if ord(c) > 127}))


def dataset_digest(samples: list[Sample]) -> str:
    """Băm nội dung các ảnh: hai lần sinh cùng seed phải cho cùng một giá trị."""
    digest = hashlib.sha256()
    for s in samples:
        digest.update(s.path.read_bytes())
    return digest.hexdigest()[:12]
