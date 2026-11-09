# PostgreSQL qua dự án Tới Nơi

Khung biên tập v1 · 07/11/2026 · **6 phần × 10 bài = 60 bài**. Mỗi phần có **3 cơ bản / 4 trung bình / 3 nâng cao**, gồm cả bài thực hành cuối phần. Đây là khung viết bài; hoàn thành khung không có nghĩa đã hoàn thành nội dung.

## Phạm vi và cách đọc

Series riêng cho PostgreSQL thuộc chuyên mục cha Database, dùng xuyên suốt hệ thống hư cấu **Tới Nơi**: đặt xe, giao đồ ăn và giao bưu kiện. Bắt đầu từ một chuyến xe, xây mô hình và SQL, quan sát lưu trữ/index, kiểm chứng giao dịch đồng thời, rồi vận hành và mở rộng. Đối tượng gồm người mới, backend developer và người vận hành database.

Đọc hồ sơ chung `du-an-mau-toi-noi` trước P01; đây là tài liệu dùng chung toàn blog, không chiếm một trong 60 bài và vẫn thuộc `system-design`. P01 không yêu cầu kiến thức PostgreSQL. Mỗi phần sau kế thừa các phần trước; mức nâng cao là độ khó trong phạm vi chủ đề, không yêu cầu kiến thức chưa được giới thiệu. CB = `beginner`, TB = `intermediate`, NC = `advanced`; toàn khung có 18 CB / 24 TB / 18 NC.

| Phần | Chủ đề | Đầu vào | Sản phẩm cuối phần |
|---|---|---|---|
| P01 | Nền tảng và mô hình dữ liệu | Hồ sơ Tới Nơi | Schema, fixture và kiểm tra ràng buộc |
| P02 | SQL cho nghiệp vụ | P01 | Bộ truy vấn và đối soát |
| P03 | Lưu trữ, index và kế hoạch thực thi | P01–P02 | Báo cáo đo và lựa chọn index |
| P04 | Giao dịch, MVCC và đồng thời | P01–P03 | Kịch bản tranh chấp và bất biến |
| P05 | Bảo mật, bảo trì và phục hồi | P01–P04 | Runbook và bằng chứng restore |
| P06 | Mở rộng, tích hợp và vận hành đầu cuối | P01–P05 | Hồ sơ vận hành và diễn tập sự cố |

## Tổ chức Database để bổ sung các series khác

`database` là chuyên mục cha. Mỗi engine có chuyên mục con, curriculum và lộ trình độc lập; không dùng một lộ trình PostgreSQL để chứa bài của engine khác.

| Engine | Slug chuyên mục | Tiền tố bài mới | Curriculum dự kiến | Trạng thái |
|---|---|---|---|---|
| PostgreSQL | `postgresql` | `postgresql-` | `postgresql-master-curriculum.md` | Dựng khung trong đợt này |
| MySQL | `mysql` | `mysql-` | `mysql-master-curriculum.md` | Dự kiến |
| MongoDB | `mongodb` | `mongodb-` | `mongodb-master-curriculum.md` | Dự kiến |
| Microsoft SQL Server (MSSQL) | `mssql` | `mssql-` | `mssql-master-curriculum.md` | Dự kiến |
| Oracle Database | `oracle` | `oracle-` | `oracle-master-curriculum.md` | Dự kiến |
| Apache Cassandra | `cassandra` | `cassandra-` | `cassandra-master-curriculum.md` | Dự kiến |
| ScyllaDB | `scylladb` | `scylladb-` | `scylladb-master-curriculum.md` | Dự kiến |
| Redis | `redis` | `redis-` | `redis-master-curriculum.md` | Dự kiến |

Mọi series engine theo cùng quy ước với PostgreSQL: chia thành nhiều phần, **mỗi phần tối đa 10 bài** và có đủ ba mức cơ bản, trung bình, nâng cao (kể cả bài thực hành cuối phần). Chỉ đăng ký chuyên mục có nội dung; các dòng dự kiến không phải liên kết công khai hay file đã có. Các engine khác cùng dùng nghiệp vụ Tới Nơi nhưng thiết kế theo mô hình và cơ chế riêng, không chỉ thay cú pháp của PostgreSQL. Bài tích hợp ở Rust, Python, .NET, AWS, Azure giữ chuyên mục hiện tại; liên kết tới đây khi cần đào sâu database. Series DA tập trung trả lời câu hỏi phân tích, series này tập trung tính đúng đắn, cơ chế và vận hành PostgreSQL.

