# Hợp đồng protobuf của Tới Nơi (định vị)

Một file duy nhất cho cả ba ngôn ngữ: [`location/v1/location.proto`](location/v1/location.proto), package `toinoi.location.v1`.

| Message | Ai gửi → ai nhận | Kênh |
|---|---|---|
| `GpsBatch` / `BatchAck` | app tài xế ⇄ `location-ingest` (Go) | WebSocket, một binary frame mỗi message |
| `DriverLocationUpdated` | `location-ingest` → Kafka → `geo-engine` (Rust), `tracking-hub` (Go), `history-writer` (Go), Fraud (Python) | topic `location.updates.v1`, key `driver_id` |
| `TrackingUpdate` / `TrackingAck` | `tracking-hub` → app khách, và ack ngược lại | WebSocket, binary frame; hợp đồng nội suy và ack ở [`tracking-contract.md`](location/v1/tracking-contract.md) |

[`location/v1/routing.proto`](location/v1/routing.proto) (cùng package) là hợp đồng gRPC của `geo-engine` (LOC-29):

| Message | Ai gửi → ai nhận | Kênh |
|---|---|---|
| `RouteRequest` / `RouteResponse` | Dispatch C#, simulator, Go → `geo-engine` (Rust) | gRPC unary `RoutingService.Route`, deadline qua header `grpc-timeout` |
| `MatrixRequest` / `MatrixResponse` | như trên | gRPC unary `RoutingService.Matrix` |

Rust biên dịch file này bằng `protox` (thuần Rust, không cần `protoc`) trong `build.rs` của `geo-engine`; C# bằng `Grpc.Tools` trong `dotnet/GeoEngineClient`.

Lịch sử lưu ở Parquet có schema riêng: [`gps-history-parquet.md`](location/v1/gps-history-parquet.md).

## Quy ước số (đừng đổi)

| Đại lượng | Kiểu | Đơn vị | Ghi chú |
|---|---|---|---|
| `lat_e7`, `lng_e7` | `sint32` | độ × 10^7 | 1 đơn vị ≈ 1,1 cm; `sint32` (zigzag) mã hoá gọn cả số âm. Thứ tự luôn là lat rồi lng |
| `*_time_ms`, `occurred_at_ms`, `received_at_ms` | `int64` | epoch mili giây UTC | |
| `accuracy_dm` | `optional uint32` | 0,1 m | vắng = app không biết; `0` là giá trị thật |
| `speed_dmps` | `optional uint32` | 0,1 m/s | vắng ≠ 0: xe đứng yên báo 0 |
| `heading_ddeg` | `optional uint32` | 0,1 độ, 0..3599 | |
| `flags` | `uint32` | bit mask của `PointFlag` | `CLOCK_SKEW=1 POOR_ACCURACY=2 SPEED_IMPLAUSIBLE=4 TELEPORT=8 MOCK_REPORTED=16 SPEED_MISMATCH=32` |

Cờ là **tín hiệu**, không phải kết luận. Chỉ Fraud service (Python) mới được kết luận gian lận.

## Sinh lại mã

Máy này không có `protoc`, nên dùng `buf` và `protoc-gen-go`, cả hai là chương trình Go và được pin phiên bản trong script:

```bash
examples/toi-noi-location/proto/generate.sh
```

Script cài `buf` v1.50.0 và `protoc-gen-go` v1.36.12 vào thư mục tạm, chạy `buf lint` rồi `buf generate` (xem `buf.yaml`, `buf.gen.yaml`), ghi `go/gen/location/v1/location.pb.go`. File sinh ra được commit; thay đổi `.proto` thì sinh lại và commit cùng lúc. Cần mạng lần đầu để tải module.

### Rust (prost) và C#

- Rust: `prost-build` với `compile_protos(&["proto/location/v1/location.proto"], &["proto"])` trong `build.rs`. `optional` của proto3 cho ra `Option<u32>`.
- C#: `Grpc.Tools` với `<Protobuf Include="..." GrpcServices="None" />`. Namespace `ToiNoi.Location.V1` đã khai báo trong file.
- Đừng cho code Rust/C# tự định nghĩa lại message: nhập file này.

## Tương thích

Theo kiến trúc §6.6: không đổi ý nghĩa hay số thứ tự field đã có, field mới thêm ở cuối, bỏ field thì `reserved`. Đổi không tương thích thì tạo `location.updates.v2`.
