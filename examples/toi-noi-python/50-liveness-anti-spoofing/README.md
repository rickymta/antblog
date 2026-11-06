# Lab 50 · Liveness và chống giả mạo cơ bản qua chất lượng ảnh

Đi kèm bài `python-liveness-anti-spoofing-chat-luong-anh`. **Đây không phải liveness thật.** Lab đo blur, độ sáng, tương phản, kích thước mặt và mẫu lặp trong phổ tần số trên ảnh MÔ PHỎNG, rồi xếp mỗi ảnh vào `ok`, `recapture` hoặc `review`. Kết quả `ok` chỉ có nghĩa là không phép đo nào báo; không có giá trị nào tên "live".

- `capture.py`: dựng ảnh mô phỏng. Khuôn mặt là Olivetti (`sklearn.datasets.fetch_olivetti_faces`, 400 ảnh 64 x 64 xám, 40 người), phóng to 160 px đặt vào khung 320 x 320 rồi "chụp" theo 9 kiểu: `genuine`, `blur`, `dark`, `bright`, `small_face`, `screen_moire`, `screen_weak`, `print_halftone`, `screen_clean`. Màn hình và ảnh in là do chính code vẽ (lưới điểm ảnh, chấm halftone), không phải ảnh chụp thật.
- `quality.py`: 5 chỉ số (`sharpness`, `brightness`, `contrast`, `face_ratio`, `moire_peak`), `calibrate` (ngưỡng từ phân vị 1% / 99% của ảnh genuine) và `assess`.
- `main.py`: chương trình demo 6 bước và `--verify`.

Hộp bao khuôn mặt do lab biết sẵn (nó tự đặt mặt vào khung). Ở hệ thống thật, hộp này do bước phát hiện khuôn mặt của bài 48 trả về.

## Chạy

Từ thư mục gốc repository. **Cần mạng ở lần chạy đầu**: scikit-learn tải Olivetti (tải 4,26 MB, bản cache 1,4 MB) vào `~/scikit_learn_data`. Bộ khuôn mặt Olivetti do AT&T Laboratories Cambridge thu thập; xem tài liệu scikit-learn. Không commit ảnh, không cần hạ tầng Docker.

```bash
uv run --project examples/toi-noi-python/50-liveness-anti-spoofing examples/toi-noi-python/50-liveness-anti-spoofing/main.py
uv run --project examples/toi-noi-python/50-liveness-anti-spoofing examples/toi-noi-python/50-liveness-anti-spoofing/main.py --verify
```

Một lượt chạy mất cỡ một phút trên máy rảnh (hàng nghìn ảnh 320 x 320 được sinh và đo); lâu hơn khi máy bận.

## Kết quả kỳ vọng

- Chương trình sinh dữ liệu bằng `numpy.random.default_rng` với seed cố định, nên mọi tỉ lệ in ra lặp lại được từ lần chạy này sang lần khác (đã chạy lại, giống hệt) với cùng phiên bản thư viện. Chỉ phần 6 (thời gian, kèm load average) thay đổi theo máy.
- `--verify` kiểm các bất biến, kết thúc bằng `OK 20/20` (exit code 0); lệch thì in `SAI: <tên ca>` rồi `FAIL n/20` với n là số ca đạt. Ca đặc biệt: `screen_clean` (màn hình phân giải cao, không dấu vết) PHẢI gần như không bị bắt; đây là giới hạn được kiểm chứng, không phải lỗi. Không có ngưỡng tốc độ nào trong `--verify`.
- Đã chạy với Python 3.14.7, numpy 2.5.4, opencv-python-headless 5.0.0.93 (bản trong `uv.lock`), scikit-learn 1.9.1.
