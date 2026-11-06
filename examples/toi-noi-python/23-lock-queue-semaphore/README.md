# Lab 23 · Lock, Queue, Semaphore: tránh nhận trùng chuyến

Đi kèm bài `python-concurrency-lock-queue-semaphore`. Chỉ dùng thư viện chuẩn, không cần mạng và không cần hạ tầng.

Nội dung `main.py`:

1. Race check-then-act khi hai (rồi 20) coroutine cùng nhận chuyến `TN-0001`; sửa bằng `asyncio.Lock`; `asyncio.Lock` không tái nhập.
2. Cùng race với thread (lịch xen kẽ điều khiển bằng `threading.Barrier`); sửa bằng `threading.Lock`; bảng race không có Barrier (2000 chuyến x 4 thread, có/không lời gọi hàm giữa hai dòng, `sys.setswitchinterval` mặc định và 1e-6 s) kèm bytecode của hai hàm.
3. `asyncio.Semaphore(10)` giới hạn 100 lời gọi báo giá.
4. `asyncio.Queue` producer/consumer: `maxsize` (backpressure), `join`/`task_done`, dừng bằng sentinel hoặc `Queue.shutdown()` (Python 3.13+).

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/23-lock-queue-semaphore examples/toi-noi-python/23-lock-queue-semaphore/main.py
uv run --project examples/toi-noi-python/23-lock-queue-semaphore examples/toi-noi-python/23-lock-queue-semaphore/main.py --verify

# Chỉ in bảng race; chạy được cả trên build free-threaded (cần `uv python install 3.14t`)
uv run --no-project --python 3.14t examples/toi-noi-python/23-lock-queue-semaphore/main.py --race-table
```

## Kết quả kỳ vọng

- Số người thắng không khóa: 2 và 20 (asyncio), 8 (thread có Barrier); có khóa: 1. `Semaphore(10)`: đỉnh đồng thời 10, mất khoảng 0,5 s. Queue `maxsize=5`: `peak qsize` 5, producer bị giữ lại khoảng 0,2 s; `maxsize=0`: `peak qsize` 30, producer xong tức thì. Số giây dao động theo máy; dòng «driver cuối» của thread không khóa có thể khác giữa các lần chạy.
- Bảng race (không kiểm bằng `--verify`, vì phụ thuộc máy và build): trên build có GIL, ba dòng đầu cho 0 chuyến hỏng và dòng «có một lời gọi hàm, interval 1e-6 s» cho cỡ 10 đến 30 chuyến trên 2000; trên 3.14t cả bốn dòng cho cỡ vài chục. In kèm load average lúc đo.
- `--verify` kết thúc bằng `OK 20/20` (exit code 0); ca sai in `SAI: <tên ca>`, rồi `FAIL n/20` (n là số ca đạt) và thoát mã 1. Ca về thời gian so theo ngưỡng rộng.
