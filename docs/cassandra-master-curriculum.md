# Apache Cassandra 5.0 qua dự án Tới Nơi

Khung biên tập v1 · 16/11/2026 · **6 phần × 10 bài = 60 bài**. Mỗi phần có **3 cơ bản / 4 trung bình / 3 nâng cao**, gồm cả bài thực hành cuối phần. Đây là khung viết bài; hoàn thành khung không có nghĩa đã hoàn thành nội dung.

## Phạm vi và cách đọc

Series riêng cho **Apache Cassandra 5.0** thuộc chuyên mục cha Database, chuyên mục con `cassandra`. Dùng xuyên suốt hệ thống hư cấu **Tới Nơi** (đặt xe, giao đồ ăn, giao bưu kiện), nhưng **chỉ đặt vào Cassandra những dữ liệu hợp với nó**: vị trí tài xế (≈ 6.250 điểm/giây lúc cao điểm), dòng thời gian sự kiện chuyến (~16 triệu dòng/ngày), lịch sử chuyến theo khách và theo tài xế, chat tài xế–khách, hộp thư thông báo, bộ đếm và khoá idempotency. Tiền, số dư ví và trạng thái chuyến cần ràng buộc liên bảng vẫn ở PostgreSQL/SQL Server; series nói rõ ranh giới này thay vì ép mọi thứ vào Cassandra.

Cassandra khác hẳn database quan hệ: **thiết kế bảng theo truy vấn** (query-first), khoá chính gồm partition key và clustering key quyết định dữ liệu nằm ở node nào và theo thứ tự nào, không có JOIN, không có transaction nhiều dòng, mọi câu ghi là upsert, xoá sinh tombstone, độ nhất quán chọn theo từng câu lệnh. Series dạy các cơ chế đó bằng thí nghiệm chạy thật, không dịch cú pháp SQL sang CQL. Đối tượng: người mới, backend developer và người vận hành database.

Đọc hồ sơ chung `du-an-mau-toi-noi` trước P01; P01 không yêu cầu kiến thức Cassandra. Người đã học series PostgreSQL có thể đọc song song: mỗi bài có mục "So với PostgreSQL" ngắn khi cơ chế khác nhau thật sự. CB = `beginner`, TB = `intermediate`, NC = `advanced`; toàn khung có 18 CB / 24 TB / 18 NC.

| Phần | Chủ đề | Đầu vào | Sản phẩm cuối phần |
|---|---|---|---|
| P01 | Nền tảng và mô hình hoá theo truy vấn | Hồ sơ Tới Nơi | Mô hình lịch sử chuyến và dòng thời gian sự kiện, có kiểm tra truy vấn |
| P02 | CQL cho nghiệp vụ | P01 | Chat tài xế–khách, hộp thư thông báo, bộ đếm và khoá idempotency |
| P03 | Kiến trúc phân tán | P01–P02 | Cụm 3 node, bằng chứng replication/consistency, diễn tập node chết |
| P04 | Hiệu năng, nhất quán và đồng thời | P01–P03 | Mô hình vị trí tài xế, tranh chấp nhận chuyến bằng LWT, số đo có điều kiện |
| P05 | Bảo mật, bảo trì và phục hồi | P01–P04 | Runbook xác thực, repair, snapshot/restore, thay node |
| P06 | Mở rộng, tích hợp và vận hành đầu cuối | P01–P05 | Hồ sơ vận hành, nâng cấp, di chuyển dữ liệu và quyết định kiến trúc |

## Môi trường và công cụ

