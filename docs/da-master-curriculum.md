# Data Analysis qua dự án Tới Nơi

Khung biên tập v1 · 07/11/2026 · **8 phần × 10 bài = 80 bài**. Mỗi phần gồm 3 bài cơ bản, 4 bài trung bình và 3 bài nâng cao; bài thực hành cuối phần đã nằm trong giới hạn 10 bài. DA trong series là **Data Analysis / Data Analyst — phân tích dữ liệu / người làm phân tích dữ liệu**.

## Concept và phạm vi

Giữ concept antblog: tiếng Việt dễ hiểu, giải thích thuật ngữ khi xuất hiện, bắt đầu từ vấn đề thực tế của hệ thống hư cấu **Tới Nơi** rồi tạo ra sản phẩm thực hành. Dùng xuyên suốt ba dịch vụ đặt xe, giao đồ ăn và giao bưu kiện; các câu hỏi xoay quanh tỷ lệ huỷ, thời gian đón, hoàn thành chuyến, thanh toán, khách quay lại và hiệu quả vận hành.

Trục học: **câu hỏi kinh doanh → định nghĩa chỉ số → thu thập và kiểm tra dữ liệu → chọn nơi lưu và cách truy vấn → phân tích → truyền đạt → quyết định → đo lại**. DA không chỉ viết SQL hay dựng dashboard; công việc còn gồm làm rõ yêu cầu, xác minh nguồn, xử lý chất lượng, kiểm tra giả thuyết, giải thích bất định và theo dõi tác động.

Đối tượng: người mới học DA; BA, developer, tester và vận hành muốn phân tích dữ liệu; DA muốn hiểu lựa chọn database và các đánh đổi của hệ thống. P01 không yêu cầu lập trình; P02 xây nền SQL. Python là công cụ bổ trợ ở các phần sau, không phải điều kiện để bắt đầu.

Khung gồm 80 bài dự kiến; **DA-001 đã viết trong source**, các bài còn lại đang ở kế hoạch. Chưa có bộ dữ liệu thực thi. Không thêm bài rỗng vào seed, không tạo URL công khai cho bài chưa viết. Các lựa chọn công nghệ dưới đây là chủ đề cần khảo sát và thực nghiệm, chưa phải quyết định thay kiến trúc Tới Nơi.

## Cấp độ và lộ trình

- **CB / beginner:** giải thích khái niệm, làm theo ví dụ và kiểm chứng kết quả trên dữ liệu nhỏ.
- **TB / intermediate:** tự chọn phương pháp, nối nhiều nguồn, xử lý ngoại lệ và tạo đầu ra tái lập được.
- **NC / advanced:** đánh giá giả định, sai lệch, bất định và đánh đổi xuyên hệ thống; bảo vệ khuyến nghị bằng bằng chứng.

Cấp độ là độ khó trong từng chủ đề. Đọc tuần tự P01 → P08; bài NC cuối phần cần các bài trước của phần đó. Tổng cộng **24 CB / 32 TB / 24 NC**. P01–P03 xây nền, P04–P06 đưa vào hệ thống, P07–P08 đào sâu suy luận và vận hành.

| Phần | Chủ đề | Đầu ra chính |
|---|---|---|
| P01 | DA là gì và làm những công việc nào? | Analysis brief và hợp đồng chỉ số |
| P02 | SQL và đọc dữ liệu nghiệp vụ | Bộ truy vấn đã đối soát |
| P03 | Thu thập, chất lượng và khám phá dữ liệu | Báo cáo chất lượng và notebook khám phá |
| P04 | Chọn database phù hợp hệ thống DA | Ma trận lựa chọn và kế hoạch benchmark |
| P05 | Mô hình dữ liệu và pipeline phân tích | Thiết kế data mart, lineage và kiểm thử |
| P06 | Phân tích sản phẩm, vận hành và dashboard | Dashboard kèm bản khuyến nghị |
| P07 | Thống kê, thử nghiệm và các yếu tố ảnh hưởng | Hồ sơ đánh giá tác động và bất định |
| P08 | Hiệu năng, quản trị và capstone | Hồ sơ phân tích đầu cuối, runbook bàn giao |

