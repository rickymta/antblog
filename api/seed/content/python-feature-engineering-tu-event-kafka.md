---
slug: python-feature-engineering-tu-event-kafka
title: "Feature engineering: từ event Kafka tới đặc trưng velocity, thiết bị, GPS"
summary: "Bài 55: biến event Kafka thành đặc trưng gian lận: cửa sổ thời gian với polars rolling, đếm theo thiết bị/khách/tài xế, haversine và tốc độ phi lý, tái hiện leakage bằng số đo và tính online khớp tính theo lô."
category: programming
level: advanced
tags: [python, toi-noi, fraud, polars, feature-engineering]
language: vi
---

Fraud-scoring của [Tới Nơi](/articles/du-an-mau-toi-noi) nhận ba loại event qua Kafka: `BookingCreated` lúc đặt, `TripCompleted` lúc kết thúc, `PaymentCaptured` lúc thu tiền. Một event riêng lẻ nói rất ít: chuyến 4,5 km giá 23.250 đồng là sạch hay không? Dấu vết gian lận nằm ở chỗ khác, ở bộ ba event nối với nhau và với lịch sử: chuyến thứ ba của cùng một điện thoại trong 40 phút, tài xế «bay» hơn 100 km giữa hai chuyến, năm tài khoản chung một thiết bị cùng ăn mã giảm một nửa. **Đặc trưng** (feature) là con số rút từ chuỗi event để luật và mô hình đọc được. Bài này dựng chúng, rồi chỉ ra lỗi đắt nhất khi dựng: dùng thông tin mà lúc chấm điểm thật chưa có, gọi là **leakage**.

Đây là bài 55 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần bộ dữ liệu tổng hợp và nhãn trễ của [bài 54](/articles/python-fraud-toi-noi-bai-toan-nhan-du-lieu-metric), event Kafka ở [bài 38](/articles/python-kafka-aiokafka-producer-consumer-schema) và polars ở [bài 29](/articles/python-polars-pandas-parquet-phan-tich-chuyen). Dữ liệu là tổng hợp, số đo chỉ nói về bộ này; điểm rủi ro là đầu vào cho người duyệt, không phải quyết định khóa tài khoản.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một đặc trưng «số chuyến của thiết bị trong giờ đó» được tính bằng cách gom theo giờ lịch (<code>group_by_dynamic</code>, mỗi giờ một ô) rồi gắn tổng của ô vào từng chuyến. Nó có dùng thông tin tương lai không?</p>
<details><summary>Xem đáp án</summary><p><b>Có.</b> Chuyến lúc 10:05 nhận tổng của cả giờ 10:00 đến 11:00, kể cả chuyến lúc 10:40 chưa xảy ra. Lab đo bằng phép thử cắt cụt: tính lại đặc trưng của một chuyến từ các event có sẵn đến lúc nó được thu tiền. Với 40 chuyến mẫu, đặc trưng gom theo giờ đổi giá trị ở 23 chuyến, đặc trưng cửa sổ trượt <code>[t - 1h, t)</code> không đổi ở chuyến nào.</p></details>
</div>

## Từ event đến bảng chuyến, và quy tắc «tại thời điểm chấm»

Đặc trưng theo chuyến cần một dòng mỗi chuyến, nên bước đầu là nối ba loại event theo `trip_id` (`features.py`, trích):

```python
def trip_table(events: pl.DataFrame) -> pl.DataFrame:
    """Nối ba loại event theo trip_id thành một dòng mỗi chuyến, sắp theo thời điểm chấm điểm paid_at."""
    booking = only("BookingCreated", [...], {"occurred_at": "booked_at"})
    done = only("TripCompleted", [...], {"occurred_at": "completed_at"})
    pay = only("PaymentCaptured", ["amount_vnd", "discount_vnd"], {"occurred_at": "paid_at"})
    return (
        booking.join(done, on="trip_id", how="inner").join(pay, on="trip_id", how="inner")
        .with_columns(started_at=pl.col("completed_at") - pl.duration(seconds=pl.col("duration_s")))
        .sort("paid_at")
    )
```

```text
trip_table: nối 3 loại event theo trip_id -> 200,000 chuyến x 22 cột trong 0.84 giây
```

Chuyến được chấm điểm ở thời điểm `PaymentCaptured`, nên gọi `paid_at` là **thời điểm chấm** (as-of time). Quy tắc xuyên suốt: đặc trưng của một chuyến chỉ được dùng ba event của chính nó và các event **xảy ra trước** `paid_at`. Lab đơn giản hóa thêm một bước: chỉ đếm những chuyến trước đã thu tiền, chuyến đang chạy chưa tính. Mọi leakage trong bài này là một cách vi phạm quy tắc đó. Hình 1 minh họa cho một thiết bị.

