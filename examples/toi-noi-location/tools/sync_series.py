#!/usr/bin/env python3
"""Đồng bộ dữ liệu loạt bài theo các file đang có trong api/seed/content.

- Viết lại dòng «Bài trước / Bài tiếp», chỉ link bài đã có.
- Cập nhật cột Trạng thái trong curriculum.
- Ghi ba lộ trình vào api/seed/learning-paths.json, chỉ gồm bài đã có.
"""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CONTENT = ROOT / "api/seed/content"
CURRICULUM = ROOT / "docs/location-master-curriculum.md"
PATHS_FILE = ROOT / "api/seed/learning-paths.json"
ROW = re.compile(r"^\| LOC-(\d+) \| `([^`]+)` \|", re.M)
PATHS = [
    (
        "location-nen-tang-qua-toi-noi",
        "Service định vị · Nền tảng định vị",
        "Từ tọa độ tới engine định tuyến đường bộ bằng Rust, nhận và làm sạch GPS bằng Go và Rust, có bộ mô phỏng làm thước đo cho mọi tối ưu.",
        "intermediate",
        range(1, 31),
    ),
    (
        "location-toi-uu-quang-duong-qua-toi-noi",
        "Service định vị · Tối ưu quãng đường hai phía",
        "Trọng tâm của loạt: chọn điểm đón, điểm trả, ghép chuyến và giảm km rỗng cho cả khách lẫn tài xế, mọi cải thiện có số đo trên dữ liệu mô phỏng.",
        "advanced",
        range(31, 51),
    ),
    (
        "location-du-lieu-van-hanh-qua-toi-noi",
        "Service định vị · Dữ liệu và vận hành",
        "Python huấn luyện ETA và dự báo nhu cầu rồi đưa vào Rust; workspace hoàn chỉnh, kiểm thử tải, quan sát và triển khai.",
        "advanced",
        range(51, 65),
    ),
]


def main():
    text = CURRICULUM.read_text("utf-8")
    order = [(int(n), s) for n, s in ROW.findall(text)]
    exists = {n: s for n, s in order if (CONTENT / f"{s}.md").exists()}

    for n, slug in exists.items():
        path = CONTENT / f"{slug}.md"
        body = path.read_text("utf-8")
        parts = []
        if n - 1 in exists:
            parts.append(f"Bài trước: [bài {n - 1:02d}](/articles/{exists[n - 1]}).")
        if n + 1 in exists:
            parts.append(f"Bài tiếp: [bài {n + 1:02d}](/articles/{exists[n + 1]}).")
        line = " ".join(parts) or "Bài trước: chưa có."
        if n == 1 and n + 1 not in exists:
            line = "Bài trước: đây là bài đầu tiên."
        new, count = re.subn(r"^Bài (?:trước|tiếp):.*$", line, body, count=1, flags=re.M)
        if count == 0:
            new = re.sub(r"^## Tài liệu gốc\s*$", line + "\n\n## Tài liệu gốc", body, count=1, flags=re.M)
        path.write_text(new, "utf-8")

    def status(m):
        n = int(m.group(1))
        return m.group(0).replace("Kế hoạch |", "Đã viết |") if n in exists else m.group(0)

    text = re.sub(r"^\| LOC-(\d+) \|.*$", status, text, flags=re.M)
    CURRICULUM.write_text(text, "utf-8")

    data = json.loads(PATHS_FILE.read_text("utf-8"))
    keep = [p for p in data if not p["slug"].startswith("location-")]
    for slug, title, desc, level, rng in PATHS:
        articles = [exists[n] for n in rng if n in exists]
        if articles:
            keep.append(
                {
                    "slug": slug,
                    "title": title,
                    "description": desc,
                    "level": level,
                    "category": "system-design",
                    "articles": articles,
                    "status": "published",
                }
            )
    PATHS_FILE.write_text(json.dumps(keep, ensure_ascii=False, indent=2) + "\n", "utf-8")
    print(f"{len(exists)} bài; lộ trình cập nhật")


if __name__ == "__main__":
    main()
