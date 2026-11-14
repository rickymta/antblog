# Khảo sát ngôn ngữ: nhóm dotnet10 / csharp / da / nhom / design-pattern / du-an-mau

Thư mục: `H:\Work\tech-blog\api\seed\content`
Chế độ: chỉ đọc, không sửa file nội dung.

## 1. Phạm vi

- Số file trong nhóm: 64.
- Đọc kỹ (văn xuôi, bỏ khối code, khoảng 300 dòng đầu): 8 bài.
  - `da-la-gi-tu-so-lieu-toi-quyet-dinh.md`
  - `da-join-cardinality-dem-trung.md`
  - `da-tien-hoan-tien-doi-soat.md`
  - `csharp-bien-kieu-du-lieu-decimal.md`
  - `csharp-async-await-cancellation.md`
  - `dotnet10-transaction-concurrency-idempotency.md`
  - `nhom-hanh-vi-dong-goi-hanh-vi.md`
  - `du-an-mau-toi-noi.md`
- Rà `title:` và `summary:` của toàn bộ 64 file.
- Grep mẫu lỗi trên toàn nhóm, kiểm tra thêm một số dòng ở các file chưa đọc kỹ (dotnet10-efcore-*, dotnet10-outbox-*, dotnet10-use-case-*, csharp-concurrency-*, csharp-interface-*, da-thuc-hanh-*, da-kpi-*, da-ngay-gio-*, design-pattern-la-gi.md).

Ghi chú về ký hiệu: A = thuật ngữ dịch sai hoặc gượng; B = diễn đạt không thuần Việt. Mức độ: cao = sai nghĩa/khó hiểu; trung bình = gượng; thấp = có thể tốt hơn.

## 2. Lỗi trong các bài đã đọc kỹ

### csharp-async-await-cancellation.md

| Dòng | Câu gốc (trích) | Loại | Mức | Đề xuất |
|---|---|---|---|---|
| 25 | "`Task<QuoteReply>` biểu diễn một tác vụ cuối cùng có thể trả kết quả, lỗi hoặc bị hủy" | B | TB | "biểu diễn một tác vụ sẽ hoàn tất, có thể trả kết quả, lỗi hoặc bị hủy" |
| 33 | "HttpClient của lab sống cho cả nhóm thao tác" | B | TB | "HttpClient của lab được dùng chung cho mọi thao tác trong bài" |
| 33 | "dùng chiến lược lifetime thích hợp" | A | TB | "dùng vòng đời (lifetime) phù hợp" |
| 42 | Tiêu đề "Cancellation là một đề nghị có hợp tác" | B | TB | "Hủy là một yêu cầu mang tính hợp tác" |
| 44 | "Cancel không giống cưỡng bức giết thread và không tự rollback điều đã commit" | A/B | TB | "Hủy không phải là buộc dừng thread, và không tự hoàn tác điều đã commit ở hệ thống khác" |
| 46 | "Pre-cancelled token giúp kiểm tra rằng lời gọi không thành công khi hủy đã được yêu cầu" | A/B | TB | "Token đã bị hủy từ trước giúp kiểm tra lời gọi thất bại khi việc hủy đã được yêu cầu" |
| 48 | "phân biệt token caller với timeout" | B | Thấp | "phân biệt token của bên gọi với timeout" |
| 52 | "'Fire-and-forget' trong một request có thể làm lỗi mất người quan sát" | B | Cao | "Fire-and-forget trong một request có thể khiến lỗi không còn ai theo dõi, và dependency scoped bị dispose trước khi việc nền hoàn tất" |
| 54 | "nó không tự giới hạn fan-out" | A | Thấp | "không tự giới hạn số lời gọi song song" |
| 56 | "Sau khi hủy HTTP, không mặc định server chưa nhận hoặc chưa thực hiện thao tác" | B | TB | "Sau khi hủy HTTP, không thể giả định server chưa nhận hoặc chưa thực hiện thao tác" |
| 56 | "Async client không xóa bài toán nhất quán liên service" | B | TB | "Dùng async không giải quyết được bài toán nhất quán giữa các service" |
| 31 | "có nguy cơ deadlock khi kết hợp context" | B | Thấp | "khi kết hợp với synchronization context" |
| 69 | "Hủy còn làm pending kết thúc nhanh không?" (đáp án tương tự) | A | Thấp | "tác vụ đang chờ (pending)" |
| 48 | "Giới hạn 5 giây trong WaitAsync chỉ là watchdog" | A | Thấp | "chỉ là cơ chế canh thời gian (watchdog)" |

