# Khảo sát ngôn ngữ: nhóm flutter-*.md và dart-*.md

Thư mục: H:\Work\tech-blog\api\seed\content
Chỉ đọc, không sửa file nội dung nào.

## Phạm vi và cách làm

- Nhóm có 52 file: 5 file `dart-*.md` và 47 file `flutter-*.md`, tổng khoảng 26.500 dòng.
- Đã rà toàn bộ dòng `title:` và `summary:` của 52 file.
- Đã đọc phần văn xuôi (bỏ khối code và HTML/SVG) của 9 bài, mỗi bài khoảng 120 đến 190 dòng đầu, không đọc hết toàn bài:
  - dart-bien-kieu-du-lieu-null-safety-tinh-cuoc.md (dòng 1-200)
  - flutter-accessibility-responsive-adaptive-ios-android.md (dòng 11-170)
  - flutter-authentication-otp-jwt-refresh-token-secure-storage.md (dòng 11-170)
  - flutter-flavors-dart-define-cau-hinh-moi-truong-dev-staging-prod.md (dòng 11-110)
  - flutter-go-router-deep-link-shellroute-guard.md (dòng 11-110)
  - flutter-hieu-nang-rebuild-isolate-devtools.md (dòng 11-130)
  - flutter-van-hanh-sau-phat-hanh-force-update-remote-config-shorebird-rollback.md (dòng 11-130)
  - flutter-vi-tri-geolocator-permission-foreground-background.md (dòng 11-110)
  - flutter-xu-ly-loi-retry-timeout-connectivity-mang-di-dong.md (dòng 11-130)

Nhận xét chung: văn phong nhìn chung tự nhiên, thuật ngữ kỹ thuật giữ tiếng Anh hợp lý. Lỗi tập trung ở ba kiểu: (1) thuật ngữ trạng thái/vận hành bị dịch gượng hoặc để lẫn tiếng Anh; (2) câu lược động từ hoặc chủ ngữ theo kiểu "X thành Y", "Package import ..."; (3) đại từ và cụm danh từ mơ hồ.

## Danh sách lỗi

Ký hiệu loại: A = thuật ngữ, B = diễn đạt. Mức: cao = sai nghĩa hoặc khó hiểu; trung bình = gượng; thấp = có thể tốt hơn.

