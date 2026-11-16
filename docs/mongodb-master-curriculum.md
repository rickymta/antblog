# MongoDB 8.0 qua dự án Tới Nơi

Khung biên tập v1 · 16/11/2026 · **6 phần × 10 bài = 60 bài**. Mỗi phần có **3 cơ bản / 4 trung bình / 3 nâng cao**, gồm cả bài thực hành cuối phần. Đây là khung viết bài; hoàn thành khung không có nghĩa đã hoàn thành nội dung.

## Phạm vi và cách đọc

Series riêng cho **MongoDB 8.0** (Community Server) thuộc chuyên mục cha Database, chuyên mục con `mongodb`. Dùng xuyên suốt hệ thống hư cấu **Tới Nơi** (đặt xe, giao đồ ăn, giao bưu kiện) như series PostgreSQL và SQL Server, cùng dữ liệu mẫu, nhưng **thiết kế theo cơ chế của MongoDB**, không dịch bảng quan hệ sang collection: document và BSON, nhúng hay tham chiếu theo cách đọc, validator thay cho ràng buộc khai báo, nguyên tử trong một document, aggregation pipeline, replica set và oplog, sharding. Đối tượng: người mới, backend developer và người vận hành database.

Đọc hồ sơ chung `du-an-mau-toi-noi` trước P01; P01 không yêu cầu kiến thức MongoDB hay SQL. Người đã học series PostgreSQL/SQL Server có thể đọc song song: mỗi bài có mục "So với PostgreSQL / SQL Server" ngắn khi cơ chế khác nhau thật sự. CB = `beginner`, TB = `intermediate`, NC = `advanced`; toàn khung có 18 CB / 24 TB / 18 NC.

| Phần | Chủ đề | Đầu vào | Sản phẩm cuối phần |
|---|---|---|---|
| P01 | Nền tảng và mô hình document | Hồ sơ Tới Nơi | Collection, validator, fixture và kiểm tra bất biến trên MongoDB 8.0 |
| P02 | Truy vấn và aggregation cho nghiệp vụ | P01 | Bộ pipeline báo cáo và đối soát có expected output |
| P03 | Index, explain và hiệu năng | P01–P02 | Báo cáo đo trước/sau index, bằng chứng `explain("executionStats")` |
| P04 | Nguyên tử, transaction và đồng thời | P01–P03 | Kịch bản tranh chấp nhận chuyến, read/write concern, có kiểm bất biến |
| P05 | Bảo mật, vận hành và phục hồi | P01–P04 | Runbook xác thực/phân quyền, backup/restore đã đối soát |
| P06 | Replica set, sharding và tích hợp | P01–P05 | Diễn tập failover, change stream, khoá shard và nâng cấp |

## Môi trường và công cụ

- **Engine:** MongoDB Community **8.0** chạy bằng **Docker Desktop**, ảnh chính thức `mongo:8.0` ghim theo digest. Bản đã kiểm chứng khi viết P01: **8.0.38**. Chạy dưới dạng **replica set một node** (`rs0`) ngay từ P01 để P04 (transaction) và P06 (change stream, oplog) dùng cùng môi trường. Compose và runner ở `examples/toi-noi-mongodb`. P06 thêm node vào compose khi cần.
- **Công cụ GUI trong bài:** **DBStudio** (công cụ quản trị đa database của tác giả, Rust + Tauri, bản phát hành 0.1.7). Bài dùng DBStudio cho kết nối bằng URI, cây database/collection, lưới document, query editor chạy cú pháp mongosh, xem kết quả, explain, export và backup (`mongodump`). **Ảnh chụp màn hình trong bài lấy từ DBStudio 0.1.7 trên Windows 11** chạy với lab thật. Khi DBStudio chưa hỗ trợ một việc, bài dùng `mongosh` trong container và nói rõ.
- **Chạy lab tái lập:** `mongosh` **bên trong container**, gọi qua `node run.mjs`; không cần cài MongoDB hay mongosh trên máy. Runner không có dependency npm.
- Không dùng tính năng chỉ có trên MongoDB Atlas (Atlas Search, Vector Search…) hay Enterprise (LDAP, auditing, encryption at rest) trong lab; bài nhắc tới thì ghi rõ "không có trong Community".
- **Lab P01–P04 không bật xác thực**, cổng chỉ mở trên `127.0.0.1`; P05 bật xác thực và phân quyền. Bài nhắc điều này ở mọi chỗ hướng dẫn kết nối.

