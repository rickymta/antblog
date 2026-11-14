# Style guide: tiếng Việt cho bài kỹ thuật

Mục tiêu là người đọc hiểu ngay ở lần đọc đầu, không phải dịch ngược sang tiếng Anh trong đầu để đoán ý. Thuật ngữ cụ thể xem ở [terminology.md](terminology.md).

## 1. Không lược mất danh từ chính

Tiếng Anh cho phép dùng danh từ làm định ngữ rất gọn ("fare test", "rain surcharge"). Dịch sát như vậy sang tiếng Việt thì câu cụt và khó hiểu.

| Sai | Đúng |
|---|---|
| năm test đầu tiên cho **cước** Tới Nơi | năm test case đầu tiên cho **chức năng tính cước** của Tới Nơi |
| phụ phí **mưa** | phụ phí **trời mưa** |
| hạ **máy** đã cập nhật về bản cũ | hạ **phiên bản** của máy đã cập nhật về bản cũ |

## 2. Không tự ghép từ Việt mới bằng cách dịch từng chữ

Nếu chưa có từ Việt quen dùng, giữ thuật ngữ tiếng Anh (có thể giải nghĩa ở lần đầu).

| Sai | Đúng |
|---|---|
| có đáng để **kiểm sâu** | có đáng để **kiểm thử chuyên sâu** |
| **biên kỳ** báo cáo | **ranh giới kỳ** báo cáo |
| **Vận hành** đề nghị thêm chat | **Bộ phận vận hành** đề nghị thêm chat (hoặc viết lại câu cho rõ chủ thể) |

## 3. Câu phải có đủ chủ ngữ và vị ngữ

| Sai | Đúng |
|---|---|
| không phải kết luận một ngôn ngữ độc quyền một loại bài toán | điều này **không có nghĩa là** một ngôn ngữ độc quyền một loại bài toán |
| Fire-and-forget có thể làm lỗi mất người quan sát | Fire-and-forget có thể khiến lỗi xảy ra mà **không ai theo dõi** |
| COUNT(s.id) bỏ null nhưng vẫn đếm ticket, không tự trở thành số chuyến có ticket | COUNT(s.id) **bỏ qua** null nhưng vẫn đếm từng ticket, **nên không cho ra** số chuyến có ticket |

## 4. Tránh cấu trúc câu kiểu tiếng Anh

- Hạn chế câu bị động "được … bởi …". Chuyển sang chủ động: "Hàm `quote()` tính phí" thay cho "Phí được tính bởi hàm `quote()`".
- Hạn chế dùng "Nó" để chỉ sự vật trừu tượng ở đầu câu. Nhắc lại danh từ: "Smoke test trả lời…" thay cho "Nó trả lời…".
- Các calque cần tránh: "không âm thầm" (→ "không bỏ qua mà không báo lỗi"), "đẩy ra một hàm" (→ "tách thành một hàm riêng"), "đề nghị có hợp tác" (→ viết lại theo nghĩa thật).

## 5. Thống nhất thuật ngữ trong bài và trong series

Một khái niệm chỉ có một tên. Không xen kẽ "test case", "ca kiểm thử" và "case" trong cùng một bài. Chọn theo bảng thuật ngữ; lần đầu có thể ghép cả hai dạng.

## 6. Tiếng Anh trong văn xuôi

- Giữ nguyên tiếng Anh với thuật ngữ trong bảng có `keepEnglish: true`.
- Không để những từ tiếng Anh thông thường lọt vào văn xuôi khi đã có từ Việt tự nhiên: "candidate" → "ứng viên", "skip" → "bỏ qua", "owner" → "người phụ trách".
- Tên hàm, biến, lệnh, khoá cấu hình luôn đặt trong `inline code`.

## 7. Chính tả

- **Vị trí dấu thanh với "oa", "oe", "uy" ở âm tiết mở: dùng kiểu cũ**, đặt dấu trên nguyên âm thứ hai: huỷ, khoá, xoá, hoá, hoạ, toạ, tuỳ, hoà, thoả, luỹ, uỷ, khoẻ, nhoè. *(Đã chốt 11/2026.)* Âm tiết có phụ âm cuối thì hai kiểu như nhau (hoàn, khoảng, thuyền), không cần sửa. `content:lint` có 15 rule `chinh-ta-*` tự sửa trường hợp này. Lint không đụng vào khối code và inline code, nên chữ trong code (comment, chuỗi) vẫn giữ nguyên như mã lab.
- Dùng « » hoặc " " thống nhất trong một bài. Không trộn lẫn.
- Số và đơn vị: `5 km`, `30%`, `200 ms`. Tiền: `25.000 đ`.

## 8. Tiêu đề và tóm tắt

`title` và `summary` hiển thị ở thẻ bài, kết quả tìm kiếm và ảnh OG, nên cần kỹ nhất:

- Tiêu đề phải đứng độc lập được: người đọc chưa vào bài vẫn hiểu bài nói về gì.
- Mẫu chung: `<Thuật ngữ/kỹ thuật>: <bài toán cụ thể của Tới Nơi>`, với vế sau là một cụm danh từ đầy đủ.
