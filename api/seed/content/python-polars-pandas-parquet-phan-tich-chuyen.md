---
slug: python-polars-pandas-parquet-phan-tich-chuyen
title: "polars, pandas và Parquet: phân tích 2 triệu chuyến mỗi ngày"
summary: "Bài 29: sinh 2 triệu chuyến giả, so CSV với Parquet, truy vấn lazy bằng polars, đặt pandas cạnh polars để so thời gian và bộ nhớ, và bẫy tràn Int32. Có số đo thật và lab chạy được."
category: python-intermediate
level: intermediate
tags: [python, toi-noi, polars, pandas, parquet]
language: vi
---

Mỗi sáng bộ phận phân tích của [Tới Nơi](/articles/du-an-mau-toi-noi) hỏi cùng một câu: hôm qua mỗi thành phố cao điểm lúc mấy giờ, và doanh thu các chuyến `ride` đã hoàn thành là bao nhiêu? Dữ liệu là file chuyến của một ngày, 2 triệu dòng. Vòng `for` qua module `csv` như ở bài 09 vẫn cho đáp án đúng, nhưng với 730 triệu dòng mỗi năm thì cách đọc từng dòng vào list các dict không còn là một lựa chọn: ta cần một công cụ coi cả cột là một đơn vị dữ liệu.

Đây là bài 29 của lộ trình **Python trung bình qua Tới Nơi**. Bạn cần quen đọc file CSV bằng `pathlib` ([bài 09](/articles/python-file-json-csv-pathlib)); lab không cần Docker hay dịch vụ nào, chỉ cần mạng ở lần cài thư viện đầu tiên.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Truy vấn chỉ cần 3 trong 9 cột của bảng chuyến. Nếu dữ liệu nằm trong file CSV, ta có tránh được việc đọc 6 cột còn lại không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> CSV xếp theo dòng, các cột của một dòng nằm liền nhau trên đĩa và chỉ phân tách bằng dấu phẩy, nên muốn lấy cột thứ ba bộ đọc vẫn phải đi qua byte của cả dòng. Parquet xếp theo cột, nên đọc riêng được từng cột. Bài sẽ đo chênh lệch này.</p></details>
</div>

## Một ngày dữ liệu chuyến trong tay

Lab sinh 2.000.000 chuyến cho ngày 02/03/2026 bằng `numpy.random.default_rng(2026)`, nên cùng phiên bản numpy thì ra cùng dữ liệu. Chín cột (`trip_id` và `driver_id` được dựng từ `trip_no` và `driver_no` bằng `pl.format`), và mỗi lựa chọn kiểu có lý do (`main.py`, trích):

```python
df = pl.DataFrame(
    {
        "trip_no": np.arange(1, n + 1, dtype=np.int64),
        "requested_at": pl.Series(requested_us).cast(pl.Datetime("us")).dt.replace_time_zone("UTC"),
        "city": pl.Series(np.array(CITIES)[city]).cast(CITY_DTYPE),      # Enum 6 giá trị
        "service": pl.Series(np.array(SERVICES)[service]).cast(SERVICE_DTYPE),
        "status": pl.Series(np.where(cancelled, "cancelled", "completed")).cast(STATUS_DTYPE),
        "driver_no": driver_no.astype(np.int32),
        "distance_m": distance_m,       # int32, tính bằng mét
        "duration_s": duration_s,
    }
)
```

- **Thời điểm lưu UTC, aware.** Giờ cao điểm tính theo giờ Việt Nam bằng `convert_time_zone("Asia/Ho_Chi_Minh")` lúc truy vấn, đúng quy ước lưu UTC, hiển thị theo múi giờ.
- **Thành phố, dịch vụ, trạng thái là `Enum`.** Mỗi cột chỉ có vài giá trị; lưu chỉ số thay vì lặp lại chuỗi, vừa nhỏ vừa nhanh. Trạng thái chỉ có `completed` và `cancelled` (2 trong 8 trạng thái chuyến): đây là file các chuyến **đã kết thúc**, không có chuyến đang chạy.
- **Mã dài hơn mã quen thuộc.** `trip_id` có 7 chữ số (`TN-0000001`) và `driver_id` có 5 chữ số (`DR-21651`), khác `TN-0001` và `DR-001` ở các bài trước, vì 2 triệu chuyến mỗi ngày và 25.000 tài xế cần nhiều chữ số hơn.
- **Khoảng cách tính bằng mét, kiểu số nguyên, và cước tính bằng số nguyên.** Tiền không đi qua `float`. Công thức cước của loạt bài, 12.000 + (phần vượt 2 km) × 4.500, làm tròn nửa lên, viết lại bằng số nguyên mét:

```python
def fare_vnd(distance_m: pl.Expr) -> pl.Expr:
    extra_m = (distance_m - 2000).clip(lower_bound=0)
    # Int64: cột sẽ bị cộng dồn hàng triệu lần, Int32 sẽ tràn (xem bài).
    return (12_000 + (extra_m * 9 + 1) // 2).cast(pl.Int64).alias("fare_vnd")
```

Phần vượt (m) × 4.500 / 1.000 chính là phần vượt × 9 / 2; làm tròn nửa lên của số dương `x / 2` là `(x + 1) // 2`. Lab không tin công thức này mà so với bản `Decimal` của bài 03 (`calculate_fare`) trên 1.000 dòng mẫu và các test case biên 0, 2, 2,5, 5 km (12.000, 12.000, 14.250, 25.500); `--verify` kiểm tra cả hai.

Dữ liệu là giả, và **giờ cao điểm trong đó do chính bộ sinh gài sẵn** (trọng số lớn quanh 8 giờ và 18 giờ). Mục đích của lab là kiểm chứng pipeline phân tích tìm lại đúng đỉnh đã gài, không phải khám phá điều gì về nhu cầu thật.

## CSV và Parquet: cùng dữ liệu, hai cách xếp byte

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 272" role="img" aria-label="CSV xếp dữ liệu theo dòng nên đọc hai cột vẫn phải đi qua mọi byte; Parquet xếp theo cột nên chỉ đọc đúng hai cột city và fare_vnd trong chín cột."><text x="20" y="26" class="t-b">CSV: xếp theo dòng</text><rect x="20" y="44" width="400" height="34" rx="6" class="f-warn-soft s-warn"/><rect x="24" y="49" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="67" y="49" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="110" y="49" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="153" y="49" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="196" y="49" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="239" y="49" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="282" y="49" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="325" y="49" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="368" y="49" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="20" y="88" width="400" height="34" rx="6" class="f-warn-soft s-warn"/><rect x="24" y="93" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="67" y="93" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="110" y="93" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="153" y="93" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="196" y="93" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="239" y="93" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="282" y="93" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="325" y="93" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="368" y="93" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="20" y="132" width="400" height="34" rx="6" class="f-warn-soft s-warn"/><rect x="24" y="137" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="67" y="137" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="110" y="137" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="153" y="137" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="196" y="137" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="239" y="137" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="282" y="137" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="325" y="137" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="368" y="137" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="20" y="176" width="400" height="34" rx="6" class="f-warn-soft s-warn"/><rect x="24" y="181" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="67" y="181" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="110" y="181" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="153" y="181" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="196" y="181" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="239" y="181" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="282" y="181" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="325" y="181" width="40" height="24" rx="3" class="f-bg s-warn"/><rect x="368" y="181" width="40" height="24" rx="3" class="f-bg s-warn"/><text x="20" y="238" class="t-small">đọc 2 cột vẫn phải đi qua mọi byte của mọi dòng</text><text x="460" y="26" class="t-b">Parquet: xếp theo cột</text><rect x="464" y="44" width="40" height="166" rx="4" class="f-none s-dead dash"/><rect x="507" y="44" width="40" height="166" rx="4" class="f-none s-dead dash"/><rect x="550" y="44" width="40" height="166" rx="4" class="f-pk-soft s-pk"/><rect x="593" y="44" width="40" height="166" rx="4" class="f-none s-dead dash"/><rect x="636" y="44" width="40" height="166" rx="4" class="f-none s-dead dash"/><rect x="679" y="44" width="40" height="166" rx="4" class="f-none s-dead dash"/><rect x="722" y="44" width="40" height="166" rx="4" class="f-none s-dead dash"/><rect x="765" y="44" width="40" height="166" rx="4" class="f-none s-dead dash"/><rect x="808" y="44" width="40" height="166" rx="4" class="f-pk-soft s-pk"/><text x="544" y="228" class="t-mono">city</text><text x="798" y="228" class="t-mono">fare_vnd</text><text x="460" y="256" class="t-small">chỉ đọc 2 trong 9 cột; 7 cột còn lại không bị chạm tới</text></svg>
</div>
<figcaption>Hình 1. Cách xếp byte của cùng một bảng. Trong CSV, các ô của cùng một dòng nằm liền nhau; trong Parquet, các ô của cùng một cột nằm liền nhau (và được nén cùng nhau). Các ô được tô nhấn là phần bộ đọc thực sự cần.</figcaption>
</figure>