## Tổ chức Database

Theo quy ước trong [`postgresql-master-curriculum.md`](postgresql-master-curriculum.md): `database` là chuyên mục cha; mỗi engine có chuyên mục con, curriculum và lộ trình riêng. MongoDB dùng slug chuyên mục `mongodb`, tiền tố bài `mongodb-`, mã bài `mg-0NN`, lộ trình `mongodb-8-qua-toi-noi` (P01) rồi `mongodb-8-qua-toi-noi-phan-02` … `-phan-06`. Chỉ thêm vào `api/seed/learning-paths.json` các bài đã viết và kiểm tra.

Bài MongoDB đã có ở series khác giữ chuyên mục cũ và được liên kết khi cần: `python-mongodb-pymongo-motor-document-ocr`, `python-luu-ket-qua-ocr-mongodb-pii-retention` (Python), các bài DynamoDB/Cosmos DB (AWS/Azure) dùng để so sánh ở P06.

## Tiêu chuẩn viết và hoàn tất một phần

Frontmatter: `slug`, `title`, `summary`, `category: mongodb`, `level`, `tags` (gồm mã bài `mg-0NN`), `language: vi`. Giọng văn tiếng Việt theo [`content-dictionary/style-guide.md`](content-dictionary/style-guide.md) và [`terminology.md`](content-dictionary/terminology.md); giải nghĩa thuật ngữ lần đầu; mở bằng vấn đề Tới Nơi; trình bày cơ chế; ví dụ chạy thật kèm kết quả lấy từ `OUTPUT.txt`; lỗi thường gặp; đánh đổi; bài tập; đọc tiếp. Không suy rộng cơ chế MongoDB cho mọi database document. Số đo hiệu năng ghi cấu hình máy, số document, cache nóng/lạnh, số lần đo; không bịa số.

**Ảnh minh hoạ:** chụp từ cửa sổ DBStudio thật (giao diện tiếng Việt, theme sáng) với lab đang chạy, cùng quy ước với series PostgreSQL: `apps/web/public/images/articles/<slug>/dbstudio-NN.webp`, nhúng bằng `<figure class="tkh-fig"><div class="tkh-fig-box"><img … width height loading="lazy"></div><figcaption>…</figcaption></figure>`, có `alt` mô tả nội dung. P01 có 16 ảnh, chụp từ DBStudio 0.1.7 (build từ repo, thư mục dữ liệu riêng) trên Windows 11, cửa sổ 1440 px, cắt vùng editor rộng 1086 px. Ảnh không thay cho khối code/kết quả dạng chữ (người đọc cần chép được); ảnh cho thấy *thao tác ở đâu* và *giao diện trả lời gì*. Chụp lại khi DBStudio đổi giao diện đáng kể.

**Giới hạn DBStudio 0.1.7 với MongoDB đã gặp khi chụp P01** (bài viết theo đúng hành vi này; sửa bài khi DBStudio thay đổi): double-click collection trên cây không mở lưới document (`collection` chưa có trong `TABLE_KINDS` của `src/features/explorer/treeModel.ts`), nên mọi ảnh dữ liệu dùng query editor; chế độ **Bảng** gộp mỗi document vào một cột `$document` (chưa tách trường thành cột), chế độ **Tài liệu** hiện Extended JSON (`$uuid`, `$date` theo UTC); lỗi 121 chỉ hiện thông báo, chưa kèm `errInfo`; lệnh ghi chỉ báo "Ảnh hưởng N dòng", không tách `matchedCount`/`modifiedCount`; query editor hiểu một tập con cú pháp mongosh (`UUID()` không tham số bị từ chối). Index xem được ở `collection › Chỉ mục` trên cây; double-click index mở script DDL có `createCollection` kèm validator.

**Từ điển keyword:** thuật ngữ quan trọng của mỗi phần được đề xuất ở `content-dictionary/glossary-candidates/mongodb.json` (cùng cấu trúc với file PostgreSQL: `terms`, `conflicts`, `deferred`, đếm `occurrences` trên bài thật). Định nghĩa trung lập với engine khi thuật ngữ dùng chung (index, collation…) và nêu rõ MongoDB khi là khái niệm riêng (BSON, ObjectId, oplog…). Không đề xuất từ quá chung dễ bắt nhầm ("document", "collection" để `autoLink: false`).

