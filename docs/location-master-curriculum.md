# Master Curriculum — Service định vị bản đồ qua Tới Nơi (Rust, Go, Python)

Bản quản lý loạt bài về **service xử lý vị trí trên bản đồ** của Tech Knowledge Hub (antblog.xyz). Loạt bài phục vụ hai phía của một chuyến: **khách đặt xe** trên ứng dụng và **tài xế** đi tìm khách rồi tới điểm trả. Câu hỏi xuyên suốt: **làm sao tổng quãng đường và thời gian của cả hai người nhỏ nhất**, thay vì tối ưu cho riêng một bên.

Gồm **3 lộ trình**, **7 phần**, **64 bài**. Mỗi phần tối đa 10 bài. Trạng thái: **bản đề xuất, chờ chủ blog duyệt** (§7).

| Lộ trình | Phần | Số bài | Slug | Kết quả mong muốn |
|---|---|---:|---|---|
| Nền tảng định vị | P01–P03 | 30 | `location-nen-tang-qua-toi-noi` | Từ tọa độ tới engine định tuyến đường bộ tự viết bằng Rust, nhận và làm sạch GPS bằng Go + Rust, có bộ mô phỏng làm thước đo |
| Tối ưu quãng đường hai phía | P04–P05 | 20 | `location-toi-uu-quang-duong-qua-toi-noi` | Trọng tâm của loạt: chọn điểm đón, điểm trả, ghép chuyến và giảm km rỗng, mọi cải thiện có số đo so với baseline |
| Dữ liệu và vận hành | P06–P07 | 14 | `location-du-lieu-van-hanh-qua-toi-noi` | Python huấn luyện ETA và dự báo nhu cầu rồi đưa vào Rust; workspace hoàn chỉnh, tải, quan sát, triển khai |

## 1. Cách dùng

1. **Người viết**: chọn bài theo bảng §4, dùng dòng «Cần chứng minh» làm đề cương, viết theo template §3.2. Code đặt trong `examples/toi-noi-location/` và phải chạy được.
2. **Người đọc**: đi theo cột «Đọc trước». Ai chỉ quan tâm tối ưu có thể đọc LOC-01, 02, 23, 26, 30 rồi vào thẳng P04.
3. **Theo dõi tiến độ**: cột «Trạng thái» (`Kế hoạch` → `Đang viết` → `Đã xuất bản`). Khi một bài xuất bản: thêm `.md` vào `api/seed/content`, thêm slug vào `api/seed/learning-paths.json`, thêm ảnh bìa vào `apps/web/src/lib/article-covers.json` rồi chạy `npm run covers:generate -w @tkh/web` (chỉ add ảnh mới, hoàn lại ảnh cũ bị sinh lại).

## 2. Phân vai ngôn ngữ

Rust **không yếu** về thuật toán: chỉ mục không gian, đồ thị đường, contraction hierarchies và ghép cặp đều chạy tốt trên Rust, lại có p99 ổn định và kiểm soát được bộ nhớ khi giữ cả bản đồ một thành phố trong RAM. Vì vậy thuật toán chạy trên đường nóng vẫn ở Rust. Go và Python được dùng ở nơi chúng có lợi thế thật, và khớp định hướng đã có trong bài `du-an-mau-toi-noi`, mục «Đa ngôn ngữ theo trách nhiệm service».

| Thành phần | Ngôn ngữ | Lý do chọn | Sở hữu gì |
|---|---|---|---|
| `geo-engine`: snap, map matching, định tuyến, ma trận chi phí, chọn điểm đón, gợi ý ghép chuyến | **Rust** | Nhạy độ trễ và tài nguyên, không GC pause, đồ thị lớn trong bộ nhớ, sai kiểu tọa độ bị chặn lúc biên dịch | Bản đồ đã biên dịch, vị trí mới nhất phục vụ truy vấn |
| `location-ingest`, `tracking-hub`: nhận GPS, fan-out vị trí cho khách | **Go** | Rất nhiều kết nối, goroutine, backpressure; mất một điểm GPS không làm lệch tiền | Kết nối, hàng đợi có giới hạn |
| Pipeline offline: ETA, dự báo nhu cầu, phân cụm điểm đón, phân tích thí nghiệm, baseline OR-Tools | **Python** | Hệ sinh thái dữ liệu và solver; không nằm trên đường nóng | Dữ liệu huấn luyện, mô hình ONNX, bảng điểm đón học được |
| `Dispatch`, `Trip`: offer, TTL, accept/reject, assignment, trạng thái chuyến | **C# (đã có)** | Nghiệp vụ và transaction vẫn là của backend chính; loạt này không viết lại | Nguồn sự thật của assignment |
| App khách, app tài xế | **Flutter (đã có)** | Dùng lại `flutter_map` của loạt Flutter; loạt này chỉ nêu hợp đồng | Trải nghiệm hiển thị |

**Ranh giới quan trọng nhất:** Rust **đề xuất**, C# **quyết định**. `geo-engine` trả danh sách điểm đón và gợi ý ghép qua gRPC; assignment chính thức vẫn do Dispatch C# persist. Không service nào đọc trực tiếp kho của service khác.

```text
Driver App ──GPS batch──▶ location-ingest (Go) ──location.updates.v1──▶ geo-engine (Rust)
                                     │                                     ▲   │ gRPC: Route, Matrix,
                                     ▼                                     │   │ SuggestPickup, SuggestMatch
Customer App ◀──WebSocket── tracking-hub (Go)                              │   ▼
Customer App ──Quote/Booking──▶ Booking, Pricing, Dispatch (C#) ───────────┘
Lịch sử GPS, chuyến ──▶ pipeline Python ──ONNX, bảng điểm đón, dự báo──▶ geo-engine
```

## 3. Quy ước