Lab ghi cùng một DataFrame ra hai định dạng, mỗi lệnh một dòng (`write_files`):

```python
_, csv_s = timed(lambda: df.write_csv(CSV_PATH), repeat=3)
_, pq_s = timed(lambda: df.write_parquet(PARQUET_PATH), repeat=3)  # mặc định nén zstd
```

Kết quả, trích từ lần chạy cuối (polars 2.1.0, pandas 3.1.1, numpy 2.5.4, pyarrow 26.0.1):

```text
[2] Ghi: CSV 176 MiB / 0.12 s; Parquet 26 MiB / 0.08 s; Parquet nhỏ hơn CSV 6.9 lần; 16 row group

[3] Đọc bằng polars (trung vị 5 lần, file đang nằm trong page cache của hệ điều hành): CSV 0.342 s; Parquet 0.033 s; Parquet chỉ 2 cột 0.008 s
```

Parquet nhỏ hơn 6,9 lần, đọc nhanh hơn cỡ một bậc (0,342 s so với 0,033 s), và đọc đúng hai cột `city`, `fare_vnd` mất 0,008 giây. Hai lý do chính: các giá trị trong cùng một cột lặp lại nhiều nên nén tốt (thành phố chỉ có sáu giá trị), và file lưu từng cột riêng nên bỏ qua được cột không cần. File này gồm 16 *row group*, các khối dòng mà trong mỗi khối dữ liệu lại xếp theo cột; mỗi khối có thống kê riêng (như giá trị nhỏ nhất, lớn nhất) để bộ đọc có thể bỏ qua cả khối khi điều kiện lọc cho phép. Còn một khác biệt nữa dễ bị bỏ qua: **Parquet giữ lại kiểu dữ liệu**. Cùng DataFrame, đọc lại từ hai file (trích):

```text
kiểu cột đọc lại từ CSV:      {'requested_at': Datetime(time_unit='us', time_zone='UTC'), 'city': String, 'service': String, 'status': String, 'distance_m': Int64, 'duration_s': Int64, 'fare_vnd': Int64, ...}
kiểu cột đọc lại từ Parquet:  {'requested_at': Datetime(time_unit='us', time_zone='UTC'), 'city': Enum(categories=['Hà Nội', 'TP.HCM', ...]), 'service': Enum(...), 'status': Enum(...), 'distance_m': Int32, 'duration_s': Int32, 'fare_vnd': Int64, ...}
```

Từ CSV, `city` trở về `String` và hai cột `Int32` trở về `Int64`: ta phải khai báo lại kiểu mỗi lần đọc (và thời điểm chỉ đọc về đúng nhờ `try_parse_dates=True`, tức là phải đoán từ chuỗi). Parquet mang theo cả `Enum` lẫn múi giờ. Đó là một lý do thực tế để chọn Parquet cho dữ liệu trao đổi giữa các service Python, ngoài chuyện nhỏ và nhanh. Hạn chế: Parquet không đọc được bằng mắt, không `grep` được, và khó sửa tay một dòng.

## Truy vấn lazy: chỉ đọc phần cần

Câu hỏi buổi sáng viết thành một truy vấn **lazy** (`ride_revenue_lazy`): mỗi bước chỉ mô tả, chưa có gì chạy cho đến `collect()`.

```python
def ride_revenue_lazy(lf: pl.LazyFrame) -> pl.LazyFrame:
    """Doanh thu và số chuyến `ride` đã hoàn thành theo thành phố và giờ địa phương."""
    return (
        lf.filter((pl.col("status") == "completed") & (pl.col("service") == "ride"))
        .with_columns(hour=pl.col("requested_at").dt.convert_time_zone(TZ).dt.hour())
        .group_by("city", "hour")
        .agg(trips=pl.len(), revenue_vnd=pl.col("fare_vnd").sum())
        .sort("city", "hour")
    )

plan = ride_revenue_lazy(pl.scan_parquet(PARQUET_PATH))
print(plan.explain())
```

