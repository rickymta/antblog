---
slug: du-an-mau-toi-noi
title: "Dự án mẫu Tới Nơi: hệ thống đặt xe và giao hàng dùng cho cả loạt bài"
summary: "Hồ sơ của hệ thống hư cấu dùng xuyên suốt blog: nghiệp vụ, quy mô tải, vòng đời một chuyến, các bảng chính và bộ dữ liệu mẫu để tự chạy trên PostgreSQL."
category: system-design
level: beginner
tags: [du-an-mau, postgresql, system-design, workload]
language: vi
---

Mỗi bài trong blog này giải thích một cơ chế kỹ thuật — cách database cất dữ liệu, cách index được dùng, vì sao giao dịch bị khoá, khi nào nên tách dịch vụ… Để các bài nối được với nhau, tất cả dùng chung **một hệ thống mẫu**: **Tới Nơi**, một ứng dụng đặt xe và giao hàng hư cấu. Bài này là hồ sơ của hệ thống đó. Đọc một lần, các bài sau chỉ cần nói "bảng `trips` của Tới Nơi" là bạn biết nó to cỡ nào và bị ghi ra sao.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Mọi thứ ở đây là hư cấu</span>
<p>Tên công ty, số liệu tải và dữ liệu mẫu đều do blog tự đặt ra để minh hoạ. Số liệu được chọn cho nhất quán với nhau (có công thức đi kèm) chứ không lấy từ một công ty thật nào.</p>
</div>

## Tới Nơi làm gì

Tới Nơi chạy ở 6 thành phố, gộp ba dịch vụ dùng chung một đội tài xế:

- **Đặt xe** (`ride`): xe máy hoặc ô tô chở khách từ điểm đón tới điểm đến.
- **Giao đồ ăn** (`food`): tài xế tới quán lấy món rồi giao cho khách.
- **Giao bưu kiện** (`parcel`): lấy hàng ở một địa chỉ, giao tới địa chỉ khác.

Vì ba dịch vụ dùng chung tài xế, mọi yêu cầu đều quy về một khái niệm duy nhất: **chuyến** (bảng `trips`). Một chuyến luôn có khách, thành phố, giá; có thể có tài xế (sau khi có người nhận) và quán (với đơn đồ ăn).

## Quy mô tải

Đây là con số các bài sau sẽ dùng khi nói "nặng" hay "nhẹ":

| Đại lượng | Giá trị | Cách tính |
|---|---|---|
| Thành phố | 6 | |
| Tài xế đăng ký / trực tuyến lúc cao điểm | 60.000 / 25.000 | |
| Khách hoạt động mỗi tháng | 4 triệu | |
| Quán và cửa hàng | 40.000 | |
| Chuyến mỗi ngày | 2 triệu | 0,8 tr đặt xe + 0,9 tr đồ ăn + 0,3 tr bưu kiện |
| Chuyến tạo mới lúc cao điểm | ≈ 70 chuyến/giây | 2 khung 11:00–13:00 và 17:00–19:00 chiếm 50% chuyến ngày: 1 triệu ÷ (4 × 3.600 giây) |
| Lượt sửa bảng `trips` lúc cao điểm | ≈ 560 UPDATE/giây | ≈ 8 lần đổi trạng thái/giá mỗi chuyến × 70 |
| Vị trí tài xế gửi về lúc cao điểm | ≈ 6.250 dòng/giây | 25.000 tài xế × 1 lần mỗi 4 giây |
| Dòng `trips` sau một năm | ≈ 730 triệu | 2 triệu × 365 |
| Truy vấn đọc lúc cao điểm | ≈ 30.000/giây | xem báo giá, menu quán, theo dõi chuyến |

Ba con số đáng nhớ nhất: **560 UPDATE/giây** vào cùng một bảng, **6.250 dòng/giây** chỉ để ghi vị trí, và **730 triệu dòng/năm**. Gần như mọi chủ đề database trong blog đều xuất phát từ một trong ba con số này.