## P01 · DA là gì và làm những công việc nào?

Đầu vào: hồ sơ chung Tới Nơi. Câu hỏi mở đầu: “Tỷ lệ huỷ tăng có thật không, tăng ở đâu và cần tìm hiểu gì trước khi can thiệp?” Đầu ra: analysis brief, danh mục nguồn và hợp đồng chỉ số.

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| DA-001 | Phân tích dữ liệu là gì? Từ số liệu chuyến xe tới quyết định | CB | Phân biệt dữ liệu, thông tin, insight và hành động |
| DA-002 | Một DA làm gì và phối hợp với BA, BI, DE, DS ra sao? | CB | Lập bảng trách nhiệm từ yêu cầu tới theo dõi kết quả |
| DA-003 | Vòng đời một bài toán phân tích dữ liệu | CB | Lập checklist công việc, người sở hữu và đầu ra từng bước |
| DA-004 | Từ “huỷ chuyến tăng” tới câu hỏi phân tích kiểm chứng được | TB | Viết analysis brief có phạm vi, quyết định cần hỗ trợ và hạn trả lời |
| DA-005 | KPI, metric và chỉ số bảo vệ: đo đúng điều cần biết | TB | Định nghĩa tử số, mẫu số, đơn vị, cohort và thời gian |
| DA-006 | Phân tích mô tả, chẩn đoán, dự báo và đề xuất hành động | TB | Chọn loại phân tích theo câu hỏi và mức bằng chứng cần có |
| DA-007 | Nguồn dữ liệu Tới Nơi: chuyến, sự kiện, thanh toán và CSKH | TB | Lập source inventory, chủ sở hữu và giới hạn truy cập |
| DA-008 | Các yếu tố ảnh hưởng: nghiệp vụ, con người, dữ liệu và hệ thống | NC | Lập bản đồ giả thuyết và kế hoạch thu thập bằng chứng |
| DA-009 | Khi nào chưa đủ dữ liệu để kết luận? | NC | Ghi giả định, bất định và điều kiện phải dừng hoặc đo thêm |
| DA-010 | Thực hành: lập kế hoạch phân tích huỷ chuyến tại Tới Nơi | NC | Bảo vệ brief, metric contract và tiêu chí hoàn thành |

## P02 · SQL và đọc dữ liệu nghiệp vụ

Đầu vào: P01 và dữ liệu mẫu PostgreSQL của hồ sơ chung. Đầu ra: bộ truy vấn trả lời câu hỏi nghiệp vụ, kèm kiểm tra số lượng và tổng tiền.

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| DA-011 | Bảng, khoá và grain: một dòng dữ liệu đại diện cho điều gì? | CB | Phân biệt một chuyến, một sự kiện và một giao dịch |
| DA-012 | SELECT, WHERE, ORDER BY: đọc đúng tập chuyến cần phân tích | CB | Lọc theo dịch vụ, thành phố và khoảng thời gian |
| DA-013 | COUNT, SUM, AVG và GROUP BY: tổng hợp số liệu đầu tiên | CB | Tính số chuyến và chỉ số theo nhóm với mẫu số rõ ràng |
| DA-014 | JOIN và lỗi nhân bản dữ liệu khi nối chuyến với thanh toán | TB | Kiểm tra cardinality và đối soát trước/sau join |
| DA-015 | NULL, CASE và trạng thái nghiệp vụ trong báo cáo | TB | Tách chưa biết, không áp dụng và giá trị bằng không |
| DA-016 | Ngày giờ, múi giờ và cửa sổ báo cáo | TB | Quy định thời điểm ghi nhận, ngày nghiệp vụ và biên khoảng thời gian |
| DA-017 | CTE và subquery: chia truy vấn thành bước kiểm tra được | TB | Viết truy vấn có đầu ra trung gian và kiểm tra độc lập |
| DA-018 | Window function: xếp hạng, luỹ kế và so sánh kỳ trước | NC | Tính theo partition/frame phù hợp, tránh lọt dữ liệu tương lai |
| DA-019 | Tiền, hoàn tiền và đối soát: vì sao tổng thu khác tổng giá chuyến? | NC | Tách giá chuyến, dòng tiền và định nghĩa doanh thu được xác nhận |
| DA-020 | Thực hành: bộ SQL báo cáo vận hành một tuần | NC | Bàn giao truy vấn, bộ kiểm tra và giải thích chênh lệch |

