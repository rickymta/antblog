"""LOC-39: học điểm đón từ lịch sử bằng DBSCAN, phân cụm trên đơn vị MÉT.

Đầu vào: `python/artifacts/loc39/pickups_history.csv` do lab Rust `loc39_hoc_diem_don --export` ghi (điểm đón của baseline trên
các seed lịch sử, dữ liệu mô phỏng). Đầu ra: `pickup_table.csv` (bảng điểm đón cho Rust nạp) và báo cáo in ra màn hình.

CẢNH BÁO vòng tròn: lịch sử do simulator sinh từ các điểm nóng ta đặt sẵn trong `fixtures/scenario-hcm-baseline.toml`, nên cụm học
được chủ yếu tìm lại điểm nóng đã đặt. Script in khoảng cách từ mỗi điểm nóng tới hub gần nhất để thấy điều đó. Mọi kết quả chỉ
chứng minh PHƯƠNG PHÁP; việc bảng học được có làm J tốt hơn hay không do lab Rust đo trên seed giữ lại.
"""

from __future__ import annotations

import argparse
import csv
import math
import tomllib
from pathlib import Path

import numpy as np
from sklearn.cluster import DBSCAN
from sklearn.neighbors import NearestNeighbors

ROOT = Path(__file__).resolve().parents[2]
R_EARTH = 6_371_008.8


def to_metres(lat: np.ndarray, lng: np.ndarray, lat0: float, lng0: float) -> np.ndarray:
    """Chiếu equirectangular quanh (lat0, lng0): x về đông, y về bắc, đơn vị mét. Đủ chính xác trong bbox một thành phố."""
    x = np.radians(lng - lng0) * math.cos(math.radians(lat0)) * R_EARTH
    y = np.radians(lat - lat0) * R_EARTH
    return np.column_stack([x, y])


def to_latlng(xy: np.ndarray, lat0: float, lng0: float) -> tuple[np.ndarray, np.ndarray]:
    lat = lat0 + np.degrees(xy[:, 1] / R_EARTH)
    lng = lng0 + np.degrees(xy[:, 0] / (R_EARTH * math.cos(math.radians(lat0))))
    return lat, lng


def load(path: Path) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    seeds, lat, lng = [], [], []
    with path.open(newline="") as f:
        for row in csv.DictReader(f):
            if row["outcome"] != "completed":
                continue
            seeds.append(int(row["seed"]))
            lat.append(float(row["lat"]))
            lng.append(float(row["lng"]))
    return np.array(seeds), np.array(lat), np.array(lng)


def coverage_of_bbox(hx: np.ndarray, sig: np.ndarray, lat0: float, lng0: float) -> float:
    """Tỉ lệ diện tích bbox lab (lat 10,70–10,86, lng 106,60–106,78) nằm trong 2σ của ít nhất một điểm nóng, tính trên lưới 100 m."""
    lo = to_metres(np.array([10.70]), np.array([106.60]), lat0, lng0)[0]
    hi = to_metres(np.array([10.86]), np.array([106.78]), lat0, lng0)[0]
    gx, gy = np.meshgrid(np.arange(lo[0], hi[0], 100.0), np.arange(lo[1], hi[1], 100.0))
    pts = np.column_stack([gx.ravel(), gy.ravel()])
    d = np.linalg.norm(pts[:, None, :] - hx[None, :, :], axis=2) / sig
    return float((d.min(axis=1) <= 2).mean())


def fit_hubs(xy: np.ndarray, eps: float, min_samples: int) -> tuple[np.ndarray, np.ndarray]:
    """DBSCAN trên mét. Mỗi cụm cho một hub là ĐIỂM THẬT của cụm gần tâm nhất (medoid xấp xỉ), để hub luôn là nơi đã có người đón."""
    labels = DBSCAN(eps=eps, min_samples=min_samples).fit_predict(xy)
    hubs, counts = [], []
    for k in sorted(set(labels) - {-1}):
        pts = xy[labels == k]
        c = pts.mean(axis=0)
        hubs.append(pts[np.argmin(((pts - c) ** 2).sum(axis=1))])
        counts.append(len(pts))
    return (np.array(hubs) if hubs else np.zeros((0, 2))), np.array(counts)


def coverage(hubs: np.ndarray, xy: np.ndarray, radius: float) -> float:
    """Tỉ lệ điểm đón nằm trong `radius` mét của một hub."""
    if len(hubs) == 0 or len(xy) == 0:
        return 0.0
    d, _ = NearestNeighbors(n_neighbors=1).fit(hubs).kneighbors(xy)
    return float((d[:, 0] <= radius).mean())


