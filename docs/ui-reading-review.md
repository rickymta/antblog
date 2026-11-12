# Rà soát giao diện đọc — 12/11/2026

Phạm vi: giao diện công khai, nền sáng/tối, trang chủ, thẻ bài và trang đọc. Không thay thiết kế riêng của trang quản trị; typography nội dung Markdown dùng chung nên bản xem trước cũng nhận thay đổi.

## Nhận xét và điều chỉnh

- Giữ Inter có bộ ký tự tiếng Việt và JetBrains Mono cho mã nguồn. Tăng nội dung lên 18px desktop / 17px mobile, giãn dòng 1,8; mã nguồn 15px / 14px.
- Giảm độ sít chữ ở tiêu đề trang chủ và thẻ; tăng tiêu đề thẻ mobile lên 17px, danh sách đọc tiếp lên 16px ở màn hẹp.
- Giới hạn cột bài ở 768px, đoạn văn tối đa 68ch; thêm nhịp cách đoạn trong danh sách, tăng độ thoáng cho mục lục và tóm tắt đầu bài.
- Nền đọc sáng xám nhẹ, chữ xanh xám; nền tối xanh than và chữ trắng dịu. Giữ màu thương hiệu cho liên kết và nút chính.
- Làm dịu gradient banner trang chủ và giảm mật độ thị giác của chấm nền. Giữ ảnh bìa có tỉ lệ ổn định; ảnh/video trong nội dung co theo cột, giữ tỉ lệ gốc. Sơ đồ rộng và mã nguồn vẫn cuộn ngang trong khối để không ép nhỏ chữ.
- Giữ mục lục thu gọn trên màn hẹp và mục lục bám bên phải trên desktop, cùng trạng thái mục đang đọc.

## Xác minh

- Kiểm tra trực quan trang chủ và bài `page-don-vi-doc-ghi` trên trình duyệt local; xem bài ở giao diện sáng và tối.
- Kiểm tra trang bài ở viewport 390px và 1440px: không tràn ngang toàn trang, font nội dung lần lượt 17px và 18px.
- Tương phản chữ phụ với ba nền page/surface/surface-2: sáng 5,75 / 6,06 / 5,62; tối 7,96 / 6,92 / 7,44.
- Typecheck web đạt. Đây là kiểm tra mẫu đại diện, không phải xác minh toàn bộ bài viết hoặc nghiên cứu mức mỏi mắt của người dùng khi đọc lâu.
