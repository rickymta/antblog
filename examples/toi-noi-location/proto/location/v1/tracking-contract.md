# Hợp đồng tracking giữa `tracking-hub` và app khách

Mục tiêu: marker tài xế **chạy mượt dọc tuyến** thay vì nhảy mỗi 4 giây. Hub chỉ gửi điểm mới nhất; app tự nội suy.

## 1. Kết nối

- `GET /track?trip=TN-0001`, WebSocket, header `Authorization: Bearer <token khách>`. Token gắn với đúng một chuyến (claim `trip`).
- Hub chỉ nhận nếu chuyến **đã có assignment** và khách là chủ chuyến; ngược lại trả `403` (cùng một mã cho «chưa gán», «chuyến người khác», «không tồn tại», để khách không dò được chuyến nào tồn tại). Token sai/hết hạn/sai chuyến: `401`. Quá 3 kết nối nghe cho một chuyến: `429`.
- Khi chuyến kết thúc hub đóng bằng close code `1000` và lý do `trip ended`. App thấy `1000` thì ngừng nghe, không kết nối lại.
- Mất kết nối bất thường: kết nối lại với backoff có jitter (0,5 s nhân đôi tới 8 s). Hub gửi ngay điểm gần nhất khi kết nối lại.

## 2. Message và ack

Hub → app: `TrackingUpdate`. App → hub: **`TrackingAck`** sau mỗi khung đã xử lý xong (một binary frame, field `seq` là `seq` của khung vừa xử lý).

Ack là backpressure tầng ứng dụng. Hub chỉ cho tối đa **2 khung chưa ack** đang bay; khi đã đủ, hub giữ điểm mới nhất trong một ô chờ duy nhất (điểm cũ hơn bị thay, tăng `skipped_since_last`), và gửi ngay khi nhận ack kế tiếp. Không ack trong 30 giây dù đang có điểm chờ thì hub đóng kết nối với `1008 client too slow`. Lý do cần ack: nếu chỉ dựa vào việc `Write` bị chặn, bộ đệm TCP (hàng trăm KB) nuốt hàng trăm khung cũ trước khi hub biết client chậm, và khi client đọc lại thì phải xem hết khung cũ mới tới vị trí hiện tại.


Mỗi binary frame là một `TrackingUpdate`.

| Field | Ý nghĩa cho client |
|---|---|
| `sample_time_ms` | lúc điểm này đúng với thực tế, theo **giờ server**. Dùng làm trục thời gian để nội suy |
| `server_time_ms` | lúc hub gửi. `server_time_ms − sample_time_ms` là tuổi của điểm lúc rời hub |
| `seq` | tăng dần theo tài xế. Bỏ qua frame có `seq` ≤ `seq` đã nhận **trừ khi** `sample_time_ms` mới hơn (app tài xế khởi động lại thì `seq` về 0) |
| `skipped_since_last` | số điểm hub đã gộp bỏ vì client đọc chậm. Chỉ để đo; **không** coi là chỗ đứt của đường đi |
| `accuracy_dm` | lớn hơn 500 (50 m): vẽ vòng bất định, đừng vẽ marker như vị trí chính xác |
| `heading_ddeg`, `speed_dmps` | quay biểu tượng xe, ước lượng thời gian tới điểm kế |
| `route_offset_m` | nếu có: vị trí đã bám đường, tính bằng mét từ đầu route. Ưu tiên hơn tự chiếu lat/lng lên route. Lab hiện để trống (map matching ở LOC-16/17) |

Hub **không** gửi điểm mang cờ `TELEPORT`, `SPEED_IMPLAUSIBLE`, `MOCK_REPORTED`. Điểm `POOR_ACCURACY` vẫn gửi, kèm `accuracy_dm`.

## 3. Cách nội suy (phía app)

Mục tiêu là **trễ có chủ đích** để luôn có hai điểm mà nội suy giữa chúng:

1. Giữ hai điểm gần nhất `A` (trước) và `B` (mới). Thời lượng đoạn `T = B.sample_time − A.sample_time`, kẹp vào `[0,5 s; 10 s]`.
2. Khi nhận `B`, marker đang ở đâu thì chạy tiếp từ đó tới `B` trong `T` (không dịch tức thì sang `A` rồi chạy). Easing tuyến tính.
3. Có route của chuyến (từ geo-engine): chiếu `A`, `B` lên route lấy hai vị trí dọc tuyến rồi chạy **theo route** giữa hai vị trí đó; không có route thì đường thẳng. Có `route_offset_m` thì dùng thẳng.
4. Marker không bao giờ chạy lùi dọc route. Nếu `B` nằm sau marker (nhiễu GPS), giữ nguyên marker tới khi `B` vượt qua.
5. Khoảng cách `A→B` quá 500 m hoặc `T` quá 15 s: coi là mất tín hiệu rồi có lại. Dịch chuyển bằng fade, không chạy xuyên qua phố.
6. Không có điểm mới quá 10 s: hiện trạng thái «đang tìm lại vị trí», dừng marker tại điểm cuối. Không ngoại suy theo vận tốc quá 3 s (xe có thể đã rẽ vào hẻm).
7. Quay biểu tượng theo `heading` theo cung ngắn nhất, làm mượt, và không quay khi `speed_dmps` bằng 0.

Đo trong lab (độ trễ trung bình marker so với thực tế = `interp_delay` cộng độ tuổi điểm) ở `results/p02-go.md`, mục LOC-19.

## 4. Quyền riêng tư

Khách chỉ thấy tài xế **sau khi được gán** và chỉ trong chuyến đó. Hub không giữ lịch sử: chỉ một điểm gần nhất cho mỗi chuyến đang mở. Log của hub không ghi toạ độ.
