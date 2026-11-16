# SQL Server 2022 qua dự án Tới Nơi

Khung biên tập v1 · 16/11/2026 · **6 phần × 10 bài = 60 bài**. Mỗi phần có **3 cơ bản / 4 trung bình / 3 nâng cao**, gồm cả bài thực hành cuối phần. Đây là khung viết bài; hoàn thành khung không có nghĩa đã hoàn thành nội dung.

## Phạm vi và cách đọc

Series riêng cho **Microsoft SQL Server 2022** (bản 16.x) thuộc chuyên mục cha Database, chuyên mục con `mssql`. Dùng xuyên suốt hệ thống hư cấu **Tới Nơi** (đặt xe, giao đồ ăn, giao bưu kiện) như series PostgreSQL, nhưng **thiết kế theo cơ chế của SQL Server**, không dịch cú pháp từ PostgreSQL: clustered index quyết định cách lưu bảng, khoá và versioning trong tempdb, Query Store, transaction log và recovery model, Always On availability group. Đối tượng: người mới, backend developer (.NET và các nền tảng khác) và người vận hành database.

Đọc hồ sơ chung `du-an-mau-toi-noi` trước P01; P01 không yêu cầu kiến thức SQL Server. Người đã học series PostgreSQL có thể đọc song song: mỗi bài có mục "So với PostgreSQL" ngắn khi cơ chế khác nhau thật sự. CB = `beginner`, TB = `intermediate`, NC = `advanced`; toàn khung có 18 CB / 24 TB / 18 NC.

| Phần | Chủ đề | Đầu vào | Sản phẩm cuối phần |
|---|---|---|---|
| P01 | Nền tảng và mô hình dữ liệu | Hồ sơ Tới Nơi | Schema, fixture và kiểm tra ràng buộc trên SQL Server 2022 |
| P02 | T-SQL cho nghiệp vụ | P01 | Bộ truy vấn T-SQL và đối soát |
| P03 | Lưu trữ, index và execution plan | P01–P02 | Báo cáo đo, lựa chọn index và bằng chứng Query Store |
| P04 | Giao dịch, khoá và đồng thời | P01–P03 | Kịch bản tranh chấp dưới locking và RCSI, có kiểm bất biến |
| P05 | Bảo mật, bảo trì và phục hồi | P01–P04 | Runbook phân quyền, bảo trì, backup/restore đã đối soát |
| P06 | Mở rộng, tích hợp và vận hành đầu cuối | P01–P05 | Hồ sơ vận hành, diễn tập failover và nâng cấp |

## Môi trường và công cụ

- **Engine:** SQL Server 2022 Developer Edition trên Linux, chạy bằng **Docker Desktop**; ảnh `mcr.microsoft.com/mssql/server:2022-latest` ghim theo digest. Bản đã kiểm chứng khi viết P01: **16.0.4325.4 (RTM-CU28)**. Compose và runner ở `examples/toi-noi-mssql`. Phần nào cần nhiều instance (P06: availability group, log shipping) dùng thêm service trong cùng compose.
- **Công cụ GUI trong bài:** **DBStudio** (công cụ quản trị đa database của tác giả, Rust + Tauri, bản phát hành 0.1.7 có bộ cài Windows/macOS/Linux). Bài dùng DBStudio cho kết nối, cây đối tượng, query editor (chạy theo batch `GO`), kết quả nhiều result set, tab Thông điệp, chế độ transaction thủ công, EXPLAIN (SHOWPLAN_XML / STATISTICS XML), ERD, export và backup native. Khi DBStudio chưa hỗ trợ một việc (ví dụ restore file `.bak`, Extended Events), bài dùng `sqlcmd` trong container và nói rõ.
- **Chạy lab tái lập:** `sqlcmd` (mssql-tools18) **bên trong container**, gọi qua `node run.mjs`; không cần cài SQL Server hay sqlcmd trên máy. Runner không có dependency npm.
- Không dùng tính năng chỉ có trên Azure SQL. Tính năng Enterprise (online index rebuild, partition lớn…) dùng được vì Developer Edition có đủ; bài phải ghi rõ khi một tính năng **không có ở Standard/Express**.

## Tổ chức Database

Theo quy ước trong [`postgresql-master-curriculum.md`](postgresql-master-curriculum.md): `database` là chuyên mục cha; mỗi engine có chuyên mục con, curriculum và lộ trình riêng. SQL Server dùng slug chuyên mục `mssql`, tiền tố bài `mssql-`, lộ trình `mssql-2022-qua-toi-noi` (P01) rồi `mssql-2022-qua-toi-noi-phan-02` … `-phan-06`. Chỉ thêm vào `api/seed/learning-paths.json` các bài đã viết và kiểm tra.