## P03 · Thu thập, chất lượng và khám phá dữ liệu

Đầu vào: metric contract P01 và truy vấn P02. Đầu ra: dữ liệu phân tích có nguồn gốc, báo cáo chất lượng và EDA (phân tích khám phá dữ liệu).

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| DA-021 | CSV, JSON, bảng tính và Parquet trong công việc DA | CB | Nhận diện kiểu dữ liệu, schema và lỗi nhập/xuất |
| DA-022 | Data dictionary: đọc đúng ý nghĩa cột và đơn vị | CB | Ghi định nghĩa, giá trị hợp lệ, nguồn và người xác nhận |
| DA-023 | Data profiling: xem dữ liệu trước khi tính chỉ số | CB | Kiểm tra số dòng, phân bố, thiếu, trùng và miền giá trị |
| DA-024 | Làm sạch dữ liệu thiếu và trùng mà vẫn giữ dấu vết | TB | Viết quy tắc xử lý, lưu bản gốc và báo cáo tác động |
| DA-025 | Event tracking: cần ghi nhận gì để đo hành trình đặt xe? | TB | Soạn tracking plan có event ID, timestamp và version |
| DA-026 | Python cho DA: notebook, DataFrame và quy trình tái lập | TB | Tạo notebook đọc dữ liệu, biến đổi và xuất kết quả có kiểm tra |
| DA-027 | EDA: phân bố thời gian đón, ngoại lệ và phân khúc | TB | So sánh trung vị, percentile và nhóm thay vì chỉ nhìn trung bình |
| DA-028 | Dữ liệu đến trễ, sai thứ tự và sai đồng hồ | NC | Đánh giá tác động event time/ingestion time tới báo cáo |
| DA-029 | Thiên lệch chọn mẫu và thiếu dữ liệu có hệ thống | NC | Kiểm tra nhóm bị bỏ sót và giới hạn suy rộng |
| DA-030 | Thực hành: kiểm toán bộ dữ liệu huỷ chuyến | NC | Bàn giao quality report, cleaning log và EDA có giới hạn kết luận |

## P04 · Chọn database nào cho DA để phù hợp hệ thống?

Đầu vào: workload truy vấn P02, chất lượng P03 và yêu cầu nghiệp vụ P01. Đầu ra: hồ sơ lựa chọn kho phân tích, kèm giả định, phương án thay thế và kế hoạch đo. Phân biệt **database engine**, **dịch vụ warehouse**, **query engine**, **định dạng file** và **kiến trúc lake/lakehouse**; không coi chúng là những sản phẩm cùng loại.

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| DA-031 | OLTP và OLAP: vì sao truy vấn báo cáo khác giao dịch đặt xe? | CB | Mô tả workload đọc/ghi và rủi ro tranh tài nguyên |
| DA-032 | Row store, column store và cách dữ liệu được quét | CB | Giải thích ảnh hưởng của cột, bộ lọc và phép tổng hợp |
| DA-033 | Database, warehouse, data lake và lakehouse khác nhau ở đâu? | CB | Vẽ bản đồ vai trò lưu trữ, tính toán và quản trị |
| DA-034 | Khi nào kho quan hệ hiện có đủ cho DA? | TB | Đánh giá PostgreSQL/MySQL, replica và kho báo cáo riêng theo workload |
| DA-035 | Phân tích cục bộ với DuckDB và file Parquet | TB | Thiết kế thử nghiệm dữ liệu dạng file và giới hạn tài nguyên máy |
| DA-036 | Phân tích tương tác trên kho dạng cột: trường hợp ClickHouse | TB | Khảo sát nạp dữ liệu, cập nhật, concurrency và truy vấn tổng hợp |
| DA-037 | Warehouse quản lý sẵn: khảo sát BigQuery, Snowflake và Redshift | TB | Lập cùng bộ câu hỏi về vận hành, chi phí và tích hợp cho các ứng viên |
| DA-038 | Dữ liệu document, thời gian và vị trí: lưu nguồn hay chuyển sang kho phân tích? | NC | Đánh giá MongoDB, kho time-series và PostGIS theo nhiệm vụ, không chọn theo tên |
| DA-039 | Benchmark và tổng chi phí sở hữu: kiểm chứng lựa chọn database | NC | Thiết kế đo latency, freshness, độ đúng, tài nguyên và chi phí vận hành |
| DA-040 | Thực hành: bảo vệ phương án kho DA cho ba kịch bản Tới Nơi | NC | Viết decision record cho nhóm nhỏ, BI tăng trưởng và phân tích gần thời gian thực |

