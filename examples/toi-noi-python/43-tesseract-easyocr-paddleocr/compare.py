"""Chạy một engine trên bộ ảnh, lưu kết quả ra JSON, rồi dựng bảng so sánh từ các file JSON đó.

Mỗi engine ghi `out/results-<engine>.json`. Bảng chỉ có hàng của engine nào ĐÃ có file kết quả:
engine chưa chạy (hoặc không chạy được trên Python này) không có cột và không có số bịa.
"""

import json
import os
import platform
import statistics
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

from dataset import VARIANTS, Sample, non_ascii_chars
from engines import OcrEngine
from ocr_common import (
    FIELDS,
    OcrLine,
    best_substring_distance,
    cpu_seconds,
    lines_from_json,
    lines_to_json,
    load_average,
    nfc,
    score_fields,
    write_json,
)

MB = 1024 * 1024


def path_size(path: Path) -> int:
    """Kích thước file, hoặc tổng kích thước mọi file trong thư mục; 0 nếu không có."""
    if path.is_file():
        return path.stat().st_size
    return sum(f.stat().st_size for f in path.rglob("*") if f.is_file()) if path.is_dir() else 0


def machine_description() -> str:
    try:
        chip = subprocess.run(["sysctl", "-n", "machdep.cpu.brand_string"], capture_output=True, text=True,
                              check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        chip = platform.processor() or platform.machine()
    return f"{chip}, {platform.platform()}"


def run_engine(engine: OcrEngine, samples: list[Sample], *, init_seconds: float, init_cpu: float) -> dict[str, Any]:
    """Đo từng ảnh: thời gian thật (perf_counter) và CPU (user + system, kể cả tiến trình con).

    Một lần chạy khởi động (không tính) để nạp cache, cấp phát bộ nhớ lần đầu. Mỗi ảnh chạy đúng
    một lần: 24 ảnh đã cho phân phối, và chạy lặp sẽ làm lâu mà không đổi kết luận lớn.
    """
    engine.recognize(samples[0].path)
    load_before = load_average()
    images = []
    for sample in samples:
        c0, t0 = cpu_seconds(), time.perf_counter()
        lines = engine.recognize(sample.path)
        seconds, cpu = time.perf_counter() - t0, cpu_seconds() - c0
        images.append({"id": sample.sample_id, "seconds": round(seconds, 4), "cpu_seconds": round(cpu, 4),
                       "lines": lines_to_json(lines)})
    return {
        "engine": engine.name,
        "info": engine.describe(),
        "python": sys.version.split()[0],
        "machine": machine_description(),
        "cpu_count": os.cpu_count(),
        "load_before": round(load_before, 2),
        "load_after": round(load_average(), 2),
        "init_seconds": round(init_seconds, 3),
        "init_cpu_seconds": round(init_cpu, 3),
        "model_mb": round(sum(path_size(p) for p in engine.model_files()) / MB, 1),
        "chars_needed": non_ascii_chars(samples),
        "chars_missing": engine.missing_chars(non_ascii_chars(samples)),
        "images": images,
    }


def results_path(out_dir: Path, name: str) -> Path:
    return out_dir / f"results-{name}.json"


def save_results(out_dir: Path, result: dict[str, Any]) -> Path:
    path = results_path(out_dir, result["engine"])
    write_json(path, result)
    return path


def load_all(out_dir: Path, names: tuple[str, ...]) -> list[dict[str, Any]]:
    """Đọc các file kết quả đang có theo thứ tự `names`; bỏ qua engine chưa có file."""
    return [json.loads(results_path(out_dir, n).read_text(encoding="utf-8"))
            for n in names if results_path(out_dir, n).exists()]


# ---- Tổng hợp --------------------------------------------------------------------------


def summarize(result: dict[str, Any], samples: list[Sample]) -> dict[str, Any]:
    by_id = {s.sample_id: s for s in samples}
    exact = {f: 0 for f in FIELDS}
    folded = {f: 0 for f in FIELDS}
    cer_by_variant: dict[str, list[float]] = {v: [] for v in VARIANTS}
    all_folded = 0
    name_by_variant = {v: 0 for v in VARIANTS}
    per_variant = {v: 0 for v in VARIANTS}
    for row in result["images"]:
        sample = by_id[row["id"]]
        scores = score_fields(sample.fields, lines_from_json(row["lines"]))
        for f in FIELDS:
            exact[f] += scores[f].exact
            folded[f] += scores[f].exact_folded
        name_by_variant[sample.variant] += scores["full_name"].exact
        per_variant[sample.variant] += 1
        all_folded += all(sc.exact_folded for sc in scores.values())
        cer_by_variant[sample.variant].append(sum(sc.cer for sc in scores.values()) / len(FIELDS))
    n = len(result["images"])
    seconds = [r["seconds"] for r in result["images"]]
    cpu = [r["cpu_seconds"] for r in result["images"]]
    return {
        "n": n,
        "median_s": statistics.median(seconds), "min_s": min(seconds), "max_s": max(seconds),
        "median_cpu": statistics.median(cpu),
        "exact_pct": {f: 100 * exact[f] / n for f in FIELDS},
        "name_by_variant": name_by_variant, "per_variant": per_variant,
        "folded_pct": {f: 100 * folded[f] / n for f in FIELDS},
        "all_folded": all_folded,
        "cer_variant": {v: statistics.mean(x) for v, x in cer_by_variant.items() if x},
        "cer_all": statistics.mean(c for x in cer_by_variant.values() for c in x),
    }


def best_line_for(truth: str, lines: list[OcrLine]) -> OcrLine | None:
    """Dòng gần giá trị trường nhất (một dòng, không ghép cặp). Dùng để in ví dụ."""
    scored = [(best_substring_distance(nfc(truth), nfc(ln.text)), ln) for ln in lines]
    return min(scored, key=lambda pair: pair[0])[1] if scored else None


def confidence_report(result: dict[str, Any], samples: list[Sample], threshold: float = 0.8) -> dict[str, float]:
    """Dòng tốt nhất cho mỗi (ảnh, trường): confidence trung bình khi dòng đó đúng và khi sai.

    «Đúng» nghĩa là dòng chứa giá trị trường nguyên văn (giữ dấu). `wrong_above` là số dòng SAI
    mà confidence vẫn từ `threshold` trở lên: những lỗi một ngưỡng confidence sẽ bỏ lọt.
    `right_below` là số dòng ĐÚNG bị ngưỡng chặn nhầm (confidence dưới `threshold`).
    """
    by_id = {s.sample_id: s for s in samples}
    right: list[float] = []
    wrong: list[float] = []
    for row in result["images"]:
        sample = by_id[row["id"]]
        lines = lines_from_json(row["lines"])
        for name in FIELDS:
            best = best_line_for(sample.fields[name], lines)
            if best is None:
                continue
            exact = best_substring_distance(nfc(sample.fields[name]), nfc(best.text)) == 0
            (right if exact else wrong).append(best.confidence)
    return {
        "right_n": len(right), "wrong_n": len(wrong),
        "right_mean": statistics.mean(right) if right else float("nan"),
        "wrong_mean": statistics.mean(wrong) if wrong else float("nan"),
        "wrong_above": sum(c >= threshold for c in wrong),
        "right_below": sum(c < threshold for c in right),
    }


# ---- Bảng ------------------------------------------------------------------------------


def _table(header: list[str], rows: list[list[str]]) -> str:
    widths = [max(len(r[i]) for r in [header, *rows]) for i in range(len(header))]
    fmt = "  ".join(f"{{:<{w}}}" if i == 0 else f"{{:>{w}}}" for i, w in enumerate(widths))
    return "\n".join([fmt.format(*header), fmt.format(*("-" * w for w in widths)), *(fmt.format(*r) for r in rows)])


def format_report(results: list[dict[str, Any]], samples: list[Sample], threshold: float = 0.8) -> str:
    """Ba bảng (thời gian và mô hình, khớp nguyên văn theo trường, CER theo biến thể); mỗi engine một hàng."""
    if not results:
        return "(chưa có file kết quả nào trong out/)"
    sums = [summarize(r, samples) for r in results]
    n = sums[0]["n"]
    t1 = [[r["engine"], r["python"], f"{r['init_seconds']:.1f}", f"{s['median_s']:.2f}",
           f"{s['min_s']:.2f}-{s['max_s']:.2f}", f"{s['median_cpu']:.2f}", f"{r['model_mb']:.1f}",
           f"{r['load_before']:.1f}->{r['load_after']:.1f}"] for r, s in zip(results, sums, strict=True)]
    t2 = [[r["engine"], *(f"{s['exact_pct'][f]:.0f}" for f in FIELDS), f"{s['folded_pct']['full_name']:.0f}",
           f"{s['folded_pct']['hometown_city']:.0f}", f"{s['all_folded']}/{n}"]
          for r, s in zip(results, sums, strict=True)]
    t3 = [[r["engine"], *(f"{s['cer_variant'][v]:.3f}" for v in VARIANTS), f"{s['cer_all']:.3f}"]
          for r, s in zip(results, sums, strict=True)]
    t2b = [[r["engine"], *(f"{s['name_by_variant'][v]}/{s['per_variant'][v]}" for v in VARIANTS)]
           for r, s in zip(results, sums, strict=True)]
    t4 = []
    for r in results:
        c = confidence_report(r, samples, threshold)
        t4.append([r["engine"], f"{c['right_n']}", f"{c['right_mean']:.2f}", f"{c['right_below']}/{c['right_n']}",
                   f"{c['wrong_n']}", f"{c['wrong_mean']:.2f}" if c["wrong_n"] else "-",
                   f"{c['wrong_above']}/{c['wrong_n']}"])
    t5 = [[r["engine"], r["chars_needed"], "".join(r["chars_missing"]) or "(không thiếu)"]
          for r in results if r.get("chars_missing") is not None]
    parts = [
        f"Bảng 1. Thời gian trên {n} ảnh, CPU và mô hình\n" + _table(
            ["engine", "python", "nạp s", "s/ảnh (trung vị)", "min-max s", "CPU s/ảnh", "mô hình MB", "load 1 phút"], t1),
        f"Bảng 2. Phần trăm ảnh có trường khớp nguyên văn (giữ dấu); hai cột «bỏ dấu» là phần trăm khi bỏ dấu cả hai phía; "
        f"cột cuối: số ảnh đúng cả 5 trường khi bỏ dấu (/{n})\n"
        + _table(["engine", *FIELDS, "tên bỏ dấu", "quê bỏ dấu", "5 trường bỏ dấu"], t2),
        "Bảng 2b. Họ tên khớp nguyên văn theo biến thể (số ảnh trên 6)\n" + _table(["engine", *VARIANTS], t2b),
        "Bảng 3. CER trung bình theo biến thể (trung bình của 5 trường trên mỗi ảnh; thấp là tốt; "
        "chỉ so trong bài này, phép chấm khác CER gộp của bài 41)\n"
        + _table(["engine", *VARIANTS, "tất cả"], t3),
        "Bảng 4. Confidence của dòng tốt nhất cho mỗi trường (5 trường x ảnh), tách dòng đúng nguyên văn và dòng sai; "
        f"ngưỡng {threshold}\n"
        + _table(["engine", "đúng (n)", "conf TB", f"đúng mà <{threshold}", "sai (n)", "conf TB",
                  f"sai mà >={threshold}"], t4),
    ]
    if t5:
        parts.append("Bảng 5. Ký tự có dấu có trong đáp án của bộ ảnh mà bảng ký tự của mô hình nhận dạng KHÔNG có "
                     "(Tesseract không được kiểm)\n" + _table(["engine", "ký tự cần", "ký tự thiếu"], t5))
    return "\n\n".join(parts)