Trong luồng thật có hai đồng hồ. **Event time** là `occurred_at`, lúc sự việc xảy ra; **processing time** là lúc consumer nhận được event. Mọi cửa sổ trong bài dùng event time, vì processing time đổi theo độ trễ mạng, độ trễ consumer và cả việc xử lý lại sau sự cố. Mục cuối đo hậu quả của thứ tự đọc nhiều topic.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Trục thời gian của một thiết bị. Chuyến đang chấm tại thời điểm t. Cửa sổ từ t trừ 1 giờ đến t chứa ba chuyến A, B, C nên dev_trips_1h bằng 3. Chuyến Z cũ hơn một giờ và hai chuyến D, E ở tương lai không được đếm."><defs><marker id="py55-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="280" y="64" width="280" height="96" rx="8" class="f-phys-soft s-phys"/><text x="292" y="88" class="t-b">cửa sổ [t - 1h, t)</text><rect x="560" y="64" width="290" height="96" rx="8" class="f-warn-soft s-warn dash"/><text x="572" y="88" class="t-b">tương lai</text><text x="572" y="108" class="t-small">chưa xảy ra lúc chấm điểm</text><path d="M30 150 L850 150" class="f-none s-line" stroke-width="2" marker-end="url(#py55-0-arrow)"/><circle cx="150" cy="150" r="7" class="f-muted s-muted"/><text x="144" y="178" class="t-small">Z</text><circle cx="330" cy="150" r="7" class="f-phys s-phys"/><text x="324" y="178" class="t-small">A</text><circle cx="430" cy="150" r="7" class="f-phys s-phys"/><text x="424" y="178" class="t-small">B</text><circle cx="520" cy="150" r="7" class="f-phys s-phys"/><text x="514" y="178" class="t-small">C</text><circle cx="560" cy="150" r="10" class="f-pk s-pk"/><text x="500" y="204" class="t-b">chuyến đang chấm, t</text><circle cx="650" cy="150" r="7" class="f-none s-warn dash"/><text x="644" y="178" class="t-small">D</text><circle cx="750" cy="150" r="7" class="f-none s-warn dash"/><text x="744" y="178" class="t-small">E</text><text x="40" y="206" class="t-small">Z cũ hơn 1 giờ</text><text x="40" y="244" class="t-small">dev_trips_1h = 3 (A, B, C). Z quá cũ, D và E chưa xảy ra nên không được đếm.</text></svg>
</div>
<figcaption>Hình 1. Đặc trưng cửa sổ của chuyến đang chấm chỉ đếm các chuyến nằm trong khoảng một giờ trước nó. Chuyến cũ hơn và chuyến ở tương lai đều ngoài cửa sổ.</figcaption>
</figure>

## Cửa sổ thời gian: đếm theo thiết bị, khách, tài xế

Với bảng đã sắp theo `paid_at`, polars cho cửa sổ trượt theo thời gian ngay trong biểu thức: `Expr.rolling(index_column, period=..., closed=...)` kết hợp `.over(khóa)` để mỗi khóa có cửa sổ riêng. Có một cái bẫy: `rolling(...).over(khóa)` của polars 2.1.0 **không báo lỗi** khi bảng chưa sắp theo cột chỉ mục, nó trả giá trị sai âm thầm (còn `rolling` không có `over` ném `InvalidOperationError`, `group_by_dynamic` có `group_by` ném `ComputeError`). Vì vậy `add_features` tự kiểm `is_sorted` và ném `ValueError`; phần chạy bên dưới đo cái giá khi không có hàng rào. Hai hàm nhỏ của `add_features` phủ gần hết các đặc trưng cửa sổ:

```python
def count(key: str | list[str], period: str) -> pl.Expr:
    # cửa sổ [t - period, t): closed="left" loại chính dòng hiện tại
    return pl.len().rolling("paid_at", period=period, closed="left").over(key)

def distinct(col: str, key: str, period: str) -> pl.Expr:
    # closed="both": gồm cả chuyến hiện tại, để "khách mới trên thiết bị" tự được tính
    return pl.col(col).n_unique().rolling("paid_at", period=period, closed="both").over(key)
```

`closed="left"` quyết định chuyện sống còn: cửa sổ là `[t - period, t)`, không có `t`, nên chuyến đang chấm không tự đếm mình và chuyến trùng đúng thời điểm không lọt vào. Số khách khác nhau trên một thiết bị dùng `closed="both"` vì ta muốn thấy «khách thứ ba xuất hiện trên máy này», nên chuyến hiện tại phải được tính. Danh sách đặc trưng nhóm cửa sổ:

| Đặc trưng | Khóa | Cửa sổ | Nhắm tới |
|---|---|---|---|
| `dev_trips_1h`, `dev_trips_24h` | thiết bị | 1 giờ, 24 giờ | chuyến ảo dồn dập |
| `cust_trips_*`, `drv_trips_*` | khách, tài xế | 1 giờ, 24 giờ | tài khoản hoặc tài xế chạy dồn |
| `dev_customers_24h`, `dev_customers_7d` | thiết bị | 24 giờ, 7 ngày | nhiều tài khoản một máy |
| `pair_trips_7d` | cặp khách-tài xế | 7 ngày | cấu kết |
| `dev_newuser_7d`, `cust_promo_share_30d` | thiết bị, khách | 7 ngày, 30 ngày | lạm dụng khuyến mãi (số lần dùng `NEWUSER50` trên máy; tỉ lệ chuyến có mã của khách) |
| `cust_prior_trips` | khách | từ đầu | tài khoản mới tinh |

Tính 30 đặc trưng cho 200.000 chuyến mất 0,14 giây trên máy đo (load average 1 phút lúc đó 39,36 trên 10 lõi, nên số giây chỉ để biết cỡ). Trung bình theo loại cho thấy các cột tách loại thế nào:

```text
loại                 dev_trips_1h         drv_trips_1h     dev_customers_7d        pair_trips_7d       dev_newuser_7d cust_promo_share_30d
collusion                   0.022                0.118                1.945                1.085                0.000                0.000
ghost_trip                  0.545                0.886                1.610                1.059                0.000                0.000
gps_spoof                   0.005                0.944                1.015                0.001                0.021                0.073
none                        0.023                0.192                1.015                0.055                0.018                0.084
promo_abuse                 0.025                0.118                3.062                0.000                2.062                0.000
```

Các cột tách loại nhưng không sạch sẽ. Mỗi loại có một cột nổi trội: `ghost_trip` ở `dev_trips_1h` (0,545 so với 0,023 của chuyến sạch), `promo_abuse` ở `dev_customers_7d` (3,062 so với 1,015) và `dev_newuser_7d` (2,062 so với 0,018), `collusion` ở `pair_trips_7d` (1,085 so với 0,055), `gps_spoof` ở `drv_trips_1h` (0,944 so với 0,192). Nhưng các loại lấn nhau: `drv_trips_1h` của `gps_spoof` lớn hơn của `ghost_trip` (0,886), `pair_trips_7d` của `ghost_trip` (1,059) gần bằng `collusion`, `dev_customers_7d` của `collusion` (1,945) lớn hơn của `ghost_trip` (1,610). `cust_promo_share_30d` là tạo tác của bộ sinh: 0,000 ở `ghost_trip`, `promo_abuse`, `collusion` và 0,084 ở chuyến sạch, vì tài khoản gian lận không có chuyến khuyến mãi nào trước đó; `cust_promo_share_30d > 0` vì thế tố «sạch», điều sẽ không đúng với gian lận thật. Một thiết bị gian lận cụ thể, tám chuyến đầu (giờ Việt Nam): các chuyến cách nhau từ 7 đến 37 phút nên cửa sổ một giờ chỉ giữ vài chuyến gần nhất, số khách trên máy lên 2 ở chuyến thứ sáu, và `pair_trips_7d` tăng mỗi khi một khách gặp lại cùng tài xế:

```text
trip_id                   paid_at  dev_trips_1h  dev_customers_24h  pair_trips_7d   fraud_type
TN-0076347         24/03 21:20:10             0                  1              0   ghost_trip
TN-0076351         24/03 21:39:37             1                  1              1   ghost_trip
TN-0076362         24/03 22:11:44             2                  1              2   ghost_trip
TN-0076367         24/03 22:48:43             1                  1              3   ghost_trip
TN-0076374         24/03 23:14:00             1                  1              4   ghost_trip
TN-0076379         24/03 23:47:08             2                  2              0   ghost_trip
TN-0076388         24/03 23:54:31             2                  2              1   ghost_trip
TN-0076399         25/03 00:15:21             2                  2              5   ghost_trip
```

Đối chứng cho cái bẫy ở trên, từ lab:

```text
đối chứng: dev_trips_1h tính trên bảng sắp theo booked_at, không báo lỗi gì, lệch 853/200,000 chuyến
```

Đặc trưng này sinh ra từ cửa sổ **trượt**, không phải cửa sổ **rơi** (tumbling) của `group_by_dynamic`. Cửa sổ rơi chia trục thời gian thành các ô cố định, mỗi event thuộc đúng một ô: tốt cho báo cáo theo giờ hay theo ngày, nhưng có hai tật khi làm đặc trưng: một đợt dồn dập vắt qua ranh giới giờ bị chẻ đôi, và gắn tổng của ô trở lại từng chuyến thì mỗi chuyến nhận cả phần tương lai của ô (mục sau tái hiện).

## Haversine, tốc độ phi lý và nhảy vị trí

GPS giả để lại hai dấu vết hình học. Một là **trong chuyến**: điểm bắt đầu cách điểm kết thúc quá xa so với thời lượng, tức tốc độ trung bình `speed_kmh` phi lý. Hai là **giữa các chuyến**: điểm bắt đầu cách xa điểm kết thúc của chuyến trước của cùng tài xế mà khoảng cách thời gian không đủ để đi (`drv_jump_speed_kmh`). Cả hai cần khoảng cách giữa hai tọa độ. **Haversine** cho khoảng cách đường chim bay trên mặt cầu bán kính 6.371 km; viết bằng biểu thức polars thì chạy được trên cả cột:

```python
def haversine_expr(lat1: str, lng1: str, lat2: str, lng2: str) -> pl.Expr:
    """Khoảng cách đường chim bay (km) giữa hai cặp cột (độ), bán kính Trái Đất 6371 km."""
    p1, p2 = pl.col(lat1).radians(), pl.col(lat2).radians()
    dphi = p2 - p1
    dlmb = (pl.col(lng2) - pl.col(lng1)).radians()
    a = (dphi / 2).sin() ** 2 + p1.cos() * p2.cos() * (dlmb / 2).sin() ** 2
    return 2 * 6371.0 * a.sqrt().arcsin()
```

```text
Hà Nội - TP.HCM (tâm thành phố, đường chim bay): 1144 km; tại 120 km/h mất 9.5 giờ
biểu thức polars so với numpy trên 200,000 chuyến: lệch lớn nhất 1.37e-12 km
```

Hà Nội đến TP.HCM đường chim bay 1.144 km, nên hai chuyến cách nhau dưới 9,5 giờ ở 120 km/h là phi lý. Nhảy vị trí so với chuyến trước dùng `shift` trong từng tài xế, kèm một chốt chặn nhỏ:

```python
prev = pl.col("driver_id")
t = t.with_columns(
    _prev_lat=pl.col("end_lat").shift(1).over(prev), _prev_lng=pl.col("end_lng").shift(1).over(prev),
    _prev_done=pl.col("completed_at").shift(1).over(prev),
).with_columns(
    drv_jump_km=haversine_expr("_prev_lat", "_prev_lng", "start_lat", "start_lng").fill_null(0.0),
    # chờ tối thiểu 60 giây để nhiễu GPS vài chục mét không thành "hàng nghìn km/h"
    drv_gap_s=(pl.col("started_at") - pl.col("_prev_done")).dt.total_seconds().clip(lower_bound=60).fill_null(0).cast(pl.Float64),
)
```

`drv_jump_speed_kmh` là `drv_jump_km` chia khoảng cách thời gian, và hai cờ `impossible_speed`, `impossible_jump` đặt ngưỡng 120 km/h. Chuyến đầu của mỗi tài xế không có chuyến trước nên các số nhảy bằng 0. Kết quả trên 200.000 chuyến:

```text
loại           chuyến  speed_kmh>120  jump>120 km/h  một trong hai
none          196,000           0.1%           0.0%           0.1%
ghost_trip      1,400           0.0%           0.0%           0.0%
gps_spoof         800          39.0%          34.9%          39.1%
promo_abuse     1,200           0.0%           0.0%           0.0%
collusion         600           0.0%           0.0%           0.0%
chuyến sạch bị gắn cờ: 111 trên 196,000; gps_spoof bị gắn cờ: 313 trên 800
```

Luật tốc độ gần như chỉ sáng ở `gps_spoof`, nhưng chỉ bắt 39,1 % (313 trong 800 chuyến). Bộ sinh dịch 60 % số chuyến `gps_spoof` chỉ vài km trong thành phố, nên chúng không tạo tốc độ phi lý; muốn bắt cần thêm khoảng cách giữa điểm đón khách khai báo và GPS tài xế (`start_pickup_gap_km`, đặc trưng theo từng chuyến). 111 chuyến sạch bị gắn cờ là những chuyến mà bộ sinh cố ý cho GPS lệch xa. Không đặc trưng nào hoàn hảo; mô hình ở bài 57 tồn tại để kết hợp chúng.

## Leakage: tái hiện và tránh

**Leakage** là đặc trưng chứa thông tin mà lúc chấm điểm thật chưa có. Lab tái hiện ba kiểu, mỗi kiểu một cách vi phạm cùng quy tắc «chỉ dùng thứ có trước `paid_at`».

**Kiểu 1: cửa sổ nhìn về tương lai.** Chỉ cần đổi `offset` của `rolling` để cửa sổ bắt đầu sau `t` là đặc trưng nhìn trước. Biến thể hay gặp hơn là gom theo giờ lịch bằng `group_by_dynamic` rồi join ngược:

```python
def add_future_window(t: pl.DataFrame) -> pl.DataFrame:
    """Rò rỉ 1: đếm chuyến cùng thiết bị trong (t, t + 1h], tức là NHÌN VỀ TƯƠNG LAI."""
    return t.sort("paid_at").with_columns(
        dev_trips_next_1h=pl.len().rolling("paid_at", period="1h", offset="0s", closed="right").over("device_id")
    )

def add_hour_bucket(t: pl.DataFrame) -> pl.DataFrame:
    """Rò rỉ 1b: gom theo giờ lịch rồi join ngược. Mỗi chuyến nhận tổng của CẢ giờ."""
    t = t.sort("paid_at")  # group_by_dynamic đòi dữ liệu đã sắp theo cột chỉ mục
    per_hour = t.group_by_dynamic("paid_at", every="1h", group_by="device_id").agg(dev_trips_this_hour=pl.len())
    ...
```

**Kiểu 2: tổng hợp cả bảng.** `n_unique().over("device_id")` không có cửa sổ thời gian đếm trên cả 60 ngày, gồm những chuyến chưa xảy ra: `dev_customers_total` và `pair_trips_total`. **Kiểu 3: nhãn biết trễ.** «Số chuyến gian lận trước đó trên cùng thiết bị» là đặc trưng rất tự nhiên, nhưng nếu đếm theo nhãn cuối cùng thì ta dùng thứ mà lúc chấm chưa ai biết (bài 54). Hàm có cờ `honest`; khác biệt chỉ nằm ở thời điểm nhãn «đến»:

```python
arrival = pl.col("label_known_at") if honest else pl.col("paid_at")
arrivals = (
    t.select("trip_id", "device_id", "paid_at").join(labels.filter(pl.col("is_fraud")), on="trip_id")
    .select("device_id", ts=arrival).sort("ts")
    .with_columns(dev_fraud_before=pl.int_range(1, pl.len() + 1).over("device_id"))
)
# join_asof strategy="backward", allow_exact_matches=False: chỉ nhãn đã đến TRƯỚC t, không bao giờ tính chính dòng này
```

Một cách phát hiện không phụ thuộc metric là nhìn **bất biến**: đặc trưng của một chuyến tính trên thông tin đến hết `paid_at` của nó phải bằng đặc trưng tính trên toàn bộ dữ liệu. Phép thử cắt cụt tính lại từng chuyến mẫu từ các event có `occurred_at <= paid_at` của chuyến đó. Giới hạn: chỉ cắt event thì mù với rò rỉ nhãn, vì nhãn nằm ngoài event; phải cắt cả nhãn theo `label_known_at`. Lab dùng 40 chuyến mà thiết bị còn chuyến sau trong một giờ, và 40 chuyến thuộc thiết bị đã có gian lận trước đó cho phần nhãn:

```text
40 chuyến mà thiết bị còn chuyến sau trong vòng 1 giờ; mỗi chuyến tính lại từ event có occurred_at <= paid_at của nó
  30 đặc trưng sạch khác kết quả tính trên toàn bộ: không đặc trưng nào
  đặc trưng nhìn tương lai khác kết quả: dev_trips_next_1h 40/40, dev_trips_this_hour 23/40, dev_customers_total 2/40, pair_trips_total 12/40
  dev_fraud_naive, trên 40 chuyến thuộc thiết bị đã có gian lận trước đó: cắt event, giữ nguyên nhãn: khác 0/40; cắt cả nhãn theo label_known_at: khác 33/40

```

Cả 30 đặc trưng sạch không đổi ở cả 40 chuyến; các đặc trưng nhìn trước đổi giá trị: cửa sổ nhìn trước ở 40/40 và 23/40 chuyến, tổng hợp cả bảng ở 2/40 và 12/40 (mẫu được chọn theo cửa sổ nên kiểu 2 chỉ lộ ở một phần). Đó cũng là đáp án của câu hỏi đầu bài. Với kiểu 3, phép thử cho thấy giới hạn của nó: chỉ cắt event thì `dev_fraud_naive` không đổi ở chuyến nào (0/40), vì nhãn nằm ngoài event; phải cắt cả nhãn theo `label_known_at` mới thấy nó khác ở 33/40 chuyến.

Rò rỉ làm số đo đẹp hơn thực tế đến đâu? Lab huấn luyện `HistGradientBoostingClassifier` mặc định (tối đa 150 vòng; dừng sớm dùng 10 % train tách ngẫu nhiên, mô hình sạch dừng ở vòng 84, `n_iter_`) trên train (trước ngày 30) và chấm PR-AUC trên valid (từ ngày 40, như bài 54) với hai bộ giá trị: **offline** (đặc trưng tính cùng cách lúc huấn luyện, tức là có rò rỉ) và **phục vụ** (thứ hệ thống thật tính được lúc đó: `next_1h` bằng 0, đếm trong giờ chỉ đến hiện tại, tổng đếm dồn, lịch sử gian lận chỉ gồm nhãn đã đến):

```text
HistGradientBoosting mặc định: tối đa 150 vòng, mô hình sạch dừng sớm ở vòng 84 (n_iter_); dừng sớm dùng 10 % train tách ngẫu nhiên
mô hình (HistGradientBoosting)                      PR-AUC offline  PR-AUC phục vụ
đặc trưng sạch (30)                                          0.886           0.886
+ cửa sổ nhìn trước (next_1h, this_hour)                     0.888           0.879
+ tổng hợp cả bảng theo khóa (2 cột)                         0.924           0.890
+ lịch sử gian lận thiết bị, nhãn tính ngay                  0.940           0.835
+ lịch sử gian lận, chỉ nhãn đã biết (cả hai nơi)            0.914           0.914
độ nhiễu của chính mô hình sạch khi chỉ đổi random_state (0 đến 4): PR-AUC từ 0.879 đến 0.887
mô hình sạch: ROC-AUC 0.995
```

Chuyến gian lận ở valid có lịch sử gian lận trên thiết bị theo hai cách đếm, và bảng 2×2 đổi từng biến một (cùng HistGradientBoosting và hồi quy logistic chuẩn hóa, cùng train và valid):