### Tiêu chí bắt buộc khi lựa chọn

| Yếu tố | Câu hỏi cần trả lời bằng dữ liệu hoặc thử nghiệm |
|---|---|
| Quyết định nghiệp vụ | Báo cáo ngày, khám phá tự do hay giám sát vận hành? |
| Khối lượng và tăng trưởng | Bao nhiêu dòng/byte, tốc độ nạp, thời gian lưu và phần thực sự cần quét? |
| Workload | Join, aggregate, cửa sổ thời gian, truy vấn vị trí hay đọc một bản ghi? |
| Độ mới và độ trễ | Chấp nhận dữ liệu trễ bao lâu; mục tiêu thời gian trả lời ở mức đồng thời nào? |
| Cập nhật và độ đúng | Có sửa lịch sử, xoá dữ liệu, CDC, dữ liệu đến trễ và yêu cầu đối soát nào? |
| Tích hợp | Công cụ BI, SQL dialect, driver, định dạng và hệ thống nguồn có phù hợp? |
| Vận hành | Ai quản trị, phục hồi, giám sát, nâng cấp và xử lý sự cố? |
| Bảo vệ dữ liệu | Quyền truy cập, vị trí lưu, vòng đời dữ liệu và yêu cầu kiểm toán là gì? |
| Chi phí | Lưu trữ, compute, quét dữ liệu, truyền dữ liệu, nhân lực và chuyển đổi tốn bao nhiêu? |
| Khả năng thay đổi | Phụ thuộc nhà cung cấp, xuất dữ liệu và chi phí di chuyển ra sao? |

Không đặt ngưỡng “đủ lớn thì bắt buộc dùng X” khi chưa đo. Benchmark dùng cùng dữ liệu, ngữ nghĩa truy vấn và điều kiện tải; ghi rõ cache nóng/lạnh, cấu hình, phiên bản, độ đúng và thời điểm khảo sát giá. Khi viết bài, đối chiếu tài liệu chính thức hiện hành trước khi khuyến nghị sản phẩm.

## P05 · Mô hình dữ liệu và pipeline phân tích