## Tiêu chuẩn viết và hoàn tất một phần

Frontmatter: `slug`, `title`, `summary`, `category: mssql`, `level`, `tags` (gồm mã bài `ms-0NN`), `language: vi`. Giọng văn tiếng Việt theo [`content-dictionary/style-guide.md`](content-dictionary/style-guide.md) và [`terminology.md`](content-dictionary/terminology.md); giải nghĩa thuật ngữ lần đầu; mở bằng vấn đề Tới Nơi; trình bày cơ chế; ví dụ chạy thật kèm kết quả lấy từ `OUTPUT.txt`; lỗi thường gặp; đánh đổi; bài tập; đọc tiếp. Không suy rộng cơ chế SQL Server cho mọi database. Số đo hiệu năng ghi cấu hình máy, số dòng, cache nóng/lạnh, số lần đo; không bịa số.

**Từ điển keyword:** thuật ngữ mới quan trọng của mỗi phần được chuẩn bị trong `api/seed/terminology.json` (`glossary: true`, `definition` ≤ 300 ký tự, `english`), định nghĩa trung lập với engine khi thuật ngữ dùng chung (clustered index, collation…) và nêu rõ SQL Server khi là khái niệm riêng (IDENTITY, tempdb…). Không thêm từ quá chung dễ bắt nhầm.

Hoàn thành **nội dung** một phần: cập nhật trạng thái ở đây, seed lộ trình, README lab, chạy `npm test` của lab, kiểm lint thuật ngữ, rồi commit riêng `content(mssql): complete part N`. Seed/deploy là bước riêng.

## P01 · Nền tảng và mô hình dữ liệu

Đầu vào: hồ sơ chung Tới Nơi; chưa cần biết SQL. Đầu ra: schema tối thiểu cho khách, tài xế, quán, chuyến và lịch sử trạng thái; fixture hợp lệ/sai; kiểm tra bất biến.

Lộ trình: `mssql-2022-qua-toi-noi` — **đã viết đủ 10 bài ngày 16/11/2026**. Lab: `examples/toi-noi-mssql/p01` (SQL Server **2022 CU28 thật** trong Docker; 8 file, 80 kiểm tra PASS; `OUTPUT.txt`; thí nghiệm kiểu dữ liệu ở `p01/thu-nghiem/`). Chưa seed/deploy lên website.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| MS-001 | SQL Server 2022 là gì và nằm ở đâu trong Tới Nơi?<br>`mssql-la-gi-va-vai-tro-trong-toi-noi` | CB | Phân biệt instance, database, schema, bảng; edition; SQL Server trên Linux/container | Đã viết |
| MS-002 | Dựng SQL Server 2022 bằng Docker và kết nối bằng DBStudio<br>`mssql-cai-dat-docker-dbstudio` | CB | Chạy lab, kết nối, chạy batch đầu tiên, đọc phiên bản, dọn môi trường | Đã viết |
| MS-003 | Bảng, khoá chính và clustered index trong nghiệp vụ chuyến xe<br>`mssql-bang-khoa-chinh-clustered-index` | CB | Chọn khoá chính và clustered key, vẽ quan hệ khách–chuyến–tài xế | Đã viết |
| MS-004 | Kiểu dữ liệu cho tiền, thời gian, chuỗi tiếng Việt và định danh<br>`mssql-kieu-du-lieu-tien-thoi-gian-unicode` | TB | Lập data dictionary: int/decimal/money, datetimeoffset, nvarchar/UTF-8, GUID | Đã viết |
| MS-005 | CREATE TABLE, ràng buộc và IDENTITY<br>`mssql-create-table-constraints` | TB | Viết DDL có PK, FK, UNIQUE, CHECK, DEFAULT đặt tên; đọc lỗi 515/547/2627 | Đã viết |
| MS-006 | INSERT, UPDATE, DELETE và mệnh đề OUTPUT<br>`mssql-insert-update-delete-output` | TB | Đi hết vòng đời một chuyến, ghi lịch sử trong cùng transaction | Đã viết |
| MS-007 | SELECT: lọc, sắp xếp, TOP và OFFSET/FETCH<br>`mssql-select-loc-sap-xep-top-offset` | TB | Lọc theo trạng thái và ngày giờ VN, thứ tự xác định, NULL và collation | Đã viết |
| MS-008 | Chuẩn hoá và ranh giới mô hình dữ liệu<br>`mssql-mo-hinh-quan-he-chuan-hoa` | NC | Giải thích bất thường cập nhật, quy tắc liên bảng và đánh đổi khi tách bảng | Đã viết |
| MS-009 | Thay đổi schema tương thích: WITH NOCHECK và ràng buộc tin cậy<br>`mssql-schema-migration-tuong-thich` | NC | Expand → backfill → contract, validate dữ liệu cũ, kế hoạch hoàn tác | Đã viết |
| MS-010 | Thực hành: schema chuyến xe có kiểm tra ràng buộc<br>`mssql-thuc-hanh-schema-chuyen-xe` | NC | Bàn giao DDL, fixture hợp lệ/sai, bất biến liên bảng và bằng chứng chạy | Đã viết |

