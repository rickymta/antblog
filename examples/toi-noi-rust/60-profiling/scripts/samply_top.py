#!/usr/bin/env python3
"""Gộp mẫu từ profile của samply, không cần mở Firefox Profiler. Chỉ dùng thư viện chuẩn.

Tạo profile (file .syms.json nằm cạnh file .json.gz):
    samply record --save-only --unstable-presymbolicate -o target/samply-slow.json.gz \
        target/release/ledger-profiling --workload slow --n 1000

Đọc:
    python3 scripts/samply_top.py target/samply-slow.json.gz --focus "SlowBook as"

In phần trăm theo số mẫu và theo CPU (`threadCPUDelta`). Trên máy tải cao, số mẫu gồm cả lúc luồng
bị hoãn tại chỗ nên hai cột có thể lệch. Đây là bảng gộp, không phải flamegraph. Thư viện hệ thống
có thể thiếu ký hiệu: khi đó tên là `<thư viện>?`, và tên hàm hệ thống chỉ nên đọc như gợi ý.
"""

import argparse
import bisect
import collections
import gzip
import json
import pathlib
import sys


def load_symbols(path):
    doc = json.loads(path.read_text(encoding="utf-8"))
    tables = {}
    for lib in doc["data"]:
        rows = sorted(lib["symbol_table"], key=lambda row: row["rva"])
        tables[lib["debug_name"]] = ([row["rva"] for row in rows], rows)
    return tables, doc["string_table"]


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("profile", type=pathlib.Path, help="file .json.gz của samply")
    parser.add_argument("--focus", help="chuỗi con của tên hàm cần xem các hàm con trực tiếp")
    parser.add_argument("--top", type=int, default=10, help="số dòng mỗi bảng (mặc định 10)")
    args = parser.parse_args()

    syms_path = args.profile.with_name(args.profile.name.removesuffix(".gz") + ".syms.json")
    if not syms_path.exists():
        sys.exit(f"LỖI: thiếu {syms_path}; chạy samply với --unstable-presymbolicate")
    with gzip.open(args.profile, "rt", encoding="utf-8") as file:
        profile = json.load(file)
    tables, strings = load_symbols(syms_path)

    thread = max(profile["threads"], key=lambda t: t["samples"]["length"])
    frames, funcs = thread["frameTable"], thread["funcTable"]
    resources, stacks = thread["resourceTable"], thread["stackTable"]

    def frame_name(index):
        resource = funcs["resource"][frames["func"][index]]
        lib = resources["lib"][resource] if resource is not None and resource >= 0 else None
        if lib is None:
            return "?"
        lib_name = profile["libs"][lib]["debugName"]
        table = tables.get(lib_name)
        if table is None:
            return f"{lib_name}?"
        address = frames["address"][index]
        at = bisect.bisect_right(table[0], address) - 1
        if at >= 0 and address < table[1][at]["rva"] + table[1][at]["size"]:
            return strings[table[1][at]["symbol"]]
        return f"{lib_name}+?"

    def frame_lib(index):
        resource = funcs["resource"][frames["func"][index]]
        lib = resources["lib"][resource] if resource is not None and resource >= 0 else None
        return profile["libs"][lib]["debugName"] if lib is not None else "?"

    # (số mẫu, CPU µs) cho từng bảng.
    self_fn, self_lib, inclusive, children = (collections.defaultdict(lambda: [0, 0]) for _ in range(4))
    samples = thread["samples"]
    cpu_deltas = samples.get("threadCPUDelta") or [0] * samples["length"]
    total = [0, 0]

    def add(table, key, cpu):
        table[key][0] += 1
        table[key][1] += cpu

    for stack, cpu in zip(samples["stack"], cpu_deltas):
        if stack is None:
            continue
        cpu = cpu or 0
        leaf_first = []
        while stack is not None:
            leaf_first.append(stacks["frame"][stack])
            stack = stacks["prefix"][stack]
        names = [frame_name(frame) for frame in leaf_first]
        total[0] += 1
        total[1] += cpu
        add(self_fn, names[0], cpu)
        add(self_lib, frame_lib(leaf_first[0]), cpu)
        for name in set(names):
            add(inclusive, name, cpu)
        if args.focus:
            root_first = names[::-1]
            for at, name in enumerate(root_first):
                if args.focus in name:
                    add(children, root_first[at + 1] if at + 1 < len(root_first) else "(chính nó)", cpu)
                    break

    def show(title, table, limit):
        print(f"--- {title}")
        print(f"{'% mẫu':>7} {'% CPU':>7} {'mẫu':>6}  tên")
        ordered = sorted(table.items(), key=lambda item: (-item[1][0], item[0]))
        for name, (count, cpu) in ordered[:limit]:
            print(f"{count * 100 / total[0]:7.1f} {cpu * 100 / max(total[1], 1):7.1f} {count:6d}  {name[:100]}")

    if total[0] == 0:
        sys.exit("LỖI: profile không có mẫu nào")
    print(f"luồng {thread['name']}: {total[0]} mẫu, {total[1] / 1000:.0f} ms CPU")
    show("thời gian riêng theo thư viện", self_lib, args.top)
    show("thời gian riêng theo hàm", self_fn, args.top)
    show("thời gian gộp theo hàm", inclusive, args.top)
    if args.focus:
        show(f"hàm con trực tiếp của `{args.focus}`", children, args.top)


if __name__ == "__main__":
    main()
