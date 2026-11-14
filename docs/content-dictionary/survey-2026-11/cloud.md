# Khảo sát ngôn ngữ: nhóm aws-*.md và azure-*.md

Thư mục: H:\Work\tech-blog\api\seed\content (đường dẫn file rút gọn theo thư mục này)
Chỉ đọc, không sửa file nội dung nào.

## 1. Phạm vi và phương pháp

- Số file: 118 (aws 60, azure 58).
- Đã quét dòng title và summary của cả 118 file.
- Đọc kỹ văn xuôi của 8 bài (bỏ khối code và SVG):
  - aws-cloudtrail-config-security-hub-guardduty.md (đọc toàn bộ)
  - aws-dms-migration-postgresql-zero-downtime.md (đọc toàn bộ)
  - aws-observability-nang-cao-xray-adot-logs-insights-slo.md (đọc khoảng 3/4 đầu)
  - aws-iam-user-role-policy.md (đọc khoảng 4/5 đầu)
  - azure-la-gi-cloud-region-trach-nhiem-chia-se.md (đọc toàn bộ)
  - azure-chon-compute-cho-tung-service-toi-noi.md (đọc khoảng 2/3 đầu)
  - azure-ha-dr-zone-region-rto-rpo.md (đọc khoảng 1/2 đầu)
  - azure-cd-environment-approval-blue-green-canary.md (đọc khoảng 9/10 đầu)
- Đếm cụm từ bằng grep trên toàn bộ thư mục content (606 file .md) và trong nhóm aws/azure.

Chất lượng chung: khá tốt, giọng văn tự nhiên, đa số thuật ngữ được giữ đúng. Các bài beginner (IAM, Azure la-gi) sạch hơn bài advanced. Lỗi tập trung ở ba kiểu: cụm từ dịch từng chữ, câu lược mất động từ hoặc danh từ, và một số cụm mang tính calque.

## 2. Lỗi trong 8 bài đã đọc

Mức độ: cao = sai nghĩa hoặc khó hiểu; trung bình = gượng; thấp = có thể tốt hơn.

### azure-la-gi-cloud-region-trach-nhiem-chia-se.md
- Dòng 118, loại A, cao. Gốc: "Điều dễ sai nhất là cộng dồn SLA. ... SLA tổ hợp là tích của các SLA." Đề xuất: "Điều dễ nhầm nhất là cộng SLA của các thành phần lại với nhau. ... SLA tổ hợp bằng tích các SLA." Lý do: "cộng dồn" đọc như phép cộng trong khi nội dung là phép nhân (composite SLA).
- Dòng 107 (heading), loại A/B, trung bình. Gốc: "SLA 99,9% là 43 phút mỗi tháng và không cộng dồn". Đề xuất: "... và không tích luỹ sang tháng sau".
- Dòng 64, loại B, trung bình. Gốc: "kèm thời gian ngừng và công kiểm thử lại". Đề xuất: "công sức kiểm thử lại". Thiếu danh từ.
- Dòng 109 và 147 (summary), loại A, trung bình. Gốc: "đọc SLA ra phút downtime". Đề xuất: "quy đổi SLA thành số phút ngừng hoạt động".
- Dòng 44, loại B, thấp. Gốc: "có địa chỉ vật lý cụ thể". Đề xuất: "nằm ở một vị trí vật lý cụ thể".

### azure-chon-compute-cho-tung-service-toi-noi.md
- Dòng 62 (bảng), loại A, trung bình đến cao. Gốc: "State khớp tài xế theo cell địa lý nằm trong bộ nhớ" và "Giải được state ra ngoài". Đề xuất: "trạng thái ghép cặp tài xế theo ô địa lý (cell) nằm trong bộ nhớ" và "tách được state ra khỏi pod". "Khớp" và "giải được" là dịch từng chữ.
- Dòng 69, loại B, trung bình. Gốc: "việc scale phải bám một chỉ số riêng như số yêu cầu chờ khớp". Đề xuất: "số yêu cầu đang chờ ghép cặp".
- Dòng 4 (summary), 58, 131, 135, loại B, trung bình. Gốc: "điều kiện đổi quyết định", "Đổi quyết định khi". Đề xuất: "điều kiện khiến quyết định thay đổi" hoặc "Quyết định đổi khi".
- Dòng 65, loại A, trung bình. Gốc: "Bùng nổ theo sự kiện". Đề xuất: "tải tăng vọt theo sự kiện".
- Dòng 88, loại B, thấp. Gốc: "không để làm đích". Đề xuất: "không phải là đích đến".
- Dòng 131 và 133, loại A/B, trung bình. Gốc: "viết một đoạn biện hộ". Đề xuất: "đoạn lý giải" hoặc "lập luận". "Biện hộ" mang nghĩa bào chữa.
- Dòng 50, loại B, thấp. Gốc: "Do bạn đặt, trừ ingress".