## Giữ nội dung hiện có và quy ước xuất bản

- Giữ URL lộ trình `postgresql-tu-ben-trong` và ba liên kết hiện có; đổi chuyên mục lộ trình sang `postgresql`. Đây là lộ trình đang đọc được, không phải P01 đã hoàn tất.
- Giữ nguyên slug hai bài đã có: `page-don-vi-doc-ghi` → PG-022, `heap-ctid-va-primary-key` → PG-024. Chuyển chuyên mục sang `postgresql`, không viết bản sao. Khi làm P03 phải rà lại nội dung, phép đo và thứ tự đọc trước khi đánh dấu hoàn tất phần.
- Khi P01 hoàn tất, cập nhật lộ trình gốc thành P01 với 10 bài PG-001–010; hồ sơ chung trở thành tài liệu đọc trước. Hai bài cũ giữ URL và đăng ký vào P03 khi phần đó sẵn sàng. Các phần tiếp theo dùng `postgresql-tu-ben-trong-phan-02` … `-phan-06`.
- Chỉ thêm các bài đã viết và kiểm tra vào `api/seed/learning-paths.json`; tối đa 10 bài/phần. Không thêm slug kế hoạch vào seed, không tạo Markdown rỗng. Level lộ trình dự kiến: P01 beginner, P02 intermediate, P03–P06 advanced; level từng bài theo bảng.
- Danh sách chi tiết dưới đây là nguồn quản lý ID, slug và đầu ra. Trạng thái mặc định **Kế hoạch**; chỉ PG-022 và PG-024 là **Đã có — chờ rà soát**. Có 58 bài mới dự kiến và 2 bài kế thừa.

## Tiêu chuẩn viết và hoàn tất một phần

Mỗi bài có frontmatter `slug`, `title`, `summary`, `category: postgresql`, `level`, `tags`, `language: vi` theo seed hiện hành. Giọng văn tiếng Việt, giải nghĩa thuật ngữ lần đầu, mở bằng vấn đề Tới Nơi, trình bày cơ chế, ví dụ có kết quả mong đợi, lỗi thường gặp, đánh đổi, bài tập và liên kết đọc tiếp. Không suy rộng cơ chế của PostgreSQL cho tất cả database.

Lab dự kiến ở `examples/toi-noi-postgresql/p01` … `p06`: README nêu prerequisites, phiên bản PostgreSQL/extension cố định sau khi kiểm chứng, cách chạy, fixture tái lập, kết quả, kiểm tra và dọn môi trường lab. Dùng tiền và múi giờ nhất quán với hồ sơ Tới Nơi; dữ liệu hư cấu, không lấy PII thật. Những bài concurrency cần hai phiên kết nối và lịch chạy rõ ràng. Bài hiệu năng ghi cấu hình máy, số dòng, phân bố, cache nóng/lạnh, số lần đo và giới hạn kết luận; không bịa số đo. Bài phục hồi phải thực sự restore vào môi trường riêng và đối soát.

Khi viết bài, tra tài liệu PostgreSQL chính thức đúng phiên bản và tài liệu chính thức của extension/công cụ dùng trong bài. Khung này chưa chốt phiên bản hay tuyên bố benchmark đã thực hiện. Ảnh bìa theo hệ thống brief/SVG/WebP hiện có; sơ đồ nội dung có nhãn, caption và được xem lại trực quan. Kiểm frontmatter, liên kết, số bài, SQL/lab và ảnh trước commit.

Hoàn thành **nội dung** phần nào: cập nhật trạng thái, seed và README lab của phần đó, chạy kiểm tra phù hợp rồi commit riêng `content(postgresql): complete part N`. Commit dựng khung dùng `docs(postgresql): scaffold part N` để phân biệt với bài đã viết. Chỉ stage file thuộc phần PostgreSQL; không gom thay đổi khác. Seed/deploy là bước riêng, không được coi việc sửa source là đã xuất bản lên website.