- **Engine:** Apache Cassandra **5.0.9** (ảnh chính thức `cassandra:5.0.9` ghim theo digest), chạy bằng **Docker Desktop**. Compose và runner ở `examples/toi-noi-cassandra`. Mặc định một node (`node1`, cổng CQL `19042`); từ P03 bật cụm 3 node bằng profile `cluster` (`19043`, `19044`). Mỗi node quảng bá `127.0.0.1` với cổng CQL riêng để driver trên máy host (DBStudio, ứng dụng) kết nối được.
- **Công cụ GUI trong bài:** **DBStudio 0.1.7** (Rust + Tauri, mã nguồn ở github.com/rickymta/dbstudio). Bài dùng DBStudio cho kết nối, cây keyspace → bảng/MV/UDT/function, xem DDL, query editor CQL, lưới kết quả và sửa dữ liệu theo khoá (partition + clustering), ERD (bảng + cột + khoá), export/import và backup logic `.dbsbak`. Khi DBStudio không có (nodetool, cassandra-stress, `sstablemetadata`, snapshot), bài dùng công cụ trong container và nói rõ. **Đã kiểm ở 0.1.7:** nút "Giải thích kế hoạch" (Ctrl+E) bị ẩn với CQL dù backend có query tracing, nên bài về tracing (CS-028, CS-031) dùng `TRACING ON` của cqlsh cho tới khi DBStudio mở tính năng này; lưới kết quả hiển thị `timestamp` theo UTC (`…Z`) còn cqlsh của lab hiển thị `+0700`.
- **Ảnh minh hoạ:** chụp từ DBStudio thật nối với lab bằng bộ chụp tự động `examples/toi-noi-cassandra/shots` (Playwright điều khiển giao diện DBStudio, backend thật qua dev bridge). Ảnh lưu ở `apps/web/public/images/articles/cassandra/<mã bài>/`. Chụp lại toàn bộ bằng `npm run shots` khi đổi phiên bản DBStudio hoặc dữ liệu lab.
- **Chạy lab tái lập:** `cqlsh` (có sẵn trong ảnh) **bên trong container**, gọi qua `node run.mjs`; không cần cài Cassandra, Java hay Python trên máy. Runner không có dependency npm.
- Không dùng tính năng chỉ có ở DataStax Astra/DSE. Tính năng **thử nghiệm** của Cassandra 5.0 (materialized view, transient replication, SASI) phải ghi rõ nhãn experimental và lý do không dùng ở production.

## Tổ chức Database

Theo quy ước trong [`postgresql-master-curriculum.md`](postgresql-master-curriculum.md): `database` là chuyên mục cha; mỗi engine có chuyên mục con, curriculum và lộ trình riêng. Cassandra dùng slug chuyên mục `cassandra`, tiền tố bài `cassandra-`, lộ trình `cassandra-qua-toi-noi` (P01) rồi `cassandra-qua-toi-noi-phan-02` … `-phan-06`. ScyllaDB có series riêng; series này chỉ so sánh ở CS-056. Chỉ thêm vào `api/seed/learning-paths.json` các bài đã viết và kiểm tra.

## Tiêu chuẩn viết và hoàn tất một phần

Frontmatter: `slug`, `title`, `summary`, `category: cassandra`, `level`, `tags` (gồm mã bài `cs-0NN`), `language: vi`. Giọng văn tiếng Việt theo [`content-dictionary/style-guide.md`](content-dictionary/style-guide.md) và [`terminology.md`](content-dictionary/terminology.md); giải nghĩa thuật ngữ lần đầu; mở bằng vấn đề Tới Nơi; trình bày cơ chế; ví dụ chạy thật kèm kết quả lấy từ `OUTPUT.txt`; ảnh DBStudio có chú thích nói người đọc cần nhìn vào đâu; lỗi thường gặp; đánh đổi; bài tập; đọc tiếp. Không suy rộng cơ chế Cassandra cho mọi NoSQL. Số đo hiệu năng ghi cấu hình máy, số node, RF, consistency level, số dòng, số lần đo; không bịa số.

**Từ điển keyword:** thuật ngữ mới quan trọng của mỗi phần được chuẩn bị trong [`content-dictionary/glossary-candidates/cassandra.json`](content-dictionary/glossary-candidates/cassandra.json) theo cấu trúc của thư mục đó (định nghĩa ≤ 300 ký tự, đúng với Cassandra 5.0, soát trùng với `api/seed/terminology.json` và file ứng viên của series khác). Từ dễ bắt nhầm toàn site (partition, token, cluster, snapshot, compaction, consistency level…) gắn `categories: ["cassandra"]` và chờ phạm vi chủ đề trước khi gộp.

Hoàn thành **nội dung** một phần: cập nhật trạng thái ở đây, seed lộ trình, README lab, chạy `npm test` của lab, chụp ảnh, kiểm lint thuật ngữ, cập nhật ứng viên từ điển, rồi commit riêng `content(cassandra): complete part N`. Seed/deploy là bước riêng.

## P01 · Nền tảng và mô hình hoá theo truy vấn

