# Khảo sát chất lượng tiếng Việt: nhóm location-*, postgresql-*, heap-ctid-va-primary-key, page-don-vi-doc-ghi

Thư mục: `H:\Work\tech-blog\api\seed\content`
Chỉ đọc. Không sửa file nội dung nào.

## 0. Phạm vi và cách làm

- Số file trong nhóm: **100** (location-*: 62; postgresql-*: 36; heap-ctid-va-primary-key.md; page-don-vi-doc-ghi.md). Tổng 606 file .md trong thư mục content.
- Đã rà toàn bộ `title:` và `summary:` của 100 file (dòng 3–4 mỗi file).
- Đã đọc phần văn xuôi của **8 bài**:
  - Đọc đầy đủ (≤300 dòng): `location-bai-toan-quang-duong-khach-va-tai-xe.md`, `location-ham-muc-tieu-diem-hen-khach-va-tai-xe.md`, `location-kho-vi-tri-moi-nhat-h3-redis-geo-kafka.md`, `location-cuoc-quote-giai-thich-doi-diem-don-cho-khach.md`.
  - Đọc 120 dòng đầu: `postgresql-la-gi-va-vai-tro-trong-toi-noi.md`, `postgresql-mvcc-snapshot.md`, `heap-ctid-va-primary-key.md`, `page-don-vi-doc-ghi.md`.
- Đã Grep đếm cặp thuật ngữ trên toàn bộ thư mục content (606 file).

Ghi chú: số dòng trong các bảng là gần đúng (có thể lệch 1–2 dòng). Số lượng "lỗi" là ước lượng, không phải đếm chính xác tuyệt đối. Một số mục ở mức "thấp" có thể coi là lựa chọn văn phong.

## 1. Mật độ ước tính

- Phần tiêu đề và tóm tắt (100 bài): khoảng **35 mục** đáng sửa, tương đương ~0,35 mục/bài. Phần lớn là gượng hoặc khó hiểu, ít khi sai nghĩa.
- Phần thân (8 bài đã đọc): khoảng **36 mục**, tương đương ~4–5 mục/bài. Trong đó khoảng 2 mục/bài ở mức trung bình trở lên.
- Phân loại tổng (ước lượng cho các mục đã ghi dưới đây, khoảng 70 mục):
  - Loại A (thuật ngữ bị dịch sai/gượng): ~26
  - Loại B (diễn đạt không thuần Việt): ~44
  - Mức cao (sai nghĩa hoặc khó hiểu): ~7
  - Mức trung bình (gượng): ~25
  - Mức thấp (có thể tốt hơn): ~38

Nhận xét chung: lỗi phổ biến nhất là **câu cụt, thiếu động từ/danh từ** (loại B, ví dụ "cùng câu trả 7 dòng lẫn..."), và **giữ nguyên tiếng Anh rồi ghép sai cấu trúc** ("accept idempotent", "margin rủi ro", "chain giảm khách"). Thứ hai là **dịch từng chữ** một số thuật ngữ ("hệ độ", "thước nới/thước chặt", "chuỗi lùi", "kết quả rỗng"). Thứ ba là **chính tả không đồng nhất** (huỷ/hủy, khoá/khóa, luỹ/lũy) và một lỗi gõ lặp lại nhiều lần ("thóat").

## 2. Lỗi theo file (file:dòng, câu gốc, loại, đề xuất, mức độ)

### 2.1 Tiêu đề và tóm tắt (title dòng 3, summary dòng 4)

