# Lab 41 · Bài toán KYC tài xế: luồng nghiệp vụ, dữ liệu tổng hợp và metric

Đi kèm bài `python-kyc-tai-xe-bai-toan-du-lieu-metric`, bài mở đầu phần 05 (OCR giấy tờ định danh). Lab dựng ba thứ: bộ ảnh giấy tờ tổng hợp có đáp án, các metric (CER, WER, precision theo trường, cổng chuyển người duyệt) và một baseline Tesseract (`-l vie --psm 4`, cấu hình riêng của bài 41 và 42). Bộ ảnh seed 2026 này dùng cho bài 41 và 42; các bài sau sinh bộ riêng với seed và tham số biến thể riêng, nên CER chỉ so được trong cùng bài.

- `synthetic_card.py`: `make_id_card(seed)` và `make_card_fields(seed)` như lab 30 (giữ nguyên), thêm GPLX (`make_license_card`, `make_license_fields`), `make_document(doc_type, seed)`, bốn biến thể ảnh (`apply_variant`: `clean`, `blur`, `rotate`, `dark`) và `photograph(...)` đặt thẻ lên mặt bàn với góc xoay, phối cảnh, bóng, mờ, nhiễu. Tự chứa, chỉ cần numpy và pillow; bài 42 copy nguyên file.
- `dataset.py`: 25 hồ sơ (`KYC-0001`..`KYC-0025`, tài xế `DR-001`..) x 2 giấy tờ = 50 ảnh, ghi `out/images/IMG-nnn.png` và `out/manifest.json` (đường dẫn ảnh, loại giấy tờ, biến thể, đáp án).
- `metrics.py`: `levenshtein`, `cer`, `wer`, `corpus_cer`, `corpus_wer` (tỉ lệ gộp), `exact_match`, `needs_review`.
- `baseline.py`: gọi `tesseract` bằng subprocess (TSV), gom từ thành dòng, tách trường theo nhãn in trên thẻ.
- `kyc.py`: `KycStatus` (7 trạng thái) và bảng chuyển hợp lệ.
- `main.py`: chạy cả bảy bước và `--verify`.

Họ tên, ngày sinh của CCCD và GPLX cùng một hồ sơ là một. Số CCCD `099` + 1 số giới tính/thế kỷ + 2 số năm sinh + 6 số ngẫu nhiên; số GPLX là `99` + 10 số ngẫu nhiên. Mọi thứ là dữ liệu tổng hợp, ảnh có hai dòng «DỮ LIỆU TỔNG HỢP».

## Chạy

Cần uv, Python 3.14 và Tesseract với gói tiếng Việt (`brew install tesseract tesseract-lang`; kiểm bằng `tesseract --list-langs`, phải có `vie`). Không cần mạng lúc chạy (lần đầu uv tải numpy, pillow), không cần Docker. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/41-kyc-bai-toan-du-lieu-metric examples/toi-noi-python/41-kyc-bai-toan-du-lieu-metric/main.py
uv run --project examples/toi-noi-python/41-kyc-bai-toan-du-lieu-metric examples/toi-noi-python/41-kyc-bai-toan-du-lieu-metric/main.py --verify
```

Ảnh và manifest nằm trong `out/` (đã gitignore): chạy lại là sinh lại cùng nội dung với seed 2026. Chạy mất vài phút (50 ảnh x 4 chế độ `--psm`, cộng 200 ảnh của thí nghiệm mất nét; đo được 2 phút 23 giây khi máy rất bận, load average 169).

## Kết quả kỳ vọng

- `--verify` kết thúc bằng `OK 43/43` (exit 0); lệch thì in `SAI: <tên ca>` rồi `FAIL n/43` với n là số ca đạt. Thiếu Tesseract hoặc gói `vie` thì in `FAIL: ...` và exit 1.
- Đáp án và ảnh phụ thuộc font của máy (Arial trên macOS, DejaVu Sans trên Linux; không có thì bỏ dấu). Số liệu trong bài đo trên macOS với Arial, Tesseract 5.5.4. Máy khác font hoặc khác phiên bản Tesseract sẽ ra số khác chút ít; `--verify` chỉ kiểm bất biến và thứ bậc lớn (CER ảnh sạch < ảnh mờ < ảnh xoay, ảnh tối vẫn dưới 0,1), không kiểm số tuyệt đối.
- Đã chạy với Python 3.14.7, pillow 13.0.0, numpy 2.5.4, Tesseract 5.5.4.
