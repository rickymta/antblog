# Lab 45: đánh giá OCR

Bài `python-danh-gia-ocr-cer-wer-threshold`. Đo chất lượng OCR trên bộ test vàng gồm 50 thẻ giả (CCCD và GPLX, bốn biến thể chất lượng ảnh): CER/WER với `jiwer`, phân tích lỗi thay/xóa/chèn theo trường, ngưỡng confidence đưa trường vào hàng chờ người duyệt, và regression test so với `baseline.json`.

Mọi thẻ là **dữ liệu giả** sinh bằng Pillow. Kết quả OCR và ngưỡng ở đây là đầu vào cho người duyệt, không phải quyết định xác minh giấy tờ.

## Cần

- uv và Python 3.14.
- Tesseract kèm ngôn ngữ `vie`: `brew install tesseract tesseract-lang` (đã thử với Tesseract 5.5.4). Lần đầu uv cần mạng để tải numpy, pillow, rapidfuzz, jiwer, pytest. Thiếu gói `vie` thì lab in `FAIL: ...` và thoát với mã 1.
- Không cần hạ tầng docker.

## Chạy

Từ gốc repository:

```bash
uv run --project examples/toi-noi-python/45-danh-gia-ocr-cer-wer examples/toi-noi-python/45-danh-gia-ocr-cer-wer/main.py
uv run --project examples/toi-noi-python/45-danh-gia-ocr-cer-wer examples/toi-noi-python/45-danh-gia-ocr-cer-wer/main.py --verify
uv run --project examples/toi-noi-python/45-danh-gia-ocr-cer-wer pytest examples/toi-noi-python/45-danh-gia-ocr-cer-wer -q
```

`--verify` in lại toàn bộ rồi kiểm 33 điều, in `OK 33/33` (exit 0); lệch thì in tên ca sai, `FAIL n/m`, exit 1. `pytest` chạy 31 test, trong đó có test môi trường, regression so với `baseline.json` và hai test thử phá (thu ảnh còn một nửa; hậu xử lý trả None cho thẻ vốn đã vào hàng chờ) phải bị bắt.

## baseline.json

Ghi bằng `main.py --update-baseline`. File ghi cả môi trường (phiên bản Tesseract, hash `vie.traineddata` và `eng.traineddata`, hash font vẽ thẻ: Arial trên macOS, DejaVu trên Linux) và được kiểm riêng bằng `check_environment`. Môi trường lệch thì test môi trường fail với thông báo riêng; xem lại số, chạy `--update-baseline`, rồi mới commit baseline mới. Không cập nhật baseline chỉ để làm test xanh. Bộ ảnh này khác bộ của lab 41 và 42, nên CER không so chéo giữa các bài.

## Tệp

- `metrics.py`: CER/WER bằng Levenshtein tự viết, đối chiếu với `jiwer`.
- `evaluation.py`: bộ test vàng (`out/manifest.json`), chạy pipeline, CER theo trường và theo biến thể, ngưỡng, baseline và regression.
- `ocr.py`, `postprocess.py`, `synthetic_card.py`: bản copy từ lab 44 (giữ giống hệt).
- `tests/`: test của metric, của ngưỡng/regression và của bộ test vàng.
