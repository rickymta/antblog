"""LOC-50: OR-Tools làm mốc chất lượng OFFLINE cho một người giao hàng, n cặp lấy -> giao, tuyến mở.

Đọc fixtures/loc50/instances.json (do lab Rust `loc50_nhieu_diem_dung --write-instances` ghi), giải từng instance bằng
OR-Tools Routing (PARALLEL_CHEAPEST_INSERTION + GUIDED_LOCAL_SEARCH, có giới hạn thời gian), ghi
fixtures/loc50/ortools-results.json. Không nằm trên đường nóng, không cần mạng.

Mô hình giống hệt phía Rust: tuyến mở bằng một đỉnh kết thúc giả có chi phí 0 từ mọi đỉnh; lấy trước giao bằng
AddPickupAndDelivery + cùng xe + thời điểm tích lũy của lấy <= giao.

Chạy: cd examples/toi-noi-location/python/loc50 && uv run python loc50_ortools.py [--time-scale 1.0]
"""

import argparse
import json
import platform
import time
from pathlib import Path

import ortools
from ortools.constraint_solver import pywrapcp, routing_enums_pb2

ROOT = Path(__file__).resolve().parents[2]
# giới hạn thời gian (giây) theo số cặp n; instance lớn được nhiều thời gian hơn
TIME_LIMIT_S = {4: 2, 8: 2, 12: 3, 20: 5, 30: 8}


def fnv1a(matrix) -> str:
    """Cùng công thức với lab Rust: FNV-1a 64 bit trên chuỗi `a,b,c,...` của ma trận hàng-chính."""
    text = ",".join(str(x) for row in matrix for x in row).encode()
    h = 0xCBF29CE484222325
    for b in text:
        h ^= b
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{h:016x}"


def solve(matrix, n, limit_s):
    m = len(matrix)  # 2n + 1; đỉnh m là đỉnh kết thúc giả
    mgr = pywrapcp.RoutingIndexManager(m + 1, 1, [0], [m])
    routing = pywrapcp.RoutingModel(mgr)

    def dist(i, j):
        a, b = mgr.IndexToNode(i), mgr.IndexToNode(j)
        if a == m or b == m:
            return 0
        return matrix[a][b]

    cb = routing.RegisterTransitCallback(dist)
    routing.SetArcCostEvaluatorOfAllVehicles(cb)
    horizon = sum(max(r) for r in matrix) + 1
    routing.AddDimension(cb, 0, horizon, True, "Time")
    time_dim = routing.GetDimensionOrDie("Time")
    solver = routing.solver()
    for k in range(n):
        p, d = mgr.NodeToIndex(1 + 2 * k), mgr.NodeToIndex(2 + 2 * k)
        routing.AddPickupAndDelivery(p, d)
        solver.Add(routing.VehicleVar(p) == routing.VehicleVar(d))
        solver.Add(time_dim.CumulVar(p) <= time_dim.CumulVar(d))
    params = pywrapcp.DefaultRoutingSearchParameters()
    params.first_solution_strategy = routing_enums_pb2.FirstSolutionStrategy.PARALLEL_CHEAPEST_INSERTION
    params.local_search_metaheuristic = routing_enums_pb2.LocalSearchMetaheuristic.GUIDED_LOCAL_SEARCH
    params.time_limit.FromMilliseconds(int(limit_s * 1000))
    t = time.perf_counter()
    sol = routing.SolveWithParameters(params)
    wall = time.perf_counter() - t
    status = {0: "NOT_SOLVED", 1: "SUCCESS", 2: "FAIL", 3: "FAIL_TIMEOUT", 4: "INVALID", 5: "INFEASIBLE"}.get(
        routing.status(), str(routing.status())
    )
    if not sol:
        return None, None, status, wall
    route, idx = [], routing.Start(0)
    while not routing.IsEnd(idx):
        route.append(mgr.IndexToNode(idx))
        idx = sol.Value(routing.NextVar(idx))
    cost = sum(matrix[a][b] for a, b in zip(route, route[1:]))
    return route, (cost, sol.ObjectiveValue()), status, wall


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--instances", default=str(ROOT / "fixtures/loc50/instances.json"))
    ap.add_argument("--out", default=str(ROOT / "fixtures/loc50/ortools-results.json"))
    ap.add_argument("--time-scale", type=float, default=1.0, help="nhân giới hạn thời gian")
    args = ap.parse_args()
    data = json.loads(Path(args.instances).read_text())
    results = []
    for inst in data["instances"]:
        n, matrix = inst["n"], inst["matrix"]
        assert inst["matrix_fnv"] == fnv1a(matrix), "băm ma trận lệch: file instance hỏng"
        limit = TIME_LIMIT_S.get(n, 5) * args.time_scale
        route, costs, status, wall = solve(matrix, n, limit)
        r = {"id": inst["id"], "n": n, "matrix_fnv": inst["matrix_fnv"], "status": status, "wall_s": wall, "time_limit_s": limit}
        if route is not None:
            r.update(route=route, cost_ms=costs[0], objective_ms=costs[1])
        else:
            r.update(route=[], cost_ms=-1, objective_ms=-1)
        results.append(r)
        print(f"{inst['id']}: {status} cost={r['cost_ms']} wall={wall:.2f}s", flush=True)
    out = {
        "format": "loc50-ortools-v1",
        "ortools_version": ortools.__version__,
        "python": platform.python_version(),
        "first_solution_strategy": "PARALLEL_CHEAPEST_INSERTION",
        "metaheuristic": "GUIDED_LOCAL_SEARCH",
        "time_limit_s_by_n": {str(k): v * args.time_scale for k, v in TIME_LIMIT_S.items()},
        "results": results,
    }
    Path(args.out).write_text(json.dumps(out, separators=(",", ":")) + "\n")
    print(f"đã ghi {args.out}")


if __name__ == "__main__":
    main()