### 3.1 Chung

| Mục | Quy ước |
|---|---|
| Mã bài, mã phần | `LOC-01` … `LOC-64`; `P01` … `P07`. |
| Slug | Chữ thường ASCII, gạch nối, bắt đầu bằng `location-`, ≤ 60 ký tự, duy nhất. |
| Category | `system-design`. Tags luôn có `toi-noi`, `location`, thêm ngôn ngữ (`rust`, `go`, `python`) và chủ đề (`h3`, `map-matching`, `routing`, `dispatch`…). |
| Level | `intermediate` (P01–P03) · `advanced` (P04–P07). Loạt không dạy lại Rust từ đầu; Go có một bài nhập môn (LOC-11) vì blog chưa có loạt Go. |
| Phiên bản (kiểm 06/11/2026) | Rust stable 1.98, edition 2024 · Go 1.27 · Python ≥ 3.12 trong venv riêng (Python hệ thống trên máy viết là 3.9, không dùng). Phiên bản thư viện pin trong lockfile. |
| Thư viện Rust | `geo` · `rstar` · `h3o` · `osmpbf` · `geojson` · `polyline` · `petgraph` (đối chiếu) · `fast_paths` (đối chiếu CH) · `pathfinding` (Kuhn–Munkres) · `tract-onnx`/`ort` · `tonic` · `axum` · `rdkafka` · `redis` · `tracing` · `criterion` · `proptest`. |
| Thư viện Go, Python | Go: `coder/websocket`, `twmb/franz-go`, `redis/go-redis`. Python: `polars`, `lightgbm`, `scikit-learn`, `ortools`, `h3`, `onnxmltools`. |
| Dữ liệu bản đồ | OpenStreetMap, giấy phép **ODbL**, ghi «© OpenStreetMap contributors» ở mọi hình và lab dùng dữ liệu thật. Không commit file `.pbf`; script tải về rồi cắt theo bbox. Test và CI chạy trên **đồ thị tổng hợp nhỏ** được commit, nên không phụ thuộc mạng. |
| Dữ liệu chuyến, GPS | Toàn bộ do simulator sinh, có seed. Không có GPS người thật. Mô hình ETA và dự báo chỉ chứng minh **phương pháp**, không chứng minh độ chính xác ngoài đời. |
| Tọa độ | Newtype `LatLng`, đúng thứ tự lat rồi lng trong Rust; GeoJSON là lng rồi lat và phải chuyển ở một chỗ duy nhất. `f64` dùng được cho tọa độ (khác tiền), lưu gọn dạng `i32` E7. |
| Đơn vị | Khoảng cách: mét nguyên. Thời gian: giây. Tiền: giữ mô hình loạt Rust/C#, `Money` là số nguyên đồng. |
| Mã, sự kiện | Chuyến `TN-0001`, tài xế `DR-001`, khách `CU-001`; topic `location.updates.v1` key `driver_id`, event `DriverLocationUpdated` (kiến trúc §6, §8.2). Cước: 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt. |
| SLO tham chiếu | Kiến trúc §14.1: độ tươi vị trí p95 dưới 5–10 giây, lan truyền realtime p95 dưới 2–3 giây, Quote p95 dưới 500 ms. Ngân sách chọn điểm đón do LOC-35 đặt và đo. |

**Quy tắc biên tập bắt buộc (chặt hơn các loạt trước vì loạt này nói về tối ưu):**

1. Mọi con số «giảm x% km rỗng» phải đến từ simulator chạy lại được, ghi rõ seed, kịch bản và lệnh chạy, luôn so với baseline, luôn kèm câu «trên dữ liệu mô phỏng». Chưa chạy thì không ghi số.
2. Không gán số liệu cho công ty thật nếu không có nguồn công bố. Thuật toán dẫn bài báo gốc (Newson & Krumm 2009, Geisberger và cộng sự 2008…) sau khi đã kiểm tra lại nguồn.
3. Không ghi giá tiền của map provider hay cloud. Điều khoản cache của provider nêu theo tài liệu chính thức tại thời điểm viết.
4. Mỗi bài tối ưu phải nêu **bên nào được lợi, bên nào chịu thiệt**. Không tối ưu cho tài xế bằng cách bắt khách đi bộ qua đường lớn.
5. Vị trí là dữ liệu cá nhân: khách chỉ thấy tài xế sau khi được gán, log không ghi tọa độ chính xác kèm định danh, có retention.

### 3.2 Template một bài

Giống loạt Rust và C#: frontmatter (`slug`, `title`, `summary` «Bài NN: …» ≤ 234 ký tự, `category`, `level`, `tags`, `language: vi`) → tình huống Tới Nơi mở bài, câu «Đây là bài NN của lộ trình …» → callout «Thử trước khi đọc» có `<details>` → 4–7 mục `##`, ít nhất một SVG nội tuyến theo lớp `packages/ui/src/article.css` (`viewBox` rộng 880, `role="img"`, `aria-label`, caption «Hình N. …») → code trích đúng từ lab, ghi tên file → «Chạy ví dụ và kiểm chứng» → «Giới hạn của ví dụ» → **Bài tập** có `<details>` → «Bài trước / Bài tiếp» (chỉ link bài đã có) → «Tài liệu gốc». Bài thuật toán có thêm **hình bản đồ hoặc đồ thị do lab xuất ra**, không vẽ tay kết quả.

### 3.3 Tổ chức lab