`scan_parquet` không đọc dữ liệu, chỉ trả một `LazyFrame` biết file ở đâu. Vì cả kế hoạch được thấy trước khi chạy, bộ tối ưu của polars có thể đẩy các bước xuống tận bước đọc file. `explain()` cho thấy điều đó (trích, rút gọn đường dẫn):

```text
    simple π 3/3 ["city", "hour", "fare_vnd"]
       WITH_COLUMNS:
       [col("requested_at").dt.convert_time_zone().dt.hour().alias("hour")]
        simple π 3/3 ["requested_at", "city", ... 1 other column]
          Parquet SCAN [.../out/trips.parquet]
          PROJECT 5/9 COLUMNS
          SELECTION: (col("service") == "ride") & (col("status") == "completed")
          ESTIMATED ROWS: 2000000
```

Hai dòng đáng đọc: `PROJECT 5/9 COLUMNS` (chỉ 5 cột trong 9 được đọc từ file: `requested_at`, `city`, `fare_vnd` cho kết quả, cùng `service` và `status` cho bộ lọc) và `SELECTION` (điều kiện lọc được áp ngay ở bước đọc). Ta không viết `select` nào, bộ tối ưu tự suy ra.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Kế hoạch lazy gồm scan_parquet, filter, with_columns và group_by; bộ tối ưu đẩy việc chọn 5 trong 9 cột và điều kiện lọc xuống bước scan, và chỉ collect mới chạy toàn bộ."><defs><marker id="py29-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py29-1-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="30" width="180" height="76" rx="10" class="f-phys-soft s-phys"/><text x="36" y="62" class="t-b">scan</text><text x="36" y="86" class="t-mono">scan_parquet</text><rect x="235" y="30" width="150" height="76" rx="10" class="f-pk-soft s-pk"/><text x="251" y="62" class="t-b">filter</text><text x="251" y="86" class="t-small">ride, completed</text><rect x="420" y="30" width="170" height="76" rx="10" class="f-pk-soft s-pk"/><text x="436" y="62" class="t-b">with_columns</text><text x="436" y="86" class="t-small">hour (giờ Việt Nam)</text><rect x="625" y="30" width="235" height="76" rx="10" class="f-pk-soft s-pk"/><text x="641" y="62" class="t-b">group_by + agg</text><text x="641" y="86" class="t-small">city, hour: trips, revenue</text><path d="M200 68 L233 68" class="f-none s-phys" stroke-width="2" marker-end="url(#py29-1-arrow)"/><path d="M385 68 L418 68" class="f-none s-phys" stroke-width="2" marker-end="url(#py29-1-arrow)"/><path d="M590 68 L623 68" class="f-none s-phys" stroke-width="2" marker-end="url(#py29-1-arrow)"/><rect x="20" y="150" width="540" height="84" rx="10" class="f-warn-soft s-warn"/><text x="36" y="176" class="t-b">Bộ tối ưu đẩy xuống bước scan</text><text x="36" y="198" class="t-small">PROJECT 5/9 COLUMNS: chỉ đọc 5 trong 9 cột</text><text x="36" y="218" class="t-small">SELECTION: lọc service, status ngay khi đọc file</text><path d="M310 106 L310 148" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py29-1-warn)"/><path d="M110 148 L110 108" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py29-1-warn)"/><text x="600" y="196" class="t-small">Chưa gọi collect() thì chưa đọc</text><text x="600" y="216" class="t-small">dữ liệu cột nào từ file.</text></svg>
</div>
<figcaption>Hình 2. Kế hoạch lazy và những gì bộ tối ưu đẩy xuống bước đọc. Cột không cần và dòng không khớp điều kiện không đi qua các bước phía sau.</figcaption>
</figure>

Chạy `collect()` cho 144 nhóm (6 thành phố × 24 giờ), rồi `city_summary` gom lại mỗi thành phố một dòng (trích):