## Vòng đời một chuyến

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 200" role="img" aria-label="Trạng thái của một chuyến: requested, searching, accepted, arriving, picked_up, in_transit, completed; có thể bị huỷ ở searching, accepted hoặc arriving.">
<defs>
<marker id="lc-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="lc-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<text x="460" y="30" text-anchor="middle" class="t-small">mỗi mũi tên = một lần UPDATE trips + một dòng trip_events</text>
<rect x="20" y="58" width="110" height="44" rx="8" class="f-pk-soft s-pk"/><text x="75" y="77" text-anchor="middle" class="t-mono t-b">requested</text><text x="75" y="93" text-anchor="middle" class="t-small">khách đặt</text>
<rect x="150" y="58" width="110" height="44" rx="8" class="f-pk-soft s-pk"/><text x="205" y="77" text-anchor="middle" class="t-mono t-b">searching</text><text x="205" y="93" text-anchor="middle" class="t-small">tìm tài xế</text>
<rect x="280" y="58" width="110" height="44" rx="8" class="f-pk-soft s-pk"/><text x="335" y="77" text-anchor="middle" class="t-mono t-b">accepted</text><text x="335" y="93" text-anchor="middle" class="t-small">tài xế nhận</text>
<rect x="410" y="58" width="110" height="44" rx="8" class="f-pk-soft s-pk"/><text x="465" y="77" text-anchor="middle" class="t-mono t-b">arriving</text><text x="465" y="93" text-anchor="middle" class="t-small">đang tới đón</text>
<rect x="540" y="58" width="110" height="44" rx="8" class="f-pk-soft s-pk"/><text x="595" y="77" text-anchor="middle" class="t-mono t-b">picked_up</text><text x="595" y="93" text-anchor="middle" class="t-small">đã đón / lấy hàng</text>
<rect x="670" y="58" width="110" height="44" rx="8" class="f-pk-soft s-pk"/><text x="725" y="77" text-anchor="middle" class="t-mono t-b">in_transit</text><text x="725" y="93" text-anchor="middle" class="t-small">đang chở / giao</text>
<rect x="800" y="58" width="100" height="44" rx="8" class="f-phys-soft s-phys"/><text x="850" y="77" text-anchor="middle" class="t-mono t-b t-phys">completed</text><text x="850" y="93" text-anchor="middle" class="t-small">hoàn thành</text>
<line x1="130" y1="80" x2="148" y2="80" class="s-pk" stroke-width="1.6" marker-end="url(#lc-a)"/>
<line x1="260" y1="80" x2="278" y2="80" class="s-pk" stroke-width="1.6" marker-end="url(#lc-a)"/>
<line x1="390" y1="80" x2="408" y2="80" class="s-pk" stroke-width="1.6" marker-end="url(#lc-a)"/>
<line x1="520" y1="80" x2="538" y2="80" class="s-pk" stroke-width="1.6" marker-end="url(#lc-a)"/>
<line x1="650" y1="80" x2="668" y2="80" class="s-pk" stroke-width="1.6" marker-end="url(#lc-a)"/>
<line x1="780" y1="80" x2="798" y2="80" class="s-pk" stroke-width="1.6" marker-end="url(#lc-a)"/>
<rect x="320" y="146" width="160" height="44" rx="8" class="f-warn-soft s-warn"/><text x="400" y="165" text-anchor="middle" class="t-mono t-b t-warn">cancelled</text><text x="400" y="181" text-anchor="middle" class="t-small">huỷ (khách hoặc tài xế)</text>
<path d="M205 102 C 205 130, 330 130, 352 144" class="f-none s-warn dash" stroke-width="1.4" marker-end="url(#lc-w)"/>
<line x1="365" y1="102" x2="388" y2="144" class="s-warn dash" stroke-width="1.4" marker-end="url(#lc-w)"/>
<path d="M465 102 C 465 130, 445 132, 430 144" class="f-none s-warn dash" stroke-width="1.4" marker-end="url(#lc-w)"/>
</svg>
</div>
<figcaption><b>Hình 1 · Tám trạng thái của một chuyến.</b> Với 2 triệu chuyến mỗi ngày, mỗi chuyến đi qua khoảng 6–8 mũi tên ⇒ hàng chục triệu câu <code>UPDATE</code> vào cùng một bảng mỗi ngày. Đây là lý do bảng <code>trips</code> xuất hiện trong hầu hết bài database.</figcaption>
</figure>

Hai đặc điểm của vòng đời này sẽ quay lại nhiều lần:

- **Một dòng bị sửa nhiều lần trong vài chục phút**, sau đó gần như không ai sửa nữa. Dữ liệu "nóng" chỉ là phần rất nhỏ ở cuối bảng.
- **Nhiều tài xế có thể cùng bấm nhận một chuyến** trong cùng một giây. Chỉ một người được nhận — đây là bài toán khoá dòng và giao dịch.

## Các bảng chính