```text
examples/toi-noi-location/
├── README.md                khung biên tập, trạng thái từng bài
├── infra/                   Redis, Kafka (cổng riêng), PostGIS tùy chọn
├── data/                    bản đồ tải về (gitignore) + script tải, cắt
├── fixtures/                đồ thị tổng hợp nhỏ, kịch bản simulator (commit)
├── proto/                   hợp đồng gRPC, Kafka dùng chung cho Rust, Go, C#
├── rust/                    Cargo workspace: geo-core, route-engine, map-match,
│                            pickup, matcher, simulator, geo-engine (service)
├── go/                      location-ingest, tracking-hub
├── python/                  pipeline ETA, dự báo, phân cụm, OR-Tools (venv)
└── verify-pNN.sh            chạy test và kiểm kết quả ghi trong bài của từng phần
```

Khác loạt Rust (mỗi bài một crate độc lập), loạt này dùng **workspace tích lũy**, vì bài sau cần engine của bài trước. Mỗi bài có một lệnh chạy riêng; số trong bài phải tái lập được bằng lệnh đó.

### 3.4 Hàm mục tiêu dùng xuyên suốt

Đây là xương sống của loạt, chốt ở LOC-32, dùng lại ở P04, P05, P07. Với điểm hẹn ứng viên `p`:

```text
J(p) = a·T_walk(khách → p)                      khách đi bộ
     + b·max(T_walk, T_drive)                   thời gian tới lúc hai người gặp nhau
     + c·T_drive(tài xế → p)                    km rỗng của tài xế
     + d·T_trip(p → điểm trả, gồm quay đầu)     đoạn có khách, ảnh hưởng cả cước
     + phạt(cấm dừng, qua đường lớn, phía đường bất lợi, hẻm không vào được)
```

Ràng buộc cứng (loại ứng viên, không chỉ phạt): quá ngưỡng đi bộ, điểm dừng bất hợp pháp, không có đường bộ an toàn tới `p`. Bộ trọng số `a, b, c, d` là chính sách có `policy_version`, được lưu vào snapshot như quote (kiến trúc §4.3), hiệu chỉnh bằng simulator chứ không đoán. Điểm trả dùng hàm cùng dạng với hạng mục khách đi bộ tới đích thật và chi phí tài xế bắt chuyến kế. Ghép chuyến (P05) lấy tổng chi phí này trên toàn lô thay cho «tài xế gần nhất».

**Chỉ số đo trong mọi thí nghiệm:** `walk_m` (khách đi bộ), `deadhead_m` (km rỗng), `pickup_eta_s` p50/p95, `meet_gap_s` (chênh lệch lúc hai người tới điểm hẹn), `trip_m`, `u_turns`, tỷ lệ hủy mô phỏng.

## 4. Danh sách bài

Cột **NN**: ngôn ngữ chính của lab (`RS` Rust, `GO` Go, `PY` Python, `—` không có lab code đáng kể).

### Lộ trình `location-nen-tang-qua-toi-noi`

#### P01 · Bài toán và hình học địa lý (10 bài)

| Mã | Slug | Tiêu đề | NN | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|---|
| LOC-01 | `location-bai-toan-quang-duong-khach-va-tai-xe` | Quãng đường của hai người: bài toán thật của service định vị | — | Tình huống ghim sai phía đường; vì sao «tài xế gần nhất» và «đón đúng chỗ ghim» đều sai; các chỉ số `walk_m`, `deadhead_m`, `meet_gap_s`; bản đồ loạt bài | — | Đã viết |
| LOC-02 | `location-adr-rust-go-python-csharp-ranh-gioi-service` | ADR: Rust, Go, Python và C# mỗi bên làm gì trong service định vị | — | Tiêu chí chọn, nói rõ Rust không yếu thuật toán; Rust đề xuất, C# quyết định; hợp đồng gRPC/Kafka; cái giá khi có bốn ngôn ngữ; khi nào gộp lại | LOC-01 | Đã viết |
| LOC-03 | `location-toa-do-wgs84-newtype-lat-lng-do-chinh-xac` | Tọa độ: WGS84, newtype LatLng và lỗi đảo lat/lng | RS | WGS84 và VN-2000; newtype chặn đảo thứ tự lúc biên dịch; GeoJSON ngược thứ tự; `f64` vs `i32` E7, số chữ số thập phân tương ứng bao nhiêu mét; vì sao float ổn ở đây mà không ổn với tiền | LOC-02 | Đã viết |
| LOC-04 | `location-khoang-cach-haversine-equirectangular-sai-so` | Khoảng cách: haversine, equirectangular và sai số đo được | RS | Công thức, bench tốc độ, sai số đo ở vĩ độ Việt Nam và độ dài vài km; khoảng cách chim bay khác đường đi, hệ số vòng; khi nào xấp xỉ là đủ | LOC-03 | Đã viết |
| LOC-05 | `location-geometry-crate-geo-chieu-diem-polyline-bearing` | Hình học với crate geo: chiếu điểm lên polyline, bearing, GeoJSON | RS | Chiếu điểm lên đoạn, vị trí dọc tuyến, bearing, mã hóa polyline, xuất GeoJSON để xem kết quả trên bản đồ, biến debug thành thói quen | LOC-04 | Đã viết |
| LOC-06 | `location-osm-pbf-do-thi-duong-bo-xe-may-o-to` | Từ OpenStreetMap tới đồ thị đường bộ: một chiều, xe máy, ô tô | RS | Đọc `.pbf` bằng `osmpbf`, cắt bbox, thẻ `highway`/`oneway`/`motorcycle`, tách way thành cạnh, biểu diễn CSR, chất lượng dữ liệu hẻm ở Việt Nam, ODbL, kiểm tra thành phần liên thông | LOC-05 | Đã viết |
| LOC-07 | `location-rstar-r-tree-snap-diem-ve-duong` | R-tree với rstar: snap điểm GPS về đúng cạnh đường | RS | R-tree, nearest-neighbor, ứng viên theo bán kính accuracy, snap sai sang đường song song, bench so với quét toàn bộ | LOC-06 | Đã viết |
| LOC-08 | `location-h3-geohash-chi-muc-luoi-k-ring` | H3 và geohash: chia thành phố thành ô để tìm nhanh | RS | H3 bằng `h3o`, k-ring, lỗi biên ô của geohash, chọn độ phân giải theo độ dài cạnh, khi nào lưới hơn cây | LOC-04 | Đã viết |
| LOC-09 | `location-tim-tai-xe-gan-nhat-bon-cach-co-so-do` | Tìm tài xế gần nhất: quét, R-tree, H3 và Redis GEO có số đo | RS | Dựng tải theo hồ sơ Tới Nơi (6.250 điểm GPS/giây), đo cập nhật + truy vấn của từng cách, chi phí sửa R-tree so với H3, PostGIS chỉ đối chiếu vì không phục vụ tần suất này | LOC-07, LOC-08 | Đã viết |
| LOC-10 | `location-tong-hop-geo-core-kiem-thu-hinh-hoc` | Tổng hợp P01: thư viện geo-core và kiểm thử hình học | RS | Crate `geo-core`, proptest (khoảng cách đối xứng, bất đẳng thức tam giác trong sai số), đồ thị tổng hợp cho CI, những gì chưa có | LOC-09 | Đã viết |