| # | File:dòng | Câu gốc (trích ngắn) | Loại | Đề xuất | Mức |
|---|---|---|---|---|---|
| 1 | location-kho-vi-tri-moi-nhat-h3-redis-geo-kafka.md:4 | "...Kafka hoặc để nguội (đầy lại sau 4 giây)" | A | "...Kafka hoặc khởi động nguội (đầy lại sau 4 giây)" | cao |
| 2 | location-cat-tia-ung-vien-h3-ma-tran-thua.md:4 | "Bộ giải đặc nên vùng khổng lồ vẫn chậm" | A | Cần xác minh với tác giả. "đặc" nghĩa là dense, trong khi ngữ cảnh là ma trận thưa (sparse). Nếu là bộ giải ma trận thưa thì sửa thành "Bộ giải ma trận thưa nên vùng khổng lồ vẫn chậm." | cao |
| 3 | location-chain-dispatch-tai-xe-sap-tra-khach.md:4 | "...margin rủi ro không thấy giúp" | A | "...biên rủi ro không thấy giúp" | trung bình |
| 4 | location-ma-tran-many-to-many-bucket-ch.md:4 | "...đo rayon, ngân sách độ trễ" | A | "...đo bán kính tìm kiếm, ngân sách độ trễ" | trung bình |
| 5 | location-geometry-crate-geo-chieu-diem-polyline-bearing.md:3 | "...hệ độ so với hệ mét trong crate geo" | A | "...hệ tọa độ theo độ so với hệ mét trong crate geo" | trung bình |
| 6 | location-map-matching-hmm-viterbi-rust.md:4 | "...HMM 86,87% cạnh nới nút (69,09% chặt)... thước nới... thước chặt" | A | "ngưỡng nới / ngưỡng chặt" thay cho "thước nới / thước chặt" | trung bình |
| 7 | location-cua-so-lo-do-tre-doi-chat-luong-ghep.md:4 | "...Cửa sổ thích ứng không thắng cố định" | B | "...Cửa sổ thích ứng không hơn cửa sổ cố định" | trung bình |
| 8 | location-cua-so-lo-do-tre-doi-chat-luong-ghep.md:4 | "Ngày thường cửa sổ chỉ thêm chờ, kết quả rỗng" | A | "...chỉ thêm thời gian chờ, kết quả không có ý nghĩa thống kê" | trung bình |
| 9 | location-diem-hen-dong-cap-nhat-khi-hai-ben-di-chuyen.md:3 | "...và kết quả rỗng của hysteresis" | A | "...và kết quả không có hiệu ứng của hysteresis" | trung bình |
| 10 | location-bai-toan-quang-duong-khach-va-tai-xe.md:4 | "Vì sao tối ưu cho một bên đều sai" | B | "Vì sao tối ưu cho một bên thì đều sai" | thấp |
| 11 | location-lech-tuyen-dinh-tuyen-lai-ghep-lai.md:4 | "...và biên chỉ ghép lại khi lợi đủ lớn" | B | "...và chỉ ghép lại khi lợi đủ lớn vượt một ngưỡng biên" | trung bình |
| 12 | location-chi-phi-ghep-cong-bang-tai-xe-rui-ro-huy.md:4 | "...ngoại vi hoàn thành −0,031" | B | Làm rõ "ngoại vi" là "vùng ngoại vi" hoặc viết lại, ví dụ "tỉ lệ hoàn thành ở vùng ngoại vi giảm 0,031" | trung bình |
| 13 | location-chi-phi-ghep-cong-bang-tai-xe-rui-ro-huy.md:4 | "Khi ghép ngay, hai số hạng không có gì để tác động" | B | "Khi ghép ngay, hai số hạng này không có gì để tác động" | thấp |
| 14 | location-h3-geohash-chi-muc-luoi-k-ring.md:4 | "...và biết khi nào lưới hơn cây" | B | "...và biết khi nào lưới tốt hơn cây" | thấp |
| 15 | location-hoc-diem-don-tu-lich-su-dbscan-python.md:3 | "...vòng tròn và kết quả rỗng" (title); "lịch sử là vòng tròn và thưa" (summary) | A | "vòng tròn" đang mơ hồ (hình học hay vòng lặp?). Cần tác giả giải thích. Đề xuất "kết quả rỗng" thành "kết quả không có ý nghĩa" | trung bình |
| 16 | location-go-backpressure-coalesce-drop-race-detector.md:4 | "...và tắt êm trong hạn" | A | "...và dừng êm trong thời hạn cho phép" (graceful shutdown) | trung bình |
| 17 | location-goi-y-sang-dispatch-csharp-grpc-lease-idempotency.md:4 | "...accept idempotent, gợi ý cũ..." | B | "...chấp nhận có tính idempotent, gợi ý cũ..." hoặc "...accept lặp lại an toàn..." | trung bình |
| 18 | location-nhieu-diem-dung-chen-2-opt-doi-chieu-ortools.md:4 | "...cách OR-Tools 0,39 đến 2,77%" | B | "...lệch so với OR-Tools 0,39 đến 2,77%" | trung bình |
| 19 | location-profile-xe-may-o-to-di-bo-trong-hem.md:4 | "...quyền đi đọc từ thẻ OSM" | B | "...quyền đi lại đọc từ thẻ OSM" | trung bình |
| 20 | location-geocoding-dia-chi-viet-nam-autocomplete.md:4 | "...kiểm đổi tên hành chính 2025 chưa đầy đủ" | B | "...kiểm tra việc đổi tên đơn vị hành chính 2025 chưa đầy đủ" | trung bình |
| 21 | location-tong-hop-geo-core-kiem-thu-hinh-hoc.md:4 | "...danh sách việc chưa làm hoặc chưa tra của P01" | B | "...danh sách việc chưa làm và chưa kiểm chứng của P01" | trung bình |
| 22 | location-self-host-osrm-valhalla-hay-map-provider.md:4 | "...port RoutingProvider để thay adapter" | A | Chấp nhận được nếu giữ "port" làm thuật ngữ kiến trúc (hexagonal). Nếu không, "giao diện RoutingProvider" | thấp |
| 23 | location-adr-rust-go-python-csharp-ranh-gioi-service.md:4 | "Go và Python giữ chỗ chúng có lợi thật" | B | "Go và Python được giữ lại ở những chỗ chúng có lợi thật" | trung bình |
| 24 | location-hem-chung-cu-tttm-san-bay-cong-sanh.md:4 | "...chuỗi lùi không âm thầm" | A | "...chuỗi dự phòng (fallback) không âm thầm" | trung bình |
| 26 | location-toi-uu-diem-tra-cong-vao-chuyen-ke-tiep.md:4 | "...chi phí thóat của tài xế" | chính tả | "...chi phí thoát của tài xế" | cao (lỗi gõ rõ) |
| 27 | location-toi-uu-diem-tra-cong-vao-chuyen-ke-tiep.md:3 | "...dễ bắt chuyến kế" | B | "...dễ bắt chuyến tiếp theo" | thấp |
| 28 | location-cuoc-quote-giai-thich-doi-diem-don-cho-khach.md:4 | "...quote bất biến, không tăng giá lén" | B | "...giá bất biến, không tăng giá âm thầm" | thấp |
| 29 | location-ham-muc-tieu-diem-hen-khach-va-tai-xe.md:4 | "...và một lần sửa quy tắc chọn trọng số sau khi thấy kết quả, có dải seed xác nhận" | B | "...và một lần sửa quy tắc chọn trọng số sau khi thấy kết quả, kèm dải seed xác nhận" | thấp |
| 30 | location-tong-hop-mo-phong-su-kien-roi-rac-seed.md:4 | "...cùng seed ra từng byte" | B | "...cùng seed cho ra kết quả giống nhau từng byte" | thấp |
| 32 | postgresql-aggregate-group-by-having.md:4 | "tỉ lệ huỷ TP. HCM" | chính tả | "hủy" (xem mục 3.2) | thấp |
| 33 | postgresql-pagination-keyset.md:4 | "so sánh bộ (requested_at, code)" | A | "so sánh theo bộ giá trị (row comparison)" | thấp |
| 34 | postgresql-sort-hash-memory-spill.md:4 | "đo bộ nhớ nhân theo nút, phiên, worker" | B | "đo bộ nhớ nhân lên theo nút, phiên, worker" | trung bình |
| 35 | postgresql-thuc-hanh-runbook-phuc-hoi.md:4 | "RPO/RTO phân rã trung thực" | B | "RPO/RTO được tách ra từng phần một cách trung thực" | trung bình |
| 36 | postgresql-thuc-hanh-toi-uu-truy-van.md:3 | "Thực hành: tối ưu truy vấn bằng bằng chứng" | B | "Thực hành: tối ưu truy vấn dựa trên bằng chứng" | thấp |
| 37 | postgresql-window-function-frame.md:4 | "để luỹ kế, xếp hạng..." | chính tả | "lũy kế" | thấp |
| 38 | postgresql-upsert-on-conflict.md:4 | "chạy lại không đổi gì" | B | "chạy lại không làm thay đổi dữ liệu" | thấp |
| 39 | postgresql-explain-ke-hoach-thuc-thi.md:4 | "chỗ khác giữa kế hoạch với thực thi" | B | "chênh lệch giữa kế hoạch và thực thi" | thấp |
| 40 | postgresql-thuc-hanh-sql-doi-soat.md:4 | "lời giải thích cho từng đồng chênh +268.000 đ" | B | "lời giải thích cho từng khoản chênh lệch +268.000 đ" | thấp |
| 41 | postgresql-insert-update-delete-returning.md:4 | "...và thẻ lệnh" | A | "command tag" nên giữ tiếng Anh hoặc dịch "thẻ lệnh (command tag)" ở lần đầu | thấp |
| 42 | postgresql-kieu-du-lieu-tien-thoi-gian-uuid.md:4 | "...và lập data dictionary" | A | "...và lập từ điển dữ liệu" | thấp |
| 43 | postgresql-thuc-hanh-tranh-chap-nhan-chuyen.md:4 | "bàn giao lịch chạy" | A | "bàn giao kịch bản chạy" hoặc "bàn giao lịch chạy lab" | thấp |
| 44 | postgresql-role-grant-least-privilege.md:4 | "và cái bẫy default privileges" | A | Chấp nhận, giữ tiếng Anh có chú thích | thấp |

