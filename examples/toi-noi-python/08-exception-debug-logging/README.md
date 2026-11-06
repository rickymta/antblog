# Lab 08: exception, debug và logging

Bài viết: `api/seed/content/python-exception-debug-logging.md`.

`fare.py` chứa exception tùy biến (`FareError`, `DistanceOutOfRangeError`, `DeviceDataError`), `quote_from_metres()` có lỗi đổi đơn vị cố ý (bật bằng `--buggy`), `handle_request()` minh họa `try/except/else/finally` và `logging`. `main.py` là điểm vào: cấu hình `logging.basicConfig` đúng một lần và có chế độ `--verify`.

## Chạy

```bash
L=examples/toi-noi-python/08-exception-debug-logging
uv run --project $L $L/main.py                      # 3 ca: 2500, -100, abc
uv run --project $L $L/main.py 2500 --buggy         # báo giá sai nhưng không có lỗi
uv run --project $L $L/main.py -100 --unhandled     # traceback thô
uv run --project $L $L/main.py 2500 --buggy --pdb   # dừng ở breakpoint() trong quote_from_metres()
uv run --project $L $L/main.py 2500 --log-level DEBUG
```

## Verify

```bash
uv run --project $L $L/main.py --verify
```

Kết quả kỳ vọng: dòng cuối `OK 21/21`, exit code 0. Có ca sai thì in tên ca, dòng `FAIL n/21` và exit code 1.
