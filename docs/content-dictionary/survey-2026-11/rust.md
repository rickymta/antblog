# Khảo sát chất lượng tiếng Việt: nhóm rust-*.md

Thư mục: `H:\Work\tech-blog\api\seed\content`
Nhóm: 68 file `rust-*.md` (trên tổng 606 file .md trong thư mục).
Phạm vi: chỉ đọc, không sửa file nội dung nào.

## 1. Phạm vi đã đọc

- Tiêu đề (`title:`) và tóm tắt (`summary:`) của cả 68 file: đã rà soát.
- Đọc kỹ phần văn xuôi (khoảng 300 dòng đầu, bỏ khối code) của 8 bài:
  - rust-async-await-tokio-task-timeout-cancellation.md
  - rust-kieu-so-tien-vnd-i64-decimal.md
  - rust-ownership-move-borrow-but-toan.md
  - rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung.md
  - rust-redis-cache-so-du-rate-limit-lock-ttl.md
  - rust-lam-tron-chia-tien-largest-remainder-khong-mat-dong.md
  - rust-kafka-rdkafka-consumer-producer-schema.md
  - rust-transaction-isolation-append-only-idempotency-key.md
- Đếm từ khóa bằng grep trên toàn thư mục để kiểm tra mức độ lặp lại.

## 2. Tổng hợp lỗi

Ước lượng từ mẫu (không phải kiểm kê đầy đủ):

| Nhóm | Số lượng (ước tính) |
|---|---|
| Loại A (thuật ngữ dịch sai/gượng) | khoảng 12 |
| Loại B (diễn đạt không thuần Việt) | khoảng 60 |
| Mức cao | khoảng 7 (gồm các chỗ đã ghi rõ "cao" trong mục 3) |
| Mức trung bình | khoảng 30 |
| Mức thấp | khoảng 40 |

Mật độ: khoảng 5 lỗi/bài khi đọc kỹ (8 bài, chủ yếu mức trung bình và thấp); khoảng 1 lỗi/bài ở phần tóm tắt của các bài còn lại. Ước tính chung cho nhóm: khoảng 2 lỗi/bài, trong đó 1 bài mức cao trở lên trên mỗi 15 bài.

Không có lỗi về code, khối code hay inline code trong báo cáo này.

## 3. Lỗi chi tiết (file: dòng)

