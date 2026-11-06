"""Đồ thị khách - tài xế - thiết bị dựng THEO CỬA SỔ THỜI GIAN, và đặc trưng đồ thị cho từng chuyến.

Quy tắc điểm thời gian (như bài 55): một chuyến được chấm lúc `PaymentCaptured` (paid_at). Đồ thị phục vụ
chuyến đó là ảnh chụp (snapshot) lúc 00:00 giờ Việt Nam của NGÀY chuyến được thu tiền, dựng từ các chuyến đã
thu tiền trong cửa sổ [đầu ngày - 28 ngày, đầu ngày). Chuyến đang chấm và mọi chuyến cùng ngày chưa nằm trong
đồ thị. Không có nhãn nào được dùng để dựng đồ thị hay chọn cạnh.

Ba loại nút: khách (CU-...), tài xế (DR-...), thiết bị (DEV-...). Hai loại cạnh:
  - khách - tài xế, có trọng số = số chuyến trong cửa sổ, CHỈ GIỮ cặp từ STRONG_MIN_TRIPS chuyến trở lên;
  - tài khoản - thiết bị, CHỈ GIỮ thiết bị có từ 2 tài khoản khác nhau (khách đặt bằng máy đó, hoặc tài xế dùng máy đó).
Thành phần liên thông lớn hơn MAX_GROUP_NODES được tách tiếp bằng Louvain (community detection) thành "nhóm".
"""

from collections.abc import Iterable
from dataclasses import dataclass
from typing import NamedTuple
from datetime import timedelta

import networkx as nx
import polars as pl

from synthetic_fraud import START

WINDOW_DAYS = 28
STRONG_MIN_TRIPS = 2
MAX_GROUP_NODES = 40
LOUVAIN_SEED = 0

# Tiền tố ID -> loại nút (ID do bộ sinh gán: CU-00001, DR-00001, DEV-00001)
KIND = {"CU": "customer", "DR": "driver", "DEV": "device"}

# Đặc trưng đồ thị gắn vào từng chuyến (xem add_graph_features)
GRAPH_FEATURES = [
    "g_edge_trips", "g_same_group", "g_suspect_group",
    "gc_group_size", "gc_cycle_rank", "gc_density", "gc_core", "gc_betweenness",
    "gd_group_size", "gd_cycle_rank", "gd_density", "gd_core", "gd_betweenness", "gd_degree",
]


def kind_of(node: str) -> str:
    return KIND[node.split("-", 1)[0]]


def window_slice(trips: pl.DataFrame, day: int, window_days: int = WINDOW_DAYS) -> pl.DataFrame:
    """Các chuyến đã thu tiền trong [đầu ngày - window_days, đầu ngày); `day` đếm từ START (00:00 giờ Việt Nam)."""
    end = START + timedelta(days=day)
    return trips.filter((pl.col("paid_at") >= end - timedelta(days=window_days)) & (pl.col("paid_at") < end))


def edge_tables(w: pl.DataFrame, strong_min: int = STRONG_MIN_TRIPS) -> tuple[pl.DataFrame, pl.DataFrame]:
    """(cặp khách-tài xế mạnh, cạnh tài khoản-thiết bị dùng chung). Lọc bằng polars trước khi đưa vào networkx."""
    # sort: group_by không giữ thứ tự, mà thứ tự thêm cạnh quyết định thứ tự nút trong networkx và kết quả Louvain
    pairs = w.group_by("customer_id", "driver_id").agg(trips=pl.len()).filter(pl.col("trips") >= strong_min).sort("customer_id", "driver_id")
    accounts = pl.concat([
        w.select(account="customer_id", device="device_id"),
        w.select(account="driver_id", device="driver_device_id"),
    ]).unique()
    shared = accounts.group_by("device").agg(n=pl.col("account").n_unique()).filter(pl.col("n") >= 2).select("device")
    return pairs, accounts.join(shared, on="device").sort("account", "device")


def build_graph(pairs: pl.DataFrame, shared: pl.DataFrame) -> nx.Graph:
    g = nx.Graph()
    g.add_edges_from((c, d, {"weight": n, "kind": "trip"}) for c, d, n in pairs.select("customer_id", "driver_id", "trips").iter_rows())
    g.add_edges_from((a, dev, {"weight": 1, "kind": "device"}) for a, dev in shared.select("account", "device").iter_rows())
    nx.set_node_attributes(g, {n: kind_of(n) for n in g}, "kind")
    return g


def ordered_subgraph(g: nx.Graph, nodes: Iterable[str]) -> nx.Graph:
    """Đồ thị con có thứ tự nút và cạnh đã sắp. `g.subgraph(set)` duyệt theo thứ tự của tập hợp, mà thứ tự đó đổi theo
    PYTHONHASHSEED của mỗi lần chạy; Louvain đọc cạnh theo thứ tự duyệt nên kết quả sẽ đổi theo."""
    ordered = sorted(nodes)
    sub = nx.Graph()
    sub.add_nodes_from(ordered)
    sub.add_edges_from((u, v, g[u][v]) for u in ordered for v in sorted(g[u]) if u < v and v in sub)
    return sub