```text
chuyến gian lận ở valid có dev_fraud_naive > 0: 62.7%; có dev_fraud_known > 0: 42.8%
PR-AUC trên valid:                          13 đặc trưng theo từng chuyến  30 đặc trưng
                          hồi quy logistic             0.508                    0.778
                          HistGradientBoosting         0.603                    0.886
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Năm mô hình, mỗi mô hình hai thanh PR-AUC, offline và phục vụ. Đặc trưng sạch 0,886 và 0,886; thêm cửa sổ nhìn trước 0,888 và 0,879; thêm tổng hợp cả bảng 0,924 và 0,890; thêm lịch sử gian lận tính nhãn ngay 0,940 và 0,835; thêm lịch sử chỉ nhãn đã biết 0,914 và 0,914."><rect x="360" y="14" width="14" height="14" class="f-phys-soft s-phys"/><text x="382" y="26" class="t-small">offline: như lúc huấn luyện</text><rect x="620" y="14" width="14" height="14" class="f-warn-soft s-warn"/><text x="642" y="26" class="t-small">phục vụ: chỉ thấy quá khứ</text><text x="20" y="84" class="t-small">đặc trưng sạch (30)</text><rect x="360" y="64" width="425" height="18" rx="3" class="f-phys-soft s-phys"/><text x="793" y="78" class="t-small">0,886</text><rect x="360" y="88" width="425" height="18" rx="3" class="f-warn-soft s-warn"/><text x="793" y="102" class="t-small">0,886</text><text x="20" y="146" class="t-small">+ cửa sổ nhìn trước</text><rect x="360" y="126" width="426" height="18" rx="3" class="f-phys-soft s-phys"/><text x="794" y="140" class="t-small">0,888</text><rect x="360" y="150" width="422" height="18" rx="3" class="f-warn-soft s-warn"/><text x="790" y="164" class="t-small">0,879</text><text x="20" y="208" class="t-small">+ tổng hợp cả bảng theo khóa</text><rect x="360" y="188" width="444" height="18" rx="3" class="f-phys-soft s-phys"/><text x="812" y="202" class="t-small">0,924</text><rect x="360" y="212" width="427" height="18" rx="3" class="f-warn-soft s-warn"/><text x="795" y="226" class="t-small">0,89</text><text x="20" y="270" class="t-small">+ lịch sử gian lận, nhãn tính ngay</text><rect x="360" y="250" width="451" height="18" rx="3" class="f-phys-soft s-phys"/><text x="819" y="264" class="t-small">0,94</text><rect x="360" y="274" width="401" height="18" rx="3" class="f-warn-soft s-warn"/><text x="769" y="288" class="t-small">0,835</text><text x="20" y="332" class="t-small">+ lịch sử gian lận, nhãn đã biết</text><rect x="360" y="312" width="439" height="18" rx="3" class="f-phys-soft s-phys"/><text x="807" y="326" class="t-small">0,914</text><rect x="360" y="336" width="439" height="18" rx="3" class="f-warn-soft s-warn"/><text x="807" y="350" class="t-small">0,914</text></svg>
</div>
<figcaption>Hình 2. PR-AUC trên valid với và không có rò rỉ. Thanh dưới là số hệ thống phục vụ thật sẽ đạt; thanh trên là số offline hứa.</figcaption>
</figure>

Có bốn điều đọc ra được. **Một**, đặc trưng sạch cho PR-AUC 0,886, so với 0,537 của bộ chấm bài 54. Bảng 2×2 tách hai nguồn: từ 13 đặc trưng theo từng chuyến lên đủ 30, PR-AUC tăng 0,603 lên 0,886 (cây tăng cường) và 0,508 lên 0,778 (hồi quy logistic); đổi mô hình chỉ thêm 0,095 (13 đặc trưng) và 0,108 (30 đặc trưng). Đặc trưng nhiều chuyến góp phần lớn hơn, nhưng mô hình không vô can. **Hai**, cửa sổ nhìn trước gần như vô hình trong metric: 0,888 offline, 0,879 phục vụ, và 0,879 nằm gần nhiễu của chính mô hình sạch khi đổi `random_state` (0,879 đến 0,887). Một PR-AUC không đẹp bất thường không chứng minh pipeline sạch; phép thử cắt cụt mới thấy. **Ba**, tổng hợp cả bảng cho khoảng chênh ảo 0,034 (0,924 và 0,890), nhưng bản phục vụ, đếm dồn trung thực, đạt 0,890, nhỉnh hơn mô hình sạch: đếm dồn có ích, chỉ khoảng chênh offline là ảo. **Bốn**, lịch sử gian lận tính nhãn ngay là tệ nhất: offline 0,940, phục vụ 0,835, thấp hơn mô hình không dùng nó. Không phải vì cột trống: 62,7 % chuyến gian lận ở valid có `dev_fraud_naive > 0`, nhưng chỉ 42,8 % có `dev_fraud_known > 0`; mô hình học trên bản đầy hơn rồi gặp bản nghèo hơn. Dùng cùng `dev_fraud_known` ở huấn luyện và phục vụ cho 0,914 ở cả hai nơi, cao hơn mô hình sạch. Ở top 1 % điểm cao nhất của valid, mô hình sạch bắt 69,9 % `ghost_trip`, 56,0 % `promo_abuse`, 49,5 % `gps_spoof` và 36,1 % `collusion`; mảnh còn thiếu là đồ thị (bài 58). Mọi số trên đo ở một seed (42).

## Cùng đặc trưng ở hai nơi: online từ envelope Kafka

Huấn luyện chạy theo lô trên bảng, còn phục vụ nhận từng event. Hai đường code cho cùng một đặc trưng là nơi sinh **train/serve skew**: hai bản tính khác nhau ở những chuyến hiếm mà ta không để ý. Cách phòng là viết đường online theo đúng định nghĩa rồi kiểm bằng so sánh, trên cả 30 đặc trưng. `online.py` giữ cho mỗi khóa một danh sách thời điểm sắp xếp, chèn đúng chỗ bằng `bisect`, và chỉ ghi chuyến vào trạng thái sau khi đã đọc xong đặc trưng:

```python
def count_before(self, t: int, width: int) -> int:
    """Số mục trong [t - width, t)."""
    return bisect_left(self.ts, t) - bisect_left(self.ts, t - width)

# trong OnlineFeatures._close, thứ tự là điều quan trọng:
f = {"dev_trips_1h": w["dev"].count_before(t, US["1h"]), ...}   # 1. đọc cửa sổ [t - 1h, t)
w["dev_cust"].add(t, cust)                                      # 2. khách hiện tại tự được tính vào số khách khác nhau
f["dev_customers_24h"] = w["dev_cust"].distinct_upto(t, US["24h"])
for name in ("dev", "cust", "drv", "pair"):                    # 3. ghi chuyến SAU khi đã đọc xong
    w[name].add(t)