## P01 · Nền tảng và mô hình dữ liệu

Đầu vào: Hồ sơ chung Tới Nơi; chưa cần biết SQL. Đầu ra: Schema tối thiểu cho khách, tài xế và chuyến, fixture nhỏ và kiểm tra dữ liệu sai.

Lộ trình: `postgresql-tu-ben-trong` — **đã cập nhật thành P01 (10 bài) ngày 08/11/2026**; hồ sơ chung là bài đọc trước, hai bài cũ chờ đăng ký vào P03. Lab: `examples/toi-noi-postgresql/p01` (PostgreSQL 17.5 qua PGlite; đường Docker + psql chưa chạy trên máy viết bài).

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| PG-001 | PostgreSQL là gì và nằm ở đâu trong Tới Nơi?<br>`postgresql-la-gi-va-vai-tro-trong-toi-noi` | CB | Phân biệt DBMS, database, schema, bảng và trách nhiệm lưu dữ liệu | Đã viết |
| PG-002 | Dựng môi trường lab và kết nối bằng psql<br>`postgresql-cai-dat-lab-psql-ket-noi` | CB | Tạo lab riêng, chạy truy vấn đầu tiên, ghi phiên bản và cách dọn | Đã viết |
| PG-003 | Bảng, dòng, cột và khoá trong nghiệp vụ chuyến xe<br>`postgresql-bang-dong-cot-primary-foreign-key` | CB | Vẽ quan hệ khách–chuyến–tài xế và nhận diện một dòng | Đã viết |
| PG-004 | Chọn kiểu dữ liệu cho tiền, thời gian và định danh<br>`postgresql-kieu-du-lieu-tien-thoi-gian-uuid` | TB | Lập data dictionary có đơn vị, NULL và múi giờ | Đã viết |
| PG-005 | CREATE TABLE và ràng buộc dữ liệu<br>`postgresql-create-table-constraints` | TB | Viết DDL với primary key, foreign key, unique, check và not null | Đã viết |
| PG-006 | Thêm, sửa, xoá và đọc kết quả với RETURNING<br>`postgresql-insert-update-delete-returning` | TB | Thực hiện vòng đời một chuyến trên fixture có thể chạy lại | Đã viết |
| PG-007 | SELECT đầu tiên: lọc, sắp xếp và giới hạn<br>`postgresql-select-filter-order-limit` | TB | Lấy chuyến theo trạng thái, thời gian và thứ tự xác định | Đã viết |
| PG-008 | Chuẩn hoá và ranh giới mô hình dữ liệu<br>`postgresql-mo-hinh-quan-he-chuan-hoa` | NC | Giải thích bất thường cập nhật và đánh đổi khi tách bảng | Đã viết |
| PG-009 | Thay đổi schema mà dữ liệu cũ vẫn đọc được<br>`postgresql-schema-migration-tuong-thich` | NC | Thực hiện migration nhỏ, kiểm dữ liệu và kế hoạch khôi phục | Đã viết |
| PG-010 | Thực hành: schema chuyến xe có kiểm tra ràng buộc<br>`postgresql-thuc-hanh-schema-chuyen-xe` | NC | Bàn giao DDL, fixture hợp lệ/sai và bằng chứng kiểm tra | Đã viết |

## P02 · SQL cho nghiệp vụ

Đầu vào: P01 và fixture chuyến xe. Đầu ra: Bộ truy vấn có expected output, đối soát số dòng và tổng tiền.