Đầu vào: lựa chọn P04 và định nghĩa dữ liệu P01–P03. Đầu ra: thiết kế data mart cho chuyến và thanh toán, luồng cập nhật và kiểm tra đối soát. DA phối hợp với DE/backend; phần này không mặc định DA phải sở hữu toàn bộ nền tảng.

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| DA-041 | Raw, staging và mart: các lớp dữ liệu phục vụ mục đích gì? | CB | Vẽ luồng từ nguồn tới báo cáo và trách nhiệm mỗi lớp |
| DA-042 | Fact, dimension và star schema qua dữ liệu chuyến | CB | Chốt grain và thiết kế fact chuyến, dimension dịch vụ/thành phố |
| DA-043 | ETL, ELT và batch: đưa dữ liệu tới nơi phân tích | CB | Mô tả lịch chạy, phụ thuộc và đầu ra của pipeline |
| DA-044 | Lịch sử thay đổi: đọc đúng thuộc tính tại thời điểm phát sinh | TB | Thiết kế hiệu lực dimension, phân biệt trạng thái hiện tại và lịch sử |
| DA-045 | Incremental load và CDC: xử lý bản ghi mới, sửa và xoá | TB | Xác định khoá, checkpoint và cơ chế đối soát |
| DA-046 | Transformation bằng SQL và kiểm thử dữ liệu | TB | Kiểm tra uniqueness, not-null, quan hệ và invariant nghiệp vụ |
| DA-047 | Metric layer: dùng chung định nghĩa chỉ số cho nhiều báo cáo | TB | Quản lý version, owner và thay đổi định nghĩa metric |
| DA-048 | Backfill, replay và idempotency của pipeline | NC | Lập cách chạy lại không nhân đôi và kiểm tra báo cáo bị ảnh hưởng |
| DA-049 | Lineage và data contract khi schema nguồn thay đổi | NC | Truy vết tác động tới mart, dashboard và người sử dụng |
| DA-050 | Thực hành: thiết kế data mart chuyến và thanh toán | NC | Bàn giao sơ đồ, SQL dự kiến, kiểm tra và kế hoạch cập nhật |

## P06 · Phân tích sản phẩm, vận hành và trực quan hoá

Đầu vào: mart P05 và dữ liệu đã kiểm tra. Đầu ra: báo cáo có thể hành động, dashboard và bản giải thích cho stakeholder.

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| DA-051 | Chọn biểu đồ phù hợp câu hỏi và kiểu dữ liệu | CB | Dùng trục, đơn vị và tỷ lệ nhất quán, tránh trình bày gây hiểu sai |
| DA-052 | Thiết kế dashboard theo người dùng và quyết định | CB | Phác thảo màn hình vận hành có ngưỡng, owner và hành động |
| DA-053 | Kể chuyện bằng dữ liệu: từ phát hiện tới khuyến nghị | CB | Viết bản một trang gồm bằng chứng, giới hạn và bước tiếp |
| DA-054 | Funnel đặt xe: từ yêu cầu tới hoàn thành | TB | Chốt đơn vị phân tích, thứ tự bước và cửa sổ chuyển đổi |
| DA-055 | Cohort và retention: khách có quay lại sử dụng Tới Nơi? | TB | Tính quay lại theo cohort, tránh so sánh cohort chưa đủ thời gian |
| DA-056 | Phân tích huỷ chuyến theo thành phố, dịch vụ và thời điểm | TB | Tách thay đổi trong từng nhóm với thay đổi cơ cấu nhóm |
| DA-057 | Thời gian đón và cung cầu: nhìn đuôi phân bố | TB | Phân tích percentile, vùng và khung giờ với độ phủ dữ liệu |
| DA-058 | Khuyến mãi và hiệu quả kinh tế trên mỗi chuyến | NC | Phân biệt doanh số, chi phí, biên đóng góp và giả định phân bổ |
| DA-059 | Phân tích vị trí và vùng phục vụ từ góc nhìn DA | NC | Kiểm tra độ chính xác GPS, cách chia vùng và giới hạn riêng tư |
| DA-060 | Thực hành: dashboard tuần và bản khuyến nghị cho vận hành | NC | Đối soát số, giải thích biến động và đề xuất cách kiểm chứng hành động |

## P07 · Thống kê, thử nghiệm và các yếu tố ảnh hưởng