### 2.2 Phần thân (đã đọc 8 bài)

#### location-bai-toan-quang-duong-khach-va-tai-xe.md

| Dòng | Câu gốc (trích ngắn) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 122 | "...và hoàn thiện workspace, tải, quan sát, triển khai." | B | "...và hoàn thiện workspace, phần tải, quan sát và triển khai." | trung bình |
| 52 | "...năm phương án hòa nhau" | B | Chấp nhận, nhưng "ngang nhau" tự nhiên hơn | thấp |

#### location-ham-muc-tieu-diem-hen-khach-va-tai-xe.md

| Dòng | Câu gốc (trích ngắn) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 105 | "Ràng buộc cứng khác khoản phạt." | B | "Ràng buộc cứng khác với khoản phạt." (thiếu "với") | trung bình |
| 81 | "Và nó không cân đều ở biên" | B | "Và nó không cân đều trong trường hợp biên" hoặc viết rõ hơn: "không cân đều khi tài xế là bên tới sau" | trung bình |
| 107 | "...chỉ bị thắng khi có ứng viên J nhỏ hơn" | B | Chấp nhận | thấp |
| 58 | "Mã chỉ là một biểu thức:" | B | "Phần mã chỉ là một biểu thức:" | thấp |
| 219 | "Cổng p95 không thấy gì, vì p95 của baseline..." | B | "Cổng p95 không phát hiện được vấn đề, vì p95 của baseline..." | trung bình |
| 225 | "...trượt cả hai, dù trượt cổng trung bình chỉ 0,6 m" | B | "...trượt cả hai; dù chỉ vượt ngưỡng cổng trung bình 0,6 m" | thấp |
| 125 | "Khử một phần: sau khi hai lần chạy rẽ nhánh (ghép khác nhau)" | A | Chấp nhận, "rẽ nhánh" là cách dùng quen thuộc | thấp |
| 223 | "...một ví dụ của «khu vườn những lối rẽ»" | A | Chấp nhận, nhưng "lối rẽ" lặp lại nhiều lần; cân nhắc "nhánh quyết định" | thấp |
| 146–151 | "...khoảng tin cậy ... loại 0" (nhiều chỗ trong bài) | A | "khoảng không chứa 0" | trung bình |
| 268 | "Trọng số a = c là một lập trường" | A | Chấp nhận | thấp |