def split_groups(g: nx.Graph, max_nodes: int = MAX_GROUP_NODES) -> list[frozenset[str]]:
    """Thành phần liên thông nhỏ giữ nguyên; thành phần lớn (một khu vực có tài xế đông khách quen) tách bằng Louvain."""
    groups: list[frozenset[str]] = []
    for comp in nx.connected_components(g):
        if len(comp) <= max_nodes:
            groups.append(frozenset(comp))
        else:
            parts = nx.community.louvain_communities(ordered_subgraph(g, comp), weight="weight", seed=LOUVAIN_SEED)
            groups.extend(frozenset(p) for p in parts)
    return sorted(groups, key=min)  # số hiệu nhóm ổn định giữa các lần chạy


def group_stats(g: nx.Graph, members: frozenset[str]) -> tuple[dict, dict[str, float]]:
    """Thống kê của một nhóm và độ trung gian (betweenness, chuẩn hóa) của từng nút trong nhóm."""
    sg = g.subgraph(members)
    n, e = sg.number_of_nodes(), sg.number_of_edges()
    betw = nx.betweenness_centrality(sg, normalized=True) if n >= 3 else dict.fromkeys(members, 0.0)
    kinds = [g.nodes[x]["kind"] for x in members]
    stats = {
        "size": n, "customers": kinds.count("customer"), "drivers": kinds.count("driver"), "devices": kinds.count("device"),
        "edges": e, "density": nx.density(sg) if n > 1 else 0.0,
        # số chu trình độc lập = E - N + số thành phần: cây (hub và nan hoa) bằng 0, lưới nhiều đường nối thì lớn
        "cycle_rank": e - n + nx.number_connected_components(sg),
        "trips": sum(d["weight"] for _, _, d in sg.edges(data=True) if d["kind"] == "trip"),
        "top_betweenness": max(betw.values(), default=0.0),
    }
    return stats, betw


class Rule(NamedTuple):
    """Luật gắn cờ nhóm. Ngưỡng chọn bằng quét lưới trên các ảnh chụp ngày 7 đến 29 (main.py, bước 4), với sự thật nền chỉ từ chuyến train."""
    min_drivers: int = 2
    min_customers: int = 3
    min_cycle_rank: int = 2
    min_density: float = 0.15
    max_top_betweenness: float = 0.6


RULE = Rule()


def is_suspect(s: dict, rule: Rule = RULE) -> bool:
    """Nhóm có từ 2 tài xế, 3 khách, nhiều đường nối chéo (chu trình), đủ dày và không phải một hub duy nhất."""
    return (s["drivers"] >= rule.min_drivers and s["customers"] >= rule.min_customers and s["cycle_rank"] >= rule.min_cycle_rank
            and s["density"] >= rule.min_density and s["top_betweenness"] <= rule.max_top_betweenness)


def suspect_expr(rule: Rule = RULE) -> pl.Expr:
    """Cùng luật, viết bằng biểu thức polars để chấm cả bảng nhóm cùng lúc (đối chiếu với is_suspect trong verify)."""
    return ((pl.col("drivers") >= rule.min_drivers) & (pl.col("customers") >= rule.min_customers)
            & (pl.col("cycle_rank") >= rule.min_cycle_rank) & (pl.col("density") >= rule.min_density)
            & (pl.col("top_betweenness") <= rule.max_top_betweenness))


@dataclass
class Snapshot:
    day: int
    graph: nx.Graph
    window_trips: int
    pairs_all: int  # số cặp khách-tài xế khác nhau trong cửa sổ (trước khi lọc cạnh mạnh)
    groups: pl.DataFrame  # một dòng mỗi nhóm
    nodes: pl.DataFrame  # một dòng mỗi nút thuộc nhóm
    pairs: pl.DataFrame  # các cặp khách-tài xế mạnh của ảnh chụp (day, customer_id, driver_id, trips)


def snapshot(trips: pl.DataFrame, day: int, window_days: int = WINDOW_DAYS, strong_min: int = STRONG_MIN_TRIPS,
             lookahead_days: int = 0) -> Snapshot:
    """Ảnh chụp đồ thị tại đầu ngày `day`. `trips` có thể chứa cả chuyến sau đó: cửa sổ tự cắt, không cần cắt trước.

    `lookahead_days` > 0 đẩy cuối cửa sổ về TƯƠNG LAI; chỉ để tái hiện lỗi, không bao giờ dùng khi chấm điểm.
    """
    w = window_slice(trips, day + lookahead_days, window_days)
    pairs, shared = edge_tables(w, strong_min)
    g = build_graph(pairs, shared)
    core = nx.core_number(g) if g.number_of_edges() else {}
    group_rows, node_rows = [], []
    for gid, members in enumerate(split_groups(g)):
        s, betw = group_stats(g, members)
        s |= {"day": day, "group_id": gid, "suspect": is_suspect(s), "members": sorted(members)}
        group_rows.append(s)
        for x in members:
            node_rows.append({"day": day, "node": x, "group_id": gid, "degree": g.degree(x), "core": core[x],
                              "betweenness": betw[x], **{f"g_{k}": s[k] for k in ("size", "cycle_rank", "density", "suspect")}})
    n_pairs = w.select("customer_id", "driver_id").n_unique() if w.height else 0
    return Snapshot(day, g, w.height, n_pairs, _frame(group_rows, GROUP_SCHEMA), _frame(node_rows, NODE_SCHEMA),
                    pairs.with_columns(day=pl.lit(day, pl.Int32)).select("day", "customer_id", "driver_id", "trips"))