Đầu vào: kỹ năng SQL/EDA, metric contract và báo cáo P06. Đầu ra: hồ sơ phân tích có đánh giá bất định; phân biệt mô tả, dự báo và kết luận nhân quả.

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| DA-061 | Tổng thể, mẫu và độ biến thiên trong số liệu vận hành | CB | Phân biệt biến động mẫu với thay đổi cần điều tra |
| DA-062 | Trung bình, trung vị, phương sai và percentile | CB | Chọn thống kê mô tả phù hợp phân bố thời gian đón |
| DA-063 | Tương quan khác quan hệ nhân quả như thế nào? | CB | Vẽ giả thuyết về thời tiết, cung tài xế, thời gian đón và huỷ |
| DA-064 | Khoảng tin cậy và bootstrap: diễn đạt sự không chắc chắn | TB | Ước lượng với đơn vị lấy mẫu phù hợp và nêu giả định |
| DA-065 | Kiểm định giả thuyết, effect size và ý nghĩa thực tiễn | TB | Đọc kết quả cùng độ lớn tác động, không chỉ nhìn p-value |
| DA-066 | Mùa vụ, ngày lễ, thời tiết và chiến dịch làm lệch so sánh | TB | Lập danh sách yếu tố gây nhiễu và thiết kế so sánh tương đương |
| DA-067 | Hồi quy và dự báo nhu cầu: baseline trước mô hình phức tạp | TB | Chia tập theo thời gian, tránh leakage và đánh giá sai số |
| DA-068 | A/B test trong thị trường hai phía: ngẫu nhiên hoá và ảnh hưởng chéo | NC | Cân nhắc đơn vị thử nghiệm, power, guardrail và spillover |
| DA-069 | Khi không thể A/B test: giới hạn của dữ liệu quan sát | NC | Đánh giá before/after, nhóm đối chứng và giả định difference-in-differences |
| DA-070 | Thực hành: đánh giá thay đổi điểm đón có giảm huỷ chuyến không? | NC | Bảo vệ thiết kế đo, kết quả, bất định và quyết định tiếp theo |

Các yếu tố phải kiểm tra xuyên suốt: định nghĩa chỉ số; chất lượng ghi nhận; độ phủ mẫu; biến động ngẫu nhiên; thành phần khách/dịch vụ/thành phố; mùa vụ; thời tiết; cung tài xế; giá và khuyến mãi; thay đổi sản phẩm/chính sách; sự cố hệ thống; độ trễ pipeline; yếu tố gây nhiễu và tác động chéo. Đây là giả thuyết cần kiểm chứng, không mặc định là nguyên nhân.

## P08 · Vận hành, quản trị và capstone

Đầu vào: toàn bộ P01–P07. Đầu ra: portfolio DA đầu cuối cho Tới Nơi, kèm cách bàn giao và theo dõi sau quyết định.

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| DA-071 | Tái lập phân tích: Git, phiên bản dữ liệu và môi trường | CB | Ghi nguồn, snapshot, tham số và lệnh chạy lại |
| DA-072 | Phân quyền và vòng đời dữ liệu khách, tài xế, vị trí | CB | Xác định dữ liệu tối thiểu, quyền đọc và nhu cầu lưu giữ |
| DA-073 | Bàn giao báo cáo: owner, lịch cập nhật và runbook | CB | Viết hướng dẫn sử dụng, xử lý lỗi và đầu mối xác nhận |
| DA-074 | EXPLAIN, index và partition cho truy vấn phân tích | TB | Đo trước/sau tối ưu trên engine đã chọn, kiểm tra độ đúng |
| DA-075 | Aggregate, materialized view và cache: nhanh hơn đánh đổi gì? | TB | Định nghĩa freshness và cơ chế làm mới dữ liệu |
| DA-076 | Theo dõi pipeline: freshness, completeness và cảnh báo | TB | Thiết kế SLO dữ liệu, cảnh báo có hành động và kiểm tra số sai |
| DA-077 | Chi phí DA: ngân sách truy vấn, lưu trữ và vận hành | TB | Theo dõi cost theo workload và rà soát lựa chọn P04 |
| DA-078 | Sự cố số liệu: báo cáo sai đã dẫn tới quyết định thì xử lý ra sao? | NC | Lập incident timeline, phạm vi ảnh hưởng, sửa và thông báo lại |
| DA-079 | Review phân tích có AI hỗ trợ và kiểm chứng độc lập | NC | Kiểm tra SQL, nguồn, suy luận, dữ liệu nhạy cảm và tính tái lập |
| DA-080 | Capstone: từ câu hỏi huỷ chuyến tới quyết định và đo lại | NC | Bảo vệ brief, kho dữ liệu, pipeline, phân tích, dashboard và kế hoạch theo dõi |

## Quy chuẩn mỗi bài