## P02 · T-SQL cho nghiệp vụ

Đầu vào: P01. Đầu ra: bộ truy vấn có expected output, đối soát số dòng và tổng tiền.

Lộ trình: `mssql-2022-qua-toi-noi-phan-02` — **đã viết đủ 10 bài ngày 17/11/2026**. Lab: `examples/toi-noi-mssql/p02` (SQL Server **2022 CU28 thật**; lược đồ P01 + bộ dữ liệu 30 ngày **giống hệt từng dòng** lab PostgreSQL P02, đã đối chiếu 28.576 dòng; 16 file, 331 kiểm tra PASS; `OUTPUT.txt`). Chưa seed/deploy lên website.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| MS-011 | NULL, CASE, ISNULL và COALESCE<br>`mssql-null-case-isnull-coalesce` | CB | Phân biệt thiếu dữ liệu, không áp dụng và 0; khác biệt kiểu trả về của ISNULL | Đã viết |
| MS-012 | Tổng hợp: GROUP BY, HAVING, ROLLUP và GROUPING SETS<br>`mssql-group-by-rollup-grouping-sets` | CB | Báo cáo theo thành phố/dịch vụ có dòng tổng, mẫu số rõ ràng | Đã viết |
| MS-013 | JOIN, APPLY và số dòng sau khi nối<br>`mssql-join-apply-cardinality` | CB | Không nhân đôi tổng tiền; CROSS/OUTER APPLY lấy "N dòng mới nhất mỗi nhóm" | Đã viết |
| MS-014 | Subquery, EXISTS, CTE và bảng tạm<br>`mssql-subquery-exists-cte-temp-table` | TB | Tách truy vấn đối soát thành bước kiểm chứng; #temp vs biến bảng | Đã viết |
| MS-015 | Window function và mệnh đề WINDOW (SQL Server 2022)<br>`mssql-window-function-window-clause` | TB | Luỹ kế, xếp hạng, LAG/LEAD với frame tường minh | Đã viết |
| MS-016 | Ngày giờ, AT TIME ZONE, DATETRUNC và DATE_BUCKET<br>`mssql-ngay-gio-at-time-zone-date-bucket` | TB | Bộ lọc ngày nghiệp vụ theo giờ VN, gom theo khung 15 phút | Đã viết |
| MS-017 | Upsert: MERGE và các mẫu an toàn hơn<br>`mssql-upsert-merge` | TB | Nhập dữ liệu có quy tắc cập nhật; rủi ro của MERGE khi đồng thời | Đã viết |
| MS-018 | Phân trang lịch sử chuyến: OFFSET/FETCH và keyset<br>`mssql-phan-trang-keyset` | NC | Chứng minh điều kiện trang kế tiếp với khoá sắp xếp trùng | Đã viết |
| MS-019 | Recursive CTE với cây khu vực<br>`mssql-recursive-cte-cay-khu-vuc` | NC | Duyệt cây có MAXRECURSION, phát hiện chu trình | Đã viết |
| MS-020 | Thực hành: truy vấn vận hành và đối soát thanh toán<br>`mssql-thuc-hanh-tsql-doi-soat` | NC | Bàn giao T-SQL, expected output và giải thích chênh lệch | Đã viết |

## P03 · Lưu trữ, index và execution plan

