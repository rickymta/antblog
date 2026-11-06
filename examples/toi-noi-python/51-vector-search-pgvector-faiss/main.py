"""Bài 51: vector search với pgvector và FAISS: tìm một người đăng ký nhiều tài khoản.

Chạy:   uv run --project examples/toi-noi-python/51-vector-search-pgvector-faiss examples/toi-noi-python/51-vector-search-pgvector-faiss/main.py
Verify: ... main.py --verify
Tùy chọn: --distractors N (số tài khoản nền tổng hợp, mặc định 100000), --keep (không xóa schema lab51)

Cần hạ tầng chung: docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
Cần mạng ở lần chạy đầu: scikit-learn tải Olivetti (~1,3 MB). Mọi thứ nằm trong schema `lab51`, xóa khi xong.
Embedding là PCA trên Olivetti, tài khoản nền là vector tổng hợp: xem docstring của embedding.py.
"""

import argparse
import itertools
import os
import subprocess
import sys
from collections import namedtuple
from collections.abc import Callable
from pathlib import Path
from typing import Any

import faiss
import numpy as np
import psycopg
from threadpoolctl import threadpool_limits

import ann
import embedding
import store

K = ann.TOP_K
Row = namedtuple("Row", "name p50 p95 recall found top1")


def row_of(driver: str) -> int:
    return int(driver.removeprefix("DR-")) - 1


def show(row: Row) -> None:
    print(f"  {row.name:<34}{row.p50:>8.2f}{row.p95:>8.2f}{row.recall:>10.3f}{row.found:>9.3f}{row.top1:>8.3f}")


def header() -> None:
    print(f"  {'cấu hình':<34}{'p50 ms':>8}{'p95 ms':>8}{'recall@10':>10}{'found@10':>9}{'top1':>8}")