```text
┌───────────┬────────────┬───────────┬────────────┬─────────────┐
│ city      ┆ ride_trips ┆ peak_hour ┆ peak_trips ┆ revenue_vnd │
╞═══════════╪════════════╪═══════════╪════════════╪═════════════╡
│ TP.HCM    ┆ 359382     ┆ 8         ┆ 32892      ┆ 9530110962  │
│ Hà Nội    ┆ 295678     ┆ 8         ┆ 27257      ┆ 7836339079  │
│ Đà Nẵng   ┆ 126691     ┆ 18        ┆ 11618      ┆ 3365833636  │
│ Hải Phòng ┆ 106207     ┆ 18        ┆ 9788       ┆ 2820244332  │
│ Nha Trang ┆ 84467      ┆ 8         ┆ 7613       ┆ 2238063295  │
│ Cần Thơ   ┆ 84433      ┆ 8         ┆ 7744       ┆ 2240991361  │
└───────────┴────────────┴───────────┴────────────┴─────────────┘
```

Giờ cao điểm (giờ địa phương) là 8 hoặc 18, đúng đỉnh đã gài. Hai giờ này ngang nhau về trọng số nên thành phố nào lọt về bên nào phụ thuộc may rủi của mẫu, đừng đọc ra ý nghĩa nghiệp vụ. `ride_trips` xếp theo đúng tỉ lệ thành phố đã gài (TP.HCM 34%, Hà Nội 28%...).

## pandas đặt cạnh polars

Cùng truy vấn đó viết bằng pandas (`ride_revenue_pandas`) thì là eager: mỗi dòng code dựng một bảng trung gian.

```python
def ride_revenue_pandas(df: pd.DataFrame) -> pd.DataFrame:
    done = df[(df["status"] == "completed") & (df["service"] == "ride")]
    hour = done["requested_at"].dt.tz_convert(TZ).dt.hour.rename("hour")
    out = (
        done.groupby(["city", hour], observed=True)
        .agg(trips=("fare_vnd", "size"), revenue_vnd=("fare_vnd", "sum"))
        .reset_index()
    )
    ...
```

Dòng đầu tiên đã tạo một bản sao của mọi cột cho các dòng khớp, kể cả cột không dùng. `--verify` xác nhận hai engine ra cùng 144 nhóm (sau khi đưa về cùng kiểu cột: pandas trả `category` còn polars trả `Enum`). Còn chi phí thì sao? Mỗi kịch bản chạy trong một tiến trình riêng, ba lần, lấy thời gian trung vị và bộ nhớ RSS lớn nhất. Kịch bản eager đo tách hai bước, đọc rồi truy vấn; kịch bản lazy chỉ có một con số tổng vì kế hoạch gộp đọc và truy vấn. Trích từ lần đo chốt (load average của máy nêu ngay trong output):

```text
[6] pandas đọc Parquet 0.038 s; trong RAM 116 MiB (polars: 86 MiB)

[7] Cùng truy vấn, mỗi kịch bản một tiến trình, 3 lần (giây trung vị, RSS tối đa):
    load average trước khi đo (1/5/15 phút): 64.15 49.35 44.18
  kịch bản                tổng     đọc  truy vấn  RSS MiB
  baseline               0.000       -         -      153
  pandas_csv             0.575   0.430     0.145      745
  pandas_parquet         0.145   0.072     0.074      569
  polars_csv             0.289   0.275     0.014      448
  polars_scan_csv        0.307       -         -      397
  polars_scan_parquet    0.036       -         -      231
  polars_streaming       0.022       -         -      229
    load average sau khi đo (1/5/15 phút): 50.04 47.72 43.82
```

**Máy lúc đo rất bận**: load average 64,15 (1 phút) trên 10 lõi lúc bắt đầu đo, 50,04 lúc kết thúc, vì máy còn các phiên làm việc khác chạy song song; đây là lần đo chốt lúc không có tiến trình nào của chính loạt bài này chạy, nhưng máy không bao giờ rảnh hẳn. Vì vậy chỉ nên đọc theo bậc độ lớn, không đọc chênh lệch nhỏ. Hai con số 0,038 s ở `[6]` và 0,072 s ở cột `đọc` của `pandas_parquet` cũng không thể so sánh trực tiếp với nhau: `[6]` là trung vị 5 lần đọc lặp trong tiến trình đã khởi động, `[7]` là lần đọc đầu trong tiến trình mới.

