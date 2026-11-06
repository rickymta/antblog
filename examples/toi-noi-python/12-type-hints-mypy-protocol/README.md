# Lab 12: type hints, mypy, Protocol và generics

Bài viết: `api/seed/content/python-type-hints-mypy-protocol-generics.md`.

- `domain.py`: thư viện nhỏ có kiểu: alias `type`, `Trip`/`Driver` (`X | None`, tham chiếu tiến không cần dấu nháy), `Registry[T]`, `first[T]`, `cheapest[T: SupportsFare]`, Protocol `SupportsFare`, `TypedDict` cho sự kiện `BookingCreated`.
- `main.py`: chạy thử từng khái niệm; `--verify` còn gọi mypy thật.
- `forward_ref_probe.py`: tham chiếu tiến không dấu nháy, so 3.14 với 3.13.
- `broken.py`: 9 lỗi kiểu cố ý (dòng kết thúc bằng chú thích «E:») để mypy bắt. Không chạy file này.

Cần Python 3.14 và uv; lần đầu uv tải mypy nên cần mạng. Lab đã chạy với mypy 2.4.1, CPython 3.14.7.

```bash
L=examples/toi-noi-python/12-type-hints-mypy-protocol
uv run --project $L $L/main.py
uv run --project $L $L/main.py --verify
uv run --project $L mypy $L/main.py --strict      # kỳ vọng: Success: no issues found in 1 source file
uv run --project $L mypy $L/broken.py --strict    # kỳ vọng: Found 9 errors in 1 file
```

Kết quả kỳ vọng của `--verify`: dòng cuối `OK 21/21`, exit code 0. Có ca sai thì in tên ca, `FAIL n/21` (n là số ca đạt) và exit code 1.

Thử riêng tham chiếu tiến không dấu nháy (không nằm trong `--verify`; cần cả hai phiên bản Python):

```bash
uv run --python 3.14 --no-project python $L/forward_ref_probe.py   # chạy xong
uv run --python 3.13 --no-project python $L/forward_ref_probe.py   # (cần mạng nếu chưa có Python 3.13) NameError: name 'Driver' is not defined
```