| # | File:dòng | Câu gốc (trích) | Loại | Đề xuất | Mức |
|---|---|---|---|---|---|
| 1 | flutter-van-hanh-...-rollback.md:11 | "cả hai cửa hàng đều không có nút hạ máy đã cập nhật về bản cũ" | A | "...không có nút hạ cấp bản đã cập nhật về bản cũ" (cũng dòng 44: "Không biện pháp nào hạ máy đã c...") | cao |
| 2 | flutter-van-hanh-...-rollback.md:13, 105 | "runbook halt hay pause theo crash-free"; "Chỉ chính sách hợp lệ..." (halt/pause) | A | "runbook tạm dừng hoặc dừng phát hành theo crash-free"; giải thích một lần "dừng phát hành (halt rollout)" | trung bình |
| 3 | flutter-van-hanh-...-rollback.md:29 | "bản đang phát dần vẫn tải tay được, nên pause không chặn người tự bấm cập nhật" | A+B | "bản đang phát dần vẫn có thể cập nhật thủ công, nên việc tạm dừng không chặn người dùng tự bấm cập nhật" | trung bình |
| 4 | flutter-van-hanh-...-rollback.md:51 | "mọi giới hạn phiên bản do app tự thực thi và tự báo người dùng" | B | "...và tự thông báo cho người dùng" | thấp |
| 5 | flutter-van-hanh-...-rollback.md:13 | "kiểm phiên bản tối thiểu" | A | "kiểm tra phiên bản tối thiểu" | thấp |
| 6 | flutter-van-hanh-...-rollback.md:105 | "Lỗi mạng, hết giờ, JSON hỏng" | A | "hết thời gian chờ" | thấp |
| 7 | flutter-authentication-...md:13 | "đăng nhập vẫn là giả" | B | "đăng nhập vẫn chỉ là bản giả lập" | trung bình |
| 8 | flutter-authentication-...md:50 | "mã sai là ClientFailure 422, sai ba lần là 429" | B | "nhập sai mã thì nhận ClientFailure 422, nhập sai ba lần thì nhận 429" | trung bình |
| 9 | flutter-authentication-...md:148 | "Access token không bị thu hồi khi họ bị thu hồi" | B | "khi họ token bị thu hồi" (chủ ngữ "họ" không rõ là gì) | trung bình |
| 10 | flutter-authentication-...md:152 | "sau lần đọc kho đầu" | B | "sau lần đọc đầu tiên từ kho lưu trữ" | trung bình |
| 11 | flutter-authentication-...md:152 | "Request bị 401 gọi vào đây" | B | "Request nhận lỗi 401 sẽ gọi tới đây" | thấp |
| 12 | flutter-accessibility-...md:94 | "màn chi tiết bật nó vì trạng thái đổi sau khi huỷ" | B | làm rõ "huỷ" là huỷ gì và ai huỷ, ví dụ "vì trạng thái có thể đổi sau khi chuyến bị huỷ" | trung bình |
| 13 | flutter-accessibility-...md:162-178 | Bảng và test: "trượt", "qua" | A | "không đạt", "đạt" (giữ nhất quán trong toàn bài) | trung bình |
| 14 | flutter-accessibility-...md:131 | "tham số semanticsEnabled có giá trị true" | B | "tham số semanticsEnabled mặc định là true" | thấp |
| 15 | flutter-xu-ly-loi-...md:80 | "đóng kết nối trước header cho connectionError" | B | "đóng kết nối trước khi gửi header" | trung bình |
| 16 | flutter-xu-ly-loi-...md:102 | "Package import dart:io trực tiếp" | B | "Package này import trực tiếp dart:io" (thiếu chủ ngữ) | trung bình |
| 17 | flutter-xu-ly-loi-...md:127 | "đo khoảng cách giữa các lần dữ liệu về" | B | "đo khoảng cách giữa các đợt dữ liệu" | thấp |
| 18 | flutter-hieu-nang-...md:13 | "số lần build, số lần paint" | A | "số lần dựng lại widget, số lần vẽ" (nếu giữ "build" thì giữ thống nhất) | thấp |
| 19 | flutter-hieu-nang-...md:52 | "mỗi pha không nên vượt ngân sách" | B | "không pha nào nên vượt ngân sách" | thấp |
| 20 | flutter-hieu-nang-...md:83 | "ổn ở 60 Hz và hụt ở 120 Hz" | A | "vượt ngân sách ở 120 Hz" hoặc "không đạt ở 120 Hz" ("hụt" dễ hiểu ngược) | trung bình |
| 21 | flutter-flavors-...md:92-93 | "Mọi giá trị thành chuỗi"; "`null` trong JSON thành chuỗi"; "Một đối tượng lồng nhau thành" | B | "Mọi giá trị đều trở thành chuỗi"... (thiếu động từ, lặp kiểu "X thành Y" nhiều chỗ) | trung bình |
| 22 | flutter-flavors-...md:94 | "`bool.fromEnvironment` chỉ nhận đúng chữ `true`: `True` cho `false`" | B | "`True` cho ra `false`" | trung bình |
| 23 | flutter-flavors-...md:13 | "địa chỉ API, mức log, thời gian chờ" | A | "mức ghi log" | thấp |
| 24 | flutter-vi-tri-geolocator-...md:99 | "chỉ lấy vị trí ngoài 'có màn hình mở' và 'có foreground service' mới cần quyền nền" | B | "chỉ khi lấy vị trí lúc không có màn hình mở và không có foreground service mới cần quyền nền" (đảo trật tự, câu khó đọc) | trung bình |
| 25 | dart-bien-kieu-du-lieu-null-safety-tinh-cuoc.md:86 | "các hằng giá ở đầu fare.dart" | B | "các hằng giá khai báo ở đầu fare.dart" | thấp |

Ghi chú: không tìm thấy lỗi "kiểm sâu" trong nhóm này. Cụm đó chỉ xuất hiện ở các nhóm bài khác trong thư mục.

