# Lab 10 - Unit test với pytest: kiểm thử chính sách cước

Bài viết: `api/seed/content/python-pytest-kiem-thu-chinh-sach-cuoc.md`.

- `fare.py`: `calculate_fare(distance_km, policy, *, service)` với `FarePolicy` (giá cơ bản 12.000, 2 km đầu, 4.500 VND mỗi km vượt, `ROUND_HALF_UP`), miền 0..100 km.
- `tests/conftest.py`: fixture `standard_policy` (scope `session`) và `policy_5000`.
- `tests/test_fare.py`: giá trị biên (bảng ca của bài 03) bằng `@pytest.mark.parametrize`, `pytest.raises` cho ngoài miền/sai kiểu, chính sách thay thế.

Chạy từ thư mục gốc repository (lần đầu uv tải pytest nên cần mạng):

```bash
uv run --project examples/toi-noi-python/10-pytest-chinh-sach-cuoc pytest examples/toi-noi-python/10-pytest-chinh-sach-cuoc -q
```

Kết quả kỳ vọng: `36 passed` và exit code 0.
