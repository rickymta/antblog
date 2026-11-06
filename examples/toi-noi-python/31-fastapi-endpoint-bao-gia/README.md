# Lab 31 - HTTP và FastAPI: xuất bản endpoint báo giá

Bài viết: `api/seed/content/python-fastapi-http-endpoint-bao-gia.md`. Lab tự chứa, không cần hạ tầng; lần resolve đầu cần mạng để tải `fastapi`, `uvicorn`, `httpx`.

- `app/main.py`: `create_app()` với `POST /v1/quotes` (201 + `Location`, 400 khi ngoài miền 0..100 km) và `GET /v1/quotes/{quote_id}`. Báo giá lưu trong dict của tiến trình (`app.state`), mất khi restart.
- `app/schemas.py`: `QuoteRequest` (pydantic v2, chỉ kiểm hình dạng) và `QuoteResponse` (Decimal ra JSON là chuỗi).
- `app/domain.py`: `calculate_fare`, `Quote`, `Money` (copy rút gọn từ bài 13); không biết gì về HTTP.
- `app/raw.py`: một hàm WSGI và một hàm ASGI viết tay, để thấy FastAPI đứng trên giao diện nào.
- `workers-demo.sh`: chạy khi server mở với `--workers 2`; 12 POST song song cho thấy hai worker cấp trùng `QT-0001`, và GET trả lúc báo giá này lúc báo giá kia.
- `main.py`: các demo in trong bài và chế độ `--verify`; gọi app bằng `httpx` + `ASGITransport` ngay trong tiến trình, không cần server.

Phiên bản đã chạy: Python 3.14.7, FastAPI 0.162.0, Starlette 1.10.0, pydantic 2.14.3, uvicorn 0.54.0, httpx 0.28.1 (xem `uv.lock`).

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/31-fastapi-endpoint-bao-gia examples/toi-noi-python/31-fastapi-endpoint-bao-gia/main.py
uv run --project examples/toi-noi-python/31-fastapi-endpoint-bao-gia examples/toi-noi-python/31-fastapi-endpoint-bao-gia/main.py --verify
```

Server thật (cổng 8031), rồi gọi bằng `curl` từ terminal khác; dừng bằng Ctrl+C:

```bash
uv run --project examples/toi-noi-python/31-fastapi-endpoint-bao-gia uvicorn app.main:app --app-dir examples/toi-noi-python/31-fastapi-endpoint-bao-gia --port 8031
curl -i -X POST http://127.0.0.1:8031/v1/quotes -H "Content-Type: application/json" -d '{"distance_km": "2.5", "service": "ride"}'
```

Với hai worker (để thấy mã báo giá bị trùng), thêm `--workers 2` vào lệnh uvicorn rồi chạy `examples/toi-noi-python/31-fastapi-endpoint-bao-gia/workers-demo.sh`.

Trang tài liệu tương tác ở `http://127.0.0.1:8031/docs` (trang này nạp giao diện Swagger UI từ CDN nên cần mạng), đặc tả ở `/openapi.json`.

## Kết quả kỳ vọng

- Lệnh đầu in năm nhóm: WSGI/ASGI trần, báo giá 0/2/2,5/5/100 km ra 12000/12000/14250/25500/453000, bốn loại lỗi (400, 422, 404, 405), tóm tắt OpenAPI, và phép đo 5 request đồng thời với `async def` chặn, `async def` + `await`, `def` thường (có in `load average` lúc đo).
- `--verify` kết thúc bằng `OK 39/39` (exit code 0). Nếu lệch: in `SAI: <tên ca>`, rồi `FAIL n/m` và exit 1.
- Số giây ở phần cuối phụ thuộc máy; `--verify` chỉ kiểm bất biến (chặn thì tuần tự >= 1,0 s, không chặn thì nhanh hơn chặn ít nhất 2 lần), không kiểm số tuyệt đối.
