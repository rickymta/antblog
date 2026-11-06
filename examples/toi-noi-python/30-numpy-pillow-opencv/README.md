# Lab 30 · NumPy, Pillow và OpenCV: ảnh là mảng

Đi kèm bài `python-numpy-pillow-opencv-tien-xu-ly-anh`. Sinh thẻ căn cước giả, đọc nó thành `ndarray`, đổi xám bằng broadcasting, resize, ngưỡng hóa (Otsu và thích nghi trên ảnh bị bóng), đo vòng lặp Python so với phép toán mảng, rồi đo thread trên một lô thẻ.

- `synthetic_card.py`: `make_id_card(seed) -> PIL.Image` (1012 x 638, RGB) và `make_card_fields(seed)` trả đúng chữ đã vẽ lên thẻ, làm đáp án cho OCR ở các bài sau. Tự chứa, chỉ cần numpy và pillow; phần 05 copy nguyên file này (giữ tên `make_id_card`, `make_card_fields`, `CARD_WIDTH`, `CARD_HEIGHT`).
  - Bộ trường chung: `full_name` (chữ hoa), `id_number`, `date_of_birth` (dd/mm/yyyy; khi lưu MongoDB đổi sang ISO), `sex`, `hometown_city`.
  - Số thẻ: `099` + 1 số giới tính/thế kỷ (thế kỷ 20: nam 0, nữ 1; thế kỷ 21: nam 2, nữ 3) + 2 số cuối năm sinh + 6 số ngẫu nhiên. Mã tỉnh `099` không tồn tại.
  - Dữ liệu tổng hợp: tên mẫu, ô ảnh chỉ là hình chữ nhật xám, hai dòng ghi «DỮ LIỆU TỔNG HỢP» / «KHÔNG PHẢI GIẤY TỜ THẬT».
- `main.py`: các hàm tiền xử lý và chương trình demo, `--verify`.
- Font: ưu tiên Arial (macOS, Windows) hoặc DejaVu Sans (Linux) vì có dấu tiếng Việt; thiếu thì dùng font mặc định của Pillow và bỏ dấu khỏi chữ trên thẻ lẫn trong `make_card_fields` (font mặc định không có glyph chữ Việt có dấu). Vì vậy ảnh và các trường chữ phụ thuộc font của máy; số, ngày và mọi kết quả đếm theo vùng chữ có thể lệch nhẹ trên máy khác font.

## Chạy

Từ thư mục gốc repository (lần đầu uv tải numpy, pillow, opencv-python-headless nên cần mạng):

```bash
uv run --project examples/toi-noi-python/30-numpy-pillow-opencv examples/toi-noi-python/30-numpy-pillow-opencv/main.py
uv run --project examples/toi-noi-python/30-numpy-pillow-opencv examples/toi-noi-python/30-numpy-pillow-opencv/main.py --verify
```

Ảnh sinh ra nằm trong `out/` (đã gitignore): `card.png`, `gray.png`, `card_half.png`, `card_shadow.png`, `otsu_clean.png`, `otsu_shadow.png`, `adaptive_shadow.png`.

## Kết quả kỳ vọng

- Lệnh đầu in bảy bước. Ổn định khi font giống bản đã chạy: shape `(638, 1012, 3)`, `200 + 100` trong `uint8` ra `44`, ảnh bị bóng làm Otsu đen 99,96% dải phải còn ngưỡng thích nghi 2,75%. Các số thời gian (bước 6 và 7) phụ thuộc máy và tải máy; lab in load average.
- `--verify` kết thúc bằng `OK 21/21` (exit code 0); nếu lệch thì in `SAI: ...` rồi `FAIL n/21` với n là số ca đạt. Chỉ có một ca so tốc độ (vectorized nhanh hơn vòng lặp ít nhất 5 lần, thực tế cỡ 50 lần); số liệu thread của bước 7 chỉ được in.
- Đã chạy với Python 3.14.7, numpy 2.5.4, pillow 13.0.0, và opencv-python-headless cả 4.14.0 lẫn 5.0.0.93 (bản trong `uv.lock`), cùng `OK 21/21`.