Đầu vào: hồ sơ chung Tới Nơi; chưa cần biết CQL. Đầu ra: keyspace `toinoi`, các bảng lịch sử chuyến theo khách, theo tài xế và dòng thời gian sự kiện chuyến; fixture; bộ truy vấn kiểm tra đúng thứ tự, đúng partition.

Lộ trình: `cassandra-qua-toi-noi` — **đã viết đủ 10 bài ngày 17/11/2026**. Lab: `examples/toi-noi-cassandra/p01` (Cassandra **5.0.9 thật** trong Docker; 9 file CQL + 1 script, 20 kiểm tra PASS, `OUTPUT.txt`, `npm test`). 17 ảnh DBStudio 0.1.7 chụp tự động bằng `shots/`. Ứng viên từ điển: `glossary-candidates/cassandra.json` (35 mục). Chưa seed/deploy lên website.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| CS-001 | Cassandra là gì và nằm ở đâu trong Tới Nơi?<br>`cassandra-la-gi-va-vai-tro-trong-toi-noi` | CB | Phân biệt cluster, node, keyspace, bảng, partition; chọn dữ liệu nào hợp Cassandra | Đã viết |
| CS-002 | Dựng Cassandra 5.0 bằng Docker và kết nối bằng DBStudio<br>`cassandra-cai-dat-docker-dbstudio` | CB | Chạy lab, cqlsh trong container, kết nối DBStudio, đọc phiên bản, dọn môi trường | Đã viết |
| CS-003 | Keyspace, bảng và khoá chính: partition key và clustering key<br>`cassandra-keyspace-bang-khoa-chinh` | CB | Đọc khoá chính `((a, b), c, d)`, biết dòng nằm ở partition nào và theo thứ tự nào | Đã viết |
| CS-004 | Kiểu dữ liệu CQL cho tiền, thời gian, chuỗi và định danh<br>`cassandra-kieu-du-lieu-cql` | TB | Chọn bigint/decimal, timestamp/date, text, uuid/timeuuid; tránh mất độ chính xác | Đã viết |
| CS-005 | INSERT, UPDATE, DELETE: mọi câu ghi là upsert<br>`cassandra-insert-update-delete-upsert` | TB | Giải thích upsert, last-write-wins, `USING TIMESTAMP`, tombstone đầu tiên, `IF EXISTS` | Đã viết |
| CS-006 | SELECT theo khoá: WHERE, thứ tự clustering và phân trang<br>`cassandra-select-where-clustering-order` | TB | Viết truy vấn hợp lệ, hiểu vì sao bị từ chối, `ALLOW FILTERING`, `PER PARTITION LIMIT`, paging | Đã viết |
| CS-007 | Thiết kế bảng theo truy vấn<br>`cassandra-thiet-ke-theo-truy-van` | TB | Liệt kê access pattern, vẽ sơ đồ truy vấn, suy ra khoá chính từng bảng | Đã viết |
| CS-008 | Phi chuẩn hoá: một truy vấn một bảng và giữ các bản sao khớp nhau<br>`cassandra-phi-chuan-hoa-nhieu-bang` | NC | Ghi một chuyến vào nhiều bảng bằng logged batch, đối soát lệch dữ liệu | Đã viết |
| CS-009 | Kích thước partition và bucketing theo thời gian<br>`cassandra-partition-size-bucketing` | NC | Ước lượng số dòng/byte mỗi partition, chọn bucket ngày/giờ, đo bằng `nodetool tablestats` | Đã viết |
| CS-010 | Thực hành: mô hình lịch sử chuyến và dòng thời gian sự kiện<br>`cassandra-thuc-hanh-mo-hinh-lich-su-chuyen` | NC | Bàn giao CQL, fixture, truy vấn kiểm tra và bằng chứng chạy | Đã viết |

## P02 · CQL cho nghiệp vụ