```

Mỗi envelope đi vào `on_envelope`; khi một `trip_id` đủ ba event, chuyến «đóng» và bộ tính trả đặc trưng. Thử trên event của 10 ngày đầu, theo thứ tự thời gian, rồi so với bản polars tính trên toàn bộ dữ liệu:

```text
[5] Tính online từ envelope (event của 10 ngày đầu)
100,013 envelope -> 33,331 chuyến đã đủ ba event, 20 chuyến còn thiếu event; 0.80 giây (124,282 envelope/giây)
so với tính theo lô trên toàn bộ dữ liệu: 0 chuyến lệch (trên 33,331, 30 đặc trưng, đủ cả 30 đặc trưng của features.py)
```

Không chuyến nào lệch ở cả 30 đặc trưng, gồm `cust_promo_share_30d` và hai cờ 120 km/h; 20 chuyến còn thiếu event là những chuyến bị cắt ở biên 10 ngày. Số envelope mỗi giây chỉ để biết cỡ, vì máy đang bận (load average 351,99 ở bước Kafka ngay sau đó). Bây giờ cho event đi qua Kafka thật: ba topic `lab55.booking.created.v1`, `lab55.trip.completed.v1`, `lab55.payment.captured.v1`, mỗi topic một partition, một consumer đọc cả ba. Kafka chỉ bảo đảm thứ tự **trong một partition**; ba topic là ba nhật ký độc lập, nên thứ tự consumer nhìn thấy giữa chúng là thứ tự giao bản ghi, không phải thứ tự thời gian:

```text
Qua Kafka thật (3 topic, mỗi topic 1 partition, một consumer đọc cả ba):
gửi 100,013 event trong 44.8 giây, đọc lại 100,013 bản ghi trong 6.6 giây; load average: 351.99 378.25 392.53
bản ghi theo topic: {'lab55.booking.created.v1': 33351, 'lab55.payment.captured.v1': 33331, 'lab55.trip.completed.v1': 33331}
32,503/33,331 chuyến có PaymentCaptured được đọc TRƯỚC TripCompleted của chính nó
xử lý đúng THỨ TỰ ĐỌC của lần chạy này: 6,717/33,330 cặp chuyến đóng liên tiếp bị đảo so với paid_at; 18 chuyến có đặc trưng lệch ({'none': 18}); theo đặc trưng:
  dev_trips_1h              8 chuyến lệch
  dev_trips_24h            10 chuyến lệch
  cust_trips_1h             7 chuyến lệch
  cust_trips_24h            9 chuyến lệch
  dev_customers_24h         1 chuyến lệch
  dev_customers_7d          1 chuyến lệch
  cust_prior_trips         16 chuyến lệch
  cust_promo_share_30d      4 chuyến lệch
phát lại xen kẽ cố định (lấy N bản ghi của mỗi topic theo vòng), số chuyến có đặc trưng lệch so với bản theo lô:
  lô      10:    99 chuyến lệch
  lô      50:    55 chuyến lệch
  lô     500:    10 chuyến lệch
  lô   2,000:     0 chuyến lệch
  lô 100,000:     0 chuyến lệch
sắp lại theo occurred_at rồi xử lý: 0 chuyến lệch (33,331 chuyến)
```

Kafka giao đủ 100.013 bản ghi, nhưng 32.503 trên 33.331 chuyến bị đọc `PaymentCaptured` trước `TripCompleted`, và 6.717 cặp chuyến đóng liên tiếp bị đảo thứ tự thanh toán. Xử lý đúng thứ tự đọc lần này làm 18 chuyến có đặc trưng khác bản theo lô, cả 18 là chuyến sạch; con số đó không đại diện, vì thứ tự đọc đổi theo từng lần chạy và cách consumer chia lô. Để đo được, lab phát lại xen kẽ cố định, lấy N bản ghi của mỗi topic theo vòng: lô 10 cho 99 chuyến lệch, lô 50 cho 55, lô 500 cho 10, lô 2.000 trở lên cho 0. Không có con số «đúng» chung; verify chỉ đòi lô nhỏ cho lệch lớn hơn 0. Sắp lại theo `occurred_at` rồi mới xử lý thì không còn chuyến nào lệch. Đó là «watermark vô hạn»: đọc hết rồi sắp, điều luồng thật không làm được. Production chọn một độ trễ cho phép (allowed lateness), đợi event đến muộn trong ngần ấy giây rồi chốt, và quyết định trước điều gì xảy ra với event đến sau.

## Chạy và kiểm chứng

Cần uv, Python 3.14 và Kafka trong hạ tầng dùng chung (`localhost:9094`, ghi đè bằng `TOINOI_KAFKA_BOOTSTRAP`); lần đầu cần mạng để tải polars, scikit-learn, aiokafka. Lab tạo ba topic `lab55.*`, xóa khi chạy xong, và không tạo consumer group. Từ thư mục gốc repository:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/55-feature-engineering-event examples/toi-noi-python/55-feature-engineering-event/main.py
uv run --project examples/toi-noi-python/55-feature-engineering-event examples/toi-noi-python/55-feature-engineering-event/main.py --verify
```

Bước cuối ghi đặc trưng ra Parquet và kiểm đọc lại (cột `trip_id`, `paid_at` và 30 đặc trưng, không có nhãn):

```text
[6] Lưu feature Parquet
out/features.parquet: 200,000 dòng x 32 cột, 12.2 MiB; đọc lại giống hệt: True

Kiểm chứng: OK 40/40
```