Dòng `baseline` chỉ import thư viện: 153 MiB là nền cố định, nên «bộ nhớ thêm» của mỗi kịch bản là con số sau khi trừ đi phần nền. Gọn lại:

| Kịch bản | Thời gian: tổng (đọc + truy vấn) | RSS thêm so với nền |
| --- | --- | --- |
| pandas, đọc CSV | 0,58 s (0,43 + 0,15) | khoảng 592 MiB |
| pandas, đọc Parquet | 0,15 s (0,07 + 0,07) | khoảng 416 MiB |
| polars, `read_csv` rồi truy vấn | 0,29 s (0,28 + 0,01) | khoảng 295 MiB |
| polars, `scan_csv` lazy | 0,31 s | khoảng 244 MiB |
| polars, `scan_parquet` lazy | 0,04 s | khoảng 78 MiB |

- **Kết luận được.** Khoảng cách lớn nhất, pandas đọc CSV 0,58 s so với polars quét Parquet 0,04 s (cỡ một bậc, gần 16 lần) và 592 MiB so với 78 MiB bộ nhớ thêm, vượt xa độ nhiễu của máy bận. Với polars, truy vấn trên bảng đã nằm trong RAM chỉ mất 0,01 s; `polars_csv` gần như toàn là thời gian đọc CSV.
- **Không kết luận được.** Thứ bậc giữa các biến thể sít nhau, như `polars_csv` (0,29 s) với `scan_csv` (0,31 s): các lần chạy trước của chính lab này cho hai biến thể đổi chỗ cho nhau (`scan_csv` 0,26 s và `polars_csv` 0,27 s ở một lần, 0,48 s và 0,36 s ở lần khác). Cũng không quy được phần nhanh của `scan_parquet` cho riêng việc chọn cột hay riêng việc lọc, vì lab không đo từng tối ưu riêng; ta chỉ thấy kế hoạch có `PROJECT 5/9 COLUMNS`, `SELECTION` và RSS thêm nhỏ (78 MiB), khớp với một kế hoạch chỉ đọc một phần dữ liệu.

Ở 2 triệu dòng mọi kịch bản đều dưới một giây, sự khác biệt chưa thành vấn đề; nó thành vấn đề khi truy vấn chạy trăm lần mỗi ngày hoặc dữ liệu lớn hơn. Mọi lần đo đều đọc file nằm trong page cache; đọc từ đĩa lạnh chậm hơn, nhất là CSV 176 MiB.

Ngoại suy tuyến tính lên 730 triệu dòng mỗi năm (chỉ là phép nhân, không phải đo):

```text
[8] Ngoại suy tuyến tính lên 730 triệu dòng/năm: CSV 63 GiB, Parquet 9.1 GiB, polars trong RAM 31 GiB
```

Một năm dữ liệu trong RAM (31 GiB cho polars, nhiều hơn cho pandas) vượt quá dung lượng RAM của phần lớn máy; nên chia theo ngày thành nhiều file Parquet và chỉ quét phần cần, đó là việc của bài 65. Kịch bản `polars_streaming` là một nấc trung gian: engine streaming xử lý theo từng lô thay vì nạp cả bảng, và `--verify` xác nhận nó cho đúng cùng kết quả (ở kích cỡ này RSS của nó, 229 MiB, gần bằng `scan_parquet` thường (231 MiB), nên chưa thấy tiết kiệm thêm).

## Khi nào không nên dùng pandas, và bẫy cộng dồn

pandas không hỏng. pandas là cầu nối của cả hệ sinh thái: scikit-learn (bài 57), matplotlib và đa số thư viện phân tích nhận hoặc trả DataFrame của pandas. Với dữ liệu nhỏ và thao tác tương tác, pandas dùng thoải mái hơn. Ranh giới để chọn polars là:

- Dữ liệu từ vài trăm MiB trở lên hoặc truy vấn lặp lại: cần kế hoạch lazy, đọc chọn lọc và đa luồng.
- Dữ liệu lớn hơn RAM, hoặc sắp lớn lên: `scan_parquet` với streaming trước khi nghĩ tới cluster.
- Pipeline cần kiểu rõ ràng và tái lập (Enum, aware datetime, số nguyên đúng kích cỡ), không muốn đoán kiểu mỗi lần đọc.