Cụm "khoảng ... loại 0" xuất hiện nhiều lần trong bài này và trong bài cuoc-quote (xem 2.3).

#### location-kho-vi-tri-moi-nhat-h3-redis-geo-kafka.md

| Dòng | Câu gốc (trích ngắn) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 13 | "hồ sơ tham chiếu ở bài 09 là 70" | B/A | "tải tham chiếu ở bài 09 là 70 truy vấn/giây" | trung bình |
| 25 | "giờ ingest nhận lô theo đồng hồ server" | B | Chấp nhận, nhưng "giờ nhận lô" tự nhiên hơn | thấp |
| 201 | "...nên readiness probe ... phải chờ kho đủ ấm thay vì chỉ chờ tiến trình lên." | B | "...phải chờ kho đủ dữ liệu thay vì chỉ chờ tiến trình khởi động xong." | trung bình |
| 201 | "...kho tự đầy lại trong 4 giây..." / "đủ ấm" | A | "đủ ấm" nên là "đã nạp đủ dữ liệu" | thấp |
| 225 | "...giữ một điểm «còn sống» mỗi 60 giây khi đứng yên" | A | "...giữ một điểm nhịp (heartbeat) mỗi 60 giây khi đứng yên" | trung bình |
| 225 | "...9 nhịp, 0 cờ" | B | "...9 điểm nhịp, 0 điểm có cờ" | thấp |
| 190 | (bảng) "Nguội hoàn toàn (Start::Latest)" | A | "Khởi động nguội (không nạp dữ liệu)" | thấp |

