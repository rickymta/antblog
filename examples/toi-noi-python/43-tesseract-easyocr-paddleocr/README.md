# Lab 43 · Tesseract, EasyOCR và PaddleOCR trên cùng bộ ảnh

Đi kèm bài `python-tesseract-easyocr-paddleocr-so-sanh`. Sinh 24 ảnh (6 thẻ căn cước giả x 4 biến thể: sạch, mờ, nghiêng, tối), chạy ba engine OCR trên đúng bộ ảnh đó, đo thời gian, CPU và lỗi theo trường, rồi in bảng so sánh. Kết quả OCR là đầu vào cho người duyệt, không phải quyết định.

- `synthetic_card.py`: copy nguyên từ lab 30 (`make_id_card`, `make_card_fields`). Đáp án mọi ảnh là `make_card_fields(seed)`, không gán nhãn tay. Chữ trên thẻ phụ thuộc font hệ thống (Arial trên macOS).
- `dataset.py`: `build_dataset(out_dir)` sinh ảnh vào `out/images/` và `out/manifest.json`; seed `(1, 2, 4, 7, 21, 50)` chọn để đủ sáu tên mẫu và sáu thành phố.
- `ocr_common.py`: `OcrLine(text, confidence 0..1, box)`, Levenshtein, CER, khớp giá trị trường với các dòng OCR, đo CPU.
- `engines.py`: `TesseractEngine`, `EasyOcrEngine`, `PaddleOcrEngine` sau một giao diện `recognize(path) -> list[OcrLine]`.
- `compare.py`: đo từng ảnh, ghi `out/results-<engine>.json`, dựng 5 bảng chỉ cho engine đã có file kết quả.
- `main.py`: chương trình chính và `--verify`. `exercise_crop.py`: lời giải bài tập 1.

## Cài

Tesseract cài ngoài Python (macOS): `brew install tesseract tesseract-lang` (cần gói `vie` và `eng`). Lab đã chạy với Tesseract 5.5.4.

Lõi chỉ cần numpy, pillow, pytesseract. EasyOCR là extra (kéo theo PyTorch). PaddleOCR nằm ở project con `paddle/` chạy Python 3.13, vì kiểm ngày 2026-11-06 `paddlepaddle` 3.3.1 chỉ có wheel cp39 đến cp313, không có cp314 (`uv pip install paddlepaddle` trên 3.14 báo không có wheel khớp ABI). EasyOCR 1.7.2 và PyTorch 2.15.0 chạy bình thường trên 3.14.7.

## Chạy

Từ thư mục gốc repository. EasyOCR và PaddleOCR cần mạng ở lần chạy đầu để tải mô hình vào cache của chính chúng (`~/.EasyOCR/model` khoảng 94 MB, `~/.paddlex/official_models` vài chục đến vài trăm MB), không vào repository.

```bash
L=examples/toi-noi-python/43-tesseract-easyocr-paddleocr
uv run --project $L $L/main.py                                   # Tesseract: quét -l x --psm, đo, so sánh
uv run --project $L --extra easyocr $L/main.py --engine easyocr  # Python 3.14
uv run --project $L/paddle $L/main.py --engine paddleocr         # Python 3.13, PP-OCRv6 (mặc định của lang='vi')
uv run --project $L/paddle $L/main.py --engine paddleocr-v5      # Python 3.13, PP-OCRv5 latin
uv run --project $L $L/main.py --compare                         # chỉ in bảng từ out/results-*.json
uv run --project $L $L/exercise_crop.py                          # bài tập 1
uv run --project $L $L/main.py --verify                          # chỉ cần Tesseract
```

## Kết quả kỳ vọng

- `--verify` kết thúc bằng `OK 31/31` (exit code 0); lệch thì in `SAI: <tên ca>` rồi `FAIL n/31` với n là số ca đạt và exit 1. Chỉ cần Tesseract: không kiểm tốc độ tuyệt đối, và ca «thiếu easyocr» giả lập bằng `sys.modules`.
- Bảng so sánh chỉ có hàng của engine đã chạy. Số đo thời gian phụ thuộc máy và tải (lab ghi load average 1 phút trước và sau mỗi lần đo); số lỗi theo trường ổn định hơn nhưng phụ thuộc font của máy.
- Đã chạy trên Apple M1 Max, 10 lõi, macOS 26.6.2, trong lúc máy đang bận (load average 1 phút từ 36 đến 167 trong lần đo cuối): Python 3.14.7 với EasyOCR 1.7.2, PyTorch 2.15.0, pytesseract 0.3.13; Python 3.13.15 với paddlepaddle 3.3.1, paddleocr 3.7.0.