### azure-ha-dr-zone-region-rto-rpo.md
- Dòng 78, loại A, cao. Gốc: "consumer phải chịu trùng". Đề xuất: "consumer phải xử lý được tin trùng lặp".
- Dòng 87, loại B, trung bình. Gốc: "Database còn là replica chỉ đọc thì mọi lệnh ghi đều lỗi." Đề xuất: "Khi database vẫn là replica chỉ đọc thì mọi lệnh ghi đều lỗi."
- Dòng 91, loại B, trung bình. Gốc: "Số đo vượt mục tiêu thì chọn". Đề xuất: "Nếu số đo vượt mục tiêu thì phải chọn một trong ba hướng sau."
- Dòng 87, loại A, thấp đến trung bình. Gốc: "Hình 2 cho thấy điểm lệch". Đề xuất: "điểm khác biệt".
- Dòng 60, loại A, trung bình. Gốc: "Azure Cache for Redis đang nghỉ hưu". Đề xuất: "đang ngừng hỗ trợ" hoặc "đã có lịch ngừng cung cấp". Lưu ý: "nghỉ hưu" lặp lại trong cả loạt Azure.
- Dòng 29, loại A, thấp. Gốc: "không giữ dữ liệu gốc". Đề xuất: "không giữ dữ liệu chính".
- Dòng 258, loại B, thấp. Gốc: "endpoint ghi đi theo primary khi promote". Đề xuất: "endpoint ghi tự trỏ về primary mới khi chuyển".
- Summary dòng 4, loại B, trung bình. Gốc: "mô phỏng tắt origin chính để đo thời gian chuyển". Đề xuất: "đo thời gian chuyển đổi dự phòng".

### azure-cd-environment-approval-blue-green-canary.md
- Dòng 166, loại B, trung bình. Gốc: "vì dự án này đã ngừng được duy trì từ 2026". Đề xuất: "vì dự án này đã ngừng bảo trì từ năm 2026".
- Dòng 125, loại B, trung bình. Gốc: "bạn mất đường lui một lệnh". Đề xuất: "bạn không còn đường lui chỉ bằng một lệnh".
- Dòng 13, loại A, trung bình. Gốc: "một truy vấn KQL quyết định tiếp tục hay quay lại". Đề xuất: "tiếp tục hay rollback". "Quay lại" mơ hồ.
- Dòng 21, loại B, thấp. Gốc: "Build lại ở staging hay prod là sai về cơ chế". Đề xuất: "là sai nguyên tắc".
- Dòng 168, loại B, thấp. Gốc: "cùng đọc ghi vào psql". Đề xuất: "cùng đọc và ghi".
- Dòng 170, loại B, thấp. Gốc: "tổng connection là của cả hai revision cộng lại". Đề xuất: "tổng số connection bằng tổng của cả hai revision".

### aws-cloudtrail-config-security-hub-guardduty.md
- Dòng 124 và 130, loại A, cao đến trung bình. Gốc: "Diệt" (bước 4 trong hình và trong văn bản NIST). Đề xuất: "Loại bỏ mối đe doạ" hoặc "Tiêu diệt" đều được. "Diệt" gây khó hiểu.
- Dòng 138, loại B, trung bình. Gốc: "kết quả cho thời gian, principal, access key, IP nguồn và user agent". Đề xuất: "kết quả cho biết thời gian, ...".
- Dòng 51, loại B, trung bình. Gốc: "tính phí theo dung lượng log nạp và lưu". Đề xuất: "tính phí theo dung lượng log được nạp vào và lưu trữ".
- Dòng 124, loại B, trung bình. Gốc: "xoay mọi secret key đó từng đọc được". Đề xuất: "xoay mọi secret mà key đó từng đọc được". "Secret key" dễ nhầm với secret của AWS.
- Dòng 4 (summary) và 124, loại B, thấp. Gốc: "playbook access key lộ". Đề xuất: "playbook cho access key bị lộ".

