# Lab 03 · Điều kiện, vòng lặp và hàm

Đi kèm bài `python-dieu-kien-vong-lap-ham`. Tách quy tắc tính cước thành `calculate_fare(distance_km, *, service="ride")` cho ba dịch vụ `ride`, `food`, `parcel`. Ở mô hình học tập này **cả ba dịch vụ cùng giá** (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, không phụ phí); `service` được kiểm trong `SERVICES` và là tham số keyword-only. Khoảng cách ngoài `[0, 100]` km bị từ chối bằng `ValueError`.

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/03-dieu-kien-vong-lap-ham examples/toi-noi-python/03-dieu-kien-vong-lap-ham/main.py
uv run --project examples/toi-noi-python/03-dieu-kien-vong-lap-ham examples/toi-noi-python/03-dieu-kien-vong-lap-ham/main.py --verify
```

## Kết quả kỳ vọng

- Lệnh đầu in bảng cước cho ba dịch vụ (ví dụ `5 km` = `25.500 VND` ở cả ba), các dòng `-1` và `100.5` km hiện `TỪ CHỐI`, hai dòng ngân sách (30.000 VND đi tối đa 6.0 km, 20.000 VND đi tối đa 3.5 km), rồi minh họa mặc định `list` bị dùng chung.
- Lệnh `--verify` kết thúc bằng `OK 22/22` (exit code 0); nếu có ca sai thì in dòng `SAI: …`, rồi `FAIL n/m` và thoát với mã 1.