`--verify` in `OK 40/40`: đặc trưng sạch bất biến khi cắt event, đặc trưng nhìn trước thì không (và chỉ cắt event thì mù với nhãn), `add_features` từ chối bảng chưa sắp, online theo thứ tự thời gian khớp bản polars ở cả 30 đặc trưng, thứ tự đọc nhiều topic làm lệch, haversine của polars khớp numpy, mô hình có rò rỉ đẹp offline hơn phục vụ. Nó không kiểm số tuyệt đối hay tốc độ, và đạt ở seed 1 và 2 của bộ sinh.

| Hiện tượng | Kiểm tra trước |
|---|---|
| PR-AUC offline đẹp, chạy thật tụt | Chạy phép thử cắt cụt; tìm đặc trưng gom cả bảng hoặc dùng nhãn cuối cùng |
| Đặc trưng online khác bản theo lô ở vài chuyến | Thứ tự event giữa các topic: sắp theo `occurred_at` hoặc dùng watermark |
| `dev_trips_1h` của chuyến đầu tiên khác 0 | `closed="left"` có bị đổi thành `"right"` hoặc `"both"` không |
| Cửa sổ trả số lạ, không có lỗi nào | `rolling().over()` không báo lỗi khi bảng chưa sắp theo cột chỉ mục; `join` không giữ thứ tự, nên `sort("paid_at")` lại |
| Tốc độ hàng nghìn km/h giữa hai chuyến sát nhau | Chốt khoảng cách thời gian tối thiểu (lab dùng 60 giây) trước khi chia |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Dữ liệu là tổng hợp (200.000 chuyến trong 60 ngày), nên số PR-AUC chỉ nói về bộ này, đo ở một seed. Hai tạo tác của bộ sinh ảnh hưởng mọi con số: tỉ lệ chuyến sạch dùng <code>NEWUSER50</code> giảm từ 7,8 % xuống 0,9 % theo thời gian (bài 54), và tài khoản gian lận không dùng mã nào ngoài <code>NEWUSER50</code> của <code>promo_abuse</code>, nên <code>cust_promo_share_30d &gt; 0</code> tố «sạch». Lab chỉ đếm chuyến đã thu tiền; hệ thống thật còn phải xử lý chuyến đang chạy. Bộ tính online giữ trạng thái trong danh sách Python của một tiến trình: đủ để so với bản theo lô, không đủ cho nhiều worker hay khởi động lại; trạng thái dùng chung trên Redis là việc của bài 59. Kafka trong lab là một broker, ba topic một partition, một consumer, đọc hết rồi sắp lại; production dùng key và partition theo khách hay thiết bị, watermark và chính sách cho event đến muộn. 30 đặc trưng chọn tay, mô hình dùng siêu tham số mặc định. Đã chạy với Python 3.14.7, polars 2.1.0, numpy 2.5.4, scikit-learn 1.9.1, aiokafka 0.15.0 và Kafka 3.9 của hạ tầng dùng chung.</p>
</div>

**Bài tập:** (1) Đổi `closed="left"` thành `closed="right"` trong hàm `count`. `dev_trips_1h` của chuyến đầu tiên của mỗi thiết bị thành bao nhiêu, và bản online (đọc cửa sổ rồi mới ghi chuyến) còn khớp bản theo lô không? (2) Tài xế kết thúc một chuyến ở Hà Nội lúc 10:00 và bắt đầu chuyến kế ở TP.HCM lúc 18:00. Cờ `impossible_jump` bật hay không? Nếu chuyến kế bắt đầu lúc 20:00 thì sao? Dùng khoảng cách 1.144 km của bài.

<details><summary>Xem lời giải</summary><p>(1) Cửa sổ <code>(t - 1h, t]</code> gồm cả dòng hiện tại, nên mọi đếm tăng thêm 1 và chuyến đầu tiên của mỗi thiết bị có <code>dev_trips_1h = 1</code> thay vì 0 (lab có ca kiểm cho giá trị 0). Bản online đọc cửa sổ trước khi ghi chuyến nên vẫn trả 0, tức là lệch 1 ở mọi chuyến: một skew hệ thống, rất dễ bỏ sót vì cả hai bản đều «trông hợp lý». (2) Với 8 giờ, tốc độ là 1.144 / 8 = <b>143 km/h</b>, lớn hơn 120 nên cờ bật. Với 10 giờ là 1.144 / 10 = <b>114,4 km/h</b>, dưới ngưỡng nên cờ tắt: đường chim bay là chặn dưới của quãng đường thật, nên cờ chỉ bắt được kiểu «bay» rõ ràng và không bao giờ chứng minh ai đó thật sự ở đó.</p></details>

Bài trước: [bài 54](/articles/python-fraud-toi-noi-bai-toan-nhan-du-lieu-metric). Bài tiếp: [bài 56](/articles/python-rule-engine-scoring-strategy-explainability).

## Tài liệu gốc

- [Polars: Expr.rolling](https://docs.pola.rs/api/python/stable/reference/expressions/api/polars.Expr.rolling.html).
- [Polars: DataFrame.group_by_dynamic](https://docs.pola.rs/api/python/stable/reference/dataframe/api/polars.DataFrame.group_by_dynamic.html).
- [Polars: DataFrame.join_asof](https://docs.pola.rs/api/python/stable/reference/dataframe/api/polars.DataFrame.join_asof.html).
- [scikit-learn: HistGradientBoostingClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.HistGradientBoostingClassifier.html).
- [aiokafka: documentation](https://aiokafka.readthedocs.io/en/stable/).
- [Python: bisect](https://docs.python.org/3.14/library/bisect.html).