### dotnet10-transaction-concurrency-idempotency.md

| Dòng | Câu gốc (trích) | Loại | Mức | Đề xuất |
|---|---|---|---|---|
| 29 | "BookTrip thêm TripRow cùng BookingRequestRow trong một lần SaveChanges: không được commit khóa xử lý nhưng mất chuyến, hoặc commit chuyến mà mất khóa." | B | Cao | Câu bị cụt, sai cấu trúc. Đề xuất: "BookTrip thêm TripRow và BookingRequestRow trong cùng một lần SaveChanges, nên không thể xảy ra trường hợp khóa xử lý được commit mà chuyến bị mất, hoặc chuyến được commit mà khóa bị mất." |
| 31 | "rollback database không có nghĩa mọi object trong tracker tự khôi phục trạng thái ban đầu cho bạn" | B | TB | "rollback ở database không có nghĩa là các object đang được change tracker theo dõi cũng tự trở về trạng thái ban đầu." |
| 33 | "tránh I/O ngoài database bên trong" | B | TB | "tránh thao tác I/O ra bên ngoài database khi đang trong transaction" |
| 37-39 | Caption: "không có một commit riêng dễ tạo khoảng trống" | B | TB | "không có commit riêng nào có thể tạo khoảng trống giữa hai bước" |
| 15 | "Bắt concurrency exception rồi SaveChanges lại ngay" | A | Thấp | "Bắt ngoại lệ xung đột đồng thời rồi SaveChanges lại ngay" |
| 25 | "dịch vụ tài chính Rust hoặc scoring Python" | A/B | TB | "dịch vụ tài chính viết bằng Rust hoặc dịch vụ chấm điểm (scoring) bằng Python" |
| 4 | Tóm tắt: "để một ý định đặt chuyến có một kết quả bền" | B | Thấp | "để một ý định đặt chuyến chỉ cho ra một kết quả được lưu bền vững" |
| 46 | "Nếu một writer quên đổi version" | A/B | TB | "Nếu một tiến trình ghi quên đổi version" |
| 48 | "Không tự động retry ghi đè bằng version mới chỉ để exception biến mất" | B | TB | "Không tự động thử lại và ghi đè bằng version mới chỉ để lỗi không còn xuất hiện" |
| 52 | "khác input bị từ chối" | B | TB | "khác dữ liệu đầu vào thì bị từ chối" |
| 52 | "Các field tham gia ý định phải được so đầy đủ" | A/B | TB | "Các trường tham gia vào ý định phải được so khớp đầy đủ" |
| 54 | "candidate thua bị rollback trong SaveChanges" | A | Cao | "bản ghi ứng viên thua bị hoàn tác trong SaveChanges" (từ "candidate" không rõ nghĩa) |
| 56 | "Retention, quyền truy cập và response snapshot" | A/B | TB | "Thời gian lưu giữ, quyền truy cập và snapshot phản hồi đầy đủ" |
| 56 | "không âm thầm đổi ý nghĩa fingerprint" | A/B | TB | "không làm lệch âm thầm dấu vân tay (fingerprint) của ý định" (từ "âm thầm" nên đổi, xem mục 3) |
| 67 | "Hai lời gọi Task.WhenAll là yêu cầu chồng lấp, không khẳng định một lịch lock PostgreSQL cụ thể được ép tại từng bước." | B | Cao | "Hai lời gọi trong Task.WhenAll chỉ tạo hai yêu cầu chạy chồng lấp nhau; bài không khẳng định một thứ tự khóa cụ thể của PostgreSQL được ép ở từng bước." |
| 71 | Đáp án: "khoảng giữa kiểm tra và ghi vẫn race" | A | TB | "vẫn còn tranh chấp (race condition) trong khoảng giữa kiểm tra và ghi" |
| 75 | "Fixture từ chối database khác tên hoặc host ngoài localhost, không âm thầm skip khi thiếu kết nối; nó chỉ tạo và dọn schema GUID riêng." | A/B | Cao | "Fixture từ chối database có tên khác hoặc host ngoài localhost; khi thiếu kết nối thì không lặng lẽ bỏ qua test; nó chỉ tạo và dọn một schema riêng có tên GUID." (Cùng câu xuất hiện ở dotnet10-efcore-postgresql-mapping-migration.md:75, dotnet10-efcore-tracking-projection-n-plus-one.md:75, dotnet10-outbox-inbox-idempotent-consumer.md:75.) |

