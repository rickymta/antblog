# Lab 37 - gRPC: service chấm điểm gian lận cho Dispatch

Bài viết: `api/seed/content/python-grpc-fraud-scoring-service.md`.

- `proto/fraud_scoring.proto`: hợp đồng `FraudScoring.Score` (`ScoreRequest` -> `ScoreResponse`). Các file `fraud_scoring_pb2.py`, `fraud_scoring_pb2.pyi`, `fraud_scoring_pb2_grpc.py` do `grpc_tools.protoc` sinh và được commit; `--verify` sinh lại ra thư mục tạm và so từng byte.
- `service.py`: bộ luật chấm điểm mẫu (`score_trip`, thuần), servicer `grpc.aio`, `start_server` (cổng 0 = cổng trống do hệ điều hành chọn).
- `interceptors.py`: server interceptor ghi (method, status, thời gian, `x-request-id`); client interceptor gắn `x-request-id`.
- `client.py`: `FraudClient` với deadline mặc định 0,5 giây và giới hạn số lời gọi đang bay (Semaphore).
- `main.py`: khởi động server trong tiến trình và chạy 6 phần: dữ liệu trên dây, gọi thường, lỗi/status code, deadline, interceptor, đồng thời.

Không cần hạ tầng ngoài, không cần mạng khi chạy (lần resolve đầu cần mạng để tải `grpcio`, `protobuf`, `grpcio-tools`). Điểm trả về là đầu vào cho người duyệt, không phải quyết định hay kết luận gian lận; bộ luật là luật viết tay để minh họa, không phải mô hình thật.

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/37-grpc-fraud-scoring examples/toi-noi-python/37-grpc-fraud-scoring/main.py
uv run --project examples/toi-noi-python/37-grpc-fraud-scoring examples/toi-noi-python/37-grpc-fraud-scoring/main.py --verify
```

Kết quả `--verify` kỳ vọng: `OK 41/41`, exit code 0. Các số mili giây phụ thuộc máy và tải; `--verify` chỉ kiểm bất biến (mã trạng thái, số đếm, tính khớp file sinh), không kiểm thời gian tuyệt đối.

Chế độ phụ:

```bash
# chỉ chạy server ở cổng cố định để tự gọi bằng client khác
uv run --project examples/toi-noi-python/37-grpc-fraud-scoring examples/toi-noi-python/37-grpc-fraud-scoring/main.py --serve --port 50051
# thí nghiệm bắn 2000 lời gọi cùng lúc: mặc định / Semaphore ở client / nâng ngưỡng hàng chờ của server (số bị CANCELLED đổi theo máy)
uv run --project examples/toi-noi-python/37-grpc-fraud-scoring examples/toi-noi-python/37-grpc-fraud-scoring/main.py --burst 2000
```

## Sinh lại file từ .proto

```bash
uv run --project examples/toi-noi-python/37-grpc-fraud-scoring python -m grpc_tools.protoc \
  -I examples/toi-noi-python/37-grpc-fraud-scoring/proto \
  --python_out=examples/toi-noi-python/37-grpc-fraud-scoring \
  --pyi_out=examples/toi-noi-python/37-grpc-fraud-scoring \
  --grpc_python_out=examples/toi-noi-python/37-grpc-fraud-scoring \
  examples/toi-noi-python/37-grpc-fraud-scoring/proto/fraud_scoring.proto
```

File sinh nằm cạnh `main.py` vì `fraud_scoring_pb2_grpc.py` import tuyệt đối `import fraud_scoring_pb2`. Mã sinh ghi phiên bản `grpcio-tools` đã dùng (1.85.1) nên `pyproject.toml` pin `grpcio>=1.85.1` và `protobuf>=7.35.1`.