Lộ trình: `postgresql-tu-ben-trong-phan-02` — **đã tạo (10 bài) ngày 10/11/2026**. Lab: `examples/toi-noi-postgresql/p02` (chạy trên lược đồ P01, fixture 30 ngày tất định; thí nghiệm và lời giải bài tập ở `p02/thu-nghiem/`; PostgreSQL 17.5 qua PGlite, đường Docker + psql chưa chạy).

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| PG-011 | NULL, CASE và COALESCE trong dữ liệu chuyến<br>`postgresql-null-case-coalesce` | CB | Phân biệt thiếu dữ liệu, không áp dụng và giá trị 0 | Đã viết |
| PG-012 | COUNT, SUM, GROUP BY và HAVING<br>`postgresql-aggregate-group-by-having` | CB | Tổng hợp theo thành phố và dịch vụ với mẫu số rõ ràng | Đã viết |
| PG-013 | JOIN và số dòng sau khi nối bảng<br>`postgresql-join-quan-he-cardinality` | CB | Kiểm quan hệ một–nhiều, không nhân đôi tổng tiền | Đã viết |
| PG-014 | Subquery, EXISTS và CTE<br>`postgresql-subquery-exists-cte` | TB | Tách truy vấn đối soát thành bước kiểm chứng được | Đã viết |
| PG-015 | Window function và cửa sổ tính toán<br>`postgresql-window-function-frame` | TB | Tính luỹ kế, xếp hạng và so sánh chuyến trước với frame rõ ràng | Đã viết |
| PG-016 | Ngày giờ và múi giờ trong truy vấn<br>`postgresql-date-time-timezone` | TB | Viết bộ lọc ngày nghiệp vụ với kiểm tra hai biên | Đã viết |
| PG-017 | UPSERT và xung đột khoá duy nhất<br>`postgresql-upsert-on-conflict` | TB | Nhập dữ liệu có quy tắc cập nhật và fixture trùng khoá | Đã viết |
| PG-018 | Phân trang lịch sử chuyến bằng keyset<br>`postgresql-pagination-keyset` | NC | Xử lý khoá sắp xếp trùng và chứng minh điều kiện trang kế tiếp | Đã viết |
| PG-019 | Recursive CTE với cây khu vực<br>`postgresql-recursive-cte-cay-khu-vuc` | NC | Duyệt cây có điều kiện dừng, phát hiện dữ liệu chu trình | Đã viết |
| PG-020 | Thực hành: truy vấn vận hành và đối soát thanh toán<br>`postgresql-thuc-hanh-sql-doi-soat` | NC | Bàn giao SQL, expected output và giải thích chênh lệch | Đã viết |

## P03 · Lưu trữ, index và kế hoạch thực thi

Đầu vào: P01–P02; đã biết lọc, nối và tổng hợp. Đầu ra: Báo cáo trước/sau index, dữ liệu đo và giới hạn kết luận.

Lộ trình khi hoàn tất: `postgresql-tu-ben-trong-phan-03`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| PG-021 | Đọc EXPLAIN từ truy vấn tìm chuyến<br>`postgresql-explain-ke-hoach-thuc-thi` | CB | Nhận diện scan, join, estimate và phân biệt kế hoạch với thực thi | Kế hoạch |
| PG-022 | Đơn vị đọc ghi nhỏ nhất của cơ sở dữ liệu là gì?<br>`page-don-vi-doc-ghi` | CB | Rà bài hiện có, đối chiếu page, buffer và phạm vi kết luận | Đã có — chờ rà soát |
| PG-023 | B-tree index giúp tìm chuyến như thế nào?<br>`postgresql-btree-index-co-ban` | CB | So sánh đường tìm dữ liệu khi có và chưa có index | Kế hoạch |
| PG-024 | Clustered index thật và clustered index nhìn thấy trong PostgreSQL<br>`heap-ctid-va-primary-key` | TB | Rà bài hiện có, làm rõ heap, ctid, primary key và tránh hiểu sai thuật ngữ | Đã có — chờ rà soát |
| PG-025 | Index nhiều cột, partial và expression<br>`postgresql-index-nhieu-cot-partial-expression` | TB | Chọn index theo điều kiện truy vấn và chi phí ghi | Kế hoạch |
| PG-026 | Index-only scan, INCLUDE và visibility map<br>`postgresql-index-only-scan-visibility-map` | TB | Quan sát heap fetch và điều kiện hiệu quả của covering index | Kế hoạch |
| PG-027 | Statistics, ANALYZE và ước lượng cardinality<br>`postgresql-statistics-analyze-cardinality` | TB | Tìm lệch estimate/actual trên dữ liệu phân bố không đều | Kế hoạch |
| PG-028 | Khi nào cân nhắc GIN, GiST và BRIN?<br>`postgresql-gin-gist-brin-lua-chon` | NC | Lập ma trận workload, operator và giới hạn từng loại index | Kế hoạch |
| PG-029 | Sort, hash và truy vấn tràn ra đĩa<br>`postgresql-sort-hash-memory-spill` | NC | Đo spill và đánh đổi tài nguyên với tải đồng thời | Kế hoạch |
| PG-030 | Thực hành: tối ưu truy vấn bằng bằng chứng<br>`postgresql-thuc-hanh-toi-uu-truy-van` | NC | Bàn giao plan, fixture, phép đo lặp và chi phí ghi của index | Kế hoạch |