#### location-cuoc-quote-giai-thich-doi-diem-don-cho-khach.md

| Dòng | Câu gốc (trích ngắn) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 13 | "...không tăng giá lén" | B | "...không tăng giá âm thầm" | thấp |
| 132 | "...cước giảm nhưng khách đi bộ thêm 90 m vẫn là một thay đổi khách nên biết" | B | "...vẫn là một thay đổi mà khách nên biết" | trung bình |
| 160 | "Giấu quãng đi bộ khi cước giảm là cách làm khách thấy mình bị chọn thay." | B | "...là cách khiến khách cảm thấy bị quyết định thay." | thấp |
| 164 | "...chúng là bản nháp có số liệu đúng." | B | Chưa rõ ý. Cân nhắc "chúng là bản nháp, số liệu đã kiểm" | trung bình |
| 185 | "Phần P04 chạy trên một nguyên tắc: không chỉnh tới khi đẹp." | B | "Phần P04 chạy theo nguyên tắc không chỉnh cho đến khi kết quả đẹp." | trung bình |
| 196 | "...cơ chế an toàn (chuỗi lùi ghi cảnh báo)" | A | "...cơ chế an toàn (chuỗi dự phòng ghi cảnh báo)" | trung bình |
| 199 (bảng) / 201 | "...ngưỡng đặt trước quá chặt..., kết quả rỗng" | A | "kết quả không có ý nghĩa thống kê" | cao |
| 218 | "...hai khoảng... không nên đọc là chắc chắn... khoảng vừa đủ loại 0" | A | "...khoảng vừa đủ không chứa 0" | trung bình |
| 218 | "Cùng cấu hình, khác dải seed, hai chỉ số đổi nhãn." | A | "...hai chỉ số đổi kết luận" | thấp |
| 273 | "...băng đường của khách..." | B | Chấp nhận, nhưng "thời gian băng qua đường" rõ hơn | thấp |
| 271 | "...câu chữ chưa được thử với người dùng, chỉ có số liệu đúng." | B | Ý chưa rõ. Đề xuất: "câu chữ chưa được thử với người dùng thật; hiện chỉ kiểm tra số liệu" | trung bình |
| 240 | "...chưa loại được một mức giảm lớn" | A | "chưa loại trừ được một mức giảm lớn" | thấp |

#### postgresql-la-gi-va-vai-tro-trong-toi-noi.md (120 dòng đầu)

