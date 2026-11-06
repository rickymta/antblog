# Lab 21 · asyncio: coroutine, Task, timeout và hủy

Đi kèm bài `python-asyncio-coroutine-task-timeout-cancellation`. «Dịch vụ báo giá» là mô phỏng: độ trễ mạng được thay bằng `asyncio.sleep` với jitter sinh từ `random.Random(42)`. Lab không cần mạng và không cần hạ tầng; chỉ dùng thư viện chuẩn.

Nội dung `main.py`: tuần tự và đồng thời (`gather`, `TaskGroup`), một lời gọi lỗi, `asyncio.timeout`, hủy và dọn dẹp (`CancelledError`; kể cả thí nghiệm `asyncio.timeout` bọc coroutine nuốt hủy), chặn event loop và `asyncio.to_thread` (chạy riêng với `debug=True`).

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/21-asyncio-task-timeout examples/toi-noi-python/21-asyncio-task-timeout/main.py
uv run --project examples/toi-noi-python/21-asyncio-task-timeout examples/toi-noi-python/21-asyncio-task-timeout/main.py --verify
```

## Kết quả kỳ vọng

- Lệnh đầu in 5 mục. Phần logic (cước, nội dung các dòng sự kiện) lặp lại; thứ tự A/B bị hủy hoặc chạy xong không được asyncio bảo đảm nên lab sắp xếp các dòng này trước khi in; số giây đo được dao động nhẹ theo máy: tuần tự khoảng 1,9 s, `gather`/`TaskGroup` khoảng 0,24 s, timeout dừng ở khoảng 0,30 s.
- `--verify` kết thúc bằng `OK 20/20` (exit code 0). Ca sai in `SAI: <tên ca>`, rồi `FAIL n/20` (n là số ca đạt) và thoát mã 1. Các ca về thời gian so theo ngưỡng rộng (ví dụ `gather` nhanh hơn tuần tự ít nhất 3 lần), không so số giây tuyệt đối; trên máy rất bận, chạy lại là đủ.
