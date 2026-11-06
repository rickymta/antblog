# Lab 44: hậu xử lý OCR

Bài `python-hau-xu-ly-ocr-regex-parse-truong-checksum`. Biến kết quả thô của Tesseract (từ, khung chữ nhật, confidence) thành trường có kiểu: chuẩn hóa Unicode, nhãn bằng so khớp mờ, regex có tên, sửa ký tự dễ nhầm theo ngữ cảnh, kiểm cấu trúc số, so khớp mờ với danh sách đóng, confidence của trường là nhỏ nhất của các từ.

Mọi thẻ là **dữ liệu giả** sinh bằng Pillow (số CCCD `099...`, GPLX `99...`, tên mẫu). Kết quả của lab là đầu vào cho người duyệt, không phải quyết định xác minh giấy tờ. Lab không khẳng định CCCD có checksum công khai; nó chỉ kiểm tính nhất quán giữa số, ngày sinh và giới tính trên cùng thẻ.

## Cần

- uv và Python 3.14.
- Tesseract kèm ngôn ngữ `vie`: `brew install tesseract tesseract-lang` (đã thử với Tesseract 5.5.4). Lần đầu uv cần mạng để tải numpy, pillow, rapidfuzz.
- Không cần hạ tầng docker.

## Chạy

Từ gốc repository:

```bash
uv run --project examples/toi-noi-python/44-hau-xu-ly-ocr-regex-checksum examples/toi-noi-python/44-hau-xu-ly-ocr-regex-checksum/main.py
uv run --project examples/toi-noi-python/44-hau-xu-ly-ocr-regex-checksum examples/toi-noi-python/44-hau-xu-ly-ocr-regex-checksum/main.py --verify
```

`--verify` in lại toàn bộ rồi kiểm 40 điều và in `OK 40/40` (exit 0); lệch thì in tên ca sai, `FAIL n/m`, exit 1. Thiếu Tesseract thì in `FAIL: không chạy được tesseract ...` và exit 1.

Các số của phần «cả bộ» (64 ảnh) phụ thuộc phiên bản Tesseract, gói ngôn ngữ và font hệ thống (Arial trên macOS); `--verify` chỉ kiểm các bất biến (hậu xử lý không làm tụt số trường đúng, phân loại cộng đủ tổng...), không kiểm số tuyệt đối của OCR.

## Tệp

- `synthetic_card.py`: sinh thẻ CCCD giả (copy từ lab 30), thêm GPLX giả và hàm `degrade` (clean, blur, rotate, dark). Bộ ảnh này khác bộ của lab 41 và 42 (GPLX cùng seed là người khác, tham số biến thể khác): CER chỉ so trong cùng một bài.
- `ocr.py`: gọi `tesseract ... tsv`, đọc từng từ kèm khung và confidence; cache trong `out/ocr_cache/`.
- `postprocess.py`: toàn bộ hậu xử lý.
- `main.py`: demo từng bước và `--verify`.