| Bảng | Một dòng là | Tải đặc trưng |
|---|---|---|
| `cities` | một thành phố | gần như không đổi |
| `customers` | một khách | đọc theo số điện thoại khi đăng nhập |
| `drivers` | một tài xế | đổi trạng thái `offline` / `online` / `busy` liên tục |
| `merchants` | một quán / cửa hàng | đọc rất nhiều, sửa ít |
| `trips` | một chuyến | **chèn 70/giây, sửa 560/giây lúc cao điểm**, đọc theo khách, theo tài xế, theo thành phố + thời gian |
| `trip_events` | một lần đổi trạng thái | chỉ chèn, không sửa; ~16 triệu dòng/ngày |
| `driver_locations` | một điểm GPS | chỉ chèn, 6.250 dòng/giây; hầu như chỉ đọc vài phút gần nhất |

Lược đồ lõi (PostgreSQL 17 trở lên). Các bảng khác (thanh toán, menu, đánh giá…) sẽ được thêm đúng ở bài cần đến chúng.

```sql
CREATE TABLE cities (
  id    smallint PRIMARY KEY,
  code  text NOT NULL UNIQUE,
  name  text NOT NULL
);

CREATE TABLE customers (
  id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  phone      text NOT NULL UNIQUE,
  full_name  text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE drivers (
  id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  code         integer NOT NULL UNIQUE,
  full_name    text NOT NULL,
  city_id      smallint NOT NULL REFERENCES cities (id),
  vehicle_type text NOT NULL CHECK (vehicle_type IN ('bike', 'car')),
  status       text NOT NULL DEFAULT 'offline' CHECK (status IN ('offline', 'online', 'busy')),
  created_at   timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE merchants (
  id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  city_id    smallint NOT NULL REFERENCES cities (id),
  name       text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE trips (
  id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  code         bigint GENERATED ALWAYS AS IDENTITY (START WITH 100000),
  kind         text NOT NULL CHECK (kind IN ('ride', 'food', 'parcel')),
  city_id      smallint NOT NULL REFERENCES cities (id),
  customer_id  uuid NOT NULL REFERENCES customers (id),
  driver_id    uuid REFERENCES drivers (id),
  merchant_id  uuid REFERENCES merchants (id),
  status       text NOT NULL DEFAULT 'requested' CHECK (status IN
                 ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed', 'cancelled')),
  fare         integer NOT NULL,              -- đồng
  requested_at timestamptz NOT NULL DEFAULT now(),
  accepted_at  timestamptz,
  completed_at timestamptz,
  updated_at   timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT uq_trips_code UNIQUE (code)
);
CREATE INDEX ix_trips_city_requested     ON trips (city_id, requested_at);
CREATE INDEX ix_trips_customer_requested ON trips (customer_id, requested_at DESC);
CREATE INDEX ix_trips_driver_id          ON trips (driver_id);

CREATE TABLE trip_events (
  id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  trip_id     uuid NOT NULL REFERENCES trips (id),
  status      text NOT NULL,
  occurred_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX ix_trip_events_trip ON trip_events (trip_id, occurred_at);

CREATE TABLE driver_locations (
  driver_id   uuid NOT NULL REFERENCES drivers (id),
  recorded_at timestamptz NOT NULL,
  lat         double precision NOT NULL,
  lng         double precision NOT NULL,
  PRIMARY KEY (driver_id, recorded_at)
);
```

Vài lựa chọn trong lược đồ là **cố ý chưa tối ưu**, để các bài sau có chỗ mổ xẻ:

- `trips.id` là uuid **ngẫu nhiên** (v4) — bài về heap và primary key giải thích nó ảnh hưởng gì.
- `trips` có 5 index — mỗi index là một cái giá phải trả ở mỗi lần UPDATE.
- `trip_events` và `driver_locations` chưa partition — sẽ có bài riêng khi bảng phình tới hàng trăm triệu dòng.

## Dựng dữ liệu mẫu trên máy

Bộ dữ liệu dưới đây tạo 6 thành phố, 50.000 khách, 5.000 tài xế, 2.000 quán và **200.000 chuyến** trải đều trong 30 ngày — đủ để kế hoạch truy vấn của PostgreSQL bắt đầu khác với bảng đồ chơi. Chạy sau lược đồ ở trên, trên một database trống:

```sql
INSERT INTO cities (id, code, name) VALUES
  (1, 'SG', 'TP. Hồ Chí Minh'), (2, 'HN', 'Hà Nội'), (3, 'DN', 'Đà Nẵng'),
  (4, 'HP', 'Hải Phòng'), (5, 'CT', 'Cần Thơ'), (6, 'NT', 'Nha Trang');

INSERT INTO customers (phone, full_name)
SELECT '09' || lpad(g::text, 8, '0'), 'Khách ' || g
FROM generate_series(1, 50000) AS g;

INSERT INTO drivers (code, full_name, city_id, vehicle_type, status)
SELECT 8000 + g, 'Tài xế ' || g, 1 + g % 6,
       CASE WHEN g % 4 = 0 THEN 'car' ELSE 'bike' END, 'online'
FROM generate_series(1, 5000) AS g;

INSERT INTO merchants (city_id, name)
SELECT 1 + g % 6, 'Quán ' || g
FROM generate_series(1, 2000) AS g;

-- Chuyến được thêm đúng thứ tự thời gian đặt ⇒ code tăng theo giờ đặt, giống hệ thống thật.
WITH c AS (SELECT array_agg(id) AS ids FROM customers),
     d AS (SELECT array_agg(id) AS ids FROM drivers),
     m AS (SELECT array_agg(id) AS ids FROM merchants),
     s AS (
       SELECT g,
              now() - interval '30 days' + (g / 200000.0) * interval '30 days' AS t,
              random() AS r
       FROM generate_series(1, 200000) AS g
     )
INSERT INTO trips (kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
SELECT k.kind,
       1 + s.g % 6,
       c.ids[1 + (random() * 49999)::int],
       CASE WHEN st.status = 'searching' THEN NULL ELSE d.ids[1 + (random() * 4999)::int] END,
       CASE WHEN k.kind = 'food' THEN m.ids[1 + (random() * 1999)::int] END,
       st.status,
       (15 + (random() * 150)::int) * 1000,
       s.t,
       CASE WHEN st.status <> 'searching' THEN s.t + interval '2 minutes' END,
       CASE WHEN st.status = 'completed' THEN s.t + interval '25 minutes' END,
       s.t
FROM s
CROSS JOIN c CROSS JOIN d CROSS JOIN m
CROSS JOIN LATERAL (SELECT CASE WHEN s.r < 0.40 THEN 'ride'
                                WHEN s.r < 0.85 THEN 'food'
                                ELSE 'parcel' END AS kind) AS k
CROSS JOIN LATERAL (SELECT CASE WHEN s.g > 199800 THEN 'searching'
                                WHEN random() < 0.06 THEN 'cancelled'
                                ELSE 'completed' END AS status) AS st
ORDER BY s.g;

ANALYZE;
```

Kết quả: khoảng 40% chuyến đặt xe, 45% giao đồ ăn, 15% bưu kiện; 6% bị huỷ; 200 chuyến cuối vẫn đang `searching` — đúng những chuyến các bài sau sẽ "nhận tài xế" để quan sát một câu UPDATE thật. Muốn tải nặng hơn, tăng `200000` (hai chỗ) và `199800`.

<div class="tkh-callout">
<span class="tkh-callout-title">Kiểm tra nhanh sau khi chạy</span>
<p><code>SELECT kind, status, count(*) FROM trips GROUP BY 1, 2 ORDER BY 1, 2;</code> — phải ra 9 nhóm, tổng 200.000 dòng.</p>
</div>

## Lộ trình các bài dựa trên Tới Nơi

| Câu hỏi của Tới Nơi | Chủ đề | Chuyên mục |
|---|---|---|
| Một dòng `trips` nằm ở đâu trên đĩa? Primary key có quyết định chỗ đó không? | Heap, `ctid`, primary key | Database |
| Vì sao 560 UPDATE/giây làm bảng phình và chậm dần? | MVCC, HOT update, VACUUM | Database |
| Hai tài xế cùng bấm nhận một chuyến thì sao? | Giao dịch, khoá dòng, `SKIP LOCKED` | Database |
| 730 triệu dòng/năm thì truy vấn tháng này có chậm không? | Partition theo thời gian | Database |
| Tính giá chuyến theo nhiều loại xe, khuyến mãi | Strategy, Chain of Responsibility | Design Pattern |
| Tách dịch vụ điều phối tài xế khỏi phần còn lại? | Event-driven, monolith hay microservices | Architecture |
| Theo dõi vị trí 25.000 tài xế theo thời gian thực | Hàng đợi, cache, xử lý đồng thời | System Design, Lập trình |

Bảng này sẽ được cập nhật mỗi khi có bài mới.