Đầu vào: P01. Đầu ra: bảng chat tài xế–khách, hộp thư thông báo, bộ đếm chuyến theo ngày, khoá idempotency; truy vấn có expected output. Lộ trình dự kiến `cassandra-qua-toi-noi-phan-02`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| CS-011 | Collection: set, list, map<br>`cassandra-collection-set-list-map` | CB | Lưu nhãn, ảnh, thuộc tính nhỏ; biết khi nào collection sinh tombstone và read-before-write | Dự kiến |
| CS-012 | User-defined type, tuple và frozen<br>`cassandra-udt-tuple-frozen` | CB | Gói địa chỉ đón/trả thành UDT; hiểu frozen và giới hạn khi sửa | Dự kiến |
| CS-013 | TTL và dữ liệu tự hết hạn<br>`cassandra-ttl-du-lieu-tam` | CB | Trạng thái online của tài xế, mã OTP; đọc `TTL()`, `WRITETIME()`, `default_time_to_live` | Dự kiến |
| CS-014 | Counter: đếm chuyến và lượt xem<br>`cassandra-counter` | TB | Bảng counter, giới hạn (không idempotent, không TTL), khi nào không dùng counter | Dự kiến |
| CS-015 | Static column: dữ liệu chung của một partition<br>`cassandra-static-column` | TB | Thông tin cuộc trò chuyện dùng chung cho mọi tin nhắn trong partition | Dự kiến |
| CS-016 | Batch: logged, unlogged và khi nào batch có hại<br>`cassandra-batch-logged-unlogged` | TB | Dùng batch cho tính nguyên tử giữa bảng phi chuẩn hoá, không dùng để "tăng tốc" | Dự kiến |
| CS-017 | Hàm, aggregate, JSON và UDF<br>`cassandra-ham-aggregate-json` | TB | `toTimestamp`, `minTimeuuid`, `COUNT/SUM` trong partition, `SELECT JSON`, rủi ro của UDF | Dự kiến |
| CS-018 | Storage-Attached Index (SAI) và secondary index<br>`cassandra-sai-secondary-index` | NC | Tạo SAI của Cassandra 5.0, đọc giới hạn, so với bảng tra cứu và legacy 2i | Dự kiến |
| CS-019 | Materialized view: tiện nhưng thử nghiệm<br>`cassandra-materialized-view` | NC | Tạo MV, quan sát độ trễ đồng bộ, vì sao production nên tự quản bảng tra cứu | Dự kiến |
| CS-020 | Thực hành: chat tài xế–khách và hộp thư thông báo<br>`cassandra-thuc-hanh-chat-hop-thu` | NC | Bàn giao bảng, truy vấn, TTL, counter, idempotency và expected output | Dự kiến |

## P03 · Kiến trúc phân tán

Đầu vào: P01–P02; máy có ≥ 6 GB RAM cho Docker. Đầu ra: cụm 3 node, bằng chứng token/replica, consistency level và node chết. Lộ trình dự kiến `-phan-03`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| CS-021 | Ring, token và partitioner<br>`cassandra-ring-token-partitioner` | CB | Tính token của partition key, đọc `token()`, vnode và `num_tokens` | Dự kiến |
| CS-022 | Dựng cụm 3 node và đọc nodetool status<br>`cassandra-cum-3-node-nodetool` | CB | Bật profile cluster, đọc UN/DN, load, owns; kết nối DBStudio tới cụm | Dự kiến |
| CS-023 | Replication factor, NetworkTopologyStrategy và datacenter<br>`cassandra-replication-factor-strategy` | CB | Chọn RF, `nodetool getendpoints`, đổi RF và vì sao phải repair sau đó | Dự kiến |
| CS-024 | Consistency level: ONE, QUORUM, LOCAL_QUORUM, ALL<br>`cassandra-consistency-level` | TB | Tính R + W > RF, chọn CL theo nghiệp vụ, đọc lỗi Unavailable/Timeout | Dự kiến |
| CS-025 | Gossip, phát hiện lỗi và snitch<br>`cassandra-gossip-snitch` | TB | Đọc `nodetool gossipinfo`, phi accrual, rack/dc từ GossipingPropertyFileSnitch | Dự kiến |
| CS-026 | Hinted handoff, read repair và anti-entropy repair<br>`cassandra-hinted-handoff-read-repair` | TB | Tái hiện node vắng mặt, quan sát hint, read repair và `nodetool repair` | Dự kiến |
| CS-027 | Đường ghi: commit log, memtable và SSTable<br>`cassandra-duong-ghi-commitlog-memtable-sstable` | TB | Quan sát flush, file SSTable, `sstablemetadata`; vì sao ghi nhanh | Dự kiến |
| CS-028 | Đường đọc: bloom filter, index, cache và tracing trong DBStudio<br>`cassandra-duong-doc-tracing` | NC | Đọc trace của một SELECT, số SSTable phải đọc, key cache, bloom filter | Dự kiến |
| CS-029 | Compaction: STCS, LCS, TWCS và UCS<br>`cassandra-compaction-strategy` | NC | Chọn chiến lược theo tải (TWCS cho vị trí, UCS mặc định mới), đo khuếch đại | Dự kiến |
| CS-030 | Thực hành: một node chết giữa giờ cao điểm<br>`cassandra-thuc-hanh-node-chet` | NC | Bàn giao kịch bản, CL đã chọn, bằng chứng không mất ghi và quy trình phục hồi | Dự kiến |