GROUP_SCHEMA = {
    "day": pl.Int32, "group_id": pl.Int32, "size": pl.Int32, "customers": pl.Int32, "drivers": pl.Int32, "devices": pl.Int32,
    "edges": pl.Int32, "density": pl.Float64, "cycle_rank": pl.Int32, "trips": pl.Int64, "top_betweenness": pl.Float64,
    "suspect": pl.Boolean, "members": pl.List(pl.String),
}
NODE_SCHEMA = {
    "day": pl.Int32, "node": pl.String, "group_id": pl.Int32, "degree": pl.Int32, "core": pl.Int32, "betweenness": pl.Float64,
    "g_size": pl.Int32, "g_cycle_rank": pl.Int32, "g_density": pl.Float64, "g_suspect": pl.Boolean,
}


def _frame(rows: list[dict], schema: dict) -> pl.DataFrame:
    return pl.DataFrame(rows, schema=schema) if rows else pl.DataFrame(schema=schema)


def snapshots(trips: pl.DataFrame, days: Iterable[int], window_days: int = WINDOW_DAYS) -> tuple[pl.DataFrame, pl.DataFrame, pl.DataFrame]:
    """(bảng nút, bảng nhóm, bảng cặp mạnh) gộp qua nhiều ngày."""
    parts = [snapshot(trips, day, window_days) for day in days]
    return (pl.concat([p.nodes for p in parts]), pl.concat([p.groups for p in parts]),
            pl.concat([p.pairs for p in parts]))


def add_graph_features(trips: pl.DataFrame, nodes: pl.DataFrame, pairs_by_day: pl.DataFrame | None = None, fixed_day: int | None = None) -> pl.DataFrame:
    """Gắn GRAPH_FEATURES vào bảng chuyến. Chuyến thanh toán ngày d dùng ảnh chụp đầu ngày d (hàng `day == d` của `nodes`).

    `pairs_by_day` (day, customer_id, driver_id, trips) là các cặp mạnh của từng ảnh chụp, dùng cho g_edge_trips.
    Nút không thuộc nhóm nào (không có cạnh mạnh, không dùng chung thiết bị) nhận 0 ở mọi đặc trưng.
    `fixed_day` chỉ để TÁI HIỆN LỖI: mọi chuyến dùng cùng một ảnh chụp (ví dụ dựng một lần trên cả 60 ngày).
    """
    day = pl.lit(fixed_day, pl.Int32) if fixed_day is not None else ((pl.col("paid_at") - pl.lit(START)).dt.total_seconds() // 86400).cast(pl.Int32)
    t = trips.with_columns(day=day)

    def side(prefix: str, key: str, with_degree: bool) -> pl.DataFrame:
        cols = {"g_size": "group_size", "g_cycle_rank": "cycle_rank", "g_density": "density", "core": "core", "betweenness": "betweenness"}
        if with_degree:
            cols["degree"] = "degree"
        out = nodes.select("day", pl.col("node").alias(key), "group_id", *cols).rename({k: f"{prefix}_{v}" for k, v in cols.items()})
        return out.rename({"group_id": f"{prefix}_gid"})

    t = t.join(side("gc", "customer_id", False), on=["day", "customer_id"], how="left")
    t = t.join(side("gd", "driver_id", True), on=["day", "driver_id"], how="left")
    sus = nodes.group_by("day", "group_id").agg(gc_suspect=pl.col("g_suspect").first()).rename({"group_id": "gc_gid"})
    t = t.join(sus, on=["day", "gc_gid"], how="left")
    t = t.with_columns(
        g_same_group=(pl.col("gc_gid") == pl.col("gd_gid")).fill_null(False).cast(pl.Float64),
        g_suspect_group=((pl.col("gc_gid") == pl.col("gd_gid")) & pl.col("gc_suspect")).fill_null(False).cast(pl.Float64),
    )
    if pairs_by_day is not None:
        t = t.join(pairs_by_day.rename({"trips": "g_edge_trips"}), on=["day", "customer_id", "driver_id"], how="left")
    else:
        t = t.with_columns(g_edge_trips=pl.lit(0))
    t = t.with_columns(pl.col("g_edge_trips").fill_null(0).cast(pl.Float64))
    fill = [c for c in GRAPH_FEATURES if c not in ("g_edge_trips", "g_same_group", "g_suspect_group")]
    return t.with_columns(pl.col(fill).fill_null(0).cast(pl.Float64)).drop("gc_gid", "gd_gid", "gc_suspect")