1. Mở đầu bằng tình huống Tới Nơi; ghi rõ dữ kiện giả lập, giả thuyết và đề xuất.
2. Ghi ID, phần, mức, bài tiên quyết, mục tiêu và sản phẩm cần hoàn thành.
3. Giải thích khái niệm bằng ngôn ngữ dễ hiểu trước khi đưa công cụ.
4. Ví dụ có định nghĩa dữ liệu, grain, khoảng thời gian và đơn vị; giải thích cách kiểm tra kết quả.
5. Nêu lỗi thường gặp, yếu tố ảnh hưởng và giới hạn kết luận.
6. Bài tập có tiêu chí tự đánh giá, dữ liệu đầu vào và gợi ý lời giải.
7. Kết thúc bằng checklist áp dụng và câu hỏi ôn tập có giải thích.
8. Dẫn nguồn chính thức theo chủ đề và chỉ điều hướng tới bài đã tồn tại.

Mục tiêu khoảng 10–18 phút đọc; bài thực hành 18–25 phút, điều chỉnh theo nội dung thực tế. Bài chọn database phải có workload và cách kiểm chứng; bài thống kê phải có giả định; bài dashboard phải chỉ ra người ra quyết định và hành động dự kiến.

## Hồ sơ Tới Nơi và tính nhất quán

- Nền nghiệp vụ: [Dự án mẫu Tới Nơi](../api/seed/content/du-an-mau-toi-noi.md). Giữ nguyên trạng thái, chính sách và kiến trúc đã định nghĩa; mở rộng phục vụ DA phải được ghi là đề xuất học tập.
- Hồ sơ riêng: [Thực hành DA](../examples/toi-noi-da/README.md). Dữ liệu dự kiến là dữ liệu tổng hợp, không dùng thông tin khách/tài xế thật.
- Kế thừa câu hỏi điểm đón từ [series BA](ba-master-curriculum.md), kỹ thuật dữ liệu từ [series Python](python-master-curriculum.md) và ngữ cảnh vị trí từ [series Location](location-master-curriculum.md). Các nguồn này không chứng minh hiệu quả của một can thiệp.
- Không gộp số liệu của các bài tập BA vào dataset DA khi chưa có mapping và kiểm tra grain, cohort, thời gian. Số cuộc gọi CSKH không đồng nghĩa số khách hay số chuyến.
- Chốt riêng tỷ lệ huỷ trên toàn bộ yêu cầu và tỷ lệ huỷ trước đón trên lượt đã được nhận; không đổi mẫu số giữa baseline và kỳ đánh giá.
- Hợp đồng metric phải có tên, ý nghĩa, grain, tử số/mẫu số, điều kiện loại trừ, cửa sổ thời gian, timezone, chính sách dữ liệu trễ, owner và version.
- Mọi bảng/mart/event mới trong khung đều là thiết kế dự kiến cho bài học; không ngụ ý đã có trong schema ứng dụng.

## Tiến độ và thứ tự triển khai

**Đã dựng khung P01–P08; DA-001 đã viết trong source, DA-002–DA-080 còn ở kế hoạch. Source chưa đồng nghĩa đã xuất bản.**

| ID | Slug | Trạng thái source |
|---|---|---|
| DA-001 | `da-la-gi-tu-so-lieu-toi-quyet-dinh` | Đã viết |

Viết P01 trước để chốt câu hỏi và metric contract. Tiếp đó tạo dataset nhỏ có kết quả đối soát cho P02–P03, rồi mới mở rộng dữ liệu và workload để khảo sát database ở P04. Mỗi phần hoàn thành cùng hồ sơ thực hành và kiểm tra đầu ra trước khi chuyển phần tiếp theo.

Khi đưa bài vào source: dùng ID `DA-xxx`, slug ASCII có tiền tố `da-`, level `beginner` / `intermediate` / `advanced`; đề xuất category `data-analysis` và đăng ký khi có nội dung thật. Cập nhật series/learning path theo cơ chế hiện có, mỗi phần tối đa 10 bài, không coi việc viết source là đã xuất bản.
