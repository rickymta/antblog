# Lab 49 · Embedding khuôn mặt, cosine, ngưỡng, FAR/FRR và ROC

Đi kèm bài `python-face-embedding-cosine-threshold-far-frr`. Căn chỉnh 400 ảnh Olivetti (40 người x 10 ảnh) bằng YuNet, lấy embedding bằng **SFace** (OpenCV Zoo, Apache 2.0, 128 chiều, chạy bằng `cv2.FaceRecognizerSF`), so cosine, tự tính FAR/FRR, ROC (TAR theo FAR), AUC, EER bằng numpy, rồi chọn ngưỡng theo mục tiêu FAR, theo chi phí, theo ba vùng gợi ý, và kiểm trên nhiều cách chia người rời nhau. Ảnh YuNet không lấy được mặt **không** được nhúng: lab đếm riêng và coi là `needs_review`, không lùi âm thầm sang cách khác.

Cờ `--with-arcface` thêm ArcFace R50 (InsightFace, 512 chiều, onnxruntime) làm hàng so sánh thang điểm. Xem mục giấy phép trước khi dùng.

- `facepipe.py`: YuNet dò mặt và căn chỉnh 112x112 (copy tự chứa từ lab 48); `aligned()` trả `None` nếu không có mặt.
- `embedders.py`: `SFace` (mặc định) và `ArcFaceR50` (chỉ nạp khi có cờ). Vector ra đã chuẩn hóa L2.
- `metrics.py`: `split_scores` (hàng NaN = ảnh không lấy được mặt, phải truyền `allow_missing=True`), `far_frr`, `roc_points`, `auc`, `equal_error_rate`, `threshold_at_far`, `threshold_at_frr`, `best_threshold_by_cost`, `suggest`. Chỉ numpy.
- `degrade.py`: giả lập ảnh thẻ in/quét (mờ, tối, nhiễu, JPEG) hai mức `nhẹ` và `nặng`.
- `models.py`: tải mô hình vào `~/.cache/toinoi-python/models` (đổi bằng `TOINOI_MODEL_CACHE`), in URL và kích thước, kiểm SHA-256.
- `tests/test_metrics.py`: 15 test toán thuần, không cần mô hình hay ảnh.

## Chạy

Cần mạng ở lần chạy đầu: YuNet 232.589 byte, SFace 38.696.353 byte, và bộ Olivetti qua sklearn (khoảng 4,26 MB, lưu `olivetti_py3.pkz` 1.405.392 byte). Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/49-face-embedding-cosine-threshold examples/toi-noi-python/49-face-embedding-cosine-threshold/main.py
uv run --project examples/toi-noi-python/49-face-embedding-cosine-threshold examples/toi-noi-python/49-face-embedding-cosine-threshold/main.py --verify
uv run --project examples/toi-noi-python/49-face-embedding-cosine-threshold pytest examples/toi-noi-python/49-face-embedding-cosine-threshold -q
# tùy chọn: thêm ArcFace R50 để so thang điểm (đọc mục giấy phép trước)
uv run --project examples/toi-noi-python/49-face-embedding-cosine-threshold examples/toi-noi-python/49-face-embedding-cosine-threshold/main.py --verify --with-arcface
```

Embedding sinh ra nằm trong `out/` (`embeddings_sface.npy`, `labels.npy`, và `embeddings_arcface_r50.npy` khi có cờ); đó là dữ liệu sinh trắc học, đã gitignore, và bài 53 bàn về thời hạn lưu.

## Kết quả kỳ vọng

- Số cặp, số ảnh không lấy được mặt (0, 8 và 59 trên 400 ở ba mức), AUC, EER, ngưỡng, FAR/FRR lặp lại giữa các lần chạy vì dữ liệu và xuống cấp ảnh sinh từ seed cố định và suy luận CPU xác định. Thời gian mỗi ảnh phụ thuộc máy và tải máy; lab in load average.
- `--verify` kết thúc bằng `OK n/n` (exit code 0); nếu lệch thì in `SAI: ...` rồi `FAIL n/m` với n là số ca đạt. Số ca khác nhau khi có hay không có `--with-arcface`. Không có ca nào kiểm tốc độ tuyệt đối.
- Điểm so khớp là đầu vào cho người duyệt hồ sơ KYC (`RV-001`), không phải quyết định xác minh.

## Phiên bản, giấy phép và hạn chế

- Kiểm ngày 2026-11-06 với Python 3.14.7, onnxruntime 1.31.1, OpenCV 5.0.0 (headless), numpy 2.5.4, scikit-learn 1.9.1 (bản trong `uv.lock`); mọi thư viện đều có wheel cho 3.14. Gói `insightface` 2.1 cũng cài được trên 3.14 nhưng lab không dùng nó: lab nạp thẳng file ONNX bằng onnxruntime để khỏi kéo thêm OpenCV bản khác (hai gói `opencv-*` cùng ghi đè thư mục `cv2`). Lab này dùng ngưỡng điểm YuNet 0,6, lab 48 dùng 0,7.
- **Giấy phép cần đọc trước khi chạy.** Mã nguồn InsightFace theo MIT, nhưng README của họ ghi các mô hình huấn luyện sẵn, trong đó có gói `buffalo_l` (file `w600k_r50.onnx`), chỉ dùng cho nghiên cứu phi thương mại; muốn dùng khác phải liên hệ InsightFace để xin giấy phép. Repository này không chứa file mô hình: cờ `--with-arcface` tải trực tiếp từ trang phát hành của họ (gói 288.621.354 byte, giải nén `w600k_r50.onnx` 174.383.860 byte rồi xóa zip) về máy bạn, và lab chỉ dùng nó để so thang điểm giữa hai mô hình; lab in cảnh báo trước khi tải. Nếu bạn chạy trong công ty hoặc dự án có thu phí, hãy hỏi người phụ trách pháp lý trước, hoặc chỉ chạy SFace (OpenCV Zoo ghi Apache 2.0; YuNet ghi MIT). Giấy phép của file mô hình không tự trả lời câu hỏi về dữ liệu đã dùng để huấn luyện. Đoạn này không phải tư vấn pháp lý.
- Bộ khuôn mặt Olivetti do AT&T Laboratories Cambridge thu thập; xem tài liệu scikit-learn. Đó là 40 người chụp trong phòng thí nghiệm 1992-1994, ảnh xám 64x64; mọi ảnh «thẻ» là ảnh xuống cấp bằng code. Số đo cho biết cách các công cụ hoạt động, không phải tỉ lệ lỗi ngoài đời. Chi phí (nhầm nhận 100, nhầm từ chối 1, tỉ lệ mạo danh 2%) là giả định minh họa.