#### P02 · Nhận, làm sạch vị trí và bám đường (10 bài)

| Mã | Slug | Tiêu đề | NN | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|---|
| LOC-11 | `location-go-goroutine-channel-nhap-mon-cho-ingest` | Go đủ dùng cho ingest: goroutine, channel, context | GO | Nhập môn tối thiểu, đối chiếu với tokio/Rust, `select`, `context`, `errgroup`, `-race`; vì sao bài toán này hợp Go | LOC-02 | Đã viết |
| LOC-12 | `location-go-ingest-websocket-batch-gps-tai-xe` | Location ingest bằng Go: nhận GPS theo lô từ app tài xế | GO | WebSocket vs gRPC stream khi mạng di động chập chờn, batching phía app, hợp đồng protobuf, xác thực, chịu tải 6.250 điểm/giây có số đo | LOC-11 | Đã viết |
| LOC-13 | `location-go-backpressure-coalesce-drop-race-detector` | Backpressure trong ingest: coalesce, drop có chủ đích, tắt êm | GO | Hàng đợi giới hạn, giữ điểm mới nhất theo tài xế, chính sách bỏ khi quá tải theo kiến trúc §10, `-race`, graceful shutdown, thí nghiệm khi Kafka chậm | LOC-12 | Đã viết |
| LOC-14 | `location-validate-gps-timestamp-accuracy-gia-mao` | Kiểm tra GPS: đồng hồ lệch, accuracy, nhảy vị trí, dấu hiệu giả mạo | GO | Clock skew, điểm cũ đến sau, tốc độ phi lý, chỉ gắn cờ và chuyển cho Fraud Python, không tự kết luận gian lận; bộ test có điểm xấu | LOC-13 | Đã viết |
| LOC-15 | `location-kalman-loc-nhieu-gps-rust` | Lọc nhiễu GPS bằng Kalman: giúp được bao nhiêu và trả giá gì | RS | Mô hình vận tốc không đổi, nhiễu theo accuracy, đo trên nhiễu tổng hợp có ground truth, độ trễ phải trả, khi không cần Kalman | LOC-10 | Đã viết |
| LOC-16 | `location-map-matching-hmm-viterbi-rust` | Map matching bằng HMM và Viterbi: tài xế đang ở đường nào | RS | Newson–Krumm: xác suất phát và chuyển, ứng viên từ R-tree, Viterbi, độ chính xác so với ground truth của simulator | LOC-07, LOC-15 | Đã viết |
| LOC-17 | `location-map-matching-online-cua-so-cau-vuot-duong-song-song` | Map matching online: cửa sổ trượt, cầu vượt và đường song song | RS | Cửa sổ, độ trễ quyết định, sửa lại điểm cũ, các ca khó của đô thị Việt Nam, xử lý mất tín hiệu, ngân sách CPU theo tài xế | LOC-16 | Đã viết |
| LOC-18 | `location-kho-vi-tri-moi-nhat-h3-redis-geo-kafka` | Kho vị trí mới nhất: bộ nhớ theo ô H3, Redis GEO và Kafka | RS | Tiêu thụ `location.updates.v1`, cấu trúc đọc nhiều ghi nhiều, TTL cho tài xế im lặng, khởi động lại thì nạp từ đâu, Redis là bản phụ, sampling sang lịch sử | LOC-09, LOC-14 | Đã viết |
| LOC-19 | `location-go-fan-out-tracking-khach-noi-suy-marker` | Tracking cho khách: fan-out bằng Go, nội suy marker dọc tuyến | GO | Hub theo chuyến, coalesce theo client chậm, chỉ mở sau khi gán tài xế, đo p95 độ trễ, hợp đồng để app nội suy theo tuyến thay vì nhảy | LOC-13, LOC-17 | Đã viết |
| LOC-20 | `location-lich-su-gps-parquet-retention-quyen-rieng-tu` | Lịch sử GPS: Parquet, retention và quyền riêng tư | GO | Luồng lịch sử tách luồng nóng, ghi Parquet theo ngày và thành phố, giảm mẫu, xóa theo hạn, mức định danh, nghĩa vụ pháp lý ở Việt Nam (kiểm lại văn bản hiện hành khi viết) | LOC-18 | Đã viết |

#### P03 · Định tuyến đường bộ (10 bài)