### aws-dms-migration-postgresql-zero-downtime.md
- Dòng 4 (summary) và 28 (bảng), loại A, trung bình. Gốc: "kiểm đối soát", "Không có công cụ đối soát". Đề xuất: "kiểm tra đối chiếu dữ liệu" và "không có công cụ kiểm tra dữ liệu". "Đối soát" là thuật ngữ tài chính, ở đây nghĩa là validation.
- Dòng 11, loại B, trung bình. Gốc: "dừng hệ thống vài giờ để copy là điều nghiệp vụ không chấp nhận". Đề xuất: "là điều mà nghiệp vụ không chấp nhận được".
- Dòng 46, loại B, trung bình. Gốc: "chỉ đặt tên slot có sẵn khi dùng điểm bắt đầu CDC gốc cho task chỉ-CDC". Đề xuất: "chỉ cần đặt tên slot có sẵn khi task chỉ-CDC dùng điểm bắt đầu CDC gốc".
- Dòng 111, loại A, trung bình. Gốc: "chọn cỡ theo số đo của staging". Đề xuất: "theo số liệu đo được ở staging".
- Dòng 129, loại B, thấp. Gốc: "tính hợp lệ của task settings khai báo một phần". Đề xuất: "khi task settings chỉ khai báo một phần".
- Dòng 156, loại A, thấp. Gốc: "theo hồ sơ tải". Đề xuất: "theo mẫu tải đã ghi nhận".

### aws-observability-nang-cao-xray-adot-logs-insights-slo.md
- Dòng 4 (summary) và 78, loại A, trung bình. Gốc: "composite alarm giảm ồn". Đề xuất: "giảm cảnh báo nhiễu".
- Dòng 50, loại A, trung bình đến thấp. Gốc: "ghi mọi trace là phí tiền". Đề xuất: "ghi mọi trace thì tốn kém".
- Dòng 48, loại A, thấp. Gốc: "Không có propagation vào database". Đề xuất: "Không có truyền ngữ cảnh vào database" (đoạn trên đã dùng "truyền ngữ cảnh").
- Dòng 78, loại A, thấp. Gốc: "chống alert fatigue". Đề xuất: "chống mệt mỏi vì cảnh báo".

### aws-iam-user-role-policy.md
- Dòng 27, loại B, trung bình. Gốc: "người nên đi qua Identity Center". Đề xuất: "người dùng nên đăng nhập qua Identity Center".
- Dòng 11, loại B, thấp. Gốc: "từ nay có thể xoá database". Đề xuất: "có thể dẫn tới việc xoá database".

Ghi chú: các mục sau không báo vì hợp lý: Identity Center, Session Manager, Policy Simulator, Deny, cold start, canary, revision, digest, RTO/RPO, staging, và các tên riêng.

## 3. Lỗi trong title và summary (118 dòng)

Title nhìn chung ổn. Một số summary cần sửa:
- aws-iam-nang-cao-permission-boundary-abac-access-analyzer.md (summary): "tìm quyền ra ngoài" nên là "quyền truy cập từ bên ngoài". Trung bình.
- aws-cloudwatch-budgets-tagging-co-ban.md (summary): "để không mất tiền" nên là "để tránh phát sinh chi phí". Thấp.
- aws-ecs-fargate-task-definition-roles.md (summary): "service giữ số task mong muốn" nên là "service giữ đúng số lượng task mong muốn". Thấp.
- azure-load-testing-autoscale-chaos-studio.md (summary): "chỉnh scale rule ... theo số đo" nên là "theo kết quả đo". Trung bình.
- azure-multi-region-data-residency-multi-tenant.md (summary): "cho khách đội xe" nên là "cho khách hàng là đội xe". Trung bình.
- aws-tai-khoan-iam-identity-center-cli.md (summary): "để không giật mình khi có hoá đơn" nên là "để không bất ngờ khi có hoá đơn". Thấp.

## 4. Cặp thuật ngữ và cụm từ lặp lại

Số liệu: "toàn bộ" là số lần xuất hiện trong 606 file .md của content. "nhóm" là số lần trong 118 file aws/azure.