Hoàn thành **nội dung** một phần: cập nhật trạng thái ở đây, seed lộ trình, README lab, chạy `npm test` của lab, kiểm lint thuật ngữ, rồi commit riêng `content(mongodb): complete part N`. Seed/deploy là bước riêng.

## P01 · Nền tảng và mô hình document

Đầu vào: hồ sơ chung Tới Nơi; chưa cần biết database. Đầu ra: collection cho thành phố, khách, tài xế, quán (menu nhúng) và chuyến (lịch sử trạng thái nhúng); validator `$jsonSchema` + quy tắc liên trường; index unique; fixture hợp lệ/sai; kiểm tra bất biến liên collection.

Lộ trình: `mongodb-8-qua-toi-noi` — **đã viết đủ 10 bài ngày 16/11/2026**, đã thêm vào `api/seed/learning-paths.json`. Lab: `examples/toi-noi-mongodb/p01` (MongoDB **8.0.38 thật** trong Docker, replica set một node; 8 file, 115 kiểm tra PASS, `npm test` 4/4; `OUTPUT.txt`; thí nghiệm kiểu BSON ở `p01/thu-nghiem/`, 13 PASS). 16 ảnh DBStudio 0.1.7 ở `apps/web/public/images/articles/mongodb-*/`. Thuật ngữ đề xuất: `content-dictionary/glossary-candidates/mongodb.json` (15 mục, chưa nạp vào `terminology.json`). Chưa seed/deploy lên website.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| MG-001 | MongoDB là gì và nằm ở đâu trong Tới Nơi?<br>`mongodb-la-gi-va-vai-tro-trong-toi-noi` | CB | Phân biệt instance, database, collection, document; replica set; khi nào chọn document database | Đã viết |
| MG-002 | Dựng MongoDB 8.0 bằng Docker và kết nối bằng DBStudio<br>`mongodb-cai-dat-docker-dbstudio` | CB | Chạy lab replica set, kết nối bằng URI, chạy lệnh mongosh đầu tiên, dọn môi trường | Đã viết |
| MG-003 | Document, BSON và `_id`: một chuyến xe trông thế nào<br>`mongodb-document-bson-id` | CB | Đọc một document lồng nhau, chọn `_id`, phân biệt trường vắng và `null` | Đã viết |
| MG-004 | Kiểu BSON cho tiền, thời gian, chuỗi tiếng Việt và định danh<br>`mongodb-kieu-bson-tien-thoi-gian-unicode` | TB | Lập data dictionary: int/long/Decimal128/double, Date và múi giờ, UTF-8, ObjectId/UUID | Đã viết |
| MG-005 | Schema validation với `$jsonSchema` và index unique<br>`mongodb-schema-validation-unique-index` | TB | Viết validator có quy tắc liên trường, đọc lỗi 121 và `errInfo`, giữ khoá tự nhiên bằng index unique | Đã viết |
| MG-006 | insert, update, delete và toán tử cập nhật<br>`mongodb-insert-update-delete` | TB | Đi hết vòng đời một chuyến bằng update có điều kiện, `$push` lịch sử, `findOneAndUpdate`, bộ đếm mã | Đã viết |
| MG-007 | find: lọc, projection, sắp xếp, phân trang và collation<br>`mongodb-find-loc-sap-xep-collation` | TB | Lọc theo ngày giờ VN, thứ tự xác định, `null` vs trường vắng, tìm tiếng Việt có/không dấu | Đã viết |
| MG-008 | Nhúng hay tham chiếu: mô hình document cho Tới Nơi<br>`mongodb-nhung-hay-tham-chieu` | NC | Quyết định nhúng/tham chiếu theo cách đọc, giới hạn 16 MB và mảng không giới hạn, kiểm tham chiếu bằng `$lookup` | Đã viết |
| MG-009 | Đổi schema khi không có ALTER TABLE<br>`mongodb-schema-migration-tuong-thich` | NC | Expand → backfill → contract với `collMod`, `validationLevel`/`validationAction`, `schema_version`, hoàn tác | Đã viết |
| MG-010 | Thực hành: mô hình chuyến xe có kiểm tra ràng buộc<br>`mongodb-thuc-hanh-mo-hinh-chuyen-xe` | NC | Bàn giao validator, fixture hợp lệ/sai, bất biến liên collection và bằng chứng chạy | Đã viết |

