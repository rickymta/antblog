# Đọc lại Parquet lịch sử bằng Polars và PyArrow để kiểm tra schema v1 (xem proto/location/v1/gps-history-parquet.md).
# Chạy từ gốc repo: uv run --no-project --with polars --with pyarrow python -I examples/toi-noi-location/go/scripts/readback_history.py [thư mục gốc]
import polars as pl, pyarrow.dataset as ds
import sys
root = sys.argv[1] if len(sys.argv) > 1 else "examples/toi-noi-location/go/out/history-live"
df = pl.scan_parquet(root + "/**/*.parquet", hive_partitioning=True).collect()
print("polars", pl.__version__, "rows", df.height)
print(df.schema)
print(df.group_by("city").agg(pl.len().alias("rows")).sort("city"))
print("speed_dmps null:", df["speed_dmps"].null_count(), "| speed_dmps == 0:", (df["speed_dmps"] == 0).sum())
print("sample_reason:", df.group_by("sample_reason").agg(pl.len()).sort("sample_reason").rows())
t = ds.dataset(root, partitioning="hive").to_table()
print("pyarrow rows", t.num_rows, "| event_time type:", t.schema.field("event_time").type)
