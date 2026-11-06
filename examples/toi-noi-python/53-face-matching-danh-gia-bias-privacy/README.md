# Lab 53 - Đánh giá so khớp khuôn mặt, thiên lệch, quyền riêng tư và đồng ý

Bài viết: `api/seed/content/python-face-matching-danh-gia-bias-privacy-consent.md`.

- `embedding.py`: SFace (OpenCV Zoo, 2021dec, Apache 2.0) chạy bằng ONNX Runtime, cùng mô hình và tiền xử lý với lab 49 và 52. Tải và kiểm sha256 lần đầu, vá khai báo chiều batch (xem lab 52).
- `data.py`: Olivetti faces (400 ảnh xám 64x64, 40 người) qua `sklearn.datasets.fetch_olivetti_faces`, và YuNet (OpenCV Zoo, MIT) tìm landmark rồi căn chỉnh về crop 112x112. Ảnh không lấy được mặt bị loại và được đếm riêng. Bộ khuôn mặt Olivetti do AT&T Laboratories Cambridge thu thập; xem tài liệu scikit-learn.
- `metrics.py`: bảng cặp cùng người / khác người, FAR/FRR, ngưỡng theo mục tiêu FAR, khoảng tin cậy lấy lại mẫu theo người, chia ngẫu nhiên 20/20, học ngưỡng ở một nửa người và đo ở nửa kia (`threshold_transfer`), quy tắc ba.
- `privacy.py`: `TemplateStore` (SQLite trong bộ nhớ: lịch sử đồng ý, mẫu embedding không có cột ảnh, rút đồng ý thì xóa mẫu, hết hạn, xóa, kiểm toán), `FaceIndex` (bản sao dẫn xuất), dải điểm và cổng quyết định của người (`suggest`, `suggest_no_face`, `system_next_status`, `apply_decision` chỉ nhận hồ sơ ở `needs_review`).
- `main.py`: 8 phần: embedding, đánh giá chung, nhóm giả định, nhóm theo độ sáng, cỡ mẫu, dải điểm, riêng tư, người quyết định.

Cần mạng lần chạy đầu: SFace (38,7 MB) và YuNet (0,2 MB) vào `~/.cache/toinoi-python/models/` (đặt `TOINOI_MODEL_CACHE` để đổi chỗ; cùng thư mục với lab 49), Olivetti (tải 4,26 MB, bản cache 1,4 MB) vào `~/scikit_learn_data/`. Không commit ảnh hay file mô hình. Embedding tính xong lưu vào `out/embeddings.npz` (đã gitignore). Lab không tải mô hình InsightFace; giấy phép và lý do nằm ở bài 49.

Olivetti không có nhãn nhân khẩu học. Các "nhóm" trong lab là giả định theo số thứ tự người hoặc lát cắt đo được trên ảnh (độ sáng), không phải thuộc tính của con người; mọi tỉ lệ chỉ đúng cho 40 người này. Lab không chứng minh hệ thống không thiên lệch. Đồng ý, mục đích, số ngày lưu là giá trị minh họa, không phải tư vấn pháp lý.

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/53-face-matching-danh-gia-bias-privacy examples/toi-noi-python/53-face-matching-danh-gia-bias-privacy/main.py
uv run --project examples/toi-noi-python/53-face-matching-danh-gia-bias-privacy examples/toi-noi-python/53-face-matching-danh-gia-bias-privacy/main.py --verify
```

`--verify` in `OK n/n` (kỳ vọng `OK 54/54`) và exit 0; sai thì in tên ca và `FAIL n/m`, exit 1. Nó chỉ kiểm bất biến (số cặp, FAR không vượt mục tiêu, tính lặp lại theo seed, hành vi đồng ý/hết hạn/xóa, cổng quyết định), không kiểm một kết luận nào về thiên lệch. Output mặc định in số đo thật của lần chạy kèm tải máy; hai lần chạy trên cùng máy cho cùng FAR/FRR vì seed và dữ liệu cố định.