## P04 · Giao dịch, MVCC và đồng thời

Đầu vào: P01–P03; lab có ít nhất hai phiên kết nối. Đầu ra: Kịch bản tranh chấp nhận chuyến với kiểm tra bất biến và retry.

Lộ trình khi hoàn tất: `postgresql-tu-ben-trong-phan-04`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| PG-031 | Transaction, COMMIT và ROLLBACK<br>`postgresql-transaction-commit-rollback` | CB | Chứng minh ghi chuyến và sự kiện cùng thành công hoặc cùng huỷ | Kế hoạch |
| PG-032 | MVCC và snapshot: mỗi phiên nhìn thấy gì?<br>`postgresql-mvcc-snapshot` | CB | Chạy hai phiên để quan sát phiên bản dữ liệu | Kế hoạch |
| PG-033 | Khoá dòng, khoá bảng và thời gian chờ<br>`postgresql-lock-row-table` | CB | Nhận diện phiên giữ khoá và phiên bị chặn | Kế hoạch |
| PG-034 | Read Committed và Repeatable Read<br>`postgresql-isolation-read-committed-repeatable-read` | TB | So sánh cùng lịch thực thi dưới hai mức cô lập | Kế hoạch |
| PG-035 | Optimistic concurrency và cập nhật có điều kiện<br>`postgresql-optimistic-version-conditional-update` | TB | Ngăn ghi đè trạng thái với version và kiểm số dòng đổi | Kế hoạch |
| PG-036 | FOR UPDATE, NOWAIT và SKIP LOCKED<br>`postgresql-for-update-skip-locked` | TB | Thử hàng đợi worker và ghi rõ trường hợp bỏ qua dòng | Kế hoạch |
| PG-037 | Deadlock, thứ tự lấy khoá và timeout<br>`postgresql-deadlock-lock-order-timeout` | TB | Tái hiện vòng chờ và sửa thứ tự truy cập | Kế hoạch |
| PG-038 | Serializable và retry toàn giao dịch<br>`postgresql-serializable-retry` | NC | Tái hiện xung đột, kiểm retry giới hạn và tính đúng đắn | Kế hoạch |
| PG-039 | Idempotency và transactional outbox<br>`postgresql-idempotency-outbox-transaction` | NC | Kiểm request lặp, lỗi giữa chừng và xử lý sự kiện trùng | Kế hoạch |
| PG-040 | Thực hành: hai tài xế cùng nhận một chuyến<br>`postgresql-thuc-hanh-tranh-chap-nhan-chuyen` | NC | Bàn giao lịch chạy, invariant và bằng chứng chỉ một người thắng | Kế hoạch |

## P05 · Bảo mật, bảo trì và phục hồi

Đầu vào: P01–P04; môi trường lab có thể huỷ và dựng lại. Đầu ra: Runbook phân quyền, giám sát, bảo trì và restore đã đối soát.

