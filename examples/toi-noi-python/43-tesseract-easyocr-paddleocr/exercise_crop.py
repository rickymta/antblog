"""Lời giải bài tập 1: cắt bỏ ô ảnh chân dung (x < 310) trước khi đưa vào Tesseract.

Chạy: uv run --project <lab> <lab>/exercise_crop.py
Dòng họ tên bị Tesseract thêm hai từ rác nhỏ (confidence 27 và 47, ở x=108 và x=197, y=299) nằm
trong ô ảnh chân dung, cùng dải với chữ «ẢNH MẪU» vẽ giữa ô (tâm ở x=167, y=315); chữ họ tên thật
bắt đầu ở x=330. Ở đây ta cắt cố định theo bố cục của thẻ giả; thẻ chụp thật không có bố cục cố
định, đó là việc của bài 42 (tìm thẻ, nắn phẳng) và của mô hình phát hiện vùng chữ.
"""

import sys
from pathlib import Path

from PIL import Image

from compare import best_line_for
from dataset import build_dataset
from engines import TesseractEngine
from ocr_common import FIELDS, score_fields

OUT = Path(__file__).parent / "out"
CROP_LEFT = 310  # ô ảnh chân dung kết thúc ở x=290; chữ trường bắt đầu từ x=330 trên thẻ không xoay


def main() -> int:
    samples = [s for s in build_dataset(OUT) if s.variant in ("clean", "dark")]  # thẻ thẳng: cắt cố định mới đúng
    engine = TesseractEngine()
    cropped_dir = OUT / "cropped"
    cropped_dir.mkdir(exist_ok=True)
    first = samples[0]  # s1-clean: in dòng họ tên trước và sau khi cắt
    totals = {"nguyên ảnh": dict.fromkeys(FIELDS, 0), f"cắt x >= {CROP_LEFT}": dict.fromkeys(FIELDS, 0)}
    for s in samples:
        with Image.open(s.path) as image:
            cropped = image.crop((CROP_LEFT, 0, image.width, image.height))
        cropped_path = cropped_dir / s.path.name
        cropped.save(cropped_path)
        for label, path in zip(totals, (s.path, cropped_path), strict=True):
            if s is first:
                line = best_line_for(s.fields["full_name"], engine.recognize(path))
                print(f"{s.sample_id} {label}: dòng họ tên = {line.text!r}, hộp {line.box}")
            for name, score in score_fields(s.fields, engine.recognize(path)).items():
                totals[label][name] += score.exact
    print(f"Số ảnh có trường khớp nguyên văn, trên {len(samples)} ảnh thẻ thẳng (sạch và tối), Tesseract -l vie+eng --psm 6")
    print(f"{'':<16}" + "".join(f"{f:>15}" for f in FIELDS))
    for label, counts in totals.items():
        print(f"{label:<16}" + "".join(f"{counts[f]:>15}" for f in FIELDS))
    return 0


if __name__ == "__main__":
    sys.exit(main())