| Mã | Slug | Tiêu đề | NN | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|---|
| LOC-21 | `location-dijkstra-a-star-rust-do-thi-csr` | Dijkstra và A* trên đồ thị CSR: tuyến đường đầu tiên | RS | CSR thay cho đối tượng con trỏ, `BinaryHeap`, heuristic hợp lệ, số nút đã duyệt, đo trên bản đồ thành phố, đối chiếu `petgraph` | LOC-06 | Kế hoạch |
| LOC-22 | `location-bidirectional-alt-landmark-tang-toc-truy-van` | Bidirectional và ALT: giảm số nút duyệt mà vẫn đúng | RS | Điều kiện dừng đúng của tìm hai chiều (lỗi kinh điển), landmark, tiền xử lý so với tốc độ, test đối chiếu với Dijkstra | LOC-21 | Kế hoạch |
| LOC-23 | `location-chi-phi-re-u-turn-do-thi-theo-canh` | Chi phí rẽ và quay đầu: đồ thị theo cạnh | RS | Vì sao mô hình theo nút bỏ sót quay đầu, đồ thị theo cạnh, cấm rẽ trong OSM, chi phí đi qua dải phân cách; nền cho bài chọn phía đường | LOC-21 | Kế hoạch |
| LOC-24 | `location-profile-xe-may-o-to-di-bo-trong-hem` | Profile xe máy, ô tô, đi bộ: cùng thành phố, ba đồ thị | RS | Hẻm ô tô không vào, cấm xe máy trên cao tốc, đi bộ thì đi hai chiều được, tốc độ theo loại đường, cấu trúc dữ liệu dùng chung | LOC-23 | Kế hoạch |
| LOC-25 | `location-contraction-hierarchies-tu-cai-dat-fast-paths` | Contraction Hierarchies: tự cài đặt rồi đối chiếu fast_paths | RS | Thứ tự co, shortcut, truy vấn hai chiều lên đồ thị, chi phí tiền xử lý và bộ nhớ, tốc độ so với Dijkstra có số đo, test tính đúng trên mẫu ngẫu nhiên | LOC-22 | Kế hoạch |
| LOC-26 | `location-ma-tran-many-to-many-bucket-ch` | Ma trận chi phí many-to-many: nền của ghép chuyến | RS | Bucket-based trên CH, so với chạy lặp N×M, song song bằng `rayon`, lý do P04, P05 phụ thuộc bài này, ngân sách độ trễ | LOC-25 | Kế hoạch |
| LOC-27 | `location-toc-do-theo-gio-customizable-ch-ket-xe` | Tốc độ theo giờ và kẹt xe: cập nhật trọng số không dựng lại | RS | Hồ sơ tốc độ theo khung giờ, Customizable CH, dữ liệu tắc nghẽn là tổng hợp và nói rõ điều đó, sai lệch khi dùng tốc độ tĩnh | LOC-26 | Kế hoạch |
| LOC-28 | `location-self-host-osrm-valhalla-hay-map-provider` | Tự viết, self-host OSRM/Valhalla hay dùng map provider? | RS | ADR thứ hai: engine của lab để học, production nên đo cả ba hướng; tiêu chí lưu lượng, kẹt xe thật, điều khoản cache, cập nhật bản đồ; port `RoutingProvider` thay được adapter | LOC-26 | Kế hoạch |
| LOC-29 | `location-routing-service-tonic-cache-fallback-p99` | Dịch vụ định tuyến: gRPC, cache, timeout, fallback và p99 | RS | `tonic` Route/Matrix, deadline, cache theo ô H3 và khung giờ, fallback chim bay × hệ số vòng, client C# gọi được, đo p99 | LOC-27, LOC-28 | Kế hoạch |
| LOC-30 | `location-tong-hop-mo-phong-su-kien-roi-rac-seed` | Tổng hợp P03: bộ mô phỏng thành phố làm thước đo cho mọi tối ưu | RS | Mô phỏng sự kiện rời rạc tài xế + khách trên route engine, seed cố định, chạy lại ra đúng số, xuất các chỉ số §3.4, baseline «đón đúng ghim, tài xế gần nhất», giới hạn của mô phỏng | LOC-24, LOC-29 | Kế hoạch |

### Lộ trình `location-toi-uu-quang-duong-qua-toi-noi`

#### P04 · Tối ưu điểm đón và điểm trả (10 bài)