Đầu vào: P01–P02. Đầu ra: báo cáo trước/sau index, Query Store và giới hạn kết luận. Lộ trình dự kiến `-phan-03`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| MS-021 | Đọc execution plan trong DBStudio<br>`mssql-execution-plan-dbstudio` | CB | Phân biệt estimated/actual plan, đọc toán tử, estimate và actual rows | Dự kiến |
| MS-022 | Page, extent và cách SQL Server lưu một dòng<br>`mssql-page-extent-luu-dong` | CB | Đọc kích thước dòng, số page, logical reads với STATISTICS IO | Dự kiến |
| MS-023 | Heap, clustered và nonclustered index<br>`mssql-heap-clustered-nonclustered` | CB | So sánh seek, scan, key lookup và RID lookup | Dự kiến |
| MS-024 | Index nhiều cột, INCLUDE và filtered index<br>`mssql-index-include-filtered` | TB | Chọn index theo truy vấn và chi phí ghi; covering index | Dự kiến |
| MS-025 | Statistics và cardinality estimator<br>`mssql-statistics-cardinality-estimator` | TB | Tìm lệch estimate/actual, cập nhật statistics, compat level 160 | Dự kiến |
| MS-026 | Query Store: lịch sử plan và hồi quy hiệu năng<br>`mssql-query-store` | TB | Bắt plan regression, force plan, Query Store hints | Dự kiến |
| MS-027 | Parameter sniffing và PSP optimization<br>`mssql-parameter-sniffing-psp` | TB | Tái hiện dữ liệu lệch, quan sát Parameter Sensitive Plan của 2022 | Dự kiến |
| MS-028 | Columnstore cho truy vấn báo cáo<br>`mssql-columnstore-bao-cao` | NC | Đo batch mode, rowgroup và đánh đổi với tải ghi | Dự kiến |
| MS-029 | Memory grant, spill và feedback<br>`mssql-memory-grant-spill-feedback` | NC | Đo spill tempdb, memory grant feedback (percentile, persistence) | Dự kiến |
| MS-030 | Thực hành: tối ưu truy vấn bằng bằng chứng<br>`mssql-thuc-hanh-toi-uu-truy-van` | NC | Bàn giao plan, phép đo lặp, Query Store và chi phí ghi của index | Dự kiến |

## P04 · Giao dịch, khoá và đồng thời

Đầu vào: P01–P03; lab có ít nhất hai phiên. Đầu ra: kịch bản tranh chấp nhận chuyến dưới locking và row versioning. Lộ trình dự kiến `-phan-04`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| MS-031 | Transaction, XACT_ABORT và TRY…CATCH<br>`mssql-transaction-xact-abort-try-catch` | CB | Chứng minh chuyến và sự kiện cùng thành công hoặc cùng huỷ | Dự kiến |
| MS-032 | Khoá, blocking và sys.dm_tran_locks<br>`mssql-lock-blocking` | CB | Nhận diện phiên giữ khoá, phiên bị chặn, chuỗi blocking | Dự kiến |
| MS-033 | Isolation level dưới mô hình locking<br>`mssql-isolation-level-locking` | CB | So sánh Read Committed, Repeatable Read, Serializable bằng hai phiên | Dự kiến |
| MS-034 | RCSI và SNAPSHOT: row versioning trong tempdb<br>`mssql-rcsi-snapshot-row-versioning` | TB | Bật RCSI, quan sát version store và xung đột cập nhật 3960 | Dự kiến |
| MS-035 | Optimistic concurrency với rowversion<br>`mssql-optimistic-rowversion` | TB | Ngăn ghi đè trạng thái, kiểm số dòng đổi | Dự kiến |
| MS-036 | Hàng đợi worker với UPDLOCK, READPAST và ROWLOCK<br>`mssql-queue-updlock-readpast` | TB | Lấy việc không trùng, ghi rõ dòng bị bỏ qua | Dự kiến |
| MS-037 | Deadlock, thứ tự lấy khoá và Extended Events<br>`mssql-deadlock-extended-events` | TB | Tái hiện deadlock, đọc deadlock graph, sửa thứ tự truy cập | Dự kiến |
| MS-038 | Serializable, range lock và retry<br>`mssql-serializable-range-lock-retry` | NC | Chặn phantom bằng key-range lock, retry giới hạn | Dự kiến |
| MS-039 | Idempotency và transactional outbox<br>`mssql-idempotency-outbox` | NC | Kiểm request lặp, lỗi giữa chừng, sự kiện trùng | Dự kiến |
| MS-040 | Thực hành: hai tài xế cùng nhận một chuyến<br>`mssql-thuc-hanh-tranh-chap-nhan-chuyen` | NC | Bàn giao lịch chạy, invariant và bằng chứng chỉ một người thắng | Dự kiến |

## P05 · Bảo mật, bảo trì và phục hồi