| Cụm đang dùng | Đề xuất | Toàn bộ | Nhóm aws/azure | Ghi chú |
|---|---|---|---|---|
| số đo | số liệu đo được / kết quả đo | 324 | 16 (10 file) | Dùng lặp lại nhiều, có chỗ đúng |
| nghỉ hưu (ngừng hỗ trợ) | ngừng hỗ trợ / ngừng cung cấp | 12 | 12 (9 file) | Calque từ "retire" |
| kiểm tài liệu / kiểm SKU (kiểm = kiểm tra) | kiểm tra tài liệu | 28 | 26 (25 file) | Động từ bị lược, không sai nhưng cụt |
| đường đi (luồng) | luồng đi | 186 | 17 (16 file) | Nhiều chỗ đúng ("đường đi của gói tin"), chỉ sửa chỗ calque |
| đổi quyết định | điều kiện khiến quyết định thay đổi | 17 | 4 (2 file) | Chủ yếu trong bài chon-compute |
| cộng dồn SLA | tổ hợp / tích SLA | - | 2 (la-gi, dòng 107 và 118) | Lỗi sai nghĩa; các dòng "cộng dồn" còn lại đều đúng |
| key lộ | access key bị lộ | - | 6 (3 file) | Lược "bị" |
| điểm lệch | điểm khác biệt | 15 | 3 (3 file) | |
| bùng nổ (tải) | tăng vọt | 7 | 3 (2 file) | |
| giảm ồn | giảm cảnh báo nhiễu | 3 | 3 (2 file) | |
| đọc ghi | đọc và ghi | 27 | 5 (4 file) | Lược liên từ |
| biện hộ | lý giải / lập luận | 1 | 2 (1 file) | |
| đối soát (validation) | kiểm tra đối chiếu dữ liệu | 357 | 1 (DMS) | Phần lớn "đối soát" là nghĩa đúng (reconciliation), chỉ sửa chỗ validation |
| dữ liệu gốc (bản chính) | dữ liệu chính | 10 | 2 (2 file) | |

Các cụm chỉ xuất hiện một lần trong nhóm: "Diệt", "chịu trùng", "State khớp", "chờ khớp", "Giải được", "kiểm đối soát", "phí tiền", "đọc SLA ra", "hồ sơ tải", "ngừng được duy trì", "mất đường lui", "sai về cơ chế", "từ nay có thể", "chỉ-CDC".

## 5. Mật độ và nhận xét

- Trong 8 bài đã đọc có khoảng 60 điểm cần xem xét. Khoảng 25 điểm mức trung bình trở lên, khoảng 4 điểm mức cao.
- Ước lượng mật độ mức trung bình trở lên: khoảng 3 lỗi mỗi bài (sai số lớn vì chỉ đọc 8 bài, và bài advanced có mật độ cao hơn bài beginner).
- Ước lượng toàn nhóm (118 bài): khoảng 300 lỗi mức trung bình trở lên, mật độ ước tính 2 đến 4 lỗi mỗi bài. Đây là ước lượng ngoại suy, chưa phải số đếm.
- Kiểu lỗi phổ biến nhất:
  1. Calque tiếng Anh ở thuật ngữ hạ tầng: "State khớp", "chịu trùng", "nghỉ hưu", "số đo", "đi vào", "đọc SLA ra".
  2. Câu lược mất động từ hoặc danh từ: "kết quả cho thời gian", "công kiểm thử", "đọc ghi", "key lộ", "kiểm tài liệu".
  3. Từ dùng sai nghĩa trong ngữ cảnh: "đối soát" cho validation, "cộng dồn" cho composite, "biện hộ" cho justify, "Diệt" cho eradication.
  4. Trật tự cụm từ kiểu tiếng Anh: "điều kiện đổi quyết định", "điều nghiệp vụ không chấp nhận", "khi ... còn là replica".
- Các bài beginner (IAM, la-gi) ít lỗi hơn. Các bài advanced (DMS, HA/DR, CD, observability) có nhiều lỗi calque hơn.
- Đề xuất ưu tiên: sửa các mục cao trước (cộng dồn SLA, chịu trùng, State khớp, Diệt), sau đó thay "số đo" và "nghỉ hưu" bằng một lần rà soát có kiểm soát vì xuất hiện nhiều.