| Dòng | Câu gốc (trích ngắn) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 51 | "...luật dữ liệu ở Phần 1, truy vấn ở Phần 2, lưu trữ và index ở Phần 3, đồng thời ở Phần 4, độ bền và phục hồi ở Phần 5." | B | "...xử lý đồng thời ở Phần 4..." | trung bình |
| 81 | (cụm "không phải trang trí") | B | "không phải chi tiết thừa" (xem thêm mục 3.3) | thấp |

#### postgresql-mvcc-snapshot.md (120 dòng đầu)

| Dòng | Câu gốc (trích ngắn) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 90 | "Bản mới cùng page và không cột có index nào bị đổi nên đây là HOT update" | B | "Bản mới nằm cùng page và không có cột nào có index bị đổi, nên đây là HOT update" | trung bình |
| 99–100 | "...(commit hoặc huỷ)" | chính tả | "hủy" | thấp |
| 11 | "Màn có phải đứng chờ giao dịch kia xong không?" | B | Chấp nhận | thấp |

#### heap-ctid-va-primary-key.md (120 dòng đầu)

| Dòng | Câu gốc (trích ngắn) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 13 (tiêu đề và thân) | "Clustered index thật — lớp vật lý: tệp heap..." | A | Ghi chú cho tác giả: heap không phải clustered index. Đây là lối nói dễ hiểu sai. Đề xuất đổi thành "lớp vật lý (heap)" thay vì gọi là "clustered index thật". Đây không phải lỗi tiếng Việt, mà là lỗi khái niệm có thể gây hiểu nhầm. | trung bình |
| 18 (callout) | "...khoá..." (chính tả) | chính tả | "khóa" nếu theo chuẩn chính tả hiện hành | thấp |

#### page-don-vi-doc-ghi.md (120 dòng đầu)

| Dòng | Câu gốc (trích ngắn) | Loại | Đề xuất | Mức |
|---|---|---|---|---|
| 81 | "Dòng relnamespace không phải trang trí." | B | "Dòng relnamespace không phải chi tiết thừa." | thấp |
| 81 | "...bỏ điều kiện schema thì cùng câu trả 7 dòng lẫn của hai bảng." | B | "...bỏ điều kiện schema thì cùng một câu lệnh trả về 7 dòng, lẫn dòng của hai bảng." | trung bình |

## 3. Cặp thuật ngữ và số lần xuất hiện (Grep trên toàn bộ thư mục content, 606 file)

Ghi chú: "số lần" là số lần khớp chuỗi, không lọc theo ngữ cảnh. Phần lớn đã kiểm tra thấy đúng nghĩa, một số có thể gồm cách dùng khác.

| # | Từ đang dùng | Đề xuất | Số lần | Số file | Nhận xét |
|---|---|---|---|---|---|
| 1 | margin | biên (hoặc giữ margin có chú thích) | 34 | 12 | Dịch dở dang, nên đồng nhất |
| 2 | còn sống (điểm nhịp / heartbeat) | điểm nhịp (heartbeat) | 34 | 29 | Gượng, cần kiểm từng ca |
| 3 | kết quả rỗng (null result) | kết quả không có ý nghĩa thống kê / kết quả không có hiệu ứng | 15 | 6 | Dịch từng chữ, dễ hiểu sai là "rỗng" như dữ liệu trống |
| 4 | loại 0 (khoảng tin cậy loại 0) | khoảng không chứa 0 | 17 | 5 | Calque của "exclude zero" |
| 5 | thóat (lỗi gõ) | thoát | 35 | 4 | Lỗi gõ lặp lại, sửa hàng loạt |
| 6 | thước nới / thước chặt | ngưỡng nới / ngưỡng chặt | 14 | 3 | Dịch từng chữ, không tự nhiên |
| 7 | chuỗi lùi | chuỗi dự phòng (fallback) | 12 | 2 | Khó hiểu |
| 8 | tắt êm | dừng êm / tắt an toàn (graceful shutdown) | 13 | 4 | Gượng |
| 9 | hệ độ | hệ tọa độ theo độ | 7 | 3 | Gượng |
| 10 | lối rẽ (forking path) | nhánh quyết định | 7 | 2 | Gượng, dễ hiểu nhầm |
| 11 | tuple chết / dòng chết (dead tuple) | thống nhất một cách gọi | 31 / 17 | 5 / 6 | Đang dùng lẫn hai cách |
| 12 | huỷ / hủy | hủy (chuẩn hiện hành) | 1622 / 386 | 206 / 53 | Dùng lẫn, cần đồng nhất (cả đường dẫn lỗi chính tả) |
| 13 | khoá / khóa | khóa | 1859 / 933 | 247 / 133 | Dùng lẫn, cần đồng nhất |
| 14 | luỹ / lũy | lũy | — | — | Có ít nhất 1 ca trong nhóm (window-function-frame) |
| 15 | idempotent | giữ nguyên tiếng Anh | 209 | 87 | Chấp nhận, không nên dịch máy móc |