### da-tien-hoan-tien-doi-soat.md

| Dòng | Câu gốc (trích) | Loại | Mức | Đề xuất |
|---|---|---|---|---|
| 4 (tóm tắt) | "Giải thích chênh lệch theo trạng thái và biên kỳ" | A/B | TB | "theo trạng thái và ranh giới kỳ báo cáo" ("biên kỳ" là từ ghép tự chế) |
| 17 | "Payment succeeded cùng settled_at là khoản thu đã được xác nhận" | B | TB | "Payment có status = succeeded và có settled_at là khoản thu đã được xác nhận" (thiếu động từ, khó đọc) |
| 21 | "Mọi amount là số nguyên VND" | A | Thấp | "Mọi số tiền (amount) là số nguyên VND" |
| 42 | "T01 có attempt thất bại vẫn chỉ thu 100.000 ... Không SUM giá chuyến sau khi nối nhiều payment/refund trực tiếp." | A/B | TB | "lần thử thất bại" thay "attempt"; "Không cộng tổng giá chuyến sau khi nối trực tiếp với nhiều dòng payment/refund." |
| 82 | "Nếu hỏi mọi dòng tiền ride/city1 phát sinh trong tuần" | B | Cao (cần xác nhận) | "city1" không rõ là gì. Nếu là tên khu vực hay cohort, cần ghi rõ; nếu là lỗi gõ thì sửa. |
| 95 | "đúng currency và đúng snapshot" | A | Thấp | "đúng đơn vị tiền tệ và đúng snapshot" |
| 97 | "Một pending refund có ý nghĩa vận hành" | A | TB | "Yêu cầu hoàn tiền đang chờ xử lý có ý nghĩa vận hành" |
| 59 | "còn thiếu hợp đồng về phân chia, phí, thuế" | B | Thấp | "còn thiếu thông tin trong hợp đồng về phân chia, phí, thuế" |

### da-join-cardinality-dem-trung.md

| Dòng | Câu gốc (trích) | Loại | Mức | Đề xuất |
|---|---|---|---|---|
| 28 | "Cả hai attempt đều có amount 100.000" | A | Thấp | "Cả hai lần thử đều có số tiền 100.000" |
| 36 | "không dùng giá trị tiền như khoá dedup" | A | TB | "không dùng giá trị tiền làm khóa khử trùng lặp" ("dedup" không được Việt hóa) |
| 49 | "không tự là guardrail khiếu nại của P01" | A | TB | "không tự là chỉ số bảo vệ về khiếu nại của P01". Bài da-kpi-metric-chi-so-bao-ve.md đã dùng "chỉ số bảo vệ" nên nên thống nhất. |
| 68 | "cần cờ coverage thay vì chỉ điền 0" | A | TB | "cần cờ độ phủ dữ liệu thay vì chỉ điền 0" |
| 85 | "Một bảng review nên có" | A | Thấp | "Một bảng rà soát nên có" |
| 93 | "View da_cash còn tổng hợp refunds riêng" | A | Thấp | "View da_cash còn tổng hợp các khoản hoàn tiền riêng" |
| 103 | "Nó giữ sự xuất hiện của dòng trái không có đối tác" | B | Thấp | "LEFT JOIN vẫn giữ dòng bên trái dù không có dòng khớp bên phải" |

### da-la-gi-tu-so-lieu-toi-quyet-dinh.md

| Dòng | Câu gốc (trích) | Loại | Mức | Đề xuất |
|---|---|---|---|---|
| 32 | "Đây là điều kiện được cấp để luyện đọc số" | B | TB | "Đây là giả định cho sẵn để luyện đọc số" |
| 92 | "Chưa đủ bằng chứng quy nguyên nhân cho điểm đón" | B | TB | "Chưa đủ bằng chứng để kết luận nguyên nhân nằm ở điểm đón" |
| 104 | "chưa có căn cứ phải mua warehouse" | A/B | Thấp | "chưa có căn cứ để đầu tư kho dữ liệu (warehouse)" |

### csharp-bien-kieu-du-lieu-decimal.md