## P02 · Truy vấn và aggregation cho nghiệp vụ

Đầu vào: P01. Đầu ra: pipeline có expected output, đối soát số document và tổng tiền. Lộ trình dự kiến `mongodb-8-qua-toi-noi-phan-02`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| MG-011 | Toán tử truy vấn: so sánh, logic, mảng và `$expr`<br>`mongodb-toan-tu-truy-van` | CB | Viết bộ lọc nghiệp vụ, biết khi nào cần `$expr` | Dự kiến |
| MG-012 | Aggregation pipeline: `$match`, `$group`, `$project`, `$sort`<br>`mongodb-aggregation-pipeline-co-ban` | CB | Báo cáo theo thành phố/dịch vụ, mẫu số rõ ràng | Dự kiến |
| MG-013 | `$lookup` và số document sau khi nối<br>`mongodb-lookup-cardinality` | CB | Nối chuyến–khách–tài xế không nhân đôi tổng tiền; `$lookup` có pipeline | Dự kiến |
| MG-014 | Làm việc với mảng: `$unwind`, `$filter`, `$map`, `$reduce`<br>`mongodb-mang-unwind-filter-map` | TB | Phân tích lịch sử trạng thái nhúng, thời gian từng chặng | Dự kiến |
| MG-015 | `$setWindowFields`: luỹ kế, xếp hạng, trước/sau<br>`mongodb-set-window-fields` | TB | Doanh thu luỹ kế, thứ hạng tài xế, khoảng cách giữa hai chuyến | Dự kiến |
| MG-016 | Ngày giờ: `$dateTrunc`, múi giờ và gom theo khung<br>`mongodb-ngay-gio-date-trunc-mui-gio` | TB | Ngày nghiệp vụ theo giờ VN, gom 15 phút, `$densify` | Dự kiến |
| MG-017 | Upsert và `$merge`: nhập dữ liệu và bảng tổng hợp<br>`mongodb-upsert-merge-out` | TB | Nhập dữ liệu có quy tắc cập nhật; materialize báo cáo bằng `$merge` | Dự kiến |
| MG-018 | Phân trang lịch sử chuyến: skip/limit và keyset<br>`mongodb-phan-trang-keyset` | NC | Chứng minh điều kiện trang kế tiếp với khoá sắp xếp trùng | Dự kiến |
| MG-019 | `$graphLookup` với cây khu vực<br>`mongodb-graphlookup-cay-khu-vuc` | NC | Duyệt cây có giới hạn độ sâu, phát hiện chu trình | Dự kiến |
| MG-020 | Thực hành: pipeline vận hành và đối soát thanh toán<br>`mongodb-thuc-hanh-aggregation-doi-soat` | NC | Bàn giao pipeline, expected output và giải thích chênh lệch | Dự kiến |

## P03 · Index, explain và hiệu năng

