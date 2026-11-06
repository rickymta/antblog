# Changelog

## 0.1.0

- Bản đầu tiên, gom code của bài 02 đến 05 trong loạt Flutter qua Tới Nơi.
- `Money` (`final class`): tiền VND bất biến (`int` đồng) với `+`, `-`, `*`, so sánh và `toString` dạng `25.500 VND`; `const Money` cho hằng, `Money.fromVnd` luôn kiểm cho dữ liệu từ ngoài.
- `FarePolicy` (hằng `int`), `calculateFareVnd`, `calculateFare`: cước minh hoạ tính bằng mét nguyên.
- `ServiceKind`, `TripStatus` (8 trạng thái), `TripEvent` (7 biến thể), `CancelledBy`.
- `Trip` (`final class`) bất biến với `apply(TripEvent)`, `==` so mọi trường, `isSameTrip`, kiểm bất biến status và `driverId`; `DriverAccepted` và `Cancelled` có `==`.
- `TripIterableX`: `totalFare`, `byStatus`, `active`.
