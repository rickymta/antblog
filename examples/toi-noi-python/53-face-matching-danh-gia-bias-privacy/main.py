"""Bài 53: đánh giá so khớp khuôn mặt theo nhóm, lưu embedding thay ảnh, đồng ý, retention, người quyết định.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify

Cần mạng lần chạy đầu: SFace (38,7 MB) và YuNet (0,2 MB) của OpenCV Zoo, bộ Olivetti faces (tải 4,26 MB) vào
cache, không commit. Bộ khuôn mặt Olivetti do AT&T Laboratories Cambridge thu thập; xem tài liệu scikit-learn.
Olivetti KHÔNG có nhãn nhân khẩu học: các "nhóm" dưới đây là giả định theo số thứ tự người hoặc là phép đo trên
ảnh (độ sáng), KHÔNG phải thuộc tính của con người. Mọi tỉ lệ in ra là số đo thật của lần chạy này trên 40
người, không suy ra được cho dân số tài xế của Tới Nơi.
"""

# ruff: noqa: E402
import os

os.environ.setdefault("OPENCV_LOG_LEVEL", "ERROR")  # tắt cảnh báo backend của cv2.dnn, không liên quan tới lab

import sys
import time
from datetime import UTC, datetime, timedelta
from pathlib import Path

import numpy as np

from data import aligned_faces, load_faces
from embedding import EMBEDDING_DIM, MODEL_VERSION, embed_all
from metrics import (
    bootstrap_ci, build_pairs, group_counts, random_split_gaps, rates, rule_of_three, threshold_at_far,
    threshold_transfer,
)
from privacy import (
    RETENTION_DAYS, Band, ConsentRequired, DecisionRejected, FaceIndex, HumanDecision, KycStatus,
    ModelVersionMismatch, TemplateStore, Thresholds, apply_decision, pseudonym, suggest, suggest_no_face,
    system_next_status,
)

HERE = Path(__file__).resolve().parent
OUT = HERE / "out"
QUIET = False
PEOPLE = 40
BOOTSTRAP_REPS = 2000
BRIGHTNESS_REPS = 1000
SPLITS = 2000
TRANSFER_SPLITS = 1000
SEED = 53
NOW = datetime(2026, 11, 6, 12, 0, tzinfo=UTC)  # cố định để output lặp lại; mã thật dùng Clock (bài 14)


def say(*parts: object) -> None:
    if not QUIET:
        print(*parts)


class Checker:
    def __init__(self) -> None:
        self.total = 0
        self.failures: list[str] = []

    def check(self, name: str, got: object, expected: object) -> None:
        self.total += 1
        if got != expected:
            self.failures.append(name)
            print(f"SAI: {name}: nhận {got!r}, kỳ vọng {expected!r}")

    def ok(self, name: str, condition: bool, detail: object = "") -> None:
        self.total += 1
        if not condition:
            self.failures.append(name)
            print(f"SAI: {name} {detail}")


def pct(value: float) -> str:
    return f"{value * 100:.1f}%"


def span(interval: tuple[float, float]) -> str:
    return f"[{pct(interval[0])}, {pct(interval[1])}]"