Đầu vào: P01–P02. Đầu ra: báo cáo đo trước/sau index với `explain("executionStats")`, giới hạn kết luận. Lộ trình dự kiến `-phan-03`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| MG-021 | WiredTiger lưu document thế nào: cache, nén, checkpoint, journal<br>`mongodb-wiredtiger-luu-tru` | CB | Đọc `collStats`, giải thích kích thước trên đĩa | Dự kiến |
| MG-022 | Index B-tree đơn và kép, quy tắc ESR<br>`mongodb-index-don-kep-esr` | CB | Chọn thứ tự khoá theo Equality–Sort–Range | Dự kiến |
| MG-023 | Đọc explain: COLLSCAN, IXSCAN, FETCH, SORT<br>`mongodb-doc-explain` | CB | Đọc `executionStats` trong DBStudio, so `nReturned` với `totalDocsExamined` | Dự kiến |
| MG-024 | Covered query, multikey index và index trên mảng<br>`mongodb-covered-query-multikey` | TB | Truy vấn không FETCH, chi phí index trên mảng lịch sử | Dự kiến |
| MG-025 | Partial, sparse, TTL và unique có điều kiện<br>`mongodb-partial-ttl-index` | TB | Index cho chuyến đang chạy, tự xoá dữ liệu tạm | Dự kiến |
| MG-026 | Geospatial: `2dsphere` tìm tài xế gần nhất<br>`mongodb-geospatial-2dsphere` | TB | `$near`/`$geoNear` với GeoJSON, đo và giới hạn | Dự kiến |
| MG-027 | Query planner, plan cache và hint<br>`mongodb-query-planner-plan-cache` | TB | Giải thích vì sao planner chọn index, khi nào hint | Dự kiến |
| MG-028 | Profiler, slow query log và `$indexStats`<br>`mongodb-profiler-index-stats` | NC | Tìm truy vấn chậm, index thừa | Dự kiến |
| MG-029 | Text search và collation index cho tiếng Việt<br>`mongodb-text-search-tieng-viet` | NC | Tìm quán/món có/không dấu, giới hạn so với Atlas Search | Dự kiến |
| MG-030 | Thực hành: báo cáo đo và lựa chọn index<br>`mongodb-thuc-hanh-index-hieu-nang` | NC | Bàn giao báo cáo trước/sau có cấu hình máy, số đo, giới hạn | Dự kiến |

## P04 · Nguyên tử, transaction và đồng thời

Đầu vào: P01–P03. Đầu ra: kịch bản tranh chấp nhận chuyến có kiểm bất biến. Lộ trình dự kiến `-phan-04`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| MG-031 | Nguyên tử trong một document và update có điều kiện<br>`mongodb-nguyen-tu-mot-document` | CB | Thiết kế để một thao tác ghi giữ được bất biến | Dự kiến |
| MG-032 | Write concern và journal: khi nào "đã ghi" là đã ghi<br>`mongodb-write-concern-journal` | CB | Chọn `w`, `j`, `wtimeout` cho dữ liệu tiền | Dự kiến |
| MG-033 | Read concern và read preference<br>`mongodb-read-concern-read-preference` | CB | Đọc từ secondary an toàn tới đâu | Dự kiến |
| MG-034 | Transaction nhiều document: cú pháp, giới hạn, lỗi tạm thời<br>`mongodb-transaction-nhieu-document` | TB | Gán tài xế và đổi trạng thái chuyến trong một transaction, retry đúng | Dự kiến |
| MG-035 | Snapshot isolation và write conflict<br>`mongodb-snapshot-write-conflict` | TB | Tái hiện `WriteConflict`, `TransientTransactionError` | Dự kiến |
| MG-036 | Causal consistency và session<br>`mongodb-causal-consistency-session` | TB | Đọc lại chính dữ liệu vừa ghi khi đọc từ secondary | Dự kiến |
| MG-037 | Retryable write và idempotency<br>`mongodb-retryable-write-idempotency` | TB | Chống tạo trùng chuyến khi mạng chập chờn | Dự kiến |
| MG-038 | Hàng đợi việc bằng `findOneAndUpdate` và khoá mềm<br>`mongodb-hang-doi-khoa-mem` | NC | Worker nhận việc không trùng, hết hạn khoá | Dự kiến |
| MG-039 | Chọn: nhúng để nguyên tử hay transaction<br>`mongodb-nhung-hay-transaction` | NC | Đánh đổi thiết kế có số đo | Dự kiến |
| MG-040 | Thực hành: tranh chấp nhận chuyến và kiểm bất biến<br>`mongodb-thuc-hanh-tranh-chap-nhan-chuyen` | NC | Bàn giao kịch bản đồng thời, kết quả lặp lại và kiểm bất biến | Dự kiến |

## P05 · Bảo mật, vận hành và phục hồi