| Dòng | Câu gốc (trích) | Loại | Mức | Đề xuất |
|---|---|---|---|---|
| 30 | "ta sẽ đưa currency và quy tắc vào một kiểu Money riêng" | A | TB | "ta sẽ đưa đơn vị tiền tệ (currency) và quy tắc vào một kiểu Money riêng" |
| 49 | "không phải bắt buộc deploy lại ứng dụng mỗi lần đổi giá" | B | TB | Câu dễ hiểu ngược. Đề xuất: "không nên buộc phải deploy lại ứng dụng mỗi lần đổi giá." |
| 53 | "giữ chính xác các giá trị như 0,1 trong khả năng của nó" | B | Thấp | "giữ chính xác các giá trị thập phân như 0,1 trong phạm vi độ chính xác của nó" |

### nhom-hanh-vi-dong-goi-hanh-vi.md

| Dòng | Câu gốc (trích) | Loại | Mức | Đề xuất |
|---|---|---|---|---|
| 110 | "Việc chọn strategy nào được đẩy ra một hàm riêng" | B | TB | "Việc chọn strategy được tách sang một hàm riêng" |
| 169 | "đơn ở quan-7 ... rơi về strategy dự phòng" | B | Thấp | "chuyển sang strategy dự phòng" |
| 175 | "Strategy ... Context ... không biết bên trong" | — | — | Chấp nhận được. |

### du-an-mau-toi-noi.md

| Dòng | Câu gốc (trích) | Loại | Mức | Đề xuất |
|---|---|---|---|---|
| 11 | "các bài sau chỉ cần nói ... là bạn biết nó to cỡ nào và bị ghi ra sao" | B | Thấp | "...và biết nó bị ghi dữ liệu như thế nào" |
| 59 | "Các luồng liên service sẽ bổ sung cơ chế phù hợp như Outbox, idempotency và bù trừ" | — | — | Chấp nhận được. |
| 65 | "Bảng dưới là định hướng thiết kế của dự án mẫu, không phải kết luận một ngôn ngữ độc quyền một loại bài toán." | B | Cao | Thiếu động từ, khó hiểu. Đề xuất: "không có nghĩa là một ngôn ngữ độc quyền một loại bài toán." |
| 65 | "Tới Nơi dùng chung nghiệp vụ để nối các loạt C#, Python, Rust, Go và database" | B | TB | "Các loạt bài dùng chung nghiệp vụ Tới Nơi để kết nối C#, Python, Rust, Go và database" |
| 74 | "kết quả OCR và điểm fraud là đầu vào" | A | TB | "kết quả OCR và điểm chấm gian lận (fraud score) là đầu vào" |
| 74 | "không tự động đồng nhất với quyết định xác minh hay kết luận gian lận" | B | TB | "không tự động tương đương với quyết định xác minh hay kết luận gian lận" |
| 82 | "đo rồi quyết định cách tiến hóa" | A/B | Thấp | "đo đạc rồi quyết định hướng phát triển" |
| 84 (bảng) | Cột "Nội dung chia sẻ" | B | TB | "Nội dung chính" |
| 87 (bảng) | "Tải tái lập" | B | TB | "Tải có thể tái lập" |
| 87 (bảng) | "cấu hình/hardware, tốc độ sinh và dọn phiên bản cũ" | A/B | Thấp | "cấu hình/phần cứng, tốc độ tạo và dọn phiên bản cũ" |
| 92 | "VACUUM là cơ chế bảo trì phiên bản hàng" | A/B | Thấp | "bảo trì phiên bản dòng (row version)" |
| 94 | "là case study lịch sử ... write amplification ... khi concurrency tăng, cũng không phải case chuyển sang MongoDB" | A/B | TB | "nghiên cứu tình huống lịch sử ... khuếch đại ghi (write amplification) ... khi số giao dịch đồng thời tăng, cũng không phải trường hợp chuyển sang MongoDB" |
| 96 | "Mọi bài so sánh phải nêu cùng tiêu chí tính đúng, durability, tải" | A | Thấp | "độ bền dữ liệu (durability)" |

### Tổng hợp mức độ trong 8 bài đã đọc kỹ

- Cao: 6 (dotnet10-transaction 29, 67, 75, 54; csharp-async 52; du-an-mau 65; da-tien 82 cần xác nhận).
- Trung bình: khoảng 30.
- Thấp: khoảng 20.
- Tổng: khoảng 55 lỗi trên 8 bài, trung bình 6-7 lỗi/bài. Các bài kỹ thuật nặng như dotnet10-transaction và csharp-async có mật độ cao nhất. Bài da-la-gi và csharp-bien có mật độ thấp hơn, chủ yếu là lỗi B nhẹ.

## 3. Thuật ngữ bị dịch hoặc viết gượng lặp lại (đếm trên toàn nhóm)