Đầu vào: P01–P04. Đầu ra: runbook phân quyền, giám sát, bảo trì và restore đã đối soát. Lộ trình dự kiến `-phan-05`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| MS-041 | Login, user, role và quyền tối thiểu<br>`mssql-login-user-role-least-privilege` | CB | Tách tài khoản ứng dụng, migration và báo cáo; contained user | Dự kiến |
| MS-042 | Kết nối an toàn: TLS, chứng chỉ và TDS 8.0<br>`mssql-ket-noi-tls-tds8` | CB | Cấu hình chứng chỉ trong container, Encrypt=strict, kiểm từ DBStudio | Dự kiến |
| MS-043 | Quan sát phiên và truy vấn chậm bằng DMV<br>`mssql-dmv-quan-sat-phien` | CB | Đọc sys.dm_exec_requests, wait stats và truy vấn tốn tài nguyên | Dự kiến |
| MS-044 | Transaction log, recovery model và ADR<br>`mssql-transaction-log-recovery-model-adr` | TB | Quan sát log reuse, VLF, Accelerated Database Recovery | Dự kiến |
| MS-045 | Backup full, differential, log và restore<br>`mssql-backup-restore` | TB | Backup native (DBStudio + T-SQL), restore vào database riêng và đối soát | Dự kiến |
| MS-046 | Bảo trì index và statistics có bằng chứng<br>`mssql-bao-tri-index-statistics` | TB | Đo phân mảnh, quyết định rebuild/reorganize, resumable operations | Dự kiến |
| MS-047 | Connection pool, timeout và giới hạn kết nối<br>`mssql-connection-pool-timeout` | TB | Đo hàng đợi kết nối, command timeout và attention | Dự kiến |
| MS-048 | Khôi phục theo thời điểm với log backup<br>`mssql-point-in-time-restore` | NC | Diễn tập lỗi thao tác, STOPAT, đối chiếu RPO/RTO | Dự kiến |
| MS-049 | Row-level security, Dynamic Data Masking và Ledger<br>`mssql-rls-ddm-ledger` | NC | Cách ly tenant, che dữ liệu, bằng chứng chống sửa với ledger table | Dự kiến |
| MS-050 | Thực hành: bàn giao runbook vận hành và phục hồi<br>`mssql-thuc-hanh-runbook-phuc-hoi` | NC | Bằng chứng restore, cảnh báo và quy trình xử lý lỗi | Dự kiến |

## P06 · Mở rộng, tích hợp và vận hành đầu cuối

Đầu vào: P01–P05. Đầu ra: capstone vận hành SQL Server với kế hoạch thay đổi và diễn tập sự cố. Lộ trình dự kiến `-phan-06`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| MS-051 | Partitioning: hàm, scheme và khi nào dùng<br>`mssql-partition-co-ban` | CB | Phân biệt partition với index, chọn khoá theo workload | Dự kiến |
| MS-052 | Temporal table và lịch sử thay đổi<br>`mssql-temporal-table` | CB | Truy vấn FOR SYSTEM_TIME, retention, đánh đổi dung lượng | Dự kiến |
| MS-053 | JSON trong SQL Server 2022<br>`mssql-json-quan-he` | CB | Chọn cột quan hệ hay JSON; JSON_OBJECT, ISJSON, computed column + index | Dự kiến |
| MS-054 | Partition switching và vòng đời dữ liệu<br>`mssql-partition-switch-retention` | TB | Sliding window, TRUNCATE theo partition, kiểm tra trước/sau | Dự kiến |
| MS-055 | Change Data Capture và Change Tracking<br>`mssql-cdc-change-tracking` | TB | Bật CDC (cần SQL Agent), đọc thay đổi, giám sát độ trễ | Dự kiến |
| MS-056 | Indexed view và dữ liệu báo cáo<br>`mssql-indexed-view-bao-cao` | TB | Điều kiện của indexed view, chi phí ghi, đối soát | Dự kiến |
| MS-057 | Always On availability group trong container<br>`mssql-always-on-availability-group` | TB | Dựng read-scale AG (CLUSTER_TYPE = NONE), quan sát đồng bộ và độ trễ | Dự kiến |
| MS-058 | Failover, contained AG và tính nhất quán khi đọc<br>`mssql-failover-contained-ag` | NC | Failover thủ công, mất dữ liệu ở chế độ async, contained AG của 2022 | Dự kiến |
| MS-059 | Nâng phiên bản, compatibility level và cutover<br>`mssql-nang-cap-compat-level-cutover` | NC | Rehearsal 2019 → 2022, Query Store trước/sau, cổng go/no-go | Dự kiến |
| MS-060 | Capstone: vận hành dữ liệu Tới Nơi trên SQL Server<br>`mssql-capstone-van-hanh-toi-noi` | NC | Bàn giao schema, T-SQL, benchmark, restore, failover và quyết định kiến trúc | Dự kiến |