def run(n_distractors: int, keep: bool, verify: bool) -> int:
    conn = store.connect()
    # Cả bốn phía đo dùng một luồng tính toán: PostgreSQL không chia việc cho worker song song, FAISS và BLAS (NumPy) một luồng.
    conn.execute("SET max_parallel_workers_per_gather = 0")
    real, real_person, queries, query_person = embedding.load_embeddings()
    distractors = embedding.synthetic_distractors(real, n_distractors)
    base = np.vstack([real, distractors])
    total = len(base)
    ids = [store.driver_id(i + 1) for i in range(total)]
    persons: list[int | None] = [int(p) for p in real_person] + [None] * n_distractors
    # Danh sách "đã bị khóa": toàn bộ tài khoản thật của 4 người đầu (20 tài khoản) và cứ 100 tài khoản nền có 1 (1%).
    banned = np.array([(i < len(real) and real_person[i] < 4) or (i >= len(real) and (i - len(real)) % 100 == 0) for i in range(total)])
    statuses = ["banned" if b else "active" for b in banned]
    truth = ann.exact_topk(queries, base)
    load = os.getloadavg()
    print(f"embedding PCA {embedding.DIM} chiều; {len(real)} tài khoản thật (40 người x 5) + {n_distractors} tài khoản nền tổng hợp = {total} dòng")
    print(f"{len(queries)} truy vấn = ảnh đăng ký mới (ảnh 5-9 của 40 người), top-{K}; load average lúc bắt đầu {load[0]:.2f} {load[1]:.2f} {load[2]:.2f}")

    rows: dict[str, Row] = {}
    meta: dict[str, Any] = {}

    def measure(name: str, search: Callable[[np.ndarray], list[int]]) -> Row:
        found, p50, p95 = ann.time_queries(search, queries)
        r = Row(name, p50, p95, ann.recall_at_k(found, truth), ann.identity_found(found, query_person, real_person),
                ann.top1_same_person(found, query_person, real_person))
        rows[name] = r
        meta[name + ":found"] = found
        show(r)
        return r

    # --- 1. Chỉ có tài khoản thật
    base_found = ann.exact_topk(queries, real).tolist()
    meta["real_top1"] = ann.top1_same_person(base_found, query_person, real_person)
    meta["real_found"] = ann.identity_found(base_found, query_person, real_person)
    print("\n== 1. Chỉ 200 tài khoản thật, tìm chính xác (NumPy)")
    print(f"  top1 cùng người: {meta['real_top1']:.3f}   found@10 (tìm được bao nhiêu trong 5 tài khoản cùng người): {meta['real_found']:.3f}")

    # --- 2. Nạp
    store.reset_schema(conn, embedding.DIM)
    try:
        secs = store.load_rows(conn, ids, persons, statuses, base)
        meta["rows"] = store.count_rows(conn)
        print(f"\n== 2. Nạp vào PostgreSQL: {meta['rows']} dòng bằng COPY nhị phân trong {secs:.1f} s; bảng {store.table_size(conn) / 1e6:.1f} MB")

        # --- 3. So sánh
        print(f"\n== 3. Top-{K}, từng truy vấn một (recall so với NumPy chính xác; found = tỉ lệ 5 tài khoản cùng người nằm trong top-{K})")
        header()
        with threadpool_limits(1):
            measure("NumPy brute force (1 luồng BLAS)", lambda q: ann.exact_topk(q[None], base)[0].tolist())

        with store.exact_scan(conn):
            meta["plan_exact"] = store.plan_head(conn, queries[0], K)
            meta["pg_dist0"] = store.topk(conn, queries[0], 1)[0][1]
            measure("pgvector quét tuần tự (chính xác)", lambda q: [row_of(r[0]) for r in store.topk(conn, q, K)])

        lists = max(10, total // 1000)
        secs, size = store.build_ivfflat(conn, lists)
        meta["ivf_build"] = (secs, size)
        meta["lists"] = lists
        meta["plan_ivf"] = store.plan_head(conn, queries[0], K)
        print(f"  [pgvector IVFFlat lists={lists}: dựng {secs:.1f} s, chỉ mục {size / 1e6:.1f} MB; kế hoạch: {meta['plan_ivf']}]")
        for probes in sorted({1, 5, min(10, lists), lists}):
            conn.execute(f"SET ivfflat.probes = {probes}")
            label = f"pgvector IVFFlat probes={probes}" + (" (hết)" if probes == lists else "")
            measure(label, lambda q: [row_of(r[0]) for r in store.topk(conn, q, K)])
        store.drop_index(conn, "driver_faces_ivf")

        secs, size = store.build_hnsw(conn)
        meta["hnsw_build"] = (secs, size)
        meta["plan_hnsw"] = store.plan_head(conn, queries[0], K)
        print(f"  [pgvector HNSW m=16 ef_construction=64: dựng {secs:.1f} s, chỉ mục {size / 1e6:.1f} MB; kế hoạch: {meta['plan_hnsw']}]")
        for ef in (10, 40, 100, 400):
            conn.execute(f"SET hnsw.ef_search = {ef}")
            measure(f"pgvector HNSW ef_search={ef}", lambda q: [row_of(r[0]) for r in store.topk(conn, q, K)])
        flat = ann.build_faiss_flat(base)
        faiss.omp_set_num_threads(1)  # FAISS một luồng, cùng điều kiện với PostgreSQL không song song và NumPy một luồng
        measure("FAISS IndexFlatIP (chính xác)", lambda q: flat.search(q[None], K)[1][0].tolist())
        hnsw, secs = ann.build_faiss_hnsw(base)
        meta["faiss_hnsw_build"] = (secs, ann.faiss_bytes(hnsw))
        print(f"  [FAISS HNSW M=16 efConstruction=64: dựng {secs:.1f} s, cỡ {meta['faiss_hnsw_build'][1] / 1e6:.1f} MB khi tuần tự hóa]")
        for ef in (10, 40, 100, 400):
            hnsw.hnsw.efSearch = ef
            measure(f"FAISS HNSW efSearch={ef}", lambda q: hnsw.search(q[None], K)[1][0].tolist())
        load_end = os.getloadavg()
        print(f"  load average lúc kết thúc phép đo {load_end[0]:.2f} {load_end[1]:.2f} {load_end[2]:.2f}")
        filtered_experiment(conn, queries, query_person, real_person, base, banned, meta)
        store.drop_index(conn, "driver_faces_hnsw")
    finally:
        if not keep:
            store.drop_schema(conn)
    meta["schema_gone"] = None if keep else store.schema_exists(conn) is False


    # --- 4. Ngưỡng
    scores = queries @ base.T
    same = real_person[None, :] == query_person[:, None]
    print("\n== 4. Ngưỡng cosine thay cho top-k mù (NumPy, chính xác)")
    print(f"  {'ngưỡng':>7}{'tìm được cùng người':>22}{'tài khoản nền vượt ngưỡng/truy vấn':>38}{'người khác (thật)/truy vấn':>28}")
    thresholds = {}
    for tau in (0.5, 0.6, 0.7, 0.8):
        hit = float(np.mean((scores[:, : len(real)] >= tau)[same].reshape(len(queries), -1).sum(1) / 5))
        bg = float(np.mean((scores[:, len(real):] >= tau).sum(1)))
        other = float(np.mean(((scores[:, : len(real)] >= tau) & ~same).sum(1)))
        thresholds[tau] = (hit, bg, other)
        print(f"  {tau:>7.1f}{hit:>22.3f}{bg:>38.1f}{other:>28.2f}")
    meta["thresholds"] = thresholds

    if not verify:
        return 0
    return verify_all(n_distractors, rows, meta, queries, base, scores)


def filtered_experiment(conn: psycopg.Connection, queries: np.ndarray, query_person: np.ndarray, real_person: np.ndarray,
                        base: np.ndarray, banned: np.ndarray, meta: dict[str, Any]) -> None:
    """Vector search kèm WHERE: so khớp ảnh đăng ký mới chỉ với tài khoản đã bị khóa (1% số dòng)."""
    truth = ann.exact_topk(queries, base, allowed=banned)
    relevant = query_person < 4  # 20 truy vấn của người có tài khoản bị khóa
    print(f"\n== 3b. Kèm WHERE status = 'banned' ({int(banned.sum())} dòng, {banned.mean() * 100:.1f}%), yêu cầu top-{K}; "
          "'bắt' = số trong 20 truy vấn của người bị khóa có tài khoản cùng người trong top-10")
    print(f"  {'cấu hình':<52}{'dòng TB':>8}{'recall@10':>11}{'bắt':>7}{'p50 ms':>9}")

    def run_one(name: str, key: str, sql: str = store.BANNED_SQL, truth_rows: np.ndarray = truth, show_caught: bool = True) -> None:
        found, p50, _ = ann.time_queries(lambda q: [row_of(r[0]) for r in conn.execute(sql, (q, q, K)).fetchall()], queries)
        caught = sum(any(i < len(real_person) and real_person[i] == p for i in f) for f, p, rel in zip(found, query_person, relevant, strict=True) if rel)
        meta[key] = (ann.mean_returned(found), ann.recall_at_k(found, truth_rows), caught, p50)
        caught_text = f"{caught:>4}/20" if show_caught else f"{'-':>7}"
        print(f"  {name:<52}{meta[key][0]:>8.2f}{meta[key][1]:>11.3f}{caught_text}{p50:>9.2f}")

    conn.execute("SET hnsw.ef_search = 40")
    with store.exact_scan(conn):
        meta["plan_banned_exact"] = store.plan_head(conn, queries[0], K, store.BANNED_SQL)
        run_one("quét tuần tự (chính xác)", "banned_exact")
    meta["plan_banned_hnsw"] = store.plan_head(conn, queries[0], K, store.BANNED_SQL)
    run_one("HNSW ef_search=40, mặc định (lọc sau khi quét chỉ mục)", "banned_hnsw")
    conn.execute("SET hnsw.iterative_scan = relaxed_order")
    run_one("  + hnsw.iterative_scan = relaxed_order", "banned_iter")
    conn.execute("SET hnsw.iterative_scan = strict_order")
    run_one("  + hnsw.iterative_scan = strict_order", "banned_strict")
    conn.execute("SET hnsw.iterative_scan = relaxed_order")
    conn.execute("SET hnsw.max_scan_tuples = 200")
    run_one("  + relaxed_order, hnsw.max_scan_tuples = 200 (mặc định 20000)", "banned_capped")
    conn.execute("RESET hnsw.max_scan_tuples")
    conn.execute("RESET hnsw.iterative_scan")

    store.build_status_btree(conn)
    with store.exact_scan(conn):
        meta["plan_banned_btree"] = store.plan_head(conn, queries[0], K, store.BANNED_SQL)
        run_one("B-tree trên status, rồi sắp xếp chính xác", "banned_btree")
    store.drop_index(conn, "driver_faces_status")

    store.drop_index(conn, "driver_faces_hnsw")  # để planner chỉ còn chỉ mục một phần
    secs, size = store.build_partial_hnsw(conn, "banned")
    meta["partial_build"] = (secs, size)
    meta["plan_banned_partial"] = store.plan_head(conn, queries[0], K, store.BANNED_SQL)
    run_one(f"HNSW một phần WHERE status='banned' (dựng {secs:.2f} s, {size / 1e6:.2f} MB)", "banned_partial")
    store.drop_index(conn, "driver_faces_hnsw_banned")

    store.drop_index(conn, "driver_faces_hnsw")
    store.build_hnsw(conn)
    conn.execute("SET hnsw.ef_search = 40")
    truth_active = ann.exact_topk(queries, base, allowed=~banned)
    run_one("(đối chứng) HNSW mặc định, WHERE status = 'active' (99%)", "active_hnsw", store.filter_sql("active"), truth_active, show_caught=False)
    meta["plan_banned_note"] = "bảng nhỏ hoặc điều kiện hẹp: planner có thể chọn Seq Scan thay vì chỉ mục"
    print(f"  (kế hoạch với HNSW toàn bảng: {meta['plan_banned_hnsw']}; B-tree: {meta['plan_banned_btree']}; một phần: {meta['plan_banned_partial']})")


def fail_message_ok() -> bool:
    """Chạy lại chính lab này với URL sai: phải in FAIL có hướng dẫn compose và thoát mã 1."""
    env = {**os.environ, "TOINOI_PG_URL": "postgresql://toinoi:toinoi@localhost:1/toinoi"}
    proc = subprocess.run([sys.executable, str(Path(__file__)), "--verify"], env=env, capture_output=True, text=True, timeout=60, check=False)
    return proc.returncode == 1 and "FAIL: không nối được PostgreSQL tại" in proc.stdout and "compose.yml up -d --wait" in proc.stdout


def verify_all(n_distractors: int, rows: dict[str, Row], meta: dict[str, Any], queries: np.ndarray, base: np.ndarray, scores: np.ndarray) -> int:
    checks: list[tuple[str, bool]] = []

    def check(name: str, ok: bool) -> None:
        checks.append((name, bool(ok)))

    exact_names = ["pgvector quét tuần tự (chính xác)", "FAISS IndexFlatIP (chính xác)"]
    check("kế hoạch quét tuần tự khi tắt chỉ mục", "Seq Scan" in str(meta["plan_exact"]))
    check("kế hoạch HNSW dùng chỉ mục driver_faces_hnsw", "driver_faces_hnsw" in str(meta["plan_hnsw"]))
    check("kế hoạch IVFFlat dùng chỉ mục driver_faces_ivf", "driver_faces_ivf" in str(meta["plan_ivf"]))
    check("số dòng trong bảng bằng 200 + distractors", meta["rows"] == 200 + n_distractors)
    check("vector đã chuẩn hóa L2", bool(np.allclose(np.linalg.norm(base, axis=1), 1.0, atol=1e-4)))
    for name in exact_names:
        check(f"{name}: recall 1.000 so với NumPy", rows[name].recall >= 0.999)
    full_ivf = f"pgvector IVFFlat probes={meta['lists']} (hết)"
    check("IVFFlat probes=lists (quét hết): recall 1.000", rows[full_ivf].recall >= 0.999)
    check("IVFFlat probes=1 có recall thấp hơn probes=lists", rows["pgvector IVFFlat probes=1"].recall < rows[full_ivf].recall)
    check("pgvector HNSW: ef_search 400 có recall >= ef_search 10", rows["pgvector HNSW ef_search=400"].recall >= rows["pgvector HNSW ef_search=10"].recall)
    check("FAISS HNSW: efSearch 400 có recall >= efSearch 10", rows["FAISS HNSW efSearch=400"].recall >= rows["FAISS HNSW efSearch=10"].recall)
    check("pgvector HNSW ef_search=100: recall >= 0.90", rows["pgvector HNSW ef_search=100"].recall >= 0.90)
    check("FAISS HNSW efSearch=100: recall >= 0.90", rows["FAISS HNSW efSearch=100"].recall >= 0.90)
    check("chỉ 200 tài khoản thật: top1 cùng người >= 0.80", float(meta["real_top1"]) >= 0.80)
    check("thêm tài khoản nền làm giảm tỉ lệ tìm được cùng người (kể cả khi tìm chính xác)",
          rows["NumPy brute force (1 luồng BLAS)"].found < float(meta["real_found"]))
    check("WHERE: quét tuần tự có recall 1.000", meta["banned_exact"][1] >= 0.999 and meta["banned_exact"][0] == K)
    if "driver_faces_hnsw" in meta["plan_banned_hnsw"]:  # bảng nhỏ thì planner tự chọn quét tuần tự, không còn bẫy để đo
        check("WHERE: HNSW mặc định trả về trung bình ít hơn một nửa số dòng yêu cầu", meta["banned_hnsw"][0] < K / 2)
        check("WHERE: HNSW + iterative_scan trả đủ dòng và recall >= 0.80", meta["banned_iter"][0] == K and meta["banned_iter"][1] >= 0.80)
        check("WHERE: strict_order trả đủ dòng", meta["banned_strict"][0] == K)
        check("WHERE: iterative_scan bị chặn bởi max_scan_tuples = 200 thì trả ít dòng hơn mặc định",
              meta["banned_capped"][0] < meta["banned_iter"][0])
        check("WHERE: chỉ mục một phần dùng đúng chỉ mục, đủ dòng, recall 1.000",
              "driver_faces_hnsw_banned" in meta["plan_banned_partial"] and meta["banned_partial"][0] == K and meta["banned_partial"][1] >= 0.999)
        check("WHERE: B-tree + sắp xếp chính xác có recall 1.000", meta["banned_btree"][1] >= 0.999)
        check("WHERE status='active' (99%): HNSW mặc định trả đủ dòng", meta["active_hnsw"][0] >= K - 0.05)
    check("khoảng cách pgvector bằng 1 trừ tích vô hướng", abs(float(meta["pg_dist0"]) - (1 - float(scores[0].max()))) < 1e-4)
    bg_counts = [meta["thresholds"][t][1] for t in sorted(meta["thresholds"])]
    check("ngưỡng cao hơn thì ít tài khoản nền vượt hơn", all(a >= b for a, b in itertools.pairwise(bg_counts)) and bg_counts[0] > bg_counts[-1])
    if meta["schema_gone"] is not None:
        check("schema lab51 đã bị xóa sau khi chạy", meta["schema_gone"] is True)
    check("không nối được PostgreSQL: in FAIL và thoát mã 1", fail_message_ok())

    failed = [name for name, ok in checks if not ok]
    print()
    for name in failed:
        print(f"SAI: {name}")
    n_ok = len(checks) - len(failed)
    if failed:
        print(f"FAIL {n_ok}/{len(checks)}")
        return 1
    print(f"OK {n_ok}/{len(checks)}")
    return 0


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--verify", action="store_true", help="kiểm các bất biến, in OK n/n")
    parser.add_argument("--distractors", type=int, default=100_000, help="số tài khoản nền tổng hợp")
    parser.add_argument("--keep", action="store_true", help="giữ schema lab51 để xem bằng tay")
    args = parser.parse_args()
    sys.exit(run(args.distractors, args.keep, args.verify))


if __name__ == "__main__":
    main()