## P04 · Hiệu năng, nhất quán và đồng thời

Đầu vào: P01–P03 (cụm 3 node). Đầu ra: mô hình vị trí tài xế, tranh chấp nhận chuyến bằng LWT, số đo có điều kiện. Lộ trình dự kiến `-phan-04`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| CS-031 | Tombstone: sinh ra từ đâu và vì sao làm chậm đọc<br>`cassandra-tombstone` | CB | Đếm tombstone trong trace, `gc_grace_seconds`, ngưỡng cảnh báo/huỷ | Dự kiến |
| CS-032 | Last-write-wins, đồng hồ và USING TIMESTAMP<br>`cassandra-last-write-wins-dong-ho` | CB | Tái hiện ghi đè do lệch đồng hồ, timestamp phía client, thiết kế không phụ thuộc thứ tự ghi | Dự kiến |
| CS-033 | Partition nóng và partition lớn<br>`cassandra-hot-large-partition` | CB | Phát hiện bằng `tablestats`, `toppartitions`, cảnh báo log; sửa bằng bucketing | Dự kiến |
| CS-034 | Lightweight transaction và Paxos<br>`cassandra-lightweight-transaction-paxos` | TB | `IF NOT EXISTS`/`IF cond`, `[applied]`, SERIAL/LOCAL_SERIAL, chi phí bốn vòng | Dự kiến |
| CS-035 | Mô hình time-series cho vị trí tài xế<br>`cassandra-time-series-vi-tri-tai-xe` | TB | Bucket theo tài xế + 5 phút, TTL, TWCS, đọc "vị trí mới nhất" | Dự kiến |
| CS-036 | Đọc toàn bảng: paging và quét theo token range<br>`cassandra-paging-token-range` | TB | Duyệt bảng lớn không timeout, chia token range cho nhiều worker | Dự kiến |
| CS-037 | Driver ứng dụng: prepared statement, token-aware, retry và idempotent<br>`cassandra-driver-prepared-retry` | TB | Cấu hình driver đúng, đánh dấu câu idempotent, speculative execution | Dự kiến |
| CS-038 | Đo tải bằng cassandra-stress<br>`cassandra-stress-benchmark` | NC | Profile YAML cho bảng vị trí, đọc p99, giới hạn kết luận trên laptop | Dự kiến |
| CS-039 | Idempotency và chống trùng sự kiện<br>`cassandra-idempotency-chong-trung` | NC | Khoá idempotency bằng LWT có TTL, chèn trùng an toàn, đối soát | Dự kiến |
| CS-040 | Thực hành: hai tài xế cùng nhận một chuyến<br>`cassandra-thuc-hanh-tranh-chap-nhan-chuyen` | NC | Bàn giao kịch bản đồng thời, bất biến một người thắng, số đo LWT | Dự kiến |

## P05 · Bảo mật, bảo trì và phục hồi

