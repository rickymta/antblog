# Lab 22 · GIL, threading, multiprocessing và free-threaded 3.14

Đi kèm bài `python-gil-threading-multiprocessing-free-threaded`. Cùng một lô 20 «ảnh giả» chạy tuần tự, bằng `ThreadPoolExecutor`, `ProcessPoolExecutor` và `InterpreterPoolExecutor` (3.14) cho ba loại việc: chờ I/O (`time.sleep`), vòng lặp Python thuần (nhị phân hóa 2.000.000 điểm), và nén `zlib` (mã C thả GIL). Không cần mạng, không cần hạ tầng, không có thư viện ngoài.

- `workload.py`: ba hàm việc và `fail_on_odd`, đặt ở module riêng để tiến trình con import được (start method `spawn`).
- `main.py`: harness đo, `--verify`.

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/22-gil-threading-multiprocessing examples/toi-noi-python/22-gil-threading-multiprocessing/main.py
uv run --project examples/toi-noi-python/22-gil-threading-multiprocessing examples/toi-noi-python/22-gil-threading-multiprocessing/main.py --verify
```

Bản free-threaded (cần `uv python install 3.14t`; lab chỉ dùng thư viện chuẩn nên chạy được bằng `--no-project`):

```bash
uv run --no-project --python 3.14t examples/toi-noi-python/22-gil-threading-multiprocessing/main.py
```

## Kết quả kỳ vọng

- Lệnh đầu in môi trường (máy, loại lõi P/E, phiên bản, GIL, start method, load average) rồi ba bảng thời gian; mỗi hàng đo có thêm load average trước và sau. Số giây và tỉ lệ phụ thuộc máy và tải máy lúc chạy, nên chỉ so sánh khi load thấp và chỉ tin thứ bậc lớn: thread thắng rõ ở I/O và zlib, thread không thắng ở vòng lặp Python trên bản có GIL, process (pool sẵn) thắng ở vòng lặp Python. Máy có lõi E thì không bao giờ đạt gấp đúng số lõi.
- `--verify` kết thúc bằng `OK 10/10` (exit code 0); nếu lệch thì in `SAI: ...` rồi `FAIL n/10` với n là số ca đạt. Nó chỉ kiểm kết quả đúng của mọi mô hình so với bản tuần tự (cả `binarize` lẫn `zlib`), `PicklingError` với lambda, ngoại lệ từ worker và trạng thái GIL; không kiểm tốc độ nào vì chúng đổi theo tải máy.
