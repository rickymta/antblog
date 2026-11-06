# Lab 52 - Service so khớp khuôn mặt qua gRPC

Bài viết: `api/seed/content/python-face-match-grpc-service-batching-model-loading.md`.

- `proto/face_match.proto`: hợp đồng `FaceMatch.Embed(image) -> embedding` và `Compare(a, b) -> score`. Các file `face_match_pb2*.py` do `grpc_tools.protoc` sinh và được commit; `--verify` sinh lại ra thư mục tạm và so từng byte.
- `model.py`: tải SFace (OpenCV Zoo, 2021dec, Apache 2.0, 38,7 MB) và YuNet (MIT) vào cache, kiểm sha256; vá khai báo chiều batch của SFace (file gốc khai báo batch cố định 1); tiền xử lý đúng của `cv2.FaceRecognizerSF`; `Embedder` (ONNX Runtime, chọn `providers`); `warm_decoder`.
- `batcher.py`: `MicroBatcher` bằng `asyncio.Queue`: gom tối đa `max_batch` yêu cầu trong `max_wait_ms`, bỏ qua yêu cầu đã hủy/hết hạn, chạy suy luận trên thread riêng, hàng chờ có giới hạn.
- `service.py`: servicer `grpc.aio`, `serving()` (vòng đời: mở cổng, nạp mô hình, warm-up, `grpc.health.v1`, tắt có thứ tự), deadline hai đầu.
- `client.py`: client có deadline mặc định và hỏi health.
- `data.py`: Olivetti faces (400 ảnh xám 64x64, 40 người) qua scikit-learn; YuNet tìm landmark và căn chỉnh về crop 112x112 như bài 48, 49 (ảnh không lấy được mặt bị loại và đếm riêng); đổi crop sang PNG trong bộ nhớ. Bộ khuôn mặt Olivetti do AT&T Laboratories Cambridge thu thập; xem tài liệu scikit-learn.
- `main.py`: chạy 7 phần (file sinh, mô hình, service thật, vòng đời, deadline, hàng đầy, tắt có thứ tự) và các thí nghiệm `--bench`, `--burst`, `--providers`, `--serve`.

Cần mạng lần chạy đầu: SFace (38,7 MB) và YuNet (0,2 MB) vào `~/.cache/toinoi-python/models/` (đặt `TOINOI_MODEL_CACHE` để đổi chỗ; cùng thư mục với lab 49), Olivetti (tải 4,26 MB, bản cache 1,4 MB) vào `~/scikit_learn_data/`. Không commit ảnh hay file mô hình. Lab không tải mô hình InsightFace; giấy phép và lý do nằm ở bài 49. Ảnh gửi lên service là crop mặt đã căn chỉnh; điểm là đầu vào cho người duyệt, không phải quyết định xác minh. Chưa đo GPU (máy không có GPU NVIDIA).

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/52-face-match-grpc-service examples/toi-noi-python/52-face-match-grpc-service/main.py
uv run --project examples/toi-noi-python/52-face-match-grpc-service examples/toi-noi-python/52-face-match-grpc-service/main.py --verify
```

`--verify` in `OK n/n` (kỳ vọng `OK 56/56`) và exit 0; sai thì in tên ca và `FAIL n/m`, exit 1. Nó chỉ kiểm bất biến (batch cho cùng kết quả như chạy lẻ, tiền xử lý khớp OpenCV, trạng thái health, mã lỗi, đếm số lần bỏ qua/suy luận phí, tắt có thứ tự), không kiểm tốc độ: deadline của phần gọi thật đặt rộng để máy bận không làm đỏ.

Thí nghiệm không nằm trong `--verify` (số đổi theo máy, mỗi lệnh in tải máy lúc đo; server chạy ở tiến trình riêng):

```bash
uv run --project examples/toi-noi-python/52-face-match-grpc-service examples/toi-noi-python/52-face-match-grpc-service/main.py --bench
uv run --project examples/toi-noi-python/52-face-match-grpc-service examples/toi-noi-python/52-face-match-grpc-service/main.py --burst 3000
uv run --project examples/toi-noi-python/52-face-match-grpc-service examples/toi-noi-python/52-face-match-grpc-service/main.py --providers
```

Chỉ phục vụ để tự gọi bằng client khác: `main.py --serve --port 50052` (thêm `--max-batch`, `--max-wait-ms`, `--queue-size`, `--pending`, `--unrequested-time`, `--inline`, `--pad-to`, `--providers-list`). SIGTERM và SIGINT dẫn tới tắt có thứ tự: health `NOT_SERVING`, đợi lời gọi đang chạy tối đa 5 giây, rồi dừng.

## Sinh lại file từ .proto

```bash
uv run --project examples/toi-noi-python/52-face-match-grpc-service python -m grpc_tools.protoc \
  -I examples/toi-noi-python/52-face-match-grpc-service/proto \
  --python_out=examples/toi-noi-python/52-face-match-grpc-service \
  --pyi_out=examples/toi-noi-python/52-face-match-grpc-service \
  --grpc_python_out=examples/toi-noi-python/52-face-match-grpc-service \
  examples/toi-noi-python/52-face-match-grpc-service/proto/face_match.proto
```
