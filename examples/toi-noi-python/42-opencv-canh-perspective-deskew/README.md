# Lab 42 · Phát hiện cạnh, biến đổi phối cảnh và deskew ảnh giấy tờ

Đi kèm bài `python-opencv-phat-hien-canh-perspective-deskew`. Sinh 50 ảnh «chụp» thẻ đặt trên mặt bàn (xoay, nghiêng phối cảnh, tối, bị cắt mép), tìm tứ giác của thẻ bằng Canny và `findContours`, nắn thẳng bằng `warpPerspective`, deskew bằng `minAreaRect`, thử CLAHE và nhị phân hóa thích nghi, rồi đo CER của Tesseract trước và sau từng bước.

- `synthetic_card.py`: bản của lab 41 (cùng tên hàm: `make_id_card`, `make_card_fields`, `make_license_card`, `make_document`, `photograph`...). Tự chứa, chỉ cần numpy và pillow. `photograph` trả ảnh kèm bốn góc thật của thẻ nên đo được sai số của bước tìm góc.
- `metrics.py`, `baseline.py`: copy của lab 41 (CER gộp, gọi Tesseract qua subprocess, tách trường theo nhãn). Tesseract chạy `-l vie --psm 4` (mốc riêng của bài 41 và 42); bảng cuối đo lại ba đường hình học bằng `vie+eng --psm 6`, cấu hình của bài 43 trở đi, và cho kết luận khác.
- `vision.py`: các hàm OpenCV: `edge_map`, `find_card_quad`, `order_corners`, `warp_card`, `rectify`, `min_area_rect_angle`, `deskew`, `clahe`, `adaptive_binary`, `denoise_median`.
- `main.py`: chín bước (ảnh chụp giả, tìm tứ giác, thử bỏ kiểm tra sát mép khung, nắn bằng góc thật, deskew, nhiễu sau CLAHE, ba bảng CER: hình học, tăng cường mỗi cột đổi một thứ, và cấu hình `vie+eng --psm 6`) và `--verify`.

Bộ ảnh (seed 2026): 25 hồ sơ x (CCCD, GPLX) = 50 ảnh JPEG 1600 x 1200 trong `out/photos/` kèm `out/manifest.json` (điều kiện, tham số chụp, bốn góc thật, đáp án). Bốn điều kiện: `skew` (15 ảnh, xoay 8 đến 20 độ trên nền phẳng sáng), `tilt` (14, nghiêng phối cảnh trên ba mặt bàn), `hard` (14, như `tilt` nhưng tối, có bóng, mờ, nhiễu), `cutoff` (7, mép phải thẻ ra ngoài khung).

## Chạy

Cần uv, Python 3.14 và Tesseract với gói tiếng Việt (`brew install tesseract tesseract-lang`; `tesseract --list-langs` phải có `vie`). Lần đầu uv tải numpy, pillow, opencv-python-headless nên cần mạng; không cần Docker. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/42-opencv-canh-perspective-deskew examples/toi-noi-python/42-opencv-canh-perspective-deskew/main.py
uv run --project examples/toi-noi-python/42-opencv-canh-perspective-deskew examples/toi-noi-python/42-opencv-canh-perspective-deskew/main.py --verify
```

Ảnh nằm trong `out/` (đã gitignore). Chạy mất vài phút vì Tesseract đọc 11 đường xử lý x 50 ảnh, thêm 3 đường x 50 ảnh ở cấu hình thứ hai, cộng 50 ảnh mốc (đo được 9 phút 1 giây khi máy rất bận, load average 241); ảnh trung gian của từng đường nằm trong thư mục tạm và bị xóa khi xong.

## Kết quả kỳ vọng

- `--verify` kết thúc bằng `OK 33/33` (exit 0); lệch thì in `SAI: <tên ca>` rồi `FAIL n/33` với n là số ca đạt. Thiếu Tesseract hoặc gói `vie` thì in `FAIL: ...` và exit 1.
- Các ca kiểm là bất biến và thứ bậc lớn, không phải số tuyệt đối: hình học (hệ số homography, thứ tự góc, sai số nắn bằng góc thật), phát hiện (tìm được tứ giác ở ít nhất 90% ảnh không bị cắt, ảnh bị cắt mép bị từ chối thay vì nắn sai và tắt kiểm tra thì ra tứ giác lệch trên 25 px, ảnh không có thẻ trả `None`), góc `minAreaRect` (sai số dưới 1 độ), và OCR (nắn phối cảnh hoặc deskew giảm CER ít nhất một nửa; CLAHE rồi ngưỡng làm hỏng OCR trên bộ này; đổi C hoặc lọc trung vị cứu được ngưỡng).
- Số trong bài đo trên macOS với font Arial, Tesseract 5.5.4, opencv-python-headless 5.0.0 (bản trong `uv.lock`); góc `minAreaRect` thô đã kiểm riêng trên 4.14.0 (cùng kết quả) và 4.10.0 (góc nằm trong (0, 90], quy ước khác). Font hoặc phiên bản Tesseract khác sẽ cho CER lệch chút ít.
- Đã chạy với Python 3.14.7, pillow 13.0.0, numpy 2.5.4.