def pair_counts(person: np.ndarray, member: np.ndarray | None = None) -> tuple[int, int]:
    """(số cặp cùng người, số cặp khác người) mong đợi khi nhóm `member` có số ảnh mỗi người như thực tế."""
    chosen = person if member is None else person[member]
    sizes = np.bincount(chosen)
    total = len(chosen) * (len(chosen) - 1) // 2
    genuine = int((sizes * (sizes - 1) // 2).sum())
    return genuine, total - genuine


# ---------------------------------------------------------------- 1. ảnh và embedding


def load_embeddings() -> tuple[np.ndarray, np.ndarray, np.ndarray, int, str]:
    """(ảnh xám đã lọc, nhãn người, embedding, số ảnh không lấy được mặt, nguồn). Ảnh YuNet không lấy được
    mặt bị loại và được đếm riêng."""
    images, person = load_faces()
    OUT.mkdir(exist_ok=True)
    cache = OUT / "embeddings.npz"
    if cache.exists():
        saved = np.load(cache)
        if str(saved["model_version"]) == MODEL_VERSION and len(saved["keep"]) == len(saved["embeddings"]):
            keep = saved["keep"]
            return (images[keep], person[keep], saved["embeddings"], len(images) - len(keep),
                    "đọc từ out/embeddings.npz")
    crops, failures = aligned_faces(images)
    keep = np.array([i for i, crop in enumerate(crops) if crop is not None])
    embeddings = embed_all([crops[i] for i in keep])
    np.savez(cache, embeddings=embeddings, keep=keep, model_version=MODEL_VERSION)
    return (images[keep], person[keep], embeddings, failures,
            "tính mới bằng ONNX Runtime, lưu vào out/embeddings.npz")


def part_embeddings(c: Checker):
    say("== 1. Ảnh và embedding Olivetti: YuNet căn chỉnh, SFace nhúng ==")
    start = time.perf_counter()
    images, person, embeddings, failures, source = load_embeddings()
    say(f"400 ảnh 64x64 của 40 người; YuNet lấy được mặt ở {len(images)} ảnh, không lấy được ở {failures} ảnh "
        f"(bị loại và đếm riêng, không lùi âm thầm sang resize)")
    say(f"embedding {embeddings.shape}, {embeddings.dtype}, mô hình {MODEL_VERSION}; {source}: "
        f"{time.perf_counter() - start:.1f} giây")
    c.check("hình dạng embedding", embeddings.shape, (len(images), EMBEDDING_DIM))
    c.check("ảnh lấy được cộng ảnh loại bằng 400", len(images) + failures, 400)
    c.ok("vector chuẩn hóa L2", bool(np.allclose(np.linalg.norm(embeddings, axis=1), 1.0, atol=1e-5)))
    c.ok("còn đủ 40 người", len(set(person)) == PEOPLE, len(set(person)))
    return images, person, embeddings, failures


# ---------------------------------------------------------------- 2. đánh giá chung


def part_overall(c: Checker, pairs, person: np.ndarray):
    say("\n== 2. Đánh giá chung: một ngưỡng cho tất cả ==")
    n_gen, n_imp = len(pairs.genuine), len(pairs.impostor)
    say(f"{n_gen} cặp cùng người, {n_imp} cặp khác người; cosine trung bình: "
        f"cùng người {pairs.genuine.mean():.3f}, khác người {pairs.impostor.mean():.3f}")
    expected_gen, expected_imp = pair_counts(person)
    c.check("số cặp cùng người", n_gen, expected_gen)
    c.check("số cặp khác người", n_imp, expected_imp)
    thresholds = {}
    for far in (0.01, 0.001):
        t = threshold_at_far(pairs.impostor, far)
        got_far, got_frr, _, _ = rates(pairs, t)
        thresholds[far] = t
        say(f"mục tiêu FAR {pct(far)}: ngưỡng {t:.3f} -> FAR {pct(got_far)}, FRR {pct(got_frr)}")
        c.ok(f"FAR không vượt mục tiêu {far}", got_far <= far + 1e-12, got_far)
    c.ok("ngưỡng chặt hơn thì bỏ sót nhiều hơn",
         rates(pairs, thresholds[0.001])[1] > rates(pairs, thresholds[0.01])[1])
    return thresholds


# ---------------------------------------------------------------- 3. theo nhóm giả định


def report_group(label: str, pairs, member: np.ndarray, t: float, reps: int) -> dict:
    far, frr, n_imp, n_gen = rates(pairs, t, member)
    ci = bootstrap_ci(group_counts(pairs, t, member, PEOPLE), reps=reps, seed=SEED)
    say(f"{label:<16} {n_imp:>6} cặp khác người, {n_gen:>4} cặp cùng người | "
        f"FAR {pct(far):>6} {span(ci['far'])} | FRR {pct(frr):>6} {span(ci['frr'])}")
    return {"far": far, "frr": frr, "ci": ci, "n_imp": n_imp, "n_gen": n_gen}


def part_assumed_groups(c: Checker, pairs, person: np.ndarray, t: float) -> None:
    say("\n== 3. Nhóm GIẢ ĐỊNH theo số thứ tự người (không có nhãn nhân khẩu học) ==")
    say(f"ngưỡng chung {t:.3f}; khoảng trong ngoặc: 95%, lấy lại mẫu theo người có hoàn lại, "
        f"{BOOTSTRAP_REPS} lần")
    in_a = person < PEOPLE // 2
    group_a = report_group("A (người 0-19)", pairs, in_a, t, BOOTSTRAP_REPS)
    group_b = report_group("B (người 20-39)", pairs, ~in_a, t, BOOTSTRAP_REPS)
    gen_a, imp_a = pair_counts(person, in_a)
    c.check("số cặp khác người nhóm A", group_a["n_imp"], imp_a)
    c.check("số cặp cùng người nhóm A", group_a["n_gen"], gen_a)
    cross = int(((in_a[pairs.i] != in_a[pairs.j]) & ~pairs.same).sum())
    c.check("cặp trong A + trong B + chéo nhóm = tất cả cặp khác người",
            group_a["n_imp"] + group_b["n_imp"] + cross, len(pairs.impostor))
    again = bootstrap_ci(group_counts(pairs, t, in_a, PEOPLE), reps=BOOTSTRAP_REPS, seed=SEED)
    c.check("cùng seed cho cùng khoảng tin cậy", again, group_a["ci"])

    far_gap, frr_gap = group_a["far"] - group_b["far"], group_a["frr"] - group_b["frr"]
    far_random, frr_random = random_split_gaps(pairs, t, PEOPLE, splits=SPLITS, seed=SEED)
    far_lo, far_hi = np.percentile(far_random, [2.5, 97.5])
    frr_lo, frr_hi = np.percentile(frr_random, [2.5, 97.5])
    say(f"hiệu A - B: FAR {far_gap * 100:+.2f} điểm %, FRR {frr_gap * 100:+.1f} điểm %")
    say(f"{SPLITS} lần chia NGẪU NHIÊN 20/20: hiệu FAR trong khoảng [{far_lo * 100:+.2f}, {far_hi * 100:+.2f}] "
        f"điểm % (95%), hiệu FRR trong khoảng [{frr_lo * 100:+.1f}, {frr_hi * 100:+.1f}]")
    say(f"tỉ lệ lần chia ngẫu nhiên có |hiệu| lớn bằng hiệu quan sát: "
        f"FAR {np.mean(np.abs(far_random) >= abs(far_gap)):.2f}, FRR {np.mean(np.abs(frr_random) >= abs(frr_gap)):.2f}")
    c.check("có đủ lần chia ngẫu nhiên", (len(far_random), len(frr_random)), (SPLITS, SPLITS))
    # So phân vị, không so một lần chia: hiệu giữa hai nửa NGẪU NHIÊN phải bao quanh 0.
    c.ok("khoảng 95% hiệu FAR của các lần chia ngẫu nhiên bao quanh 0", far_lo <= 0 <= far_hi, (far_lo, far_hi))
    c.ok("khoảng 95% hiệu FRR của các lần chia ngẫu nhiên bao quanh 0", frr_lo <= 0 <= frr_hi, (frr_lo, frr_hi))

    say("ngưỡng học ở một nhóm (FAR 1% trong nhóm đó), đo ở nhóm KIA (hai chiều):")
    for learn_label, learn, test_label, test in (("A", in_a, "B", ~in_a), ("B", ~in_a, "A", in_a)):
        learn_pairs = learn[pairs.i] & learn[pairs.j] & ~pairs.same
        t_group = threshold_at_far(pairs.score[learn_pairs], 0.01)
        learn_far, learn_frr, _, _ = rates(pairs, t_group, learn)
        test_far, test_frr, _, _ = rates(pairs, t_group, test)
        say(f"  học ở {learn_label} (ngưỡng {t_group:.3f}; đo lại ở {learn_label}: FAR {pct(learn_far)}, "
            f"FRR {pct(learn_frr)}) -> đo ở {test_label}: FAR {pct(test_far)}, FRR {pct(test_frr)}")
        c.ok(f"FAR trên chính nhóm {learn_label} không vượt 1%", learn_far <= 0.01 + 1e-12, learn_far)

    far_out, frr_out = threshold_transfer(pairs, np.arange(PEOPLE), 0.01, splits=TRANSFER_SPLITS, seed=SEED)
    lo, hi = np.percentile(far_out, [2.5, 97.5])
    say(f"{TRANSFER_SPLITS} cách chia ngẫu nhiên rời người, học ngưỡng FAR 1% ở một nửa, đo ở nửa kia: "
        f"FAR đo được trung vị {pct(float(np.median(far_out)))}, khoảng 95% [{pct(lo)}, {pct(hi)}]; "
        f"FRR trung vị {pct(float(np.median(frr_out)))}, khoảng 95% "
        f"[{pct(float(np.percentile(frr_out, 2.5)))}, {pct(float(np.percentile(frr_out, 97.5)))}]")
    c.ok("khoảng 95% của FAR trên người chưa thấy bao quanh mục tiêu 1%", lo <= 0.01 <= hi, (lo, hi))


# ---------------------------------------------------------------- 4. nhóm đo được: độ sáng


def part_brightness(c: Checker, pairs, images: np.ndarray, t: float) -> None:
    say("\n== 4. Nhóm ĐO ĐƯỢC trên ảnh: độ sáng trung bình, chia ba phần bằng nhau ==")
    means = images.reshape(len(images), -1).mean(axis=1)
    low, high = np.quantile(means, [1 / 3, 2 / 3])
    say(f"độ sáng trung bình mỗi ảnh (0..1): nhỏ nhất {means.min():.3f}, lớn nhất {means.max():.3f}; "
        f"ranh giới ba phần {low:.3f} và {high:.3f}; khoảng tin cậy: {BRIGHTNESS_REPS} lần lấy lại mẫu theo người")
    groups = {"tối": means < low, "vừa": (means >= low) & (means < high), "sáng": means >= high}
    c.check("ba nhóm phủ đủ số ảnh", sum(int(m.sum()) for m in groups.values()), len(images))
    for label, member in groups.items():
        info = report_group(f"{label} ({int(member.sum())} ảnh)", pairs, member, t, BRIGHTNESS_REPS)
        c.ok(f"nhóm {label} có cặp để đo", info["n_imp"] > 0 and info["n_gen"] > 0)


# ---------------------------------------------------------------- 5. bao nhiêu cặp là đủ


def part_sample_size(c: Checker, pairs, person: np.ndarray) -> None:
    say("\n== 5. Bao nhiêu phép thử thì mới nói được FAR nhỏ ==")
    in_a = person < PEOPLE // 2
    for label, trials in (("nhóm 20 người", pair_counts(person, in_a)[1]), ("cả 40 người", len(pairs.impostor))):
        bound = rule_of_three(trials)
        say(f"{label}: {trials} cặp khác người; không thấy nhận nhầm nào thì cận trên một phía 95% của FAR "
            f"cũng chỉ xuống được khoảng {bound * 100:.4f}%, nếu các cặp độc lập")
    say("nhưng các cặp dùng chung ảnh và chung người: đơn vị độc lập thực sự là NGƯỜI, "
        "ở đây 20 hoặc 40, nên con số trên là cận lạc quan")
    c.check("quy tắc ba", rule_of_three(30000), 1e-4)


# ---------------------------------------------------------------- 6. dải điểm và khối lượng người duyệt


def part_bands(c: Checker, pairs, thresholds: dict[float, float]) -> Thresholds:
    say("\n== 6. Dải điểm là gợi ý cho người duyệt ==")
    th = Thresholds("th-2026-11-lab", review=threshold_at_far(pairs.impostor, 0.05), strong=thresholds[0.001])
    say(f"ngưỡng review {th.review:.3f} (FAR 5%), ngưỡng strong {th.strong:.3f} (FAR 0,1%); "
        "cả hai chọn trên chính dữ liệu đo nên lạc quan, xem mục 3")
    bands = np.where(pairs.score >= th.strong, "strong", np.where(pairs.score >= th.review, "review", "weak"))
    rows = {}
    for kind, mask in (("cùng người", pairs.same), ("khác người", ~pairs.same)):
        counts = {b: int((bands[mask] == b).sum()) for b in ("strong", "review", "weak")}
        rows[kind] = counts
        total = sum(counts.values())
        say(f"{kind:<11} " + ", ".join(f"{b} {n} ({pct(n / total)})" for b, n in counts.items()))
    c.check("mọi cặp rơi vào đúng một dải", sum(sum(v.values()) for v in rows.values()), len(pairs.i))
    return th


# ---------------------------------------------------------------- 7. riêng tư


def part_privacy(c: Checker, person: np.ndarray, embeddings: np.ndarray) -> None:
    say("\n== 7. Lưu embedding thay ảnh, đồng ý, retention, quyền xóa ==")
    store = TemplateStore()
    columns = store.columns("face_template")
    say(f"bảng face_template: {columns}")
    c.check("bảng mẫu không có cột ảnh", columns,
            ["driver_id", "embedding", "model_version", "created_at", "expires_at"])
    say(f"một mẫu: {embeddings[0].nbytes} byte (128 số float32); crop mặt 112x112x3 chưa nén: {112 * 112 * 3} byte")

    first = {p: int(np.flatnonzero(person == p)[0]) for p in range(4)}
    drivers = {f"DR-{p + 1:03d}": first[p] for p in range(4)}
    try:
        store.enroll("DR-001", embeddings[first[0]], MODEL_VERSION, NOW)
        refused = None
    except ConsentRequired as exc:
        refused = str(exc)
    say(f"lưu mẫu khi chưa có đồng ý: {refused}")
    c.ok("chưa đồng ý thì không lưu được", refused is not None)

    for driver, idx in drivers.items():
        store.grant_consent(driver, NOW)
        store.enroll(driver, embeddings[idx], MODEL_VERSION, NOW)
    index = FaceIndex()
    index.rebuild(store, MODEL_VERSION)
    probe = embeddings[int(np.flatnonzero(person == 1)[1])]  # ảnh khác của người số 1 (DR-002)
    best = index.search(probe, k=2)
    say(f"đưa một ảnh khác của DR-002 vào chỉ mục {len(index.vectors)} mẫu, chỉ có embedding: "
        f"top-1 {best[0][0]} ({best[0][1]:.3f}), top-2 {best[1][0]} ({best[1][1]:.3f})")
    c.check("mẫu đã lưu vẫn nhận ra đúng người", best[0][0], "DR-002")

    # Rút đồng ý: hết căn cứ xử lý thì mẫu phải biến mất ngay, không đợi tới hạn lưu; lịch sử đồng ý được giữ.
    removed = store.withdraw_consent("DR-004", NOW + timedelta(days=1))
    after_withdrawal = store.get("DR-004", MODEL_VERSION)
    history = store.consent_history("DR-004")
    store.grant_consent("DR-004", NOW + timedelta(days=2))
    say(f"DR-004 rút đồng ý: xóa {removed} mẫu; get -> {after_withdrawal}; lịch sử đồng ý {len(history)} dòng, "
        f"đã rút: {history[0][1] is not None}; cấp lại thì lịch sử có {len(store.consent_history('DR-004'))} dòng")
    c.check("rút đồng ý xóa mẫu", (removed, after_withdrawal), (1, None))
    c.ok("lịch sử đồng ý được giữ", len(history) == 1 and history[0][1] is not None, history)
    c.check("cấp lại thêm một dòng, không ghi đè", len(store.consent_history("DR-004")), 2)

    # Retention: mẫu hết hạn thì bị xóa (DR-004 đã bị xóa từ trước nên chỉ còn ba mẫu).
    c.check("chưa tới hạn: không xóa", store.purge_expired(NOW + timedelta(days=RETENTION_DAYS - 1)), 0)
    c.ok("mẫu còn nguyên", store.get("DR-001", MODEL_VERSION) is not None)
    expired = store.purge_expired(NOW + timedelta(days=RETENTION_DAYS))
    say(f"retention {RETENTION_DAYS} ngày: tại ngày {RETENTION_DAYS - 1} xóa 0 mẫu; tại ngày {RETENTION_DAYS} xóa {expired} mẫu")
    c.check("tới hạn: xóa ba mẫu còn lại", expired, 3)
    c.check("không còn mẫu", store.get("DR-001", MODEL_VERSION), None)

    # Quyền xóa: bảng gốc sạch nhưng bản sao dẫn xuất (chỉ mục) có thể còn.
    later = NOW + timedelta(days=RETENTION_DAYS)
    for driver in ("DR-002", "DR-003"):
        store.enroll(driver, embeddings[drivers[driver]], MODEL_VERSION, later)  # đồng ý của họ vẫn còn hiệu lực
    index.rebuild(store, MODEL_VERSION)
    removed = store.erase("DR-002", later)
    before_rebuild = index.search(probe, k=1)[0][0]
    index.rebuild(store, MODEL_VERSION)
    after_rebuild = index.search(probe, k=1)[0][0]
    say(f"xóa DR-002 theo yêu cầu: bảng gốc xóa {removed} mẫu; chỉ mục CHƯA dựng lại vẫn trả {before_rebuild}; "
        f"sau khi dựng lại trả {after_rebuild}")
    c.check("bảng gốc đã xóa", store.get("DR-002", MODEL_VERSION), None)
    c.check("bản sao dẫn xuất còn lộ nếu quên dựng lại", before_rebuild, "DR-002")
    c.check("dựng lại chỉ mục thì hết", after_rebuild, "DR-003")
    try:
        store.enroll("DR-002", embeddings[0], MODEL_VERSION, later)
        again = None
    except ConsentRequired as exc:
        again = str(exc)
    c.ok("sau khi rút đồng ý không lưu lại được", again is not None)

    events = store.audit_events()
    subject = pseudonym("DR-002")
    flat = " ".join(f"{e} {s}" for e, s in events)
    say(f"nhật ký kiểm toán có {len(events)} dòng, ví dụ {events[-1]}; biệt danh của DR-002 là {subject}, mã gốc không xuất hiện")
    c.ok("nhật ký không chứa mã tài xế rõ", "DR-002" not in flat)
    c.ok("nhật ký có sự kiện xóa", ("template_erased", subject) in events)
    c.ok("nhật ký không có vector", all(len(s) == 12 for _, s in events))

    # Đổi mô hình thì mẫu cũ không dùng được, và vì không lưu ảnh nên phải chụp lại.
    try:
        store.get("DR-003", "sface_2021dec@opencv-zoo/rgb112-v2")
        mismatch = None
    except ModelVersionMismatch as exc:
        mismatch = str(exc)
    say(f"đổi phiên bản mô hình: {mismatch}")
    c.ok("khác phiên bản mô hình thì từ chối so", mismatch is not None)


# ---------------------------------------------------------------- 8. người quyết định


def part_human(c: Checker, th: Thresholds) -> None:
    say("\n== 8. Hệ thống gợi ý, người quyết định ==")
    cases = [suggest("KYC-0001", 0.80, MODEL_VERSION, th),
             suggest("KYC-0002", (th.review + th.strong) / 2, MODEL_VERSION, th),
             suggest("KYC-0003", 0.10, MODEL_VERSION, th),
             suggest_no_face("KYC-0004", MODEL_VERSION, th)]
    expected = {Band.STRONG: KycStatus.FACE_CHECKED, Band.REVIEW: KycStatus.NEEDS_REVIEW,
                Band.WEAK: KycStatus.NEEDS_REVIEW, Band.NO_FACE: KycStatus.NEEDS_REVIEW}
    for s in cases:
        nxt = system_next_status(s)
        score = "không có điểm (YuNet không lấy được mặt)" if s.score is None else f"điểm {s.score:.3f}"
        say(f"{s.kyc_id}: {score} -> dải {s.band}, hệ thống đẩy hồ sơ tới {nxt}")
        c.check(f"{s.kyc_id}: hệ thống không tự duyệt", nxt, expected[s.band])
    c.ok("hệ thống không bao giờ ra approved/rejected",
         all(system_next_status(suggest("KYC-0009", s, MODEL_VERSION, th))
             not in (KycStatus.APPROVED, KycStatus.REJECTED) for s in np.linspace(-1, 1, 41)))

    good = HumanDecision("KYC-0001", KycStatus.APPROVED, "RV-001", "ảnh khớp giấy tờ, đã xem tay", NOW)
    c.check("người duyệt có tên và lý do, hồ sơ ở needs_review: nhận",
            apply_decision(KycStatus.NEEDS_REVIEW, good), KycStatus.APPROVED)
    bad = {
        "tác nhân tự động": HumanDecision("KYC-0001", KycStatus.APPROVED, "system:face-match", "điểm cao", NOW),
        "thiếu lý do": HumanDecision("KYC-0001", KycStatus.REJECTED, "RV-001", "  ", NOW),
        "kết cục không phải quyết định cuối": HumanDecision("KYC-0001", KycStatus.FACE_CHECKED, "RV-001", "x", NOW),
    }
    for name, decision in bad.items():
        try:
            apply_decision(KycStatus.NEEDS_REVIEW, decision)
            got = "nhận"
        except DecisionRejected as exc:
            got = str(exc)
        say(f"{name}: {got}")
        c.ok(f"{name}: bị từ chối", got != "nhận")
    for status in (KycStatus.SUBMITTED, KycStatus.FACE_CHECKED):
        try:
            apply_decision(status, good)
            early = "nhận"
        except DecisionRejected as exc:
            early = str(exc)
        say(f"hồ sơ ở {status}: {early}")
        c.ok(f"hồ sơ ở {status}: không nhận quyết định", early != "nhận")


# ---------------------------------------------------------------- chạy


def run() -> int:
    c = Checker()
    images, person, embeddings, _ = part_embeddings(c)
    pairs = build_pairs(embeddings, person)
    thresholds = part_overall(c, pairs, person)
    part_assumed_groups(c, pairs, person, thresholds[0.01])
    part_brightness(c, pairs, images, thresholds[0.01])
    part_sample_size(c, pairs, person)
    th = part_bands(c, pairs, thresholds)
    part_privacy(c, person, embeddings)
    part_human(c, th)
    if c.failures:
        print(f"FAIL {c.total - len(c.failures)}/{c.total}")
        return 1
    if QUIET:
        print(f"OK {c.total}/{c.total}")
    else:
        say(f"\nKiểm chứng: {c.total}/{c.total} ca đạt.")
    return 0


def main() -> None:
    global QUIET
    QUIET = "--verify" in sys.argv[1:]
    if not QUIET:
        print(f"tải máy lúc chạy (load average 1 phút): {os.getloadavg()[0]:.1f}")
    sys.exit(run())


if __name__ == "__main__":
    main()