Số liệu từ Grep trên toàn thư mục, chỉ tính các file trong nhóm.

| Từ hiện đang dùng | Đề xuất | Số lần / số file |
|---|---|---|
| `huỷ` (cách viết cũ) | Thống nhất với `hủy` | huỷ: 204 lần / 22 file. hủy: 27 lần / 8 file. Không thống nhất. |
| `khoá` | Thống nhất với `khóa` | khoá: 55 lần / 16 file. khóa: 51 lần / 23 file. Không thống nhất. |
| `biên kỳ` | ranh giới kỳ báo cáo | ít nhất 7 dòng / 5 file (da-tien, da-kpi, da-thuc-hanh-ke, da-chua-du, da-ngay-gio) |
| `không âm thầm` (calque của "silently") | không báo lỗi / lặng lẽ bỏ qua | ít nhất 5 file, gồm đoạn lặp 4 lần (dotnet10-efcore-postgresql, dotnet10-efcore-tracking, dotnet10-outbox, dotnet10-transaction, dòng 75) |
| `skip` | bỏ qua (test) | cùng đoạn trên, 4 file |
| `candidate` (bản ghi thua cuộc) | bản ghi ứng viên | ít nhất 2 file bài văn (dotnet10-transaction:54, dotnet10-use-case-dat-chuyen-end-to-end:31) |
| `warehouse` | kho dữ liệu | 3 dòng / 3 file (da-la-gi:104, da-thuc-hanh-ke:42, da-thuc-hanh-sql:111) |
| `guardrail` | chỉ số bảo vệ | ít nhất 1 dòng (da-join:49), trong khi da-kpi dùng "chỉ số bảo vệ" |
| `fingerprint` | dấu vân tay (fingerprint) của ý định | ít nhất 3 file (dotnet10-transaction:56, dotnet10-tong-hop:48, dotnet10-use-case:33) |
| `tác vụ` / `task` | thống nhất một cách | lẫn lộn trong csharp-async và dotnet10 |

Ghi chú: Các từ "fixture", "dependency", "invariant", "validation", "strategy", "interface" đều được giữ nguyên và xuất hiện dày đặc trong nhóm. Đây là lựa chọn văn phong có chủ đích và không đề xuất sửa.

## 4. Ước tính mật độ

- Mẫu 8 bài đã đọc kỹ: khoảng 6-7 lỗi/bài.
- Mật độ thực trên 64 bài có thể thấp hơn, vì các bài da-* và csharp* ngắn hơn và chủ yếu lỗi mức thấp. Ước tính toàn nhóm khoảng 3-6 lỗi/bài, tập trung ở dotnet10-* kỹ thuật cao và csharp-async.

## 5. Nhận xét về kiểu lỗi

1. Câu bị cụt hoặc thiếu động từ trong các đoạn giải thích kỹ thuật phức tạp (dotnet10-transaction:29, 67; du-an-mau:65). Đây là lỗi nguy hiểm nhất vì người đọc không biết câu nói gì.
2. Calque từ tiếng Anh: "đề nghị có hợp tác", "không mặc định", "không xóa bài toán", "đẩy ra một hàm", "rơi về", "case chuyển sang", "bản sao... lệch nhau". Gợi ý dùng cách diễn đạt tự nhiên hơn.
3. Từ ghép tự chế: "biên kỳ", "phiên bản hàng", "chia sẻ" (dùng với nghĩa "chính"), "tải tái lập".
4. Giữ nguyên tiếng Anh không cần thiết ở chỗ có từ Việt chuẩn: candidate, attempt, currency, retention, dedup, coverage, guardrail, skip, watchdog.
5. Đoạn văn lặp lại nguyên văn ở 4 file dotnet10 (dòng 75). Nên thống nhất một câu.
6. Không thống nhất chính tả huỷ/hủy và khoá/khóa giữa các file. Đây là vấn đề biên tập chung, không phải lỗi từng câu. Cần biên tập viên chọn một chuẩn và áp dụng đồng loạt.

## 6. Việc cần biên tập viên quyết định

- Chọn một chuẩn chính tả: `hủy/khóa` (dạng hiện đại) hay `huỷ/khoá` (dạng đang chiếm đa số trong da-*). Các bài dotnet10 và csharp-async đang dùng dạng ngược lại.
- Xác nhận "ride/city1" ở da-tien-hoan-tien-doi-soat.md dòng 82 là gì.
