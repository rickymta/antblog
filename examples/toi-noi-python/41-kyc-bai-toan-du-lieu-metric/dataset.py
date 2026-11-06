"""Bộ dữ liệu KYC tổng hợp: 25 tài xế x 2 giấy tờ (CCCD, GPLX) = 50 ảnh, bốn biến thể chất lượng.

Mỗi ảnh đi kèm đáp án (ground truth) do chính hàm sinh ảnh trả về, nên đo được CER mà không
cần gán nhãn tay. `out/manifest.json` ghi đường dẫn ảnh và đáp án. Bộ này (seed 2026) dùng cho bài 41 và 42; các bài
sau trong phần 05 sinh bộ riêng với seed và tham số biến thể riêng, nên CER chỉ so được trong cùng bài.
"""

import json
from dataclasses import asdict, dataclass
from pathlib import Path

from synthetic_card import DOC_TYPES, VARIANTS, apply_variant, font_has_vietnamese, make_document, make_document_fields

DRIVERS = 25
BASE_SEED = 2026
MANIFEST_NAME = "manifest.json"


@dataclass(frozen=True)
class Sample:
    sample_id: str  # IMG-001
    kyc_id: str  # KYC-0001: một hồ sơ có hai ảnh
    driver_id: str  # DR-001
    doc_type: str  # "cccd" | "gplx"
    variant: str  # "clean" | "blur" | "rotate" | "dark"
    image: str  # đường dẫn tương đối so với thư mục chứa manifest
    fields: dict[str, str]  # đáp án: tên trường -> chuỗi đúng như in trên giấy tờ


def plan(drivers: int = DRIVERS, base_seed: int = BASE_SEED) -> list[Sample]:
    """Kế hoạch bộ dữ liệu, chưa vẽ ảnh. Biến thể của GPLX lệch một bước so với CCCD của cùng tài xế,
    nên mỗi tổ hợp (loại giấy tờ, biến thể) đều xuất hiện."""
    samples = []
    for k in range(1, drivers + 1):
        seed = base_seed + k
        for j, doc_type in enumerate(DOC_TYPES):
            sample_id = f"IMG-{2 * (k - 1) + j + 1:03d}"
            samples.append(Sample(
                sample_id=sample_id, kyc_id=f"KYC-{k:04d}", driver_id=f"DR-{k:03d}",
                doc_type=doc_type, variant=VARIANTS[(k - 1 + j) % len(VARIANTS)],
                image=f"images/{sample_id}.png", fields=make_document_fields(doc_type, seed)))
    return samples


def seed_of(sample: Sample, base_seed: int = BASE_SEED) -> int:
    return base_seed + int(sample.kyc_id.removeprefix("KYC-"))


def render(sample: Sample, base_seed: int = BASE_SEED):
    """Vẽ ảnh của một mẫu: giấy tờ sạch rồi áp biến thể."""
    seed = seed_of(sample, base_seed)
    return apply_variant(make_document(sample.doc_type, seed), sample.variant, seed)


def build_dataset(out_dir: Path, drivers: int = DRIVERS, base_seed: int = BASE_SEED) -> list[Sample]:
    """Sinh ảnh vào `out_dir/images/` và ghi `out_dir/manifest.json`."""
    (out_dir / "images").mkdir(parents=True, exist_ok=True)
    samples = plan(drivers, base_seed)
    for sample in samples:
        render(sample, base_seed).save(out_dir / sample.image)
    manifest = {"seed": base_seed, "generator": "synthetic_card.py", "vietnamese_font": font_has_vietnamese(),
                "count": len(samples), "items": [asdict(s) for s in samples]}
    (out_dir / MANIFEST_NAME).write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    return samples


def load_manifest(out_dir: Path) -> list[Sample]:
    data = json.loads((out_dir / MANIFEST_NAME).read_text(encoding="utf-8"))
    return [Sample(**item) for item in data["items"]]