Tên file nằm trong `H:\Work\tech-blog\api\seed\content\`.

### 3.1 rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung.md

- **Dòng 18, 23, 25, 62, 85, 87, 97, 117 (loại A, cao).** "thẻ" dùng để dịch "tag" của HMAC (ví dụ: "thẻ bắt đầu `51f0ae52…`", "ra thẻ khác", "so sánh thẻ bằng `==`"). Trong tiếng Việt, "thẻ" đọc ra là thẻ thanh toán/thẻ tín dụng, gây hiểu nhầm. Đề xuất: "giá trị chữ ký" hoặc "mã xác thực (MAC)". Lưu ý: "số thẻ" ở rust-config-secrets-tracing-structured-logging.md và "hoàn về thẻ" ở rust-refund-reversal-chargeback-but-toan-dao-maker-checker.md là nghĩa đúng, không sửa.
- **Dòng 237 (loại B, cao).** "Hai lần ra 0 mới là điều đáng nhớ." Khó hiểu. Gốc: "Hai lần ra 0 mới là điều đáng nhớ: cuộc đua không lộ ở mọi lần chạy..." Đề xuất: "Chỉ hai lần cho kết quả 0; điều đáng chú ý là cuộc đua không xuất hiện ở mọi lần chạy, nên test xanh không chứng minh an toàn."
- **Dòng 275 (loại B, trung bình).** "...tối đa bốn `v1` mỗi header để không bị bắt tốn CPU." Dịch máy. Đề xuất: "để kẻ gửi không ép được hệ thống tốn CPU."
- **Dòng 275 (loại B, trung bình).** "Lab chưa ghi khóa nào khớp; muốn biết lúc nào gỡ được A..." Đề xuất: "Lab chưa ghi lại khóa nào khớp; muốn biết lúc nào gỡ được A, production cần đếm theo `kid`."
- **Dòng 192 (loại B, trung bình).** "Lab giả định VíXanh coi mọi 2xx là đã giao" → "coi mọi 2xx là đã nhận".
- **Dòng 87 (loại B, trung bình).** "thời gian so phụ thuộc số byte đầu đoán đúng" → "thời gian so phụ thuộc vào số byte đầu tiên đoán đúng".
- **Dòng 49 (loại B, trung bình).** "Ra ngoài chỉ một câu trả lời cho mọi lý do" → "Chỉ trả về một câu trả lời cho mọi lý do". Tương tự dòng 62: "đều ra ngoài bằng cùng một `401`" → "đều trả về cùng một `401`".
- **Dòng 64 (loại B, thấp).** "mọi thứ nằm trong app đều trích ra được" → "mọi thứ nằm trong app đều có thể bị trích xuất".
- **Dòng 11 (loại B, thấp).** "không đánh rơi thông báo đến sai thứ tự" → "không làm mất thông báo đến lệch thứ tự".
- **Dòng 150 (loại B, thấp).** "mọi webhook thật thành «cũ»" → "mọi webhook thật bị coi là quá hạn".
- **Dòng 263 (loại B, thấp).** "sau lần thứ hai ta trả tên bảng về" → "sau lần thứ hai ta đổi lại tên bảng".

### 3.2 rust-kafka-rdkafka-consumer-producer-schema.md

- **Dòng 225 (loại B, cao).** "...chỉ không commit thì chưa đủ. Vị trí đọc nằm trong bộ nhớ consumer nên thông điệp kế tiếp vẫn đến, và commit nó sẽ che luôn thông điệp lỗi" → Câu đầu khó hiểu. Đề xuất: "Chỉ bỏ qua việc commit là chưa đủ."
- **Dòng 32 (loại B, trung bình).** "muốn commit theo lô thì bật lại commit nền, đặt nó `false` và gọi `store_offset`" → "nó" không rõ đang chỉ tham số nào. Đề xuất: "muốn commit theo lô thì bật lại commit nền, đặt `enable.auto.offset.store=false` và gọi `store_offset` sau khi ghi."
- **Dòng 131 (loại B, trung bình).** "Chọn một tài khoản thì hai tài khoản còn lại không có thứ tự nào với bút toán." → "...thì thứ tự của hai tài khoản còn lại so với bút toán không được bảo đảm."
- **Dòng 206 (loại B, trung bình).** "Consumer đúng log, bỏ qua và vẫn commit" → "Consumer ghi log đúng lý do, bỏ qua và vẫn commit."
- **Dòng 241 (loại B, trung bình).** "Phần mô hình lập trình dựa vào tài liệu của hai thư viện." → "Phần mô hình lập trình chỉ dựa trên tài liệu của hai thư viện, lab không kiểm chứng."
- **Dòng 109 (loại B, thấp).** "làm key đổi chỗ" → "làm key chuyển sang partition khác".
- **Dòng 134 (loại B, thấp).** "cộng thứ tự nào cũng ra cùng kết quả" → "cộng theo thứ tự nào cũng ra cùng kết quả".
- **Dòng 255 (loại B, thấp).** "broker bận thì chậm hoặc fail rõ" → "hoặc báo lỗi rõ".
- **Dòng 132 (loại A, thấp).** "hot partition" giữ nguyên tiếng Anh. Có thể chấp nhận; nếu muốn Việt hóa thì dùng "partition nóng (hot partition)" ở lần xuất hiện đầu.

### 3.3 rust-lam-tron-chia-tien-largest-remainder-khong-mat-dong.md

- **Dòng 49 (loại B, trung bình).** "`Decimal` không biểu diễn hết 1/3" → "không biểu diễn chính xác 1/3".
- **Dòng 65 (loại B, trung bình).** "Không hàng nào có ba số 0 ở cột lệch." → "Không strategy nào cho lệch 0 ở cả ba cột."
- **Dòng 123 (loại B, trung bình).** "Largest remainder giữ mỗi bên dưới 1 đồng" → "lệch mỗi bên dưới 1 đồng so với phần chính xác".
- **Dòng 189 (loại A, trung bình).** "Về strategy: thử sửa tiêu chí phá hòa thành «theo vị trí trong đầu vào»." Từ "strategy" bị dùng lẫn với `RoundingStrategy` và proptest strategy. Đề xuất: "Thử phá code: đổi tiêu chí phá hòa thành «theo vị trí trong đầu vào»."
- **Dòng 179 (loại B, thấp).** "Không chứng minh được, vì sai: ..." → "Không thể khẳng định, vì nó sai: ..."
- **Dòng 13 (loại B, thấp).** "Bài này biến nó thành thành phần dùng thật" → "Bài này biến nó thành mã dùng được trong thực tế".
- **Dòng 153 (loại B, thấp).** "làm tròn đúng một lần, ở biên" → "ở bước cuối".
- **Dòng 47 (loại B, thấp).** "hai phần lẻ cộng lại đúng bằng 1 (hoặc 0)" → "phần lẻ của hai bên bù nhau đúng 1 đồng (hoặc 0)".

### 3.4 rust-redis-cache-so-du-rate-limit-lock-ttl.md

- **Dòng 124 (loại B, trung bình).** "Script Lua đưa đọc, tính hoàn token và ghi vào một lệnh `EVAL`" → "Script Lua gom việc đọc, tính hoàn token và ghi vào một lệnh `EVAL`."
- **Dòng 124 (loại B, thấp).** "Đây là thứ tự xấu nhất, ép ra để kết quả lặp lại được" → "buộc xảy ra để kết quả lặp lại được".
- **Dòng 256 (loại B, thấp).** "`post_entry` chỉ kiểm key" → "chỉ kiểm tra key".
- **Dòng 34 (loại B, thấp).** "kết nối ghép kênh" cho "multiplexed". Có thể giữ "multiplexed" hoặc viết "kết nối dùng chung nhiều lệnh (multiplexed)".

### 3.5 rust-transaction-isolation-append-only-idempotency-key.md

- **Dòng 282 (loại A, trung bình).** "`TRUNCATE` không kích hoạt trigger theo dòng, và chỉ nhận trigger cấp statement." → "chỉ kích hoạt trigger cấp câu lệnh".
- **Dòng 130 (loại B, trung bình).** "PostgreSQL không trả 0 dòng rồi để ta đọc lại vào chỗ trống" → "PostgreSQL không im lặng trả 0 dòng để ta đọc lại".
- **Dòng 112 (loại B, thấp).** "T1 commit thì T2 phải trả rỗng" → "T2 nhận về 0 dòng".
- **Dòng 86 (loại B, thấp).** "Cơ chế có hai nửa" → "Cơ chế gồm hai phần".
- **Dòng 206 (loại B, thấp).** "Nó không khoá chặn trước mà theo dõi..." → "Nó không khóa trước để chặn mà theo dõi...".
- **Dòng 257 (loại B, thấp).** "có thể huỷ oan" → "có thể hủy nhầm".
- **Dòng 261 (loại B, thấp).** "Xuống PostgreSQL, nó phải thành cơ chế..." → "Đẩy xuống PostgreSQL, nó phải thành cơ chế...".

### 3.6 rust-async-await-tokio-task-timeout-cancellation.md

- **Dòng 93 (loại B, trung bình).** "Mặc định `select!` kiểm từ một nhánh ngẫu nhiên" → "Mặc định, `select!` chọn ngẫu nhiên thứ tự kiểm tra các nhánh".

### 3.7 rust-kieu-so-tien-vnd-i64-decimal.md

- **Dòng 4 (summary) (loại B, trung bình).** "...và chốt lưu i64 đồng." → "...và chốt kiểu lưu trữ là i64, đơn vị đồng."
- **Dòng 146 (loại B, thấp).** "mà không một dòng cảnh báo" → "mà không có dòng cảnh báo nào".

### 3.8 rust-ownership-move-borrow-but-toan.md

- **Dòng 45 (loại B, thấp).** "`String` gồm ba từ trên stack" → "ba trường trên stack" (từ "word" trong Rust là kích thước một con trỏ máy).

### 3.9 Tiêu đề và tóm tắt (dòng 3 và 4) của các file khác trong nhóm

Chỉ liệt kê những chỗ cần sửa, không kèm câu gốc đầy đủ:

- rust-audit-append-only-hash-chain-phan-quyen-pii.md, dòng 4 (loại B, cao): "điểm neo ngoài database lộ cả chuỗi bị tính lại" → "điểm neo ngoài database cho thấy cả chuỗi đã bị tính lại". Cũng dòng 4: "đo bằng role thật" → "kiểm bằng role thật" (thấp).
- rust-criterion-benchmark-do-bo-nho-10k-but-toan.md, dòng 4 (loại B, cao): "chỉ gộp lô bằng unnest vượt 10.000 bút toán/giây" → "chỉ cách gộp lô bằng unnest mới vượt được 10.000 bút toán/giây". Dòng 3 (trung bình): "sổ cái có ghi nổi 10.000 bút toán/giây không?" → "sổ cái có ghi kịp 10.000 bút toán/giây không?" ("nổi" dùng sai nghĩa).
- rust-tong-hop-so-cai-tu-quyet-dinh... không liên quan; xem mục 3.10.
- rust-module-crate-cargo-test-clippy.md, dòng 4 (loại B, cao): "tổng kết điều phần P01 đã chứng minh và chưa có" → "tổng kết những điều P01 đã chứng minh và những gì còn thiếu".
- rust-outbox-inbox-idempotent-consumer.md, dòng 4 (loại B, cao): "tái hiện dual-write mất và sinh sự kiện bằng tiến trình con thoát giữa hai bước" → mệnh đề bị lộn xộn. Đề xuất: "tái hiện lỗi dual-write làm mất hoặc sinh thừa sự kiện, bằng cách cho một tiến trình con thoát giữa hai bước".
- rust-otel... (rust-opentelemetry-trace-tu-csharp-sang-rust-grpc-kafka.md), dòng 4 (loại B, trung bình): "metric hữu hạn" → "metric có tập nhãn hữu hạn"; "cha-con hay link" → "quan hệ cha-con hay liên kết (link)".
- rust-double-entry-journal-entry-posting.md, dòng 4 (loại A, trung bình): "dựng JournalEntry chỉ tồn tại khi cân" → "dựng JournalEntry chỉ tạo được khi cân". Cùng lỗi ở rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen.md (dòng 4 có "không tồn tại khi lệch" ở thân bài, xem mục 4).
- rust-config-secrets-tracing-structured-logging.md, dòng 4 (loại A, trung bình): "validate rồi fail fast" → "kiểm tra rồi dừng ngay khi lỗi".
- rust-trien-khai-aws-azure-secrets-health-autoscale.md, dòng 4 (loại A, trung bình): "ngân sách tắt" → "thời gian tắt tối đa"; "autoscale theo lag Kafka bị chặn bởi partition..." → "bị giới hạn bởi số partition và kết nối PostgreSQL".
- rust-graceful-shutdown-health-readiness-backpressure.md, dòng 4 (loại B, trung bình): "hàng đợi bounded" → "hàng đợi có giới hạn"; "bị SIGKILL vẫn không mất bút toán" → "dù bị SIGKILL vẫn không mất bút toán".
- rust-tong-hop-ledger-api-postgresql-kafka-end-to-end.md, dòng 4 (loại B, thấp): "TripCompleted từ Kafka thành bút toán" → "biến TripCompleted từ Kafka thành bút toán"; "kill giữa chừng" → "dừng đột ngột giữa chừng".
- rust-send-sync-arc-mutex-channel-concurrency.md, dòng 4 (loại A, trung bình): "Nhiều task ghi cùng idempotency_key chỉ ra một bút toán" → "chỉ tạo ra một bút toán".
- rust-struct-enum-match-tai-khoan-but-toan.md, dòng 4 (loại B, thấp): "máy trạng thái thanh toán trả Option" → "trả về Option"; "đối chiếu class, record, enum C#" → "đối chiếu với class, record, enum của C#".
- rust-string-str-ngay-gio-dinh-dang-tien.md, dòng 4 (loại B, trung bình): "UTF-8 với len() là byte" → "len() trả về số byte, không phải số ký tự"; "impl Display in 25.500 VND" → "impl Display để in 25.500 VND".
- rust-serde-json-but-toan-decimal-thoi-gian.md, dòng 4 (loại B, thấp): "lỗi đọc JSON có dòng, cột" → "lỗi đọc JSON kèm số dòng, cột".
- rust-result-option-toan-tu-hoi-loi-la-gia-tri.md, dòng 4 (loại B, trung bình): "ghi trùng idempotency key trả bút toán cũ" → "ghi trùng key thì trả bút toán cũ"; "bút toán lệch tổng trả Err(Unbalanced)" → "bút toán lệch tổng thì trả Err(Unbalanced)".
- rust-newtype-value-object-money-account-id.md, dòng 4 (loại B, trung bình): "cộng trừ tiền trả Result" → "phép cộng, trừ tiền trả về Result"; "chặn cộng khác loại tiền" → "chặn cộng tiền khác loại"; "chọn derive đúng" → "chọn derive phù hợp".
- rust-tong-hop-kien-truc-ledger-service-workspace.md, dòng 4 (loại B, thấp): "luồng TripCompleted tới số dư HTTP và gRPC" → "luồng TripCompleted đến số dư qua HTTP và gRPC".
- rust-vi-sao-so-cai-toi-noi-chon-rust.md, dòng 4 (loại B, thấp): "bốn tiêu chí có bằng chứng chạy được" → "bốn tiêu chí, mỗi tiêu chí có bằng chứng chạy được".
- rust-hop-dong-su-kien-xuyen-ngon-ngu-protobuf-version.md, dòng 4 (loại B, thấp): "bảng đo sáu thay đổi schema" → "bảng phân loại sáu loại thay đổi schema".
- rust-proptest-property-test-bat-bien-so-cai.md, dòng 4 (loại B, thấp): "shrinking thu nhỏ phản ví dụ về còn 1 đồng" → "shrinking thu gọn phản ví dụ xuống còn 1 đồng".
- rust-trait-generics-chinh-sach-phi-lam-tron.md, dòng 4 (loại B, thấp): "so static với dynamic dispatch" → "so sánh static dispatch với dynamic dispatch".
- rust-tonic-grpc-get-balance-post-entry-interop-dotnet.md, dòng 4 (loại B, thấp): "server tonic map LedgerError sang Status" → "ánh xạ LedgerError sang Status".
- rust-use-case-thanh-toan-chuyen-end-to-end.md, dòng 4 (loại B, trung bình): "vẫn ra cùng sổ hai bút toán" → "vẫn cho ra cùng hai bút toán trong sổ"; "abort giữa chừng" → "hủy giữa chừng".
- rust-khuyen-mai-tro-gia-but-toan-nhieu-ben-ngan-sach.md, dòng 4 (loại B, trung bình): "hết ngân sách, hoàn chuyến, trợ giá" không rõ nghĩa (liệt kê hay mệnh lệnh?). Cần tác giả xác nhận. "thành bút toán cân" → "thành một bút toán cân bằng".
- rust-ci-cd-github-actions-clippy-audit-deny-postgresql.md, dòng 4 (loại B, trung bình): "Lệnh chạy thật ở local, workflow chưa chạy trên GitHub." → "Các lệnh đã chạy thật ở máy local; workflow chưa chạy trên GitHub." "pin SHA" → "ghim SHA" (thấp).
- rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao.md, dòng 4 (loại B, thấp): "chặn test flaky" → "chặn test chập chờn (flaky)".
- rust-vi-tai-xe-chia-tien-payout-theo-lo.md, dòng 4 (loại B, thấp): "ngân hàng giả timeout" → "ngân hàng giả lập bị timeout".
- rust-van-hanh-so-cai-slo-alert-doi-soat-runbook-replay.md, dòng 3–4 (loại A, thấp): "alert lệch đối soát" → "cảnh báo lệch đối soát" (cách dùng "alert" phổ biến trong nhóm, có thể giữ).
- rust-proptest..., rust-lam-tron... đã nêu.
- rust-validation-problem-details-openapi-utoipa.md, dòng 4 (loại B, thấp): "bắt rejection của axum::Json" → "bắt lỗi từ chối (rejection) của axum::Json".

### 3.10 Mục thân bài chưa đọc (chỉ phát hiện qua grep)

- rust-repository-unit-of-work-trait-transaction.md (thân bài): "hai lần cùng key chỉ ra một bút toán" → "chỉ tạo ra một bút toán".
- rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen.md (thân bài): "nên không tồn tại khi lệch" → "nên không tạo được khi lệch".
- rust-trien-khai..., rust-criterion... đã nêu.

## 4. Cặp thuật ngữ lặp lại (đếm trên toàn thư mục content)

Lưu ý: các số đếm là số lần xuất hiện của chuỗi, chưa loại trừ các ngữ cảnh đúng. Cần đọc từng chỗ trước khi thay.

| Từ hiện dùng | Đề xuất | Rust-* | Toàn thư mục | Ghi chú |
|---|---|---|---|---|
| thẻ (nghĩa "tag" HMAC) | giá trị chữ ký / mã xác thực (MAC) | 19 (khoảng 10 là lỗi trong rust-webhook) | 491 (phần lớn là HTML hoặc thẻ thanh toán, đúng nghĩa) | Chỉ sửa ngữ cảnh HMAC |
| fail fast | dừng ngay khi lỗi | 1 | 6 (5 file) | |
| chỉ ra một bút toán | tạo ra một bút toán | 2 | 10 (8 file) | Nhiều chỗ "chỉ ra một lỗi" là đúng |
| tồn tại khi (bút toán cân) | chỉ tạo được khi / chỉ hợp lệ khi | 2 | 8 (8 file) | |
| ngân sách tắt | thời gian tắt tối đa | 1 | 1 | |

Ghi chú bổ sung (không phải lỗi dịch, nhưng nên thống nhất):
- Chính tả "khoá" và "khóa" đang dùng lẫn: rust-* có 87 "khoá" và 323 "khóa"; toàn thư mục có 1718 "khoá" và 876 "khóa". Nên chọn một dạng (khuyến nghị "khóa").
- Tương tự "huỷ" và "hủy": toàn thư mục 1534 và 365.
- Giữ nguyên hợp lý (không báo): "round trip" (rust-* 53 lần), "bounded" (10), "mapping" (7), "alert" (25), "clock" (dạng thuật ngữ), "hot partition", "backfill", "poison message", "dual-write". Nếu muốn Việt hóa thì nên áp dụng đồng đều, không sửa lẻ.

## 5. Nhận xét chung

- Kiểu lỗi phổ biến nhất là loại B: câu bị lược mất động từ hoặc danh từ (ví dụ "cộng trừ tiền trả Result", "Hai lần ra 0 mới là điều đáng nhớ", "chỉ không commit thì chưa đủ"). Các câu này đọc được nhưng phải đọc lại hai lần.
- Lối dịch máy hay gặp: "không biểu diễn hết", "bị bắt tốn CPU", "ra ngoài bằng", "ngân sách tắt", "đưa đọc", "đánh rơi", "trích ra được". Đây là dấu hiệu cần viết lại cả câu, không chỉ đổi một từ.
- Một lỗi mang tính hệ thống: trong tiêu đề và tóm tắt, nhiều câu dài nối bằng dấu chấm phẩy nhưng mệnh đề cuối bị cụt (ví dụ "tổng kết điều phần P01 đã chứng minh và chưa có"). Nên rà lại tóm tắt của toàn nhóm bằng một lượt đọc riêng.
- Thuật ngữ kỹ thuật được giữ tiếng Anh khá hợp lý (commit, rollback, partition, trigger). Điểm chưa nhất quán là "thẻ" (tag HMAC), "strategy" (dùng lẫn giữa nghĩa thuật toán làm tròn và nghĩa proptest), và cách dùng "fail fast", "hot partition", "alert".
- Phần code và khối code không được đánh giá.

## 6. Việc chưa làm

- Không đọc thân bài của 60 file còn lại (chỉ rà tiêu đề và tóm tắt). Mật độ lỗi thân bài có thể cao hơn ước tính.
- Không kiểm kê đầy đủ toàn bộ 68 file; các số trong mục 2 là ước lượng từ mẫu.
- Không sửa file nào.
