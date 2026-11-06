# Lab 35 · Upload ảnh giấy tờ, BackgroundTasks và SSE

Đi kèm bài `python-fastapi-upload-background-task-streaming`. Service FastAPI nhận ảnh giấy tờ tài xế (thẻ giả sinh bằng Pillow), kiểm ở ba lớp, lưu vào «object storage» giả (thư mục), xử lý nền bằng `BackgroundTasks` và báo tiến độ qua Server-Sent Events.

- `app/main.py`: `create_app(settings)`, route upload, job, SSE (viết tay bằng `StreamingResponse` và bản `EventSourceResponse` có sẵn của FastAPI), middleware chặn sớm theo `Content-Length`.
- `app/images.py`: kiểm ảnh bằng Pillow (định dạng thật, số điểm ảnh, cấu trúc), làm ảnh nhỏ, đo độ nét. `app/errors.py`: các lỗi có kiểu.
- `app/jobs.py`: `Job`, `JobRegistry` (dict trong RAM, mã job ngẫu nhiên), `run_job`, luồng sự kiện.
- `app/problems.py`: Problem Details cùng dạng với lab 32 (404/405 `about:blank`, 422 `errors[{field, code, message}]`, 500 `internal-error`). `app/storage.py`, `app/settings.py`: kho đối tượng giả, cấu hình (`TOINOI_UPLOAD_DIR`, `TOINOI_MAX_UPLOAD_BYTES`...). Giới hạn mặc định: 10 MiB và 50 triệu điểm ảnh mỗi ảnh; JPEG, MPO, PNG (không có HEIC: client đổi sang JPEG trước khi gửi).
- `main.py`: demo, `--verify`, `--samples`. `synthetic_card.py`: thẻ giả, copy từ lab 30 (số thẻ `099...`, chân thẻ ghi «KHÔNG PHẢI GIẤY TỜ THẬT»).
- Không cần Docker, Kafka hay cơ sở dữ liệu. Ảnh nằm trong `out/` (đã gitignore).

## Chạy

Từ thư mục gốc repository (lần đầu uv tải gói nên cần mạng):

```bash
uv run --project examples/toi-noi-python/35-fastapi-upload-background-streaming examples/toi-noi-python/35-fastapi-upload-background-streaming/main.py
uv run --project examples/toi-noi-python/35-fastapi-upload-background-streaming examples/toi-noi-python/35-fastapi-upload-background-streaming/main.py --verify
uv run --project examples/toi-noi-python/35-fastapi-upload-background-streaming examples/toi-noi-python/35-fastapi-upload-background-streaming/main.py --samples
```

`--samples` ghi `out/card.jpg` (thẻ giả), `out/ghi-chu.jpg` (file chữ đội lốt ảnh) và `out/bomb.png` (PNG 62 KB nhưng 64 triệu điểm ảnh) để thử bằng curl. Chạy service (cổng 8035; tài liệu tại `http://localhost:8035/docs`):

```bash
uv run --project examples/toi-noi-python/35-fastapi-upload-background-streaming uvicorn app.main:app --app-dir examples/toi-noi-python/35-fastapi-upload-background-streaming --port 8035
curl -si -F driver_id=DR-001 -F kind=id_card -F "file=@examples/toi-noi-python/35-fastapi-upload-background-streaming/out/card.jpg;type=image/jpeg" localhost:8035/v1/kyc/documents
curl -sN localhost:8035/v1/kyc/jobs/<job_id>/events
```

Đo hiện tượng «trạng thái job nằm trong RAM của từng tiến trình»: chạy uvicorn với `--workers 2`, upload một lần, rồi gọi lại job 20 lần (số lần 200 và 404 thay đổi theo lần chạy và theo tải máy):

```bash
JOB=$(curl -s -F driver_id=DR-001 -F kind=id_card -F "file=@examples/toi-noi-python/35-fastapi-upload-background-streaming/out/card.jpg;type=image/jpeg" localhost:8035/v1/kyc/documents | python3 -c "import sys,json; print(json.load(sys.stdin)['job_id'])")
for i in $(seq 1 20); do curl -s -o /dev/null -w '%{http_code}\n' localhost:8035/v1/kyc/jobs/$JOB; done | sort | uniq -c
```

## Kết quả kỳ vọng

- Lệnh đầu in dòng load average rồi bốn mục: 202 kèm `Location`, luồng SSE 5 sự kiện (`stored`, `read`, `thumbnail`, `quality`, `done`), trạng thái job, năm ca bị từ chối (415, 400, 400, 400, 413). Độ nét ảnh mẫu `3061.5`, khóa object `kyc/DR-001/12ae24e6e6b1f448.jpg` ổn định giữa các lần chạy; mã job ngẫu nhiên nên khác mỗi lần.
- `--verify` kết thúc bằng `OK 42/42` (exit code 0). Nhóm 2 mở uvicorn thật trên cổng ngẫu nhiên; các ca đo thời gian chỉ kiểm bất biến («202 trả về trước khi job xong»), không kiểm số tuyệt đối. Ca Pillow bị lỗi (PNG cắt đôi, sai checksum, header khai 20000x20000) phải ra 400, không ra 500.
- Đã chạy với fastapi 0.162.0, starlette 1.10.0, uvicorn 0.54.0, python-multipart 0.0.33, pillow 13.0.0, httpx 0.28.1 trên Python 3.14.7, macOS; lúc chạy máy rất bận (load average khoảng 45).
