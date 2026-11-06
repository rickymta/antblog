"""Bài 30: NumPy, Pillow và OpenCV - ảnh là mảng, tiền xử lý ảnh giấy tờ.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify

Dùng thẻ căn cước giả của synthetic_card.py. Mọi ảnh sinh ra nằm trong out/ (gitignore).
Không cần mạng, không cần hạ tầng.
"""

import os
import statistics
import sys
import time
from collections.abc import Callable
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import cv2
import numpy as np
import PIL
from PIL import Image

from synthetic_card import CARD_HEIGHT, CARD_WIDTH, font_has_vietnamese, make_card_fields, make_id_card

OUT = Path(__file__).parent / "out"
SEED = 7
WORKERS = os.process_cpu_count() or 1
# Các ảnh demo() ghi vào out/.
SAVED = ("card.png", "gray.png", "card_half.png", "card_shadow.png",
         "otsu_clean.png", "otsu_shadow.png", "adaptive_shadow.png")
# Hệ số xám ITU-R BT.601 (cùng họ với Pillow "L" và OpenCV), viết dạng số nguyên /1000.
GRAY_WEIGHTS = np.array([299, 587, 114], dtype=np.int32)


# ---- Hàm tiền xử lý (code được trích trong bài) -------------------------------------------


def to_gray_broadcast(rgb: np.ndarray) -> np.ndarray:
    """RGB (H,W,3) uint8 -> xám (H,W) uint8 bằng broadcasting.

    `weights` có shape (3,); khi nhân với (H,W,3), NumPy so shape từ cuối: 3 khớp 3, nên
    trọng số được «kéo giãn» sang mọi điểm ảnh mà không tạo bản sao. Dùng float vì phép
    nhân uint8 với số lẻ cần chỗ chứa phần thập phân.
    """
    weights = np.array([0.299, 0.587, 0.114], dtype=np.float32)
    weighted = rgb * weights  # (H,W,3) uint8 * (3,) float32 -> (H,W,3) float32
    return np.rint(weighted.sum(axis=-1)).astype(np.uint8)  # cộng theo trục kênh -> (H,W)


def add_shadow(rgb: np.ndarray, strength: float = 0.55) -> np.ndarray:
    """Giả lập ánh sáng không đều: tối dần từ trái sang phải, bằng một phép nhân broadcasting.

    `factor` có shape (1,W,1): 1 khớp H, W khớp W, 1 khớp 3 kênh.
    """
    width = rgb.shape[1]
    factor = np.linspace(1.0, 1.0 - strength, width, dtype=np.float32).reshape(1, width, 1)
    return np.clip(rgb * factor, 0, 255).astype(np.uint8)


def otsu(gray: np.ndarray) -> tuple[float, np.ndarray]:
    """Ngưỡng toàn cục tự chọn (Otsu): một ngưỡng duy nhất cho cả ảnh."""
    threshold, binary = cv2.threshold(gray, 0, 255, cv2.THRESH_BINARY + cv2.THRESH_OTSU)
    return float(threshold), binary


def adaptive(gray: np.ndarray, block_size: int = 31, c: int = 10) -> np.ndarray:
    """Ngưỡng thích nghi: mỗi điểm so với trung bình Gauss của lân cận block_size x block_size."""
    return cv2.adaptiveThreshold(gray, 255, cv2.ADAPTIVE_THRESH_GAUSSIAN_C,
                                 cv2.THRESH_BINARY, block_size, c)


