# Bài 17 · Decorator và context manager

Lab của bài `python-decorator-context-manager`: closure, `functools.wraps`, decorator có tham số (`retry`, `timed`), thứ tự chồng decorator, `@contextmanager` cho transaction giả (commit/rollback trên dict), context manager dạng class (`DriverLock`), `ExitStack` và `suppress`.

```bash
uv run --project examples/toi-noi-python/17-decorator-context-manager examples/toi-noi-python/17-decorator-context-manager/main.py
uv run --project examples/toi-noi-python/17-decorator-context-manager examples/toi-noi-python/17-decorator-context-manager/main.py --verify
```

- Lệnh đầu in từng nhóm kết quả (closure, wraps, retry, thứ tự decorator, transaction, lock/ExitStack, thứ tự thoát). Chạy dưới một giây vì mọi thời gian chờ đi qua `FakeTimer`, không ngủ thật.
- Kết quả cố định, không phụ thuộc máy; không có số đo thời gian thật.
- `--verify` phải in `OK 21/21` và thoát mã 0; sai thì in tên ca sai, `FAIL n/m` và thoát mã 1.

Python 3.14 (cú pháp generic `def f[**P, R]` từ 3.12), không cần mạng, không có thư viện ngoài.