Lưu ý cho tác giả về chính tả: "huỷ/khoá/luỹ" là cách viết cũ. Nếu đây là chuẩn của blog, nên đồng nhất toàn bộ thư mục, không chỉ nhóm này.

## 4. Ví dụ tiêu biểu (gốc → đề xuất)

1. `location-cat-tia-ung-vien-h3-ma-tran-thua.md:4`: "Bộ giải đặc nên vùng khổng lồ vẫn chậm" → "Bộ giải ma trận thưa nên vùng khổng lồ vẫn chậm" (cần xác minh với tác giả trước khi sửa).
2. `location-kho-vi-tri-moi-nhat-h3-redis-geo-kafka.md:4`: "...Kafka hoặc để nguội (đầy lại sau 4 giây)" → "...Kafka hoặc khởi động nguội (đầy lại sau 4 giây)".
3. `location-cuoc-quote-giai-thich-doi-diem-don-cho-khach.md:196`: "cơ chế an toàn (chuỗi lùi ghi cảnh báo)" → "cơ chế an toàn (chuỗi dự phòng ghi cảnh báo)".

## 5. Nhận xét ngắn

- Lỗi phổ biến nhất là **dịch từng chữ và giữ nguyên tiếng Anh một cách gượng** ("margin", "chain", "accept idempotent", "clustered index" dùng sai). Đây là kiểu A nổi bật nhất.
- Thứ hai là **câu cụt, thiếu danh từ/động từ** trong tóm tắt ("Cửa sổ thích ứng không thắng cố định", "Cổng p95 không thấy gì", "hoàn thiện workspace, tải, quan sát, triển khai"). Lỗi này nằm chủ yếu ở loại B và đọc vẫn hiểu nhưng không mượt.
- Thứ ba là **lỗi gõ và chính tả không đồng nhất**: "thóat" (35 lần), "huỷ/hủy", "khoá/khóa", "luỹ/lũy".
- Thuật ngữ thống kê "kết quả rỗng" và "khoảng ... loại 0" lặp lại nhiều lần, và nên thống nhất một cách gọi cho cả nhóm.
- Ngoài phạm vi ngôn ngữ, một chỗ có khả năng gây hiểu nhầm về khái niệm: tiêu đề bài heap-ctid-va-primary-key gọi heap là "clustered index thật". Nên tác giả xem lại.
- Phần thân các bài kỹ thuật chủ yếu tiếng Việt tự nhiên, thuật ngữ như heap, ctid, WAL, B-tree, snapshot được giữ đúng. Mức lỗi tập trung ở tóm tắt và các đoạn tự viết nhanh.

## 6. Giới hạn của khảo sát

- Chỉ đọc 8 bài (4 bài đầy đủ, 4 bài 120 dòng đầu). Phần còn lại chỉ rà qua tiêu đề và tóm tắt.
- Mục đếm lần xuất hiện là Grep chuỗi thô, chưa lọc từng ngữ cảnh. Nên kiểm tay khi sửa hàng loạt.
- Một số mục ở mức thấp có thể là lựa chọn văn phong hợp lệ. Trước khi sửa, nên thống nhất với biên tập viên chính.
- Không có file nào trong thư mục content bị sửa trong khảo sát này.
