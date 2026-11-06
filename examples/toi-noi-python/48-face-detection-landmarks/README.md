# Lab 48 · Face detection và landmarks

Đi kèm bài `python-face-detection-landmarks-mediapipe-opencv`. Ghép khuôn mặt Olivetti (400 ảnh xám 64x64, 40 người) lên thẻ căn cước giả và lên nền «phòng» để có ảnh chân dung và selfie, rồi so ba bộ phát hiện (Haar cascade, YuNet qua OpenCV DNN, BlazeFace qua MediaPipe), đo sai số góc nghiêng của landmark (YuNet 5 điểm, FaceLandmarker 478 điểm), căn chỉnh theo mẫu ArcFace 112x112, đo mờ và độ sáng, và gom thành `check_portrait` trả lý do chuyển người duyệt.

- `faces.py`: `HaarDetector`, `YuNetDetector`, `BlazeFaceDetector`, `MediaPipeLandmarker` (cùng giao diện `detect(bgr)`), `similarity_transform`, `align_face`, `face_quality`, `check_portrait`.
- `synth.py`: Olivetti + thẻ giả + nền phòng, xoay, làm mờ, đổi gamma. `synthetic_card.py` là bản copy của lab 30.
- `models.py`: tải mô hình vào `~/.cache/toinoi-python/models` (đổi bằng `TOINOI_MODEL_CACHE`), in URL và kích thước, kiểm SHA-256.
- `main.py`: chương trình demo và `--verify`.

## Chạy

Cần mạng ở lần chạy đầu: bốn mô hình (YuNet 232.589 byte, BlazeFace short range 229.746 byte, BlazeFace full range 1.083.786 byte, FaceLandmarker 3.758.596 byte) và bộ Olivetti qua sklearn (tải khoảng 4,26 MB, lưu `olivetti_py3.pkz` 1.405.392 byte trong `~/scikit_learn_data`). Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/48-face-detection-landmarks examples/toi-noi-python/48-face-detection-landmarks/main.py
uv run --project examples/toi-noi-python/48-face-detection-landmarks examples/toi-noi-python/48-face-detection-landmarks/main.py --verify
```

Ảnh minh họa sinh ra trong `out/` (đã gitignore): `card_detections.png`, `aligned_roll30.png`. Không có ảnh người hay file mô hình nào nằm trong repository.

## Kết quả kỳ vọng

- Lệnh đầu in sáu bước. Số lượng ảnh trúng, mặt thừa, sai số góc nghiêng và tỉ lệ ảnh bị gắn mờ lặp lại giữa các lần chạy vì mọi dữ liệu sinh từ seed cố định; thời gian (ms) phụ thuộc máy và tải máy, lab in load average.
- `--verify` kết thúc bằng `OK n/n` (exit code 0); nếu lệch thì in `SAI: ...` rồi `FAIL n/m` với n là số ca đạt. Không có ca nào kiểm tốc độ tuyệt đối.
- Kết quả là đầu vào cho người duyệt: `check_portrait` trả `face_checked` hoặc `needs_review` kèm lý do, không bao giờ trả «giả mạo» hay «sai người».

## Phiên bản và hạn chế

- Kiểm ngày 2026-11-06 với Python 3.14.7, `opencv-contrib-python` 4.14.0.94, `mediapipe` 1.2.2, `numpy` 2.5.4, `scikit-learn` 1.9.1 (bản trong `uv.lock`). Cả MediaPipe 1.2.2 lẫn OpenCV đều có wheel cho 3.14; không phải lùi về 3.13.
- OpenCV 5.0.0.93 (kiểm ngày 2026-11-06): bản chính `opencv-python-headless` bỏ `cv2.CascadeClassifier` và file `haarcascade_*.xml`; bản `opencv-contrib-python` còn class và `cv2.data.haarcascades` nhưng không kèm file xml, nên Haar báo `RuntimeError` khi nạp. Lab chặn `opencv-contrib-python<5`, là gói mà `mediapipe` cũng kéo theo; không cài thêm `opencv-python-headless` vì hai gói cùng ghi đè thư mục `cv2`.
- BlazeFace có hai mô hình: short range (229.746 byte) và full range (1.083.786 byte); lab so cả hai. Bộ khuôn mặt Olivetti do AT&T Laboratories Cambridge thu thập; xem tài liệu scikit-learn.
- Olivetti là ảnh xám chụp năm 1992-1994 trong phòng thí nghiệm, đã cắt sát mặt; ảnh «selfie» ở đây là ảnh ghép tổng hợp, không phải ảnh chụp thật bằng điện thoại. Số đo cho biết hành vi của từng bước trên dữ liệu này, không phải tỉ lệ lỗi ngoài đời.