| Mã | Slug | Tiêu đề | NN | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|---|
| LOC-31 | `location-geocoding-dia-chi-viet-nam-autocomplete` | Từ địa chỉ tới tọa độ: geocoding và gợi ý địa chỉ Việt Nam | RS | Chuẩn hóa có dấu/không dấu, số nhà và hẻm kiểu Việt Nam, đổi đơn vị hành chính từ 2025 (kiểm lại khi viết), chỉ mục tìm kiếm, reverse geocoding, độ tin cậy kết quả | LOC-07 | Kế hoạch |
| LOC-32 | `location-ham-muc-tieu-diem-hen-khach-va-tai-xe` | Hàm mục tiêu điểm hẹn: đặt khách và tài xế lên cùng một cân | RS | Hàm §3.4, ví dụ tay trên đồ thị nhỏ, ràng buộc cứng khác phạt, ý nghĩa từng trọng số, `policy_version`, đo đánh đổi bằng simulator | LOC-30 | Kế hoạch |
| LOC-33 | `location-sinh-ung-vien-diem-don-do-thi-di-bo` | Sinh ứng viên điểm đón: khách đi bộ tới được đâu | RS | Dijkstra trên đồ thị đi bộ có giới hạn, snap về cạnh xe vào được, loại cầu vượt, hầm, điểm cấm dừng, gom ứng viên gần nhau, đo số ứng viên | LOC-24, LOC-32 | Kế hoạch |
| LOC-34 | `location-phia-duong-huong-tiep-can-tranh-quay-dau` | Phía đường và hướng tiếp cận: bớt một vòng quay đầu | RS | Mô hình mục tiêu ở phía đường cụ thể, hướng di chuyển từ map matching, đường có dải phân cách, thí nghiệm `u_turns`, `deadhead_m` so với baseline | LOC-23, LOC-33 | Kế hoạch |
| LOC-35 | `location-xep-hang-diem-don-one-to-many-ngan-sach-do-tre` | Xếp hạng điểm đón trong ngân sách độ trễ | RS | Tính J cho tất cả ứng viên từ ma trận, trả top 3, đặt và đo ngân sách p99, chưa biết tài xế thì tính theo phân bố quanh đó, bất biến proptest: kết quả không tệ hơn điểm ghim | LOC-26, LOC-34 | Kế hoạch |
| LOC-36 | `location-diem-hen-dong-cap-nhat-khi-hai-ben-di-chuyen` | Điểm hẹn động: cập nhật khi cả hai đang di chuyển | RS | Tính lại theo sự kiện, ngưỡng trễ để tránh đổi qua đổi lại, chỉ đổi khi lợi đủ lớn và khách đồng ý, `meet_gap_s`, thí nghiệm khách đi chậm và tài xế trễ | LOC-19, LOC-35 | Kế hoạch |
| LOC-37 | `location-hem-chung-cu-tttm-san-bay-cong-sanh` | Hẻm, chung cư, trung tâm thương mại, sân bay: nơi GPS vô dụng | RS | Lớp dữ liệu cổng, sảnh, khu đón riêng; hẻm khác nhau cho xe máy và ô tô; vùng có điểm đón bắt buộc; GPS kém trong tòa nhà; dữ liệu thiếu thì lùi về đâu | LOC-33 | Kế hoạch |
| LOC-38 | `location-toi-uu-diem-tra-cong-vao-chuyen-ke-tiep` | Tối ưu điểm trả: gần đích thật, đúng phía, dễ bắt chuyến kế | RS | Hàm mục tiêu cho điểm trả, khách đi bộ tới đích thật, cổng vào, chi phí thoát ra của tài xế, đổi đích khi đang đi, đánh đổi có số đo | LOC-35, LOC-37 | Kế hoạch |
| LOC-39 | `location-hoc-diem-don-tu-lich-su-dbscan-python` | Học điểm đón từ lịch sử bằng DBSCAN | PY | Phân cụm trên đơn vị mét, tham số và cách chọn, xuất bảng điểm cho Rust nạp, kiểm định trên simulator, thiên lệch «cứ đón ở đâu thì học ở đó» | LOC-20, LOC-33 | Kế hoạch |
| LOC-40 | `location-cuoc-quote-giai-thich-doi-diem-don-cho-khach` | Đổi điểm đón thì cước ra sao, và nói với khách thế nào | RS | Tuyến đổi làm cước đổi theo mô hình 12.000 + 4.500/km, snapshot quote, không tăng giá lén, hợp đồng với Pricing C#, câu chữ và hiển thị cho khách, tài xế; tổng hợp P04 có số đo | LOC-36, LOC-38 | Kế hoạch |

#### P05 · Ghép chuyến và giảm km rỗng (10 bài)

| Mã | Slug | Tiêu đề | NN | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|---|
| LOC-41 | `location-gan-nhat-sai-o-dau-do-bang-mo-phong` | Tài xế gần nhất sai ở đâu: đo bằng mô phỏng | RS | Chim bay so với ETA đường bộ, phản ví dụ tham lam cướp tài xế của người khác, baseline cho cả phần | LOC-30 | Kế hoạch |
| LOC-42 | `location-ghep-theo-lo-hungarian-kuhn-munkres-rust` | Ghép theo lô: bài toán phân công và thuật toán Hungarian | RS | Mô hình hóa ma trận chi phí, `pathfinding` Kuhn–Munkres (trọng số nguyên, ma trận chữ nhật), xử lý thừa khách hoặc thừa xe, độ phức tạp, so với tham lam | LOC-26, LOC-41 | Kế hoạch |
| LOC-43 | `location-cua-so-lo-do-tre-doi-chat-luong-ghep` | Cửa sổ lô: đợi bao lâu là đáng | RS | Quét độ dài cửa sổ trên simulator theo mật độ, thời gian chờ đổi lấy km rỗng, cửa sổ thích ứng, ngày thường và cao điểm | LOC-42 | Kế hoạch |
| LOC-44 | `location-cat-tia-ung-vien-h3-ma-tran-thua` | Cắt tỉa ứng viên: ma trận thưa ở quy mô thành phố | RS | Lọc bằng k-ring, giới hạn k gần nhất, ma trận thưa, tách vùng độc lập, sai số do cắt tỉa so với tối ưu đầy đủ, ngân sách độ trễ | LOC-09, LOC-42 | Kế hoạch |
| LOC-45 | `location-chi-phi-ghep-cong-bang-tai-xe-rui-ro-huy` | Chi phí ghép ngoài quãng đường: công bằng cho tài xế và rủi ro hủy | RS | Thời gian rảnh của tài xế, giới hạn đón xa, xác suất hủy là đầu vào, tránh phân biệt đối xử theo khu vực, đánh đổi có số đo, trọng số là chính sách có version | LOC-43 | Kế hoạch |
| LOC-46 | `location-goi-y-sang-dispatch-csharp-grpc-lease-idempotency` | Từ gợi ý tới assignment: hợp đồng với Dispatch C# | RS | gRPC `SuggestMatch`, Rust đề xuất, C# quyết định; gợi ý có hạn, tài xế từ chối thì sao, gợi ý cũ, lease, idempotency, contract test có client .NET | LOC-44 | Kế hoạch |
| LOC-47 | `location-chain-dispatch-tai-xe-sap-tra-khach` | Ghép nối chuyến: nhận chuyến kế khi sắp trả khách | RS | Tài xế bận cũng là ứng viên, ETA còn lại có độ tin cậy, rủi ro chuyến trước kéo dài, đo km rỗng và thời gian chờ | LOC-45 | Kế hoạch |
| LOC-48 | `location-lech-tuyen-dinh-tuyen-lai-ghep-lai` | Lệch tuyến, định tuyến lại và ghép lại | RS | Phát hiện lệch tuyến từ map matching, định tuyến lại phía server hay app, khi nào được đổi tài xế, giới hạn để không làm phiền cả hai, ngưỡng trễ | LOC-17, LOC-46 | Kế hoạch |
| LOC-49 | `location-tai-xe-ranh-di-dau-dieu-huong-theo-nhu-cau` | Tài xế rảnh nên đi đâu: gợi ý theo nhu cầu mà không gây bầy đàn | RS | Bản đồ nhu cầu theo ô H3, chi phí di chuyển so với lợi kỳ vọng, phân bổ để không kéo tất cả về một chỗ, chỉ gợi ý không ép, đo trên simulator với dự báo từ LOC-55 | LOC-44 | Kế hoạch |
| LOC-50 | `location-nhieu-diem-dung-chen-2-opt-doi-chieu-ortools` | Nhiều điểm dừng: chèn, 2-opt trong Rust và đối chiếu OR-Tools | RS, PY | Giao bưu kiện có ràng buộc lấy trước giao sau, heuristic chèn + 2-opt/Or-opt đủ nhanh cho đường nóng, OR-Tools chạy offline làm mốc so sánh chất lượng, tổng hợp P05 | LOC-26, LOC-47 | Kế hoạch |

