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

## Kiến trúc đích cho các backend service chính

Các backend service chính của Tới Nơi thống nhất dùng **DDD, Clean Architecture, CQRS, Mediator, Repository và Unit of Work**. Đây là định hướng triển khai xuyên suốt các loạt bài. Ví dụ console hoặc SQL độc lập vẫn tập trung vào cơ chế đang học; khi phát triển thành service hoàn chỉnh, chúng sẽ được đặt vào kiến trúc này.

| Thành phần | Trách nhiệm trong Tới Nơi |
| --- | --- |
| DDD | Xác định bounded context, ngôn ngữ nghiệp vụ, aggregate và invariant; ranh giới service được quyết định từ nghiệp vụ |
| Clean Architecture | Tách Domain, Application, Infrastructure và API; phụ thuộc mã nguồn hướng vào Domain/Application |
| CQRS | Tách command thay đổi trạng thái với query đọc dữ liệu, mỗi use case có hợp đồng và handler rõ ràng |
| Mediator | Điều phối command/query tới handler trong cùng tiến trình; pipeline xử lý các mối quan tâm chung phù hợp |
| Repository | Cung cấp truy cập aggregate ở phía ghi theo nhu cầu domain; implementation lưu trữ thuộc Infrastructure |
| Unit of Work | Quản lý commit thay đổi trong một transaction cục bộ của service/use case |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 900 260" role="img" aria-label="Luồng ghi: API gửi command qua Mediator tới Application handler, handler dùng Domain aggregate và Repository, Unit of Work commit transaction cục bộ. Infrastructure hiện thực các abstraction.">
<rect x="20" y="35" width="170" height="80" rx="10" class="f-phys-soft s-phys"/><text x="35" y="65" class="t-b">API</text><text x="35" y="92" class="t-small">Command / Query</text>
<text x="200" y="82" class="t-b">→</text>
<rect x="235" y="35" width="170" height="80" rx="10" class="f-phys-soft s-phys"/><text x="250" y="65" class="t-b">Mediator</text><text x="250" y="92" class="t-small">Pipeline</text>
<text x="415" y="82" class="t-b">→</text>
<rect x="450" y="35" width="200" height="80" rx="10" class="f-phys-soft s-phys"/><text x="465" y="65" class="t-b">Application</text><text x="465" y="92" class="t-small">Use-case handler</text>
<text x="660" y="82" class="t-b">→</text>
<rect x="695" y="35" width="185" height="80" rx="10" class="f-phys-soft s-phys"/><text x="710" y="65" class="t-b">Domain</text><text x="710" y="92" class="t-small">Aggregate / Rules</text>
<rect x="235" y="155" width="645" height="75" rx="10" class="f-bg s-line"/><text x="255" y="184" class="t-b">Repository + Unit of Work → lưu trữ cục bộ</text><text x="255" y="210" class="t-small">Infrastructure hiện thực abstraction; không đặt luật nghiệp vụ ở đây.</text>
</svg>
</div>
<figcaption>Luồng điều phối của một use case. Mũi tên trên biểu diễn luồng xử lý; chiều phụ thuộc mã nguồn được quản lý theo Clean Architecture.</figcaption>
</figure>

CQRS không bắt buộc tách hai database hoặc dùng event sourcing. Query có thể projection trực tiếp ra DTO qua abstraction đọc riêng, không cần dựng lại aggregate chỉ để hiển thị. Repository phía ghi được thiết kế theo aggregate, tránh một generic CRUD repository che mất ý nghĩa nghiệp vụ. Với EF Core, DbContext đã có cơ chế tracking và transaction; abstraction Repository/Unit of Work cần làm rõ ranh giới nghiệp vụ và commit, không chỉ sao chép mọi API của DbContext.

Mediator ở đây là pattern điều phối trong tiến trình, không thay message broker giữa các service; lựa chọn thư viện sẽ được chốt và giải thích trong bài triển khai. Unit of Work chỉ bao trùm transaction cục bộ, không giữ transaction xuyên nhiều service. Các luồng liên service sẽ bổ sung cơ chế phù hợp như Outbox, idempotency và bù trừ khi bài toán cần.

Những Design Pattern khác được bổ sung theo từng tình huống: Strategy cho chính sách tính cước, State cho chuyển trạng thái chuyến, Adapter cho cổng thanh toán… Mỗi bài cần chỉ ra vấn đề, vị trí trong kiến trúc, lợi ích, cái giá và kiểm thử tương ứng trước khi chọn pattern.

## Đa ngôn ngữ theo trách nhiệm service

Tới Nơi dùng chung nghiệp vụ để nối các loạt C#, Python, Rust, Go và database. Bảng dưới là **định hướng thiết kế của dự án mẫu**, không phải kết luận một ngôn ngữ độc quyền một loại bài toán. Các backend chính tiếp tục giữ kiến trúc đích đã nêu; cách hiện thực abstraction cần phù hợp hệ sinh thái của từng ngôn ngữ.

| Ngôn ngữ | Service dự kiến | Điều bài viết cần chứng minh |
| --- | --- | --- |
| C# / .NET 10 | Đặt chuyến, quản lý đơn, điều phối nghiệp vụ chính | Aggregate, invariant, command/query, transaction và kiểm thử |
| Python | Fraud scoring, phân tích/dự đoán, OCR giấy tờ định danh | Pipeline dữ liệu, đánh giá mô hình, độ tin cậy kết quả, thời gian xử lý |
| Rust | Sổ cái, đối soát và các thành phần nhạy với độ trễ/tài nguyên | Tính đúng của tiền, idempotency, hiệu năng đo được, kiểm soát bộ nhớ |
| Go (Golang) | Tiếp nhận vị trí tài xế, fan-out thông báo, worker nhiều tác vụ đồng thời | Goroutine, giới hạn concurrency, backpressure, cancellation và race safety |