Điểm đáng giữ nguyên, không nên sửa: "Tới Nơi", "fail open", "rotation", "Halt rollout" khi là tên nút trên console, các thuật ngữ như "Semantics", "guideline", "sealed class", "fake", "lab".

## Thuật ngữ bị dịch lặp lại (đếm trên toàn bộ thư mục content)

Lưu ý: số đếm là số lần khớp chuỗi, gồm cả code và văn bản ngoài nhóm. Số trong ngoặc là số file trong nhóm flutter/dart có chứa cụm đó (đếm sơ bộ).

| Từ đang dùng | Đề xuất | Số lần (số file toàn thư mục) | Ghi chú |
|---|---|---|---|
| trượt (kết quả test / cache) | đạt / không đạt; cache miss | 128 (43) | nhóm flutter: 9 file. Dùng nhiều trong bảng test |
| hết giờ | hết thời gian chờ | 120 (34) | nhóm flutter: 5 file |
| pause / halt (văn xuôi) | tạm dừng / dừng phát hành | pause 120 (20), halt 61 (2) | nhiều chỗ là code; văn xuôi nhóm flutter khoảng 6 file |
| thu hồi | giữ nguyên (chuẩn) | 100 (44) | không phải lỗi |
| bộ đệm | giữ nguyên hoặc "cache" nhất quán | 50 (20) | cần thống nhất "cache" / "bộ đệm" |
| tự báo | tự thông báo | 16 (13) | |
| tự khai | khai báo | 34 (27) | nhiều chỗ đúng nghĩa "khai báo" |
| mức log | mức ghi log | 4 (4) | |
| hạ máy | hạ cấp / hạ phiên bản | 2 (1) | lỗi nặng, chỉ 1 file |
| tải tay | cập nhật thủ công | 2 (1) | |
| hụt (kết quả đo) | vượt / không đạt ngân sách | 44 (19) | "thiếu hụt" là nghĩa đúng, nhưng dùng cho kết quả đo thì mơ hồ |

## Ví dụ tiêu biểu

1. flutter-van-hanh-...md:11: "không có nút hạ máy đã cập nhật về bản cũ" → "không có nút hạ cấp bản đã cập nhật về bản cũ" (cao, sai nghĩa).
2. flutter-authentication-...md:148: "khi họ bị thu hồi" → "khi họ token bị thu hồi" (trung bình, đại từ mơ hồ).
3. flutter-vi-tri-geolocator-...md:99: "chỉ lấy vị trí ngoài 'có màn hình mở'..." → "chỉ khi lấy vị trí lúc không có màn hình mở..." (trung bình, trật tự từ).

## Thống kê

- Số file trong nhóm: 52.
- Số bài đọc kỹ (một phần): 9.
- Số lỗi tìm được: 25 (A: 11, B: 14). Cao: 1. Trung bình: 13. Thấp: 11.
  Ghi chú: phân loại A/B theo ngữ cảnh, một số mục vừa A vừa B được tính vào loại chính.
- Mật độ ước tính: khoảng 2,8 lỗi/bài trong 9 bài đã đọc (chỉ đọc một phần). Nếu áp dụng cho 52 file, ước tính khoảng 100 đến 150 lỗi, độ tin cậy thấp, vì chưa đọc hết các bài.

## Nhận xét về kiểu lỗi phổ biến

- Thuật ngữ vận hành và kiểm thử bị dịch gượng hoặc để lẫn tiếng Anh: "trượt", "hết giờ", "tải tay", "hạ máy", "halt/pause" trong văn xuôi. Đây là nhóm dễ sửa nhất và có ảnh hưởng nhất đến người đọc.
- Câu lược động từ hoặc chủ ngữ theo kiểu tiếng Anh: "X thành Y" thiếu "trở thành", "True cho false", "Package import ...". Lặp lại ở nhiều bài.
- Đại từ và cụm danh từ mơ hồ: "họ", "kho đầu", "huỷ", "hụt", khiến câu cần đọc hai lần.
- Cách dùng "khai báo" và "tự khai" còn chưa thống nhất giữa các bài.