def ink_by_thirds(binary: np.ndarray) -> list[float]:
    """Tỉ lệ điểm đen (giá trị 0) ở ba dải dọc trái, giữa, phải. Dùng để thấy ngưỡng hỏng ở đâu."""
    width = binary.shape[1]
    return [float((binary[:, i * width // 3:(i + 1) * width // 3] == 0).mean()) for i in range(3)]


def binarize_vectorized(rgb: np.ndarray) -> np.ndarray:
    """Xám rồi nhị phân hóa ngưỡng 128 cho cả ảnh trong vài phép toán mảng, không vòng lặp."""
    gray1000 = rgb.astype(np.int32) @ GRAY_WEIGHTS  # tích vô hướng theo kênh, số nguyên
    return np.where(gray1000 >= 128_000, 255, 0).astype(np.uint8)


def binarize_cv(rgb: np.ndarray) -> np.ndarray:
    """Cùng phép tính bằng OpenCV: xám rồi ngưỡng; hàm C++ chuyên dụng cho đúng việc này."""
    gray = cv2.cvtColor(rgb, cv2.COLOR_RGB2GRAY)
    return cv2.threshold(gray, 127, 255, cv2.THRESH_BINARY)[1]


def binarize_loop_indexing(rgb: np.ndarray) -> np.ndarray:
    """Cùng phép tính bằng vòng lặp for, truy cập từng phần tử ndarray: cách người mới hay viết."""
    height, width, _ = rgb.shape
    out = np.zeros((height, width), dtype=np.uint8)
    for y in range(height):
        for x in range(width):
            gray1000 = 299 * int(rgb[y, x, 0]) + 587 * int(rgb[y, x, 1]) + 114 * int(rgb[y, x, 2])
            out[y, x] = 255 if gray1000 >= 128_000 else 0
    return out


def binarize_loop_lists(rgb: np.ndarray) -> np.ndarray:
    """Vòng lặp Python thuần trên list lồng nhau (`tolist()`): bản nhanh nhất có thể của vòng lặp."""
    rows = rgb.tolist()
    result = [[255 if 299 * r + 587 * g + 114 * b >= 128_000 else 0 for r, g, b in row] for row in rows]
    return np.array(result, dtype=np.uint8)


# ---- Đo và in ----------------------------------------------------------------------------


def best_median(fn: Callable[[], object], repeats: int) -> float:
    """Trung vị thời gian tường (perf_counter) của `repeats` lần gọi, tính bằng giây."""
    samples = []
    for _ in range(repeats):
        start = time.perf_counter()
        fn()
        samples.append(time.perf_counter() - start)
    return statistics.median(samples)


def demo() -> dict[str, object]:
    """Chạy toàn bộ bước, in kết quả, trả về các số cần cho --verify."""
    OUT.mkdir(exist_ok=True)
    facts: dict[str, object] = {}

    print(f"numpy {np.__version__}, pillow {PIL.__version__}, opencv {cv2.__version__}")
    print(f"font tiếng Việt hệ thống: {'có' if font_has_vietnamese() else 'không (dùng font mặc định, bỏ dấu)'}")

    # 1. Ảnh là mảng -------------------------------------------------------------------
    print("\n[1] Ảnh là mảng")
    card = make_id_card(SEED)
    fields = make_card_fields(SEED)
    card.save(OUT / "card.png")
    rgb = np.asarray(card)
    print(f"Pillow: size={card.size} (rộng, cao), mode={card.mode}")
    print(f"ndarray: shape={rgb.shape} (cao, rộng, kênh), dtype={rgb.dtype}, "
          f"strides={rgb.strides}, nbytes={rgb.nbytes:,}, ghi được={rgb.flags.writeable}, "
          f"base={type(rgb.base).__name__}")
    print(f"thẻ giả: {fields}")
    facts["rgb"] = rgb
    facts["fields"] = fields

    bgr = cv2.imread(str(OUT / "card.png"))
    print(f"cv2.imread: shape={bgr.shape}, kênh đầu = B: bằng RGB đảo kênh: "
          f"{np.array_equal(bgr, rgb[..., ::-1])}")
    facts["bgr_equals_reversed"] = np.array_equal(bgr, rgb[..., ::-1])

    # 2. dtype: uint8 tràn số ---------------------------------------------------------
    print("\n[2] dtype: uint8 chỉ chứa 0..255")
    a = np.array([[200, 100]], dtype=np.uint8)
    b = np.array([[100, 100]], dtype=np.uint8)
    wrapped = (a + b).tolist()
    saturated = cv2.add(a, b).tolist()
    widened = np.clip(a.astype(np.int16) + b, 0, 255).astype(np.uint8).tolist()
    print(f"a + b (NumPy, uint8)           = {wrapped}   (200 + 100 = 300 -> 300 - 256 = 44)")
    print(f"cv2.add(a, b) (bão hòa)        = {saturated}")
    print(f"clip(int16(a) + b, 0, 255)     = {widened}")
    facts["wrapped"], facts["saturated"] = wrapped, saturated

    # 3. Xám bằng broadcasting, so với Pillow và OpenCV -------------------------------
    print("\n[3] Ảnh xám")
    gray_np = to_gray_broadcast(rgb)
    gray_cv = cv2.cvtColor(rgb, cv2.COLOR_RGB2GRAY)
    gray_pil = np.asarray(card.convert("L"))
    diff_cv = int(np.abs(gray_np.astype(np.int16) - gray_cv).max())
    diff_pil = int(np.abs(gray_np.astype(np.int16) - gray_pil).max())
    print(f"broadcasting: shape={gray_np.shape}, dtype={gray_np.dtype}")
    print(f"lệch tối đa so với cv2.cvtColor: {diff_cv} mức xám; so với Pillow convert('L'): {diff_pil}")
    Image.fromarray(gray_np).save(OUT / "gray.png")
    facts["diff_cv"], facts["diff_pil"] = diff_cv, diff_pil

    # 4. Resize: thứ tự (rộng, cao) và (cao, rộng) -------------------------------------
    print("\n[4] Resize")
    new_size = (CARD_WIDTH // 2, CARD_HEIGHT // 2)  # (rộng, cao)
    small_pil = card.resize(new_size, Image.Resampling.LANCZOS)
    small_cv = cv2.resize(rgb, new_size, interpolation=cv2.INTER_AREA)  # dsize cũng là (rộng, cao)
    mean_diff = float(np.abs(np.asarray(small_pil).astype(np.int16) - small_cv).mean())
    print(f"đích (rộng, cao)={new_size}; Pillow.size={small_pil.size}; "
          f"ndarray Pillow={np.asarray(small_pil).shape}; ndarray OpenCV={small_cv.shape}")
    print(f"độ lệch trung bình Pillow LANCZOS và OpenCV INTER_AREA: {mean_diff:.3f} mức/kênh")
    small_pil.save(OUT / "card_half.png")
    facts["small_shape"], facts["resize_diff"] = small_cv.shape, mean_diff

    # 5. Ngưỡng: Otsu và thích nghi, ảnh sáng đều và ảnh bị bóng --------------------------
    print("\n[5] Ngưỡng hóa")
    clean_t, clean_bin = otsu(gray_np)
    shadow_rgb = add_shadow(rgb)
    shadow_gray = to_gray_broadcast(shadow_rgb)
    shadow_t, shadow_bin = otsu(shadow_gray)
    shadow_adaptive = adaptive(shadow_gray)
    clean_ink = ink_by_thirds(clean_bin)
    otsu_ink = ink_by_thirds(shadow_bin)
    adaptive_ink = ink_by_thirds(shadow_adaptive)
    clean_adaptive_ink = ink_by_thirds(adaptive(gray_np))

    def pct(values: list[float]) -> str:
        return "  ".join(f"{v * 100:6.2f}%" for v in values)

    print(f"Otsu, ảnh sáng đều: ngưỡng {clean_t:.0f}")
    print(f"Ảnh bị bóng (nhân hệ số 1,00 -> 0,45 từ trái sang phải): Otsu chọn ngưỡng {shadow_t:.0f}")
    print(f"tỉ lệ điểm đen   trái / giữa / phải")
    print(f"  sáng đều, Otsu         {pct(clean_ink)}")
    print(f"  sáng đều, thích nghi   {pct(clean_adaptive_ink)}")
    print(f"  bị bóng,  Otsu         {pct(otsu_ink)}")
    print(f"  bị bóng,  thích nghi   {pct(adaptive_ink)}")
    Image.fromarray(shadow_rgb).save(OUT / "card_shadow.png")
    Image.fromarray(clean_bin).save(OUT / "otsu_clean.png")
    Image.fromarray(shadow_bin).save(OUT / "otsu_shadow.png")
    Image.fromarray(shadow_adaptive).save(OUT / "adaptive_shadow.png")
    facts.update(clean_t=clean_t, clean_ink=clean_ink, otsu_ink=otsu_ink, adaptive_ink=adaptive_ink,
                 clean_adaptive_ink=clean_adaptive_ink)

    # 6. Vectorize thay vòng lặp -----------------------------------------------------
    print("\n[6] Vectorize thay vòng lặp (xám + ngưỡng 128 trên ảnh 1012 x 638)")
    print(f"load average 1 phút lúc đo: {os.getloadavg()[0]:.2f} (số lõi: {os.cpu_count()})")
    vec = binarize_vectorized(rgb)
    lst = binarize_loop_lists(rgb)
    idx = binarize_loop_indexing(rgb)
    same = bool(np.array_equal(vec, lst) and np.array_equal(vec, idx))
    t_vec = best_median(lambda: binarize_vectorized(rgb), 7)
    t_cv = best_median(lambda: cv2.threshold(cv2.cvtColor(rgb, cv2.COLOR_RGB2GRAY), 127, 255, cv2.THRESH_BINARY), 7)
    t_lst = best_median(lambda: binarize_loop_lists(rgb), 3)
    t_idx = best_median(lambda: binarize_loop_indexing(rgb), 1)
    print(f"ba bản cho cùng một kết quả: {same}")
    print(f"  NumPy vectorized (trung vị 7 lần)         {t_vec * 1000:9.2f} ms")
    print(f"  OpenCV cvtColor + threshold (trung vị 7)  {t_cv * 1000:9.2f} ms")
    print(f"  vòng lặp trên list (trung vị 3 lần)       {t_lst * 1000:9.2f} ms   chậm hơn NumPy x{t_lst / t_vec:,.0f}")
    print(f"  vòng lặp truy cập ndarray (1 lần)         {t_idx * 1000:9.2f} ms   chậm hơn NumPy x{t_idx / t_vec:,.0f}")
    facts.update(same=same, t_vec=t_vec, t_lst=t_lst, t_idx=t_idx)

    # 7. Có cần song song không? Đo thread trên lô thẻ -----------------------------
    print("\n[7] Thread trên một lô thẻ (bài 22: GIL chỉ nhả khi mã C chủ động nhả)")
    print(f"số thread: {WORKERS}; OpenCV tự dùng {cv2.getNumThreads()} thread nội bộ")
    cards = [np.asarray(make_id_card(SEED + i)) for i in range(8)]
    rows = (
        ("OpenCV cvtColor + threshold", binarize_cv, 400),
        ("NumPy vectorized", binarize_vectorized, 100),
        ("vòng lặp trên list", binarize_loop_lists, 20),
    )
    thread_speedups: dict[str, float] = {}
    thread_same = True
    for name, fn, count in rows:
        batch = [cards[i % len(cards)] for i in range(count)]
        load_before = os.getloadavg()[0]
        start = time.perf_counter()
        seq = [fn(card) for card in batch]
        t_seq = time.perf_counter() - start
        start = time.perf_counter()
        with ThreadPoolExecutor(WORKERS) as pool:
            par = list(pool.map(fn, batch))
        t_par = time.perf_counter() - start
        load_after = os.getloadavg()[0]
        thread_same = thread_same and all(np.array_equal(a, b) for a, b in zip(seq, par))
        thread_speedups[name] = t_seq / t_par
        print(f"  {name:<28} {count:4d} thẻ: tuần tự {t_seq * 1000:8.1f} ms, thread {t_par * 1000:8.1f} ms, "
              f"x{t_seq / t_par:5.2f}   (load {load_before:.1f} -> {load_after:.1f})")
    print(f"kết quả thread trùng tuần tự: {thread_same}")
    facts.update(thread_same=thread_same, thread_speedups=thread_speedups)

    print(f"\nẢnh đã lưu trong {OUT.name}/: {', '.join(SAVED)}")
    return facts


def id_ok(fields: dict[str, str]) -> bool:
    """Số thẻ có đúng cấu trúc so với ngày sinh và giới tính trên cùng thẻ không."""
    number, year = fields["id_number"], int(fields["date_of_birth"][-4:])
    male = fields["sex"] == "Nam"
    century_digit = (0 if male else 1) if year < 2000 else (2 if male else 3)
    return (len(number) == 12 and number.isdigit() and number.startswith("099")
            and number[3] == str(century_digit) and number[4:6] == f"{year % 100:02d}"
            and len(fields["date_of_birth"]) == 10 and fields["full_name"] == fields["full_name"].upper())


def verify() -> None:
    """So các kết quả kỳ vọng đã in trong bài; sai thì in FAIL và thoát với mã 1."""
    failures: list[str] = []
    total = 0

    def check(name: str, ok: bool, detail: str = "") -> None:
        nonlocal total
        total += 1
        if not ok:
            failures.append(name)
            print(f"SAI: {name} {detail}".rstrip())

    facts = demo()
    print()
    rgb = facts["rgb"]
    assert isinstance(rgb, np.ndarray)
    fields = facts["fields"]
    assert isinstance(fields, dict)

    check("shape thẻ (cao, rộng, kênh)", rgb.shape == (638, 1012, 3), f"nhận {rgb.shape}")
    check("dtype thẻ uint8", rgb.dtype == np.uint8)
    check("cùng seed cho cùng ảnh", np.array_equal(np.asarray(make_id_card(SEED)), rgb))
    check("khác seed cho ảnh khác", not np.array_equal(np.asarray(make_id_card(SEED + 1)), rgb))
    id_number = str(fields["id_number"])
    check("số CCCD giả 12 chữ số, bắt đầu 099", len(id_number) == 12 and id_number.isdigit()
          and id_number.startswith("099"), id_number)
    check("bộ trường chung của thẻ", set(fields) == {"full_name", "id_number", "date_of_birth", "sex",
                                                    "hometown_city"}, str(sorted(fields)))
    # Cấu trúc số thẻ: 099 + số giới tính/thế kỷ + 2 số năm sinh + 6 số ngẫu nhiên, khớp ngày sinh và giới tính.
    check("số thẻ khớp giới tính và năm sinh", all(
        id_ok(make_card_fields(seed)) for seed in range(40)), "một seed trong 0..39 không khớp")
    check("cv2.imread trả BGR = RGB đảo kênh", facts["bgr_equals_reversed"] is True)
    check("uint8 tràn số: 200 + 100 = 44", facts["wrapped"] == [[44, 200]], str(facts["wrapped"]))
    check("cv2.add bão hòa: 255", facts["saturated"] == [[255, 200]], str(facts["saturated"]))
    check("xám broadcasting lệch <= 1 so với cv2", facts["diff_cv"] <= 1, str(facts["diff_cv"]))
    check("xám broadcasting lệch <= 1 so với Pillow", facts["diff_pil"] <= 1, str(facts["diff_pil"]))
    check("resize về (319, 506, 3)", facts["small_shape"] == (319, 506, 3), str(facts["small_shape"]))
    check("Pillow và OpenCV resize lệch trung bình < 3 mức", float(facts["resize_diff"]) < 3.0,
          str(facts["resize_diff"]))
    check("Otsu ảnh sáng đều: ngưỡng trong 100..200", 100 <= float(facts["clean_t"]) <= 200, str(facts["clean_t"]))
    clean_ink, otsu_ink, adaptive_ink = facts["clean_ink"], facts["otsu_ink"], facts["adaptive_ink"]
    assert isinstance(clean_ink, list) and isinstance(otsu_ink, list) and isinstance(adaptive_ink, list)
    check("Otsu ảnh bị bóng: dải phải đen > 90%", otsu_ink[2] > 0.90, f"{otsu_ink[2]:.3f}")
    check("thích nghi ảnh bị bóng: dải phải đen < 2 lần ảnh sáng đều",
          adaptive_ink[2] < 2 * clean_ink[2], f"{adaptive_ink[2]:.4f} so với {clean_ink[2]:.4f}")
    check("vòng lặp và vectorized cùng kết quả", facts["same"] is True)
    check("vectorized nhanh hơn vòng lặp trên list (ít nhất 5 lần, chênh một bậc độ lớn)",
          float(facts["t_lst"]) / float(facts["t_vec"]) >= 5,
          f"{float(facts['t_lst']) / float(facts['t_vec']):.1f}x")
    check("kết quả thread trùng tuần tự", facts["thread_same"] is True)
    check("đã lưu ảnh vào out/", all((OUT / n).exists() for n in SAVED))

    if failures:
        print(f"FAIL {total - len(failures)}/{total}")
        sys.exit(1)
    print(f"OK {total}/{total}")


if __name__ == "__main__":
    if "--verify" in sys.argv:
        verify()
    else:
        demo()
