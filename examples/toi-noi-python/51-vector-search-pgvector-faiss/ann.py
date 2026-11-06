"""Đáp án chính xác bằng NumPy, chỉ mục FAISS, và các phép đo recall / độ trễ."""

import statistics
import time
from collections.abc import Callable

import faiss
import numpy as np

TOP_K = 10


def exact_topk(queries: np.ndarray, base: np.ndarray, k: int = TOP_K, allowed: np.ndarray | None = None) -> np.ndarray:
    """Top-k theo cosine (vector đã chuẩn hóa L2 nên tích vô hướng = cosine). Trả về chỉ số hàng, giảm dần theo điểm.

    `allowed` là mặt nạ bool theo hàng: hàng không được phép bị loại khỏi kết quả (tương đương `WHERE`).
    """
    scores = queries @ base.T
    if allowed is not None:
        scores = np.where(allowed[None, :], scores, -np.inf)
    part = np.argpartition(-scores, k, axis=1)[:, :k]
    order = np.argsort(-np.take_along_axis(scores, part, axis=1), axis=1)
    return np.take_along_axis(part, order, axis=1)


def build_faiss_flat(base: np.ndarray) -> faiss.Index:
    """IndexFlatIP: quét toàn bộ, chính xác, không có tham số."""
    index = faiss.IndexFlatIP(base.shape[1])
    index.add(base)
    return index


def build_faiss_hnsw(base: np.ndarray, m: int = 16, ef_construction: int = 64) -> tuple[faiss.Index, float]:
    """IndexHNSWFlat với độ đo tích vô hướng. Trả về (chỉ mục, giây dựng)."""
    index = faiss.IndexHNSWFlat(base.shape[1], m, faiss.METRIC_INNER_PRODUCT)
    index.hnsw.efConstruction = ef_construction
    t0 = time.perf_counter()
    index.add(base)
    return index, time.perf_counter() - t0


def faiss_bytes(index: faiss.Index) -> int:
    """Kích thước chỉ mục khi tuần tự hóa (xấp xỉ bộ nhớ nó chiếm)."""
    return int(faiss.serialize_index(index).nbytes)


def recall_at_k(found: list[list[int]], truth: np.ndarray) -> float:
    """Recall so với đáp án chính xác: |tìm được giao đáp án| / k, trung bình theo truy vấn."""
    return float(np.mean([len(set(f) & set(t.tolist())) / len(t) for f, t in zip(found, truth, strict=True)]))


def identity_found(found: list[list[int]], query_person: np.ndarray, real_person: np.ndarray, per_person: int = 5) -> float:
    """Trung bình tỉ lệ tài khoản thật của cùng người nằm trong kết quả (mỗi người có `per_person` tài khoản).

    Chỉ số hàng < len(real_person) là tài khoản thật; phần còn lại là tài khoản nền tổng hợp.
    """
    scores = []
    for f, p in zip(found, query_person, strict=True):
        hits = sum(1 for i in f if i < len(real_person) and real_person[i] == p)
        scores.append(hits / per_person)
    return float(np.mean(scores))


def mean_returned(found: list[list[int]]) -> float:
    """Số dòng trung bình mỗi truy vấn thật sự trả về (yêu cầu k dòng)."""
    return float(np.mean([len(f) for f in found]))


def top1_same_person(found: list[list[int]], query_person: np.ndarray, real_person: np.ndarray) -> float:
    return float(np.mean([bool(f) and f[0] < len(real_person) and real_person[f[0]] == p for f, p in zip(found, query_person, strict=True)]))


def time_queries(search: Callable[[np.ndarray], list[int]], queries: np.ndarray, warmup: int = 10) -> tuple[list[list[int]], float, float]:
    """Gọi từng truy vấn một (như một yêu cầu đăng ký). Trả về (kết quả, p50 ms, p95 ms) sau khi khởi động."""
    for q in queries[:warmup]:
        search(q)
    found, times = [], []
    for q in queries:
        t0 = time.perf_counter()
        found.append(search(q))
        times.append((time.perf_counter() - t0) * 1000)
    times.sort()
    return found, statistics.median(times), times[int(len(times) * 0.95) - 1]