Rust không tự bảo đảm đúng nghiệp vụ tài chính; sổ cái cần invariant, quy tắc làm tròn và đối soát. Go được chọn cho mô hình concurrency của service dự kiến, không có nghĩa C# hoặc Rust thiếu khả năng xử lý đồng thời. Python OCR dùng dữ liệu giấy tờ tổng hợp trong bài minh hoạ; kết quả OCR và điểm fraud là đầu vào cho quy trình nghiệp vụ, không tự động đồng nhất với quyết định xác minh hay kết luận gian lận.

Service sở hữu dữ liệu của mình. Giao tiếp qua hợp đồng API/event có version, timeout, idempotency và trace xuyên ngôn ngữ; không để service khác ghi trực tiếp vào bảng nghiệp vụ nội bộ. Không mở một Unit of Work xuyên nhiều service hoặc nhiều loại database.

## Hành trình PostgreSQL, MySQL và MongoDB

**Giai đoạn đầu dùng PostgreSQL làm database chủ đạo.** Các bảng SQL và quy mô bên dưới là mô hình tham chiếu cho chặng này, không ngụ ý mọi service dùng chung một database trong kiến trúc đích.

Khi workload tăng, loạt bài sẽ tái hiện áp lực ghi, MVCC và vacuum, đo rồi quyết định cách tiến hoá. MySQL và MongoDB có mặt trong lộ trình để học so sánh, tách dữ liệu và migration theo từng service; không đặt quy luật “đạt một số giao dịch đồng thời thì bắt buộc rời PostgreSQL”.

| Chặng | Nội dung chia sẻ | Bằng chứng để ra quyết định |
| --- | --- | --- |
| PostgreSQL nền tảng | Heap, index, MVCC, transaction, autovacuum | Query plan, latency p95/p99, throughput, lock wait, WAL, dead tuples và bloat |
| PostgreSQL dưới tải | Ghi trạng thái chuyến, transaction dài, vacuum không theo kịp | Tải tái lập, cấu hình/hardware, tốc độ sinh và dọn phiên bản cũ; thử tối ưu transaction, index, vacuum và phân vùng phù hợp |
| MySQL / InnoDB | Clustered index, undo/purge, locking và replication | So sánh cùng workload và yêu cầu durability; đánh giá bottleneck và chi phí vận hành |
| MongoDB | Mô hình document cho kết quả OCR, hồ sơ phân tích hoặc read model phù hợp | Access pattern, kích thước document, index, consistency và ranh giới transaction |
| Migration có kiểm soát | Chuyển một tập dữ liệu/service khi kết quả đo đủ thuyết phục | Backfill, CDC/event, đối chiếu, cutover, rollback và quyền sở hữu dữ liệu |

`VACUUM` là cơ chế bảo trì phiên bản hàng của PostgreSQL; vacuum thông thường có thể chạy cùng hoạt động đọc/ghi, dù tiêu tốn I/O. Nó khác `VACUUM FULL`, vốn cần khoá độc quyền. Vì vậy phải mô tả đúng loại vacuum và điều kiện gây chậm, không gộp chúng thành “vacuum khoá toàn bộ hệ thống”. Xem [tài liệu PostgreSQL về routine vacuuming](https://www.postgresql.org/docs/18/routine-vacuuming.html).

Bài [Uber Engineering ngày 26/07/2016](https://www.uber.com/us/en/blog/postgres-to-mysql-migration/) là case study lịch sử, chủ yếu dựa trên PostgreSQL 9.2: Uber nêu write amplification, replication và các khó khăn vận hành, rồi dùng Schemaless xây trên MySQL cho nhiều trường hợp. Đây không phải bằng chứng rằng PostgreSQL hiện đại luôn phải nhường chỗ khi concurrency tăng, cũng không phải case chuyển sang MongoDB. Khi tái hiện cần ghi phiên bản và tách kết luận của Uber khỏi số đo của Tới Nơi.

MySQL cũng phải dọn lịch sử MVCC qua [InnoDB purge](https://dev.mysql.com/doc/refman/8.4/en/innodb-purge-configuration.html). MongoDB được chọn theo mô hình dữ liệu và truy cập, không phải thuốc chữa chung cho vacuum. Mọi bài so sánh phải nêu cùng tiêu chí tính đúng, durability, tải và ngân sách tài nguyên trước khi công bố kết quả.

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
| Tạo chuyến cho ba dịch vụ, cộng phụ phí vào cước, đi qua 8 trạng thái, ghép tài xế… nên viết thế nào? | [23 design pattern theo ba nhóm](/articles/design-pattern-la-gi) | Design Pattern |
| Tách dịch vụ điều phối tài xế khỏi phần còn lại? | Event-driven, monolith hay microservices | Architecture |
| Theo dõi vị trí 25.000 tài xế theo thời gian thực | Hàng đợi, cache, xử lý đồng thời | System Design, Lập trình |

Bảng này sẽ được cập nhật mỗi khi có bài mới.
