#!/usr/bin/env python3
"""Kiểm cấu trúc các bài LOC trong api/seed/content theo khung biên tập README.md.

Chạy từ bất kỳ đâu: python3 check_articles.py --from 1 --to 10
Thoát mã 1 nếu có lỗi. Chỉ dùng thư viện chuẩn.
"""
import argparse
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CONTENT = ROOT / "api/seed/content"
CURRICULUM = ROOT / "docs/location-master-curriculum.md"
LAB = "examples/toi-noi-location/"

SVG_CLASSES = set(
    "f-bg f-surface f-muted f-none f-dead f-phys f-phys-soft f-pk f-pk-soft f-warn f-warn-soft "
    "s-line s-muted s-dead s-phys s-pk s-warn t-b t-big t-small t-mono t-phys t-pk t-warn t-dead dash".split()
)
PATHS = {range(1, 31): "Nền tảng định vị", range(31, 51): "Tối ưu quãng đường hai phía", range(51, 65): "Dữ liệu và vận hành"}
ROW = re.compile(r"^\| LOC-(\d+) \| `([^`]+)` \| (.+?) \| ([^|]+?) \| ", re.M)


def lessons():
    return [(int(n), slug, title, lang.strip()) for n, slug, title, lang in ROW.findall(CURRICULUM.read_text("utf-8"))]


def frontmatter(text):
    m = re.match(r"---\n(.*?)\n---\n", text, re.S)
    if not m:
        return None, text
    fm = {}
    for line in m.group(1).splitlines():
        if ":" in line:
            k, v = line.split(":", 1)
            fm[k.strip()] = v.strip()
    return fm, text[m.end():]


def check(n, slug, title, lang):
    path = CONTENT / f"{slug}.md"
    if not path.exists():
        return ["thiếu file"]
    text = path.read_text("utf-8")
    fm, body = frontmatter(text)
    errs = []
    if fm is None:
        return ["thiếu frontmatter"]

    if fm.get("slug") != slug:
        errs.append(f"slug frontmatter khác: {fm.get('slug')}")
    if not fm.get("title"):
        errs.append("thiếu title")
    summary = fm.get("summary", "").strip('"')
    if not summary.startswith(f"Bài {n:02d}:"):
        errs.append(f"summary phải bắt đầu «Bài {n:02d}:»")
    if len(summary) > 234:
        errs.append(f"summary dài {len(summary)} > 234")
    if fm.get("category") != "system-design":
        errs.append("category phải là system-design")
    want_level = "intermediate" if n <= 30 else "advanced"
    if fm.get("level") not in {want_level, "advanced", "intermediate"}:
        errs.append("level sai")
    tags = [t.strip() for t in fm.get("tags", "").strip("[]").split(",") if t.strip()]
    if not {"toi-noi", "location"} <= set(tags):
        errs.append("tags phải có toi-noi, location")
    if len(tags) > 10:
        errs.append("quá 10 tags")
    if fm.get("language") != "vi":
        errs.append("language phải là vi")

    path_name = next(v for k, v in PATHS.items() if n in k)
    if f"Đây là bài {n:02d} của lộ trình **{path_name}**" not in body:
        errs.append(f"thiếu câu «Đây là bài {n:02d} của lộ trình **{path_name}**»")
    if "/articles/du-an-mau-toi-noi" not in body:
        errs.append("thiếu link dự án mẫu")
    if "Thử trước khi đọc" not in body or "tkh-callout--example" not in body:
        errs.append("thiếu callout Thử trước khi đọc")
    if "Giới hạn của ví dụ" not in body:
        errs.append("thiếu «Giới hạn của ví dụ»")
    bt = re.search(r"^## Bài tập\s*$(.*?)(?=^## |\Z)", body, re.S | re.M)
    if not bt:
        errs.append("thiếu mục ## Bài tập")
    elif bt.group(1).count("<details") < 2:
        errs.append("Bài tập cần ít nhất 2 <details> lời giải")
    if not re.search(r"^## Tài liệu gốc\s*$", body, re.M):
        errs.append("thiếu ## Tài liệu gốc")
    if not re.search(r"^Bài (trước|tiếp):", body, re.M):
        errs.append("thiếu dòng điều hướng")
    if lang != "—" and not re.search(r"^## Chạy ví dụ và kiểm chứng\s*$", body, re.M):
        errs.append("thiếu ## Chạy ví dụ và kiểm chứng")

    sections = len(re.findall(r"^## ", body, re.M))
    if sections < 6:
        errs.append(f"chỉ có {sections} mục ##")

    svgs = re.findall(r"<svg\b.*?</svg>", body, re.S)
    if not svgs:
        errs.append("thiếu hình SVG")
    for i, svg in enumerate(svgs, 1):
        if not re.search(r'viewBox="0 0 880 \d+"', svg):
            errs.append(f"SVG {i}: viewBox phải rộng 880")
        if 'role="img"' not in svg or "aria-label=" not in svg:
            errs.append(f"SVG {i}: thiếu role/aria-label")
        try:
            ET.fromstring(svg)
        except ET.ParseError as e:
            errs.append(f"SVG {i}: XML lỗi: {e}")
        for cls in re.findall(r'class="([^"]+)"', svg):
            for c in cls.split():
                if c not in SVG_CLASSES:
                    errs.append(f"SVG {i}: lớp lạ {c}")
    if len(re.findall(r"<figcaption>Hình \d+\.", body)) < len(svgs):
        errs.append("mỗi hình cần figcaption «Hình N. …»")

    for s in set(re.findall(r"\]\(/articles/([a-z0-9-]+)", body)):
        if not (CONTENT / f"{s}.md").exists():
            errs.append(f"link tới bài không tồn tại: {s}")

    prose = re.sub(r"```.*?```", "", body, flags=re.S)
    if re.search(r"(?:US)?\$\s?\d|\b\d+(?:[.,]\d+)?\s?USD\b", prose):
        errs.append("có vẻ ghi giá tiền bằng USD")
    if 31 <= n <= 50 and re.search(r"\d\s?%", prose) and "mô phỏng" not in prose:
        errs.append("có số % nhưng không nói «mô phỏng»")

    for ref in set(re.findall(re.escape(LAB) + r"[\w./-]+", body)):
        ref = ref.rstrip(".,/")
        if "<" not in ref and "NN" not in ref and not (ROOT / ref).exists():
            errs.append(f"đường dẫn lab không tồn tại: {ref}")
    if lang != "—":
        for b in set(re.findall(r"--bin (loc\d+[a-z0-9_]*)", body)):
            if not list((ROOT / LAB / "rust/labs/src/bin").glob(f"{b}.rs")):
                errs.append(f"binary không tồn tại: {b}")

    words = len(body.split())
    if words < 1500:
        errs.append(f"quá ngắn: {words} từ")
    if words > 6500:
        errs.append(f"quá dài: {words} từ")

    if re.search(r"\b(TODO|FIXME|lorem)\b", body, re.I):
        errs.append("còn TODO/FIXME")
    return errs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--from", dest="start", type=int, default=1)
    ap.add_argument("--to", dest="end", type=int, default=64)
    a = ap.parse_args()
    bad = 0
    for n, slug, title, lang in lessons():
        if a.start <= n <= a.end:
            errs = check(n, slug, title, lang)
            if errs:
                bad += 1
                print(f"LOC-{n:02d} {slug}")
                for e in errs:
                    print(f"  - {e}")
    print(f"{a.end - a.start + 1 - bad}/{a.end - a.start + 1} bài đạt")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