### Lộ trình `location-du-lieu-van-hanh-qua-toi-noi`

#### P06 · Học từ dữ liệu bằng Python (6 bài)

| Mã | Slug | Tiêu đề | NN | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|---|
| LOC-51 | `location-python-polars-etl-chuyen-gps-dac-trung` | Pipeline dữ liệu: từ Parquet GPS tới đặc trưng chuyến | PY | Polars đọc lịch sử LOC-20, nối chuyến, đặc trưng theo giờ và ô H3, không rò rỉ tương lai, chia theo thời gian | LOC-20, LOC-30 | Kế hoạch |
| LOC-52 | `location-python-eta-lightgbm-sua-sai-so-route-engine` | ETA bằng LightGBM: học phần sai số của route engine | PY | Dự đoán phần dư thay vì toàn bộ thời gian, baseline, MAE theo khung giờ và khu vực, nói rõ dữ liệu tổng hợp chỉ chứng minh phương pháp, nối bài ETA của loạt Python khi đã xuất bản | LOC-51 | Kế hoạch |
| LOC-53 | `location-onnx-python-sang-rust-tract-ort-do-p99` | Đưa mô hình vào Rust: ONNX, tract và ort | RS, PY | Xuất ONNX, chạy bằng `tract` và `ort`, so sánh kết quả hai phía, p99 và bộ nhớ, version mô hình, fallback khi mô hình lỗi | LOC-29, LOC-52 | Kế hoạch |
| LOC-54 | `location-do-tin-cay-eta-p50-p90-hieu-chuan` | Độ tin cậy của ETA: p50, p90 và hiệu chuẩn | PY | Hồi quy phân vị, kiểm tra hiệu chuẩn, app nên hiện con số nào, ảnh hưởng tới `meet_gap_s` và điểm hẹn động | LOC-52 | Kế hoạch |
| LOC-55 | `location-du-bao-nhu-cau-o-h3-kiem-tra-theo-thoi-gian` | Dự báo nhu cầu theo ô H3 | PY | Baseline theo cùng giờ tuần trước, mô hình, kiểm tra theo thời gian, ô thưa, xuất bảng cho LOC-49 | LOC-51 | Kế hoạch |
| LOC-56 | `location-switchback-thi-nghiem-thi-truong-hai-phia` | Thí nghiệm trên thị trường hai phía: vì sao A/B thường sai | PY | Nhiễu giữa hai nhóm dùng chung tài xế, switchback theo vùng và khung giờ, phân tích bằng Python trên simulator, tiêu chí dừng, chỉ số bảo vệ cho cả khách và tài xế | LOC-45, LOC-51 | Kế hoạch |

#### P07 · Service hoàn chỉnh và vận hành (8 bài)