Chuyển qua lại ở ranh giới với thư viện khác là chuyện thường: `df.to_pandas()` khi đưa sang sklearn, và ngược lại `pl.from_pandas(...)`. Lab dùng `pl.from_pandas` trong `--verify`, và pandas đọc Parquet qua pyarrow nên lab cài sẵn thư viện này.

Còn một bẫy đã cắn ngay trong lúc viết lab: kiểu số nguyên hẹp. `Int32` cho `distance_m` tiết kiệm bộ nhớ, nhưng chỉ an toàn **khi không ai cộng cột đó trên cả bảng**, mà tổng quãng đường cả ngày là phép cộng rất tự nhiên. Polars cộng `Int32` vẫn ra `Int32` và quay vòng khi vượt 2.147.483.647, không báo lỗi. Lab giữ `distance_m` ở `Int32` có chủ ý để cho thấy (polars 2.1.0):

```text
[9] Cộng hai lần 2.000.000.000: Int32 -> -294,967,296; Int64 -> 4,000,000,000
    Tổng quãng đường cả ngày (m): cột Int32 -> 1,693,032,173; ép Int64 -> 10,282,966,765
    Tổng cước chuyến completed (VND): ép Int32 -> -547,871,422; Int64 -> 46,696,768,834
```

Dòng đầu là minh hoạ tối thiểu. Dòng thứ hai nguy hiểm hơn: **kết quả sai là một số dương trông hợp lý** (1,69 tỷ mét thay vì 10,28 tỷ) và không có dấu hiệu nào báo là sai; số này chỉ là phần dư của tổng đúng khi chia cho 2^32. Cách phòng: `cast(pl.Int64)` trước khi cộng, như `df["distance_m"].cast(pl.Int64).sum()`, hoặc chọn `Int64` ngay từ đầu cho cột có thể bị cộng (`fare_vnd` là `Int64` vì lý do đó). pandas không mắc bẫy này vì tự nâng kiểu khi cộng (ta đã thử `Series` `int32` hai số 2 tỷ: ra 4.000.000.000); đó là khác biệt về mặc định, và sự khác biệt này dễ bị bỏ sót khi chuyển pipeline từ pandas sang polars. `--verify` kiểm cả hai tổng: bản `Int64` so với tổng tính độc lập bằng numpy, bản `Int32` so với phần dư theo modulo 2^32.

## Chạy và kiểm chứng

Lab ở `examples/toi-noi-python/29-polars-pandas-parquet/`. Dữ liệu sinh ra `out/` (khoảng 200 MiB, đã được `.gitignore`).

```bash
uv run --project examples/toi-noi-python/29-polars-pandas-parquet examples/toi-noi-python/29-polars-pandas-parquet/main.py
uv run --project examples/toi-noi-python/29-polars-pandas-parquet examples/toi-noi-python/29-polars-pandas-parquet/main.py --verify
```

Lệnh đầu in báo cáo như các khối trên; lệnh thứ hai kiểm tra 17 bất biến và in (output lần chạy cuối, Python 3.14.7):

```text
OK 17/17
```