def selftest() -> None:
    """Kiểm tra đường ống bằng dữ liệu BỊA (ba đám mây điểm Gauss quanh ba toạ độ cho trước): DBSCAN trên mét phải tìm lại đúng ba hub.

    Không dùng dữ liệu simulator; chỉ để kiểm tra mã chiếu mét, DBSCAN và medoid mà không cần chạy lab Rust.
    """
    rng = np.random.default_rng(7)
    lat0, lng0 = 10.78, 106.69
    centres = np.array([[0.0, 0.0], [1500.0, 200.0], [-800.0, 1200.0]])  # mét
    pts = np.vstack([c + rng.normal(0, 12.0, size=(60, 2)) for c in centres] + [rng.uniform(-2500, 2500, size=(40, 2))])
    lat, lng = to_latlng(pts, lat0, lng0)
    back = to_metres(lat, lng, lat0, lng0)
    assert np.abs(back - pts).max() < 0.5, "chiếu mét hai chiều phải khớp"
    hubs, counts = fit_hubs(back, eps=30.0, min_samples=8)
    assert len(hubs) == 3, f"cần đúng 3 hub, có {len(hubs)}"
    for c in centres:
        d = np.linalg.norm(hubs - c, axis=1).min()
        assert d < 30.0, f"hub gần tâm {c} nhất cách {d:.1f} m"
    assert coverage(hubs, back[:180], 40.0) > 0.85
    print(f"selftest OK: {len(hubs)} hub, cỡ cụm {sorted(counts.tolist())}, (dữ liệu bịa, không phải simulator)")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--selftest", action="store_true", help="chạy kiểm tra bằng dữ liệu bịa rồi thoát")
    ap.add_argument("--history", type=Path, default=ROOT / "python/artifacts/loc39/pickups_history.csv")
    ap.add_argument("--out", type=Path, default=ROOT / "python/artifacts/loc39/pickup_table.csv")
    ap.add_argument("--scenario", type=Path, default=ROOT / "fixtures/scenario-hcm-baseline.toml")
    ap.add_argument("--val-seeds", type=int, default=10, help="số seed cuối dùng để chọn tham số (chia theo seed, không xáo trộn)")
    ap.add_argument("--cover-radius", type=float, default=100.0)
    ap.add_argument("--max-hubs", type=int, default=150)
    args = ap.parse_args()
    if args.selftest:
        selftest()
        return

    seeds, lat, lng = load(args.history)
    if len(seeds) == 0:
        raise SystemExit(f"LỖI: {args.history} trống hoặc không có chuyến hoàn thành; chạy lab Rust với --export trước")
    lat0, lng0 = float(lat.mean()), float(lng.mean())
    xy = to_metres(lat, lng, lat0, lng0)
    uniq = sorted(set(seeds.tolist()))
    print("== LOC-39: DBSCAN trên mét ==")
    print(f"đầu vào: {args.history.name}: {len(seeds)} điểm đón của chuyến hoàn thành, {len(uniq)} seed lịch sử ({uniq[0]}…{uniq[-1]}); dữ liệu mô phỏng")
    print(f"gốc chiếu (lat0, lng0) = ({lat0:.5f}, {lng0:.5f}); đơn vị mét (equirectangular)")

    # chẩn đoán chọn eps: khoảng cách tới láng giềng thứ k
    train_seeds = uniq[: len(uniq) - args.val_seeds]
    val_seeds = uniq[len(uniq) - args.val_seeds :]
    tr = np.isin(seeds, train_seeds)
    va = np.isin(seeds, val_seeds)
    print(f"chia theo seed (không xáo trộn): huấn luyện {len(train_seeds)} seed ({tr.sum()} điểm), kiểm định {len(val_seeds)} seed ({va.sum()} điểm)")
    for k in (5, 10, 20):
        d, _ = NearestNeighbors(n_neighbors=k + 1).fit(xy[tr]).kneighbors(xy[tr])
        q = np.percentile(d[:, k], [25, 50, 75, 90])
        print(f"  khoảng cách tới láng giềng thứ {k:>2}: p25 {q[0]:6.1f} m  p50 {q[1]:6.1f} m  p75 {q[2]:6.1f} m  p90 {q[3]:6.1f} m")

    print("\nquét tham số trên tập huấn luyện, chấm trên tập kiểm định (seed giữ lại):")
    print(f"  độ phủ = tỉ lệ điểm đón kiểm định cách một hub ≤ {args.cover_radius:.0f} m; «ngẫu nhiên» = cùng số hub chọn ngẫu nhiên trong điểm đón huấn luyện; «vượt» = DBSCAN trừ ngẫu nhiên")
    print(f"  {'eps (m)':>8} {'min_samples':>12} {'số hub':>7} {'điểm/hub TB':>12} {'nhiễu':>7} {'phủ':>7} {'ngẫu nhiên':>11} {'vượt':>7}  ghi chú")
    rng = np.random.default_rng(20261106)
    grid = [(eps, ms) for eps in (25, 50, 100, 150, 250, 400) for ms in (5, 10, 20, 40)]
    rows = []
    for eps, ms in grid:
        hubs, counts = fit_hubs(xy[tr], eps, ms)
        labels = DBSCAN(eps=eps, min_samples=ms).fit_predict(xy[tr])
        noise = float((labels == -1).mean())
        cov = coverage(hubs, xy[va], args.cover_radius)
        rand = coverage(xy[tr][rng.choice(tr.sum(), size=max(len(hubs), 1), replace=False)], xy[va], args.cover_radius) if len(hubs) else 0.0
        mean_pts = float(counts.mean()) if len(counts) else 0.0
        ok = 5 <= len(hubs) <= args.max_hubs and mean_pts >= 10
        rows.append((eps, ms, len(hubs), mean_pts, noise, cov, rand, cov - rand, ok))
        why = "" if ok else ("loại: quá nhiều hub" if len(hubs) > args.max_hubs else "loại: dưới 5 hub hoặc dưới 10 điểm/hub")
        print(f"  {eps:>8} {ms:>12} {len(hubs):>7} {mean_pts:>12.1f} {noise:>7.2f} {cov:>7.3f} {rand:>11.3f} {cov - rand:>+7.3f}  {why}")
    print(f"\nquy tắc chọn (đặt trước): trong các cấu hình có 5 ≤ số hub ≤ {args.max_hubs} và điểm/hub trung bình ≥ 10, chọn độ phủ VƯỢT ngẫu nhiên lớn nhất; hoà thì ít hub hơn.")
    ok = [r for r in rows if r[8]]
    if not ok:
        raise SystemExit("LỖI: không cấu hình nào qua quy tắc")
    best = sorted(ok, key=lambda r: (-r[7], r[2]))[0]
    eps, ms = best[0], best[1]
    print(f"→ chọn eps = {eps} m, min_samples = {ms}: {best[2]} hub, phủ kiểm định {best[5]:.3f} so với ngẫu nhiên {best[6]:.3f}")
    print("  đọc: hub chỉ phủ vài phần trăm điểm đón kiểm định. Trong mô phỏng, khách đứng ở vị trí ngẫu nhiên quanh điểm nóng (phân phối trơn), nên KHÔNG có «điểm đón nổi tiếng» ở cấp vài chục mét để học; DBSCAN chỉ tìm ra vùng nhu cầu.")

    # hồ sơ cuối: học trên toàn bộ seed lịch sử
    hubs, counts = fit_hubs(xy, eps, ms)
    labels = DBSCAN(eps=eps, min_samples=ms).fit_predict(xy)
    print(f"\nhồ sơ cuối (học trên cả {len(uniq)} seed): {len(hubs)} hub, {float((labels == -1).mean()):.2f} điểm là nhiễu; cỡ cụm p50 {int(np.percentile(counts, 50))}, max {int(counts.max())}")
    out_lat, out_lng = to_latlng(hubs, lat0, lng0)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["hub_id", "lat", "lng", "n_points", "n_seeds", "radius_m"])
        for i, k in enumerate(sorted(set(labels) - {-1})):
            m = labels == k
            r90 = float(np.percentile(np.linalg.norm(xy[m] - hubs[i], axis=1), 90))
            w.writerow([i, f"{out_lat[i]:.7f}", f"{out_lng[i]:.7f}", int(m.sum()), len(set(seeds[m].tolist())), f"{r90:.1f}"])
    print(f"đã ghi {args.out}")

    # vòng tròn: hub so với điểm nóng đã đặt
    sc = tomllib.loads(args.scenario.read_text())
    hs = sc["demand"]["hotspot"]
    hx = to_metres(np.array([h["lat"] for h in hs]), np.array([h["lng"] for h in hs]), lat0, lng0)
    d, _ = NearestNeighbors(n_neighbors=1).fit(hubs).kneighbors(hx)
    print("\nVÒNG TRÒN (nêu thẳng): lịch sử sinh từ các điểm nóng đã đặt sẵn; khoảng cách từ mỗi điểm nóng tới hub gần nhất:")
    for h, dist in zip(hs, d[:, 0]):
        within = int((np.linalg.norm(hubs - to_metres(np.array([h["lat"]]), np.array([h["lng"]]), lat0, lng0)[0], axis=1) <= 2 * h["sigma_m"]).sum())
        print(f"  {h['name'][:44]:<44} σ {h['sigma_m']:>5.0f} m, trọng số {h['weight']:.1f}: hub gần nhất {dist:7.0f} m; số hub trong 2σ: {within}")
    sig = np.array([h["sigma_m"] for h in hs], dtype=float)
    # σ chuẩn hoá: khoảng cách tới điểm nóng chia cho σ của CHÍNH điểm nóng đó, lấy điểm nóng gần nhất theo thước này
    z = np.array([np.min(np.linalg.norm(hx - p, axis=1) / sig) for p in hubs])
    n_in = int((z <= 2).sum())
    print(f"  {n_in}/{len(hubs)} hub ({n_in / len(hubs):.0%}) nằm trong 2σ (σ riêng của từng điểm nóng) của một điểm nóng đã đặt; {int((z <= 1).sum())} hub trong 1σ; xa nhất {z.max():.1f}σ")
    print(f"  «2σ» ở đây là vòng rộng: tám vòng 2σ phủ {coverage_of_bbox(hx, sig, lat0, lng0):.0%} diện tích bbox lab, nên con số trên chỉ nói hub nằm trong vùng đã đặt, không nói hub trùng điểm nóng")


if __name__ == "__main__":
    main()