Đầu vào: P01–P04. Đầu ra: runbook phân quyền, giám sát, backup/restore đã đối soát. Lộ trình dự kiến `-phan-05`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| MG-041 | Bật xác thực: SCRAM, user và keyfile cho replica set<br>`mongodb-xac-thuc-scram-keyfile` | CB | Chuyển lab sang có xác thực không mất dữ liệu | Dự kiến |
| MG-042 | Phân quyền theo vai trò: built-in và custom role<br>`mongodb-phan-quyen-role` | CB | Tách tài khoản ứng dụng, migration, báo cáo | Dự kiến |
| MG-043 | TLS và mạng: bind, firewall, chuỗi kết nối<br>`mongodb-tls-mang` | CB | Kết nối TLS từ DBStudio và ứng dụng | Dự kiến |
| MG-044 | Bảo vệ dữ liệu cá nhân: field-level và Queryable Encryption<br>`mongodb-bao-ve-du-lieu-ca-nhan` | TB | Mã hoá số điện thoại, giới hạn của Community | Dự kiến |
| MG-045 | Backup logic: `mongodump`/`mongorestore` và DBStudio<br>`mongodb-backup-mongodump-restore` | TB | Backup có kiểm, restore vào môi trường riêng và đối soát | Dự kiến |
| MG-046 | Backup vật lý, snapshot và point-in-time bằng oplog<br>`mongodb-backup-snapshot-oplog-pitr` | TB | Khôi phục tới thời điểm trước lệnh xoá nhầm | Dự kiến |
| MG-047 | Giám sát: `serverStatus`, `currentOp`, log có cấu trúc<br>`mongodb-giam-sat-server-status` | TB | Bảng chỉ số tối thiểu và ngưỡng cảnh báo | Dự kiến |
| MG-048 | Dữ liệu tăng mãi: TTL, archive, capped và time series collection<br>`mongodb-ttl-archive-time-series` | NC | Chính sách lưu giữ vị trí GPS và log | Dự kiến |
| MG-049 | Compact, resync và sự cố đĩa đầy<br>`mongodb-compact-resync-dia-day` | NC | Runbook khi dung lượng không giảm sau khi xoá | Dự kiến |
| MG-050 | Thực hành: runbook phân quyền, backup và restore đối soát<br>`mongodb-thuc-hanh-runbook-phuc-hoi` | NC | Bàn giao runbook và bằng chứng restore | Dự kiến |

## P06 · Replica set, sharding và tích hợp

Đầu vào: P01–P05. Đầu ra: hồ sơ vận hành, diễn tập failover, khoá shard và nâng cấp. Lộ trình dự kiến `-phan-06`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| MG-051 | Replica set ba node: bầu chọn, oplog và độ trễ sao chép<br>`mongodb-replica-set-bau-chon-oplog` | CB | Mở rộng lab lên ba node, đọc `rs.status()` | Dự kiến |
| MG-052 | Failover và ứng dụng: retry, timeout, đọc sau failover<br>`mongodb-failover-ung-dung` | CB | Diễn tập tắt primary, đo thời gian gián đoạn | Dự kiến |
| MG-053 | Change stream: phát sự kiện trạng thái chuyến<br>`mongodb-change-stream` | CB | Đẩy sự kiện ra hàng đợi, resume token | Dự kiến |
| MG-054 | Sharding: khi nào cần và kiến trúc cụm<br>`mongodb-sharding-kien-truc` | TB | Đánh giá tải Tới Nơi trước khi shard | Dự kiến |
| MG-055 | Chọn khoá shard: hashed, ranged, zone theo thành phố<br>`mongodb-chon-khoa-shard` | TB | Đánh giá khoá shard cho chuyến, tránh hot shard | Dự kiến |
| MG-056 | Ứng dụng .NET/Node/Python với MongoDB: driver, pool, mapping<br>`mongodb-driver-ung-dung` | TB | Cấu hình pool, timeout, mapping kiểu tiền và thời gian | Dự kiến |
| MG-057 | MongoDB cạnh PostgreSQL: chia trách nhiệm và đồng bộ<br>`mongodb-canh-postgresql-polyglot` | TB | Ranh giới dữ liệu, outbox, đối soát hai nguồn | Dự kiến |
| MG-058 | Nâng cấp phiên bản và feature compatibility version<br>`mongodb-nang-cap-fcv` | NC | Rolling upgrade, hoàn tác FCV | Dự kiến |
| MG-059 | Reshard và di chuyển dữ liệu không dừng hệ thống<br>`mongodb-reshard-di-chuyen-du-lieu` | NC | Kế hoạch reshard có kiểm tra | Dự kiến |
| MG-060 | Thực hành: hồ sơ vận hành và diễn tập sự cố<br>`mongodb-thuc-hanh-ho-so-van-hanh` | NC | Bàn giao hồ sơ, kết quả diễn tập failover và nâng cấp | Dự kiến |