Đầu vào: P01–P04. Đầu ra: runbook xác thực, giám sát, repair, snapshot/restore, thêm và thay node. Lộ trình dự kiến `-phan-05`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| CS-041 | Xác thực, role và quyền tối thiểu<br>`cassandra-xac-thuc-role-quyen` | CB | Bật PasswordAuthenticator, tách role ứng dụng/báo cáo, RF của `system_auth` | Dự kiến |
| CS-042 | TLS cho client và giữa các node<br>`cassandra-tls-client-internode` | CB | Cấu hình keystore, bật TLS client, kiểm từ DBStudio và cqlsh | Dự kiến |
| CS-043 | Giám sát: nodetool, virtual table và metric<br>`cassandra-giam-sat-nodetool-virtual-table` | CB | Đọc `system_views`, tpstats, latency, dropped message; dựng danh sách cảnh báo | Dự kiến |
| CS-044 | Repair định kỳ và gc_grace_seconds<br>`cassandra-repair-gc-grace` | TB | Lịch repair, incremental vs full, dữ liệu "sống lại" khi quên repair | Dự kiến |
| CS-045 | Snapshot, incremental backup và restore<br>`cassandra-snapshot-backup-restore` | TB | `nodetool snapshot`, backup logic bằng DBStudio, restore vào keyspace riêng và đối soát | Dự kiến |
| CS-046 | Thêm và bớt node: bootstrap, decommission, cleanup<br>`cassandra-them-bot-node` | TB | Thêm node 4, theo dõi streaming, cleanup, bớt node an toàn | Dự kiến |
| CS-047 | Thay node chết bằng replace_address<br>`cassandra-thay-node-chet` | TB | Thay node mất đĩa, so với removenode, kiểm dữ liệu sau thay | Dự kiến |
| CS-048 | Thay đổi schema an toàn<br>`cassandra-thay-doi-schema-an-toan` | NC | ALTER thêm cột, không đổi khoá chính, migrate sang bảng mới bằng dual write + backfill | Dự kiến |
| CS-049 | Guardrails, audit log và full query log<br>`cassandra-guardrails-audit-log` | NC | Đặt guardrail chống truy vấn nguy hiểm, bật audit log, ghi lại tải để phát lại | Dự kiến |
| CS-050 | Thực hành: runbook vận hành và phục hồi<br>`cassandra-thuc-hanh-runbook-phuc-hoi` | NC | Bàn giao runbook có bằng chứng restore, repair và thay node | Dự kiến |

## P06 · Mở rộng, tích hợp và vận hành đầu cuối

Đầu vào: P01–P05. Đầu ra: capstone vận hành Cassandra cho dữ liệu sự kiện Tới Nơi, có kế hoạch thay đổi và diễn tập. Lộ trình dự kiến `-phan-06`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| CS-051 | Nhiều datacenter và LOCAL_QUORUM<br>`cassandra-nhieu-datacenter` | CB | Thêm dc2 cho đọc báo cáo, chọn CL theo DC, chi phí độ trễ | Dự kiến |
| CS-052 | Cassandra trong kiến trúc Tới Nơi: ranh giới với PostgreSQL<br>`cassandra-ranh-gioi-voi-postgresql` | CB | Phân dữ liệu nào ở đâu, nguồn dữ liệu gốc, đường đồng bộ | Dự kiến |
| CS-053 | Vector search với SAI (Cassandra 5.0)<br>`cassandra-vector-search-sai` | CB | Kiểu `vector`, index SAI, tìm quán gần giống theo embedding; giới hạn | Dự kiến |
| CS-054 | Change Data Capture của Cassandra<br>`cassandra-change-data-capture` | TB | Bật `cdc=true`, đọc commit log CDC, so với outbox phía ứng dụng | Dự kiến |
| CS-055 | Xuất dữ liệu sang phân tích<br>`cassandra-xuat-du-lieu-phan-tich` | TB | Export bằng DBStudio/`COPY TO`, quét token range ra Parquet, không làm phiền tải chính | Dự kiến |
| CS-056 | ScyllaDB và các hệ tương thích CQL<br>`cassandra-scylladb-tuong-thich` | TB | Chạy cùng schema trên ScyllaDB, khác biệt kiến trúc, khi nào cân nhắc | Dự kiến |
| CS-057 | Capacity planning và chi phí<br>`cassandra-capacity-planning` | TB | Ước lượng dung lượng × RF × compaction, số node, ngân sách đĩa | Dự kiến |
| CS-058 | Nâng phiên bản rolling 4.1 → 5.0<br>`cassandra-nang-cap-rolling` | NC | Diễn tập nâng cấp từng node, `upgradesstables`, cổng go/no-go | Dự kiến |
| CS-059 | Di chuyển dữ liệu từ PostgreSQL sang Cassandra<br>`cassandra-di-chuyen-tu-postgresql` | NC | Dual write, backfill, đối soát, cutover và hoàn tác cho bảng vị trí | Dự kiến |
| CS-060 | Capstone: vận hành dữ liệu sự kiện Tới Nơi trên Cassandra<br>`cassandra-capstone-van-hanh-toi-noi` | NC | Bàn giao mô hình, CL, số đo, runbook, diễn tập và quyết định kiến trúc | Dự kiến |