Lộ trình khi hoàn tất: `postgresql-tu-ben-trong-phan-05`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| PG-041 | Role, GRANT và quyền tối thiểu<br>`postgresql-role-grant-least-privilege` | CB | Tách tài khoản ứng dụng, migration và đọc báo cáo | Kế hoạch |
| PG-042 | Xác thực và bảo vệ kết nối PostgreSQL<br>`postgresql-ket-noi-auth-tls` | CB | Kiểm kết nối hợp lệ/bị từ chối và cấu hình TLS trong lab | Kế hoạch |
| PG-043 | Quan sát phiên làm việc và truy vấn chậm<br>`postgresql-pg-stat-activity-quan-sat` | CB | Đọc hoạt động, thời gian chờ và thông tin chẩn đoán | Kế hoạch |
| PG-044 | VACUUM, autovacuum và bloat<br>`postgresql-vacuum-autovacuum-bloat` | TB | Quan sát vòng đời tuple và lập lịch bảo trì dựa trên bằng chứng | Kế hoạch |
| PG-045 | WAL, checkpoint và độ bền dữ liệu<br>`postgresql-wal-checkpoint-durability` | TB | Mô tả đường ghi và đo ảnh hưởng checkpoint trong lab | Kế hoạch |
| PG-046 | Backup logic và khôi phục với pg_dump<br>`postgresql-pg-dump-restore` | TB | Restore vào database riêng và đối soát schema, số dòng, quyền | Kế hoạch |
| PG-047 | Connection pool, giới hạn kết nối và timeout<br>`postgresql-connection-pool-timeout` | TB | Đo tải chờ kết nối, lập ngân sách và kiểm tương thích pool | Kế hoạch |
| PG-048 | Khôi phục theo thời điểm với base backup và WAL<br>`postgresql-pitr-base-backup-wal` | NC | Diễn tập lỗi thao tác, restore và đối chiếu mục tiêu RPO/RTO | Kế hoạch |
| PG-049 | Row-level security và cách ly tenant<br>`postgresql-rls-tenant-security` | NC | Kiểm ma trận quyền, ngữ cảnh tenant và trường hợp đặc quyền | Kế hoạch |
| PG-050 | Thực hành: bàn giao runbook vận hành và phục hồi<br>`postgresql-thuc-hanh-runbook-phuc-hoi` | NC | Bàn giao bằng chứng restore, cảnh báo và quy trình xử lý lỗi | Kế hoạch |

## P06 · Mở rộng, tích hợp và vận hành đầu cuối

Đầu vào: P01–P05; có baseline đo và runbook phục hồi. Đầu ra: Capstone vận hành PostgreSQL với kế hoạch thay đổi và diễn tập sự cố.

Lộ trình khi hoàn tất: `postgresql-tu-ben-trong-phan-06`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| PG-051 | Partition là gì và dùng khi nào?<br>`postgresql-partition-co-ban` | CB | Phân biệt partition với index và chọn khoá theo workload | Kế hoạch |
| PG-052 | Primary, standby và replication lag<br>`postgresql-replication-primary-standby` | CB | Quan sát sao chép trong lab và diễn giải độ trễ đọc | Kế hoạch |
| PG-053 | JSONB và dữ liệu quan hệ<br>`postgresql-jsonb-quan-he` | CB | Chọn cột quan hệ hoặc JSONB theo ràng buộc và truy vấn | Kế hoạch |
| PG-054 | Partition pruning và vòng đời dữ liệu<br>`postgresql-partition-pruning-retention` | TB | Đo pruning, chuẩn bị partition và thử retention có kiểm tra | Kế hoạch |
| PG-055 | Logical replication và CDC<br>`postgresql-logical-replication-cdc` | TB | Thử initial copy, thay đổi dữ liệu và giám sát replication slot | Kế hoạch |
| PG-056 | Materialized view và dữ liệu báo cáo<br>`postgresql-materialized-view-reporting` | TB | Định nghĩa độ tươi, quy trình refresh và đối soát kết quả | Kế hoạch |
| PG-057 | Đánh giá PostGIS và pgvector theo workload<br>`postgresql-extension-postgis-pgvector` | TB | Thiết kế hai thử nghiệm độc lập cho vị trí và vector, nêu điều kiện dùng | Kế hoạch |
| PG-058 | Failover, split-brain và tính nhất quán khi đọc<br>`postgresql-ha-failover-read-consistency` | NC | Lập kịch bản mất primary, fencing, kết nối lại và đo mất dữ liệu | Kế hoạch |
| PG-059 | Nâng phiên bản và cutover có phương án quay lại<br>`postgresql-upgrade-migration-cutover` | NC | Rehearsal tương thích, đối soát, cổng go/no-go và giới hạn rollback | Kế hoạch |
| PG-060 | Capstone: vận hành dữ liệu Tới Nơi từ schema tới sự cố<br>`postgresql-capstone-van-hanh-toi-noi` | NC | Bàn giao schema, SQL, benchmark, restore, failover và quyết định kiến trúc | Kế hoạch |