Các bất biến gồm: số dòng, đủ 6 thành phố và 3 dịch vụ, mã chuyến duy nhất, cước số nguyên khớp `Decimal`, Parquet đọc lại bằng đúng DataFrame gốc, Parquet nhỏ hơn CSV ít nhất 3 lần, lazy trên Parquet bằng eager trong RAM, bằng `scan_csv`, bằng pandas và bằng engine streaming, tổng nhóm bằng tổng trực tiếp, `Int32` tràn còn `Int64` đúng (ví dụ hai số 2 tỷ, tổng quãng đường và tổng cước completed), giờ cao điểm Hà Nội là 8 hoặc 18. Lab không ghim tổng doanh thu cụ thể: bộ sinh số ngẫu nhiên `Generator` của numpy không cam kết giữ nguyên dòng số giữa các phiên bản, nên các tổng trong bài đúng với numpy 2.5.4. Máy ít RAM có thể chạy `--rows 500000`.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Tổng tiền ra số âm hoặc nhỏ bất thường | Kiểu cột cộng dồn có là `Int32` không; `df.schema`, rồi `cast(pl.Int64)` |
| `ImportError` về pyarrow khi `pd.read_parquet` | Cài `pyarrow` (đã có trong `pyproject.toml` của lab) |
| Cột `city` đọc từ CSV là chuỗi, không phải `Enum` | CSV không giữ kiểu; truyền `schema_overrides` hoặc dùng Parquet |
| `scan_parquet` chậm bất ngờ | `plan.explain()`: có `PROJECT n/m COLUMNS` và `SELECTION` không; thao tác nào chặn việc đẩy xuống |
| Giờ cao điểm lệch 7 giờ | Đã `convert_time_zone` sang `Asia/Ho_Chi_Minh` trước `dt.hour()` chưa; dữ liệu lưu UTC |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Dữ liệu sinh bằng numpy, không có dòng thiếu, trùng, lệch múi giờ hay mã thành phố lạ như dữ liệu thật; mã chuyến và mã tài xế dài hơn dạng <code>TN-0001</code>, <code>DR-001</code> của các bài trước. Số đo lấy từ một máy macOS Apple silicon 10 lõi đang rất bận (load average 64,15 trên 10 lõi lúc bắt đầu đo), file nằm trong page cache; đừng dùng làm cam kết hiệu năng. Ngoại suy 730 triệu dòng là phép nhân tuyến tính, thực tế còn phụ thuộc phân vùng và nén. Một file Parquet duy nhất cũng chưa phải kiến trúc kho dữ liệu: không phân vùng theo ngày, không ghi tăng dần, không idempotent; những thứ đó thuộc bài 65. Cuối cùng, phân tích ở đây là mô tả, không phải dự đoán; mô hình dự đoán nằm ở phần sau của lộ trình.</p>
</div>

**Bài tập:** (1) Hãy tìm 5 tài xế có nhiều chuyến `completed` nhất trong ngày bằng truy vấn lazy trên file Parquet. Nếu hai tài xế bằng số chuyến thì xếp theo mã. (2) Vì sao ta không cần sửa `ride_revenue_lazy` khi bảng có thêm 20 cột mới, và việc đó khác pandas thế nào?

<details><summary>Xem lời giải</summary><p>(1) Lọc, nhóm theo tài xế, sắp xếp có tiêu chí phụ để kết quả lặp lại được:</p>

```python
top = (
    pl.scan_parquet(PARQUET_PATH)
    .filter(pl.col("status") == "completed")
    .group_by("driver_id")
    .agg(trips=pl.len(), revenue_vnd=pl.col("fare_vnd").sum())
    .sort(["trips", "driver_id"], descending=[True, False])
    .head(5)
    .collect()
)
```

<p>Kết quả trên dữ liệu của lab (đã chạy): DR-23037 108 chuyến, DR-15222 107, DR-01838 104, DR-13549 103, DR-23213 102. (2) Bộ tối ưu chỉ chọn những cột mà truy vấn nhắc tới, nên 20 cột thêm vào không được đọc từ file; <code>PROJECT 5/9 COLUMNS</code> sẽ thành <code>PROJECT 5/29 COLUMNS</code> (đã thử trên file thêm 20 cột). Với <code>pd.read_parquet(path)</code> không có tham số <code>columns</code>, pandas đọc hết 29 cột vào RAM rồi mới lọc; muốn tiết kiệm phải nhớ liệt kê cột bằng tay.</p></details>

Bài trước: [bài 28](/articles/python-integration-test-testcontainers-fixture). Bài tiếp: [bài 30](/articles/python-numpy-pillow-opencv-tien-xu-ly-anh).

## Tài liệu gốc

- [Polars: Lazy API](https://docs.pola.rs/user-guide/lazy/).
- [Polars: Query optimizations](https://docs.pola.rs/user-guide/lazy/optimizations/).
- [Polars: Streaming](https://docs.pola.rs/user-guide/concepts/streaming/).
- [Polars: Expr.sum](https://docs.pola.rs/api/python/stable/reference/expressions/api/polars.Expr.sum.html).
- [Apache Parquet: Documentation](https://parquet.apache.org/docs/).
- [pandas: IO tools (CSV, Parquet)](https://pandas.pydata.org/docs/user_guide/io.html).