| Mã | Slug | Tiêu đề | NN | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|---|
| LOC-57 | `location-workspace-geo-engine-hexagonal-port-adapter` | Kiến trúc geo-engine: workspace, port và adapter | RS | Bản đồ crate, domain không phụ thuộc tokio, port `RoutingProvider`, `DriverStore`; vì sao engine thuật toán không cần đầy đủ CQRS/Mediator, lệch kiến trúc đích ở đâu và lý do | LOC-46 | Kế hoạch |
| LOC-58 | `location-phan-manh-thanh-pho-mmap-doi-ban-do-khong-dung` | Mỗi thành phố một mảnh: mmap và đổi bản đồ không dừng | RS | Shard theo thành phố, nạp đồ thị bằng mmap, đổi phiên bản bản đồ trong lúc chạy, chuyến vắt qua biên, RSS đo được | LOC-57 | Kế hoạch |
| LOC-59 | `location-kiem-thu-tuyen-dung-proptest-golden-replay` | Kiểm thử engine: CH khớp Dijkstra, golden route, replay chuyến | RS | Proptest đối chiếu thuật toán nhanh với chậm, golden route khi đổi bản đồ, replay GPS, simulator trong CI có ngưỡng hồi quy cho các chỉ số | LOC-57 | Kế hoạch |
| LOC-60 | `location-hieu-nang-flamegraph-cache-locality-bo-nho` | Hiệu năng: flamegraph, cache locality và bộ nhớ | RS | Profile thật, sắp xếp nút theo không gian, tránh cấp phát trên đường nóng, số trước và sau, khi nào dừng tối ưu | LOC-58 | Kế hoạch |
| LOC-61 | `location-kiem-thu-tai-che-do-suy-giam-khi-qua-tai` | Kiểm thử tải và chế độ suy giảm | GO | Bộ tạo tải bằng Go, ingest + geo-engine + hub chạy cùng lúc, thứ tự suy giảm: bỏ điểm hẹn động → ETA chim bay → tham lam thay ghép lô, circuit breaker, phục hồi | LOC-59, LOC-60 | Kế hoạch |
| LOC-62 | `location-opentelemetry-slo-km-rong-di-bo-canh-bao` | Quan sát: trace Go–Rust–C#, SLO và chỉ số km rỗng | RS | W3C propagation qua WebSocket, gRPC, Kafka; SLO §14.1; chỉ số nghiệp vụ `deadhead_m`, `walk_m` theo thành phố; cảnh báo khi đổi bản đồ làm xấu đi; log không lộ tọa độ | LOC-61 | Kế hoạch |
| LOC-63 | `location-trien-khai-container-du-lieu-ban-do-cloud` | Triển khai: container Rust, Go, Python và pipeline bản đồ | RS | Dockerfile đa stage, ảnh bản đồ tách khỏi ảnh code, job biên dịch bản đồ, readiness chờ nạp xong, autoscale, liên kết loạt AWS và Azure; nói rõ chưa chạy thật trên cloud nếu chưa chạy | LOC-58, LOC-62 | Kế hoạch |
| LOC-64 | `location-tong-hop-quang-duong-hai-nguoi-dau-toi-cuoi` | Tổng hợp toàn loạt: chuyến TN-0001 từ ghim bản đồ tới điểm trả | RS | Lần theo một chuyến qua mọi thành phần, bảng baseline so với bản tối ưu theo từng chỉ số (mô phỏng, có seed), việc tối ưu nào đáng làm trước, những gì lab chưa có | LOC-40, LOC-50, LOC-56, LOC-63 | Kế hoạch |

## 5. Chuỗi phụ thuộc chính

- Hình học: LOC-03 → 04 → 05 → 06 → 07 → 09 → 10; LOC-08 → 09.
- Nhận vị trí: LOC-11 → 12 → 13 → 14 → 18 → 20; LOC-13 → 19.
- Bám đường: LOC-07, 15 → 16 → 17 → 48.
- Định tuyến: LOC-21 → 22 → 25 → 26 → 27 → 29 → 30; LOC-21 → 23 → 24.
- **Trục tối ưu điểm đón**: LOC-30 → 32 → 33 → 34 → 35 → 36, 38 → 40.
- **Trục ghép chuyến**: LOC-30 → 41 → 42 → 43, 44 → 45, 46 → 47 → 50.
- Dữ liệu: LOC-20 → 51 → 52 → 53; LOC-52 → 54; LOC-51 → 55, 56.
- Vận hành: LOC-46 → 57 → 58, 59 → 60 → 61 → 62 → 63 → 64.

LOC-49 dùng kết quả của LOC-55 làm đầu vào nhưng đọc được trước: lab có sẵn bảng nhu cầu cố định, LOC-55 thay bảng đó bằng dự báo.

## 6. Quyết định kỹ thuật đề xuất

1. **Engine định tuyến tự viết để học, không tuyên bố thay được OSRM/Valhalla.** LOC-28 viết ADR này. Port `RoutingProvider` giữ đường đổi adapter.
2. **Dữ liệu bản đồ**: OSM Việt Nam tải bằng script, cắt bbox một thành phố; đồ thị tổng hợp nhỏ cho test và CI.
3. **Hạ tầng lab riêng** `examples/toi-noi-location/infra`, cổng không đụng Redis, Kafka của loạt Rust và Python đang chạy. Tái dùng image đã có trên máy (`redis:7`, `apache/kafka:4.1.0`). PostGIS là tùy chọn vì image chưa có sẵn và Docker Hub từng treo trên máy này (LOC-09 chỉ đối chiếu, không phụ thuộc).
4. **Kafka client**: `rdkafka` ở Rust (giống loạt Rust), `franz-go` ở Go.
5. **Thứ tự viết**: P01 → P03 tuần tự vì lab tích lũy; P04 và P05 có thể song song sau khi có simulator LOC-30. Sonnet viết code và bài, Opus review, chạy `verify-pNN.sh`, commit theo phần.

## 7. Điểm cần chủ blog chốt trước khi viết

1. **Quy mô 64 bài**, hay rút gọn còn khoảng 30 bài xoay quanh P03–P05 (giả định người đọc tự tìm hiểu nền tảng).
2. **Sửa ranh giới trong loạt Rust.** `docs/rust-master-curriculum.md` và RS-01 (branch `feature/quandh/rust_series`) ghi «Rust chỉ dành cho ledger và đối soát», còn bảng trong `du-an-mau-toi-noi` đã cho Rust làm «các thành phần nhạy với độ trễ/tài nguyên». Cần đổi câu ở RS-01 thành «trong loạt này» và thêm `geo-engine` vào hồ sơ dự án cùng `docs/toi-noi-overall-architecture.md`. Branch đó đang có việc dở (bài 21–30 chưa commit), nên sửa sau khi việc đó xong.
3. **Thứ tự merge.** Loạt này giả định người đọc đã xem Rust RS-01 → RS-22, Python PY-62 và Flutter, nhưng các bài đó chưa nằm trên `main`. Quy tắc «không tạo link cho bài chưa có» nên các link chéo chờ tới khi các branch đó merge.
4. **Category** đề xuất là `system-design` thay vì `programming`, vì loạt đa ngôn ngữ và thiên về thiết kế.
