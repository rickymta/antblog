---
slug: python-fastapi-upload-background-task-streaming
title: "Upload ảnh giấy tờ, background task và streaming"
summary: "Bài 35: nhận ảnh CCCD/GPLX bằng UploadFile qua ba lớp kiểm, lưu object storage giả, xử lý nền bằng BackgroundTasks (và khi nào cần worker), báo tiến độ qua SSE. Lab chạy cả uvicorn thật."
category: programming
level: intermediate
tags: [python, toi-noi, fastapi, upload, sse]
language: vi
---

Tài xế mới của [Tới Nơi](/articles/du-an-mau-toi-noi) đăng ký bằng điện thoại: chụp mặt trước thẻ căn cước và bằng lái, bấm «Gửi». Mỗi ảnh vài trăm KB đến vài MB, đi qua mạng di động chập chờn. Phía sau, `kyc-ocr` cần ảnh gốc, còn màn hình chờ cần ảnh nhỏ và một câu trả lời sớm cho người dùng: «đã nhận, đang xử lý, xong 40%». Nhận byte phụ thuộc mạng, xử lý ảnh phụ thuộc CPU, báo tiến độ phụ thuộc người đang nhìn màn hình: ba tốc độ khác nhau. Nhồi hết vào một request chạy 10 giây thì điện thoại báo timeout trước khi service kịp trả lời.

Đây là bài 35 của lộ trình **Python trung bình qua Tới Nơi**. Bài dùng lại `Depends` ở [bài 33](/articles/python-fastapi-dependency-middleware-settings-logging), Problem Details ở [bài 32](/articles/python-fastapi-validation-problem-details-openapi), Pillow ở [bài 30](/articles/python-numpy-pillow-opencv-tien-xu-ly-anh) và `asyncio` ở [bài 21](/articles/python-asyncio-coroutine-task-timeout-cancellation). Ảnh giấy tờ là thẻ giả sinh bằng Pillow, không có dữ liệu thật.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Endpoint upload chỉ nhận ảnh tối đa 10 MiB và kiểm bằng <code>len(data) &gt; 10 * 1024 * 1024</code> ngay trong handler. Một client gửi 50 MB. Kiểm tra đó có ngăn 50 MB ghi vào đĩa của server không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Starlette đọc và ghi toàn bộ phần file của <code>multipart/form-data</code> vào <code>SpooledTemporaryFile</code> (RAM tới 1 MiB, quá thì xuống file tạm) <i>trước khi</i> handler chạy. Khi dòng kiểm tra chạy thì 50 MB đã nằm trên đĩa. Muốn chặn sớm phải nhìn <code>Content-Length</code> ở lớp phía trước, và đặt giới hạn thật ở reverse proxy.</p></details>
</div>

## Một upload đi qua những lớp nào

Với `file: UploadFile`, FastAPI cần `python-multipart` để phân tích `multipart/form-data`; thiếu gói này, app báo `RuntimeError: Form data requires "python-multipart" to be installed.` ngay lúc khai báo route. Phân tích xong, handler nhận một `UploadFile`: đối tượng giống file có `filename`, `content_type`, `size`, hàm `read` bất đồng bộ, bên dưới là file tạm.

Điểm dễ hiểu nhầm là thứ tự: Starlette đọc cả body, ghi phần file vào `SpooledTemporaryFile` (ngưỡng RAM 1 MiB) rồi mới gọi code của ta. Trong mã nguồn starlette 1.10, `max_part_size` chỉ giới hạn các trường chữ của form; phần file không có trần. Vì vậy ta chồng ba lớp, mỗi lớp trả lời một câu hỏi khác nhau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Năm khối nối tiếp: lớp 1 là middleware đọc Content-Length và từ chối sớm; Starlette phân tích multipart vào file tạm; lớp 2 kiểm file.size hoặc đọc tối đa limit cộng một byte; lớp 3 để Pillow kiểm định dạng và số điểm ảnh; cuối cùng lưu object và tạo job. Chú thích cho biết chỉ lớp 1 chặn trước khi body vào đĩa."><defs><marker id="py35-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="15" y="30" width="150" height="120" rx="10" class="f-pk-soft s-pk"/><text x="29" y="56" class="t-b">Lớp 1</text><text x="29" y="82" class="t-small">Content-Length</text><text x="29" y="100" class="t-small">quá lớn: 413 sớm</text><text x="29" y="118" class="t-small">(chưa đọc body)</text><path d="M165 90 L190 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py35-0-arrow)"/><rect x="190" y="30" width="150" height="120" rx="10" class="f-warn-soft s-warn"/><text x="204" y="56" class="t-b">Starlette</text><text x="204" y="82" class="t-small">phân tích multipart</text><text x="204" y="100" class="t-small">RAM tới 1 MiB,</text><text x="204" y="118" class="t-small">quá thì file tạm</text><path d="M340 90 L365 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py35-0-arrow)"/><rect x="365" y="30" width="150" height="120" rx="10" class="f-pk-soft s-pk"/><text x="379" y="56" class="t-b">Lớp 2</text><text x="379" y="82" class="t-small">file.size, hoặc đọc</text><text x="379" y="100" class="t-small">tối đa limit + 1 byte</text><text x="379" y="118" class="t-small">thừa: 413</text><path d="M515 90 L540 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py35-0-arrow)"/><rect x="540" y="30" width="150" height="120" rx="10" class="f-pk-soft s-pk"/><text x="554" y="56" class="t-b">Lớp 3</text><text x="554" y="82" class="t-small">Pillow đọc header:</text><text x="554" y="100" class="t-small">định dạng, điểm ảnh</text><text x="554" y="118" class="t-small">sai: 415 hoặc 400</text><path d="M690 90 L715 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py35-0-arrow)"/><rect x="715" y="30" width="150" height="120" rx="10" class="f-phys-soft s-phys"/><text x="729" y="56" class="t-b">Lưu và xếp job</text><text x="729" y="82" class="t-small">put theo sha256</text><text x="729" y="100" class="t-small">tạo job, trả 202</text><text x="729" y="118" class="t-small">việc nặng chạy sau</text><rect x="15" y="185" width="850" height="80" rx="10" class="f-bg s-warn dash"/><text x="31" y="212" class="t-b">Chỉ lớp 1 chặn trước khi body vào đĩa</text><text x="31" y="236" class="t-small">Client gửi chunked (không có Content-Length) thì lớp 1 không thấy gì; lớp 2 mới là chốt cuối trong ứng dụng.</text><text x="31" y="254" class="t-small">Giới hạn thật nằm ở reverse proxy (ví dụ client_max_body_size của nginx), trước cả uvicorn.</text></svg>
</div>
<figcaption>Hình 1. Ba lớp kiểm của một upload. Starlette đứng giữa và đã nhận đủ body khi lớp 2 và lớp 3 chạy.</figcaption>
</figure>

Lớp 1 là middleware đọc `Content-Length` rồi từ chối ngay, rẻ nhưng chỉ ngăn được client ngay thẳng: header có thể vắng (chunked) hoặc nói dối. Lớp 2 đếm byte thật của phần file. Lớp 3 xem nội dung có thật là ảnh không.

## UploadFile và ba lớp kiểm

Lớp 1 trong `app/main.py` (trích); ngưỡng là giới hạn ảnh cộng 64 KiB «phần thừa» của multipart:

```python
@app.middleware("http")
async def reject_oversized_body(request: Request, call_next):
    if request.method == "POST" and request.url.path == UPLOAD_PATH:
        declared = request.headers.get("content-length", "")
        if declared.isdigit() and int(declared) > settings.max_upload_bytes + MULTIPART_OVERHEAD:
            exc = UploadTooLarge(settings.max_upload_bytes, declared)
            return problem_response(request, status=413, type_=problems.TYPE_BASE + "upload-too-large",
                                    title="Ảnh quá lớn", detail=f"Body khai báo {declared} byte, ...")
    return await call_next(request)
```

Lớp 2 là `read_limited` (`app/main.py`, trích). Starlette luôn đặt `file.size`, nên nhánh đầu là nhánh chạy thật; nhánh đọc `limit + 1` byte chỉ là chốt dự phòng khi `size` là `None` (đối tượng dựng tay).

```python
async def read_limited(file: UploadFile, limit: int) -> bytes:
    if file.size is not None and file.size > limit:
        raise UploadTooLarge(limit)
    data = await file.read(limit + 1)
    if len(data) > limit:
        raise UploadTooLarge(limit)
    return data
```

Lớp 3 là `inspect_image` trong `app/images.py`. `Content-Type` và `filename` do client tự khai, đáng tin ngang một lời nói; thứ đáng tin là byte, nên ta để Pillow đọc phần đầu file và tự báo định dạng (trích):

```python
ALLOWED_FORMATS = {"JPEG": "jpg", "MPO": "jpg", "PNG": "png"}

def inspect_image(data: bytes, max_pixels: int) -> ImageInfo:
    try:
        with Image.open(io.BytesIO(data)) as img:
            fmt, (width, height) = img.format, img.size
            if fmt not in ALLOWED_FORMATS:
                raise NotAnImage(f"định dạng {fmt} không được chấp nhận")
            if width * height > max_pixels:  # kiểm TRƯỚC khi giải nén
                raise ImageTooBig(f"{width}x{height} = {width * height} điểm ảnh, tối đa {max_pixels}")
            img.verify()
    except UnidentifiedImageError as exc:  # đứng trước OSError vì là lớp con của nó
        raise NotAnImage("nội dung không phải ảnh") from exc
    except Image.DecompressionBombError as exc:
        raise ImageTooBig("header khai kích thước vượt xa giới hạn của Pillow") from exc
    except (OSError, SyntaxError) as exc:
        raise CorruptImage(str(exc)) from exc
    return ImageInfo(fmt, ALLOWED_FORMATS[fmt], width, height)
```

`Image.open` là lười: nó chỉ đọc header, còn giải nén điểm ảnh xảy ra ở `load()`. Điều đó mở hai cửa mà giới hạn dung lượng không thấy. Cửa thứ nhất là ảnh nhỏ trên đĩa nhưng khổng lồ trong RAM: một PNG 8000 x 8000 một màu nặng 62 KB, giải nén ra 64 triệu điểm ảnh. Trần mặc định của Pillow (`Image.MAX_IMAGE_PIXELS`, khoảng 89 triệu) chỉ cảnh báo ở mức đó và chỉ ném `DecompressionBombError` từ gấp đôi, khoảng 179 triệu. Ta đặt trần riêng 50 triệu điểm ảnh (đủ cho camera 48 MP) và kiểm từ header; một PNG 66 byte khai 20000 x 20000 thì Pillow tự từ chối ngay lúc `open`, nên phải bắt `DecompressionBombError`.

Cửa thứ hai là file hỏng. `verify()` kiểm cấu trúc mà không giải nén và chỉ PNG có checksum: PNG cắt đôi ném `OSError`, PNG sai một byte ném `SyntaxError`, cả hai thành 400 `image-corrupt`. Bản đầu của lab chỉ bắt `UnidentifiedImageError` nên ba ca này ra 500 `text/plain`. Với JPEG, `verify()` gần như không kiểm gì; lỗi lộ ở job.

Khóa lưu không bao giờ dựa vào `filename` mà là `kyc/<mã tài xế>/<16 ký tự đầu của sha256>.<đuôi suy từ định dạng thật>`, nên `../../evil.jpg` chỉ còn là một chuỗi hiển thị lại trong response.

Giới hạn phải khớp với ảnh thật: điện thoại thường chụp JPEG 12 MP (4032 x 3024, 1 đến 5 MB), có máy ghi MPO (JPEG nhiều khung); mức 10 MiB, 50 triệu điểm ảnh và danh sách `JPEG`, `MPO`, `PNG` cho chúng đi qua. HEIC của iPhone không có sẵn trong Pillow, client phải đổi sang JPEG. Giải nén ảnh 50 MP tốn khoảng 150 MB RAM mỗi job, nên số job song song cần giới hạn (bài 40). Mã tài xế và loại giấy tờ đi qua `Form(...)` với `pattern=r"^DR-\d{3}$"` và một `StrEnum`; sai thì 422 dạng Problem Details của bài 32, không phản chiếu giá trị đã gửi.

## Lưu vào kho và trả 202

«Object storage» của lab là một thư mục sau giao diện hai hàm `put` và `get` (một `Protocol`, bài 14), để thay bằng S3 hay GCS mà không đụng handler. Hai chi tiết đáng giữ: ghi qua file tạm rồi `os.replace` để không ai thấy file ghi dở, và kiểm đường dẫn cuối nằm trong thư mục gốc (`is_relative_to`) phòng khi một khóa xấu lọt qua.

Khóa suy từ nội dung nên tải cùng ảnh hai lần ra cùng khóa và ghi lại là vô hại, ý tưởng idempotency của bài 25; lab kiểm: ảnh gửi lại với tên khác ra cùng khóa, nhưng tạo job mới.

Phần cuối của handler là chỗ phân biệt «nhận» với «xử lý» (trích):

```python
key = object_key(driver_id, data, info.ext)
await asyncio.to_thread(store.put, key, data)

job = jobs.create(driver_id, key)
await job.publish("stored", JobStatus.QUEUED, 0, "đã lưu ảnh, xếp hàng xử lý")
background.add_task(run_job, job, store, settings.stage_delay)

response.headers["Location"] = f"/v1/kyc/jobs/{job.job_id}"
return UploadAccepted(job_id=job.job_id, status=job.status, document=DocumentOut(...))
```

Status là `202 Accepted`, không phải 201: kết quả xử lý chưa tồn tại, chỉ có lời hứa và địa chỉ để hỏi (`Location`). `asyncio.to_thread` đẩy việc ghi đĩa đồng bộ sang thread để vòng lặp sự kiện không bị chặn (bài 21, 22).

## BackgroundTasks: chạy sau khi trả lời

`BackgroundTasks` của FastAPI chạy hàm ta thêm vào *sau khi* response đã gửi, trong cùng tiến trình: hàm `async` chạy trên vòng lặp sự kiện, hàm thường chạy trong thread pool. Hai quy tắc: việc nặng CPU không đặt thẳng trong hàm `async` vì nó chặn mọi request khác (`run_job` gọi `asyncio.to_thread(process_image, data)`); và không ai bắt ngoại lệ hộ, vì client đã nhận 202, nên `run_job` tự bắt `Exception`, ghi log và đặt trạng thái `failed` (`app/jobs.py`, trích):

```python
async def run_job(job: Job, store: ObjectStore, stage_delay: float) -> None:
    try:
        await job.publish("read", JobStatus.RUNNING, 10, "đọc ảnh từ kho")
        data = await asyncio.to_thread(store.get, job.document_key)
        await job.publish("thumbnail", JobStatus.RUNNING, 40, "tạo ảnh nhỏ và đo độ nét")
        processed = await asyncio.to_thread(process_image, data)
        ...
        await job.publish("done", JobStatus.DONE, 100, "xong; kết quả chờ người duyệt")
    except Exception as exc:
        log.exception("job %s lỗi", job.job_id)  # chi tiết (loại lỗi, stack) chỉ vào log
        await job.publish("failed", JobStatus.FAILED, 100, "Không xử lý được ảnh; hãy gửi lại ảnh khác.")
```

Ca này có thật: JPEG cắt đôi vẫn qua cửa, và `OSError: image file is truncated` chỉ lộ ở `load()` trong job. Sự kiện `failed` gửi client chỉ là một câu chung, loại lỗi và stack nằm trong log (nguyên tắc của bài 32); lab kiểm cả hai phía.

`BackgroundTasks` là tiện ích, không phải hàng đợi, và đó là điều phân biệt nó với worker.

| | BackgroundTasks | Worker và hàng đợi (bài 40) |
| --- | --- | --- |
| Sống ở đâu | Cùng tiến trình với API | Tiến trình riêng, đọc hàng đợi |
| Tiến trình chết giữa chừng | Việc và trạng thái mất | Việc còn trong hàng đợi |
| Nhiều bản API (`--workers`, nhiều pod) | Mỗi tiến trình thấy dict job của riêng mình | Trạng thái nằm ở kho chung |
| Hợp với | Việc ngắn, mất cũng không sao | Việc đắt hoặc phải chạy chắc chắn, cần retry: OCR, face match |

Dòng áp chót chạy thật trong lab bằng hai cách. Trong `--verify`, hai app riêng đóng vai hai tiến trình: job tạo ở app A thì app B trả 404. Với uvicorn thật `--workers 2`, upload một lần rồi gọi lại job 20 lần bằng vòng lặp `curl` trong README: một lần đo trên macOS lúc máy bận (load average khoảng 45) cho 5 lần `200` và 15 lần `404`; con số đổi theo từng lần chạy vì hệ điều hành tự chia kết nối. Lỗi này không lộ khi chạy thử một tiến trình, đúng loại lỗi chỉ xuất hiện khi triển khai.

Mã job cũng liên quan. Bản đầu của lab đánh số `JOB-0001`...; mỗi tiến trình có bộ đếm riêng nên hai worker phát trùng mã và một `GET` có thể đọc nhầm job của tài xế khác. Lab giờ dùng mã ngẫu nhiên (`JOB-` cộng 16 ký tự hex) và `--verify` kiểm hai tiến trình không phát trùng. Trạng thái chung cần kho chung và worker riêng, vì vậy OCR thật (bài 46) tách API khỏi worker. Bên .NET, cùng khái niệm là [BackgroundService và Channel](/articles/dotnet10-backgroundservice-channel).

## SSE: báo tiến độ từng bước

Client cần thấy tiến độ: hỏi lặp, WebSocket, hoặc **Server-Sent Events** (SSE). Với luồng một chiều như tiến độ job, SSE đơn giản nhất: HTTP thường, `GET` giữ kết nối mở, server ghi dần các khối `text/event-stream`; trình duyệt có sẵn `EventSource` và tự nối lại khi đứt.

Định dạng trên dây là các dòng `id:`, `event:`, `data:`, kết thúc một sự kiện bằng **một dòng trống**; dòng bắt đầu bằng `:` là chú thích. `JobEvent.to_sse` trong `app/jobs.py` viết đúng thế:

```python
def to_sse(self) -> str:
    name = self.status.value if self.status in TERMINAL else "progress"
    return f"id: {self.id}\nevent: {name}\ndata: {json.dumps(asdict(self), ensure_ascii=False)}\n\n"
```

Hai ý khiến luồng dùng được ngoài đời. Mỗi job giữ danh sách sự kiện có `id` tăng dần, nên client nối lại với `Last-Event-ID: 2` nhận tiếp từ sự kiện 3, không mất và không lặp. Khi chưa có gì mới, server gửi `: ping` sau mỗi 15 giây để proxy không cắt kết nối vì im lặng. Vòng lặp phát (`app/jobs.py`):

```python
async def event_stream(job: Job, last_id: int, heartbeat: float) -> AsyncIterator[str]:
    while True:
        events = await job.events_after(last_id, heartbeat)  # chờ trên asyncio.Condition
        if not events:
            if job.finished:
                return  # job xong và không còn gì để phát: đóng, không treo
            yield ": ping\n\n"
            continue
        for event in events:
            yield event.to_sse()
            last_id = event.id
        if events[-1].status in TERMINAL:
            return
```

Dòng `if job.finished: return` sửa một lỗi nặng của bản đầu. `EventSource` tự nối lại mỗi khi luồng đóng, nên client vừa nhận `done` nối lại với `Last-Event-ID: 4`; bản đầu chờ một sự kiện thứ 5 không bao giờ tới và treo vĩnh viễn (`Last-Event-ID: 99` cũng vậy). Bản sửa: `events_after` thức dậy khi job kết thúc; luồng đóng khi không còn gì để phát; endpoint trả `204 No Content` nếu job đã xong và client đã nhận hết, vì theo đặc tả SSE, 204 báo `EventSource` ngừng nối lại. Client nên `close()` ngay khi nhận `done`. `Last-Event-ID: -3` bị `Header(ge=0)` từ chối bằng 422.

Endpoint bọc vòng lặp trong `StreamingResponse(..., media_type="text/event-stream", headers={"Cache-Control": "no-cache", "X-Accel-Buffering": "no"})`; header thứ hai báo nginx đừng đệm luồng. FastAPI 0.162 của lab còn có sẵn `EventSourceResponse` và `ServerSentEvent`: một hàm sinh `yield ServerSentEvent(data=event, event=..., id=...)` là đủ, framework tự định dạng và chèn ping. Lab có cả hai endpoint (`/events` viết tay, `/events-native`) và `--verify` kiểm chúng phát cùng chuỗi bước. Bản native không trả được 204: nối lại sau khi xong cho ra luồng rỗng rồi đóng. FastAPI cũ chưa có lớp này thì bản viết tay vẫn chạy.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Ba làn thời gian. Làn handler: nhận ảnh, kiểm, lưu, tạo job rồi trả 202. Làn job nền: bốn bước read, thumbnail, quality, done chạy sau khi trả 202. Làn client SSE: mở luồng sau 202, nhận sự kiện id 0 đến 4, và nếu client ngắt giữa chừng thì job vẫn chạy tới done."><defs><marker id="py35-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="62" class="t-b">Handler</text><rect x="150" y="38" width="150" height="40" rx="8" class="f-pk-soft s-pk"/><text x="162" y="62" class="t-small">kiểm, lưu, tạo job</text><path d="M300 62 L346 106" class="f-none s-phys" stroke-width="2" marker-end="url(#py35-1-arrow)"/><path d="M290 78 L302 190" class="f-none s-phys dash" stroke-width="2" marker-end="url(#py35-1-arrow)"/><text x="306" y="52" class="t-b">202</text><text x="20" y="132" class="t-b">Job nền</text><rect x="340" y="108" width="118" height="40" rx="8" class="f-phys-soft s-phys"/><text x="352" y="132" class="t-small">read 10%</text><rect x="465" y="108" width="118" height="40" rx="8" class="f-phys-soft s-phys"/><text x="477" y="132" class="t-small">thumbnail 40%</text><rect x="590" y="108" width="118" height="40" rx="8" class="f-phys-soft s-phys"/><text x="602" y="132" class="t-small">quality 80%</text><rect x="715" y="108" width="118" height="40" rx="8" class="f-phys-soft s-phys"/><text x="727" y="132" class="t-small">done 100%</text><text x="20" y="202" class="t-b">Client SSE</text><path d="M300 198 L840 198" class="f-none s-pk" stroke-width="2"/><circle cx="305" cy="198" r="5" class="f-pk"/><circle cx="345" cy="198" r="5" class="f-pk"/><circle cx="470" cy="198" r="5" class="f-pk"/><circle cx="595" cy="198" r="5" class="f-pk"/><circle cx="720" cy="198" r="5" class="f-pk"/><text x="295" y="224" class="t-small">id 0</text><text x="335" y="244" class="t-small">id 1</text><text x="460" y="224" class="t-small">id 2</text><text x="585" y="244" class="t-small">id 3</text><text x="710" y="224" class="t-small">id 4 done</text><text x="312" y="176" class="t-small">client mở luồng</text><path d="M520 150 L520 270" class="f-none s-warn dash" stroke-width="2"/><rect x="530" y="262" width="330" height="30" rx="8" class="f-warn-soft s-warn"/><text x="542" y="282" class="t-small">client ngắt ở đây: job vẫn chạy tới done</text></svg>
</div>
<figcaption>Hình 2. Sau khi trả 202, job nền và luồng SSE chạy độc lập. Luồng chỉ là một cách nhìn vào job; đóng nó không dừng job.</figcaption>
</figure>

Ba giới hạn của SSE. `EventSource` chỉ gọi được `GET` và không đặt được header `Authorization`, nên khi gắn xác thực của bài 34, client dùng `fetch` đọc stream hoặc cookie. Mỗi người xem giữ một kết nối; nhiều người xem một job trên nhiều bản API thì sự kiện phải đi qua kho hoặc kênh pub/sub chung. Và `httpx.ASGITransport` (bài 36) không trả response từng phần, nên kiểm «sự kiện đầu đến trước khi job xong» cần server thật.

## Chạy và kiểm chứng

Mọi lệnh chạy từ gốc repository; lab cần uv, mạng ở lần đầu, không cần Docker. Lệnh sinh ảnh mẫu cho `curl`:

```bash
uv run --project examples/toi-noi-python/35-fastapi-upload-background-streaming examples/toi-noi-python/35-fastapi-upload-background-streaming/main.py --samples
```

```text
out/card.jpg: 73421 byte
out/ghi-chu.jpg: 19 byte
out/bomb.png: 62192 byte
```

Demo (không tham số) khởi động uvicorn thật, tải một thẻ giả lên rồi đọc luồng SSE (trích; mã job ngẫu nhiên nên đổi mỗi lần chạy):

```text
tải máy lúc chạy (load average 1/5/15 phút): 43.58 51.40 47.28
== 1. Upload thẻ giả (uvicorn thật, ảnh JPEG)
202 application/json Location: /v1/kyc/jobs/JOB-da4e73e59fa2422f
{
  "job_id": "JOB-da4e73e59fa2422f",
  "status": "queued",
  "document": {
    "key": "kyc/DR-001/12ae24e6e6b1f448.jpg",
    "kind": "id_card",
    "size": 73421,
    "format": "JPEG",
    "width": 1012,
    "height": 638,
    "original_filename": "cccd.jpg"
  }
}
== 2. Theo dõi tiến độ bằng SSE
  id: 3
  event: progress
  data: {"id": 3, "stage": "quality", "status": "running", "percent": 80, "message": "độ nét 3061.5"}
  
  id: 4
  event: done
  data: {"id": 4, "stage": "done", "status": "done", "percent": 100, "message": "xong; kết quả chờ người duyệt"}
```

Năm ca bị từ chối ở cửa (cùng lần chạy):

```text
== 4. Các ca bị từ chối ở cửa
  file chữ đội lốt .jpg                -> 415 unsupported-image
  PNG 62 KB, 64 triệu điểm ảnh         -> 400 image-too-many-pixels
  PNG vài chục byte khai 20000x20000   -> 400 image-too-many-pixels
  PNG cắt đôi                          -> 400 image-corrupt
  ảnh 15,9 MB                          -> 413 upload-too-large
```

Chạy thành service riêng thì thử bằng `curl` với ảnh từ `--samples` (lệnh nằm trong README của lab). Ca file chữ cho `415` với `application/problem+json`, ca ảnh thật cho `202` kèm `location` (trích, bỏ các header `date`, `server`, `content-length`):

```text
HTTP/1.1 202 Accepted
content-type: application/json
location: /v1/kyc/jobs/JOB-36e4dae2b4794c27

HTTP/1.1 415 Unsupported Media Type
content-type: application/problem+json
{"type":"https://toinoi.example/problems/unsupported-image","title":"Không phải ảnh JPEG hoặc PNG","status":415,"detail":"nội dung không phải ảnh","instance":"/v1/kyc/documents"}
```

Kiểm chứng đầy đủ là chế độ `--verify`: 42 ca, 36 ca gọi app trong tiến trình qua `httpx.ASGITransport` và 6 ca trên uvicorn thật:

```bash
uv run --project examples/toi-noi-python/35-fastapi-upload-background-streaming examples/toi-noi-python/35-fastapi-upload-background-streaming/main.py --verify
```

Output là trích (lược các dòng giữa); dòng đầu là tải máy lúc đo:

```text
tải máy lúc chạy (load average 1/5/15 phút): 62.15 51.15 49.31
Nhóm 1: gọi app trong tiến trình (ASGITransport)
  đạt  upload JPEG hợp lệ: 202, mã job ngẫu nhiên, có Location
  đạt  ASGITransport chờ cả BackgroundTasks: POST trả sau >= 3 bước job
  ...
  đạt  nối lại với Last-Event-ID: 4 sau khi job xong: 204, không treo
  đạt  nối lại với Last-Event-ID: 99 sau khi job xong: 204, không treo
  đạt  Last-Event-ID: -3 bị từ chối 422 problem+json
  ...
  đạt  PNG vài chục byte khai 20000x20000 (Pillow DecompressionBombError): 400, không 500
  đạt  PNG cắt đôi: 400 image-corrupt, không 500
  đạt  PNG sai checksum: 400 image-corrupt, không 500
  ...
  đạt  chunked, không Content-Length: lớp 1 bỏ qua, lớp 2 trả 413
  đạt  JPEG 4032x3024 như ảnh điện thoại: 202 và job done
  đạt  JPEG nhiều khung (MPO): 202, đuôi jpg, job done
  đạt  ...client chỉ nhận câu chung; chi tiết (truncated, OSError) nằm trong log
  đạt  sai phương thức: 405 about:blank, giữ header Allow
  đạt  mã job ngẫu nhiên: hai tiến trình không phát trùng mã
Nhóm 2: uvicorn thật trên cổng ngẫu nhiên
  đạt  uvicorn thật: POST trả 202 ngay, không chờ job (dưới một bước)
  đạt  SSE thật đến từng phần: sự kiện đầu tới khi job chưa xong
  đạt  client ngắt SSE giữa chừng: job vẫn chạy xong
  đạt  Last-Event-ID: 99 trên job đang chạy: luồng tự đóng khi job xong, không treo
  đạt  ảnh 15,9 MB qua socket thật: nhận 413, không đứt kết nối
OK 42/42
```

Ca thứ ba của nhóm 1 đáng chú ý: trong `ASGITransport`, `POST` trả về chỉ sau khi cả background task chạy xong, vì transport chờ app hoàn tất lời gọi ASGI và background task nằm trong đó. Ngay sau `POST` ta đã thấy job `done`, không bao giờ thấy `queued`; chỉ uvicorn thật cho thấy 202 về trước khi job xong (bài 36 dùng lại điều này). Các ca đo thời gian chỉ kiểm bất biến, không kiểm số tuyệt đối.

Phá có chủ ý để biết ca có thể đỏ. Đổi thân `read_limited` thành `return await file.read()`: lớp 1 vẫn bắt client có `Content-Length`, nhưng ca chunked lọt qua và handler nhận đủ 15.894.937 byte rồi mới lưu, đúng đáp án của câu hỏi đầu bài (trích):

```text
  SAI  chunked, không Content-Length: lớp 1 bỏ qua, lớp 2 trả 413  [{"job_id":"JOB-6284266edf42423d","status":"queued","document":{"key":"kyc/DR-001/0f197dee3956064b.png","kind":"id_card","size":15894937,...
FAIL 41/42
```

Bỏ các nhánh `except` của Pillow và nhánh trả 204 thì `--verify` báo `FAIL 37/42` (37 ca đạt trên 42): 3 ca Pillow trả 500, 2 ca nối lại trả 200 thay vì 204.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Ảnh hỏng ra 500 thay vì 400 | Bắt `OSError`, `SyntaxError`, `DecompressionBombError` quanh `Image.open` và `verify()` |
| SSE treo sau khi job xong, client nối lại liên tục | Đóng luồng khi job xong và hết sự kiện; trả 204; client `close()` khi nhận `done` |
| `GET` job đôi khi 404 khi chạy nhiều bản | Trạng thái nằm trong RAM từng tiến trình: kho chung và worker |
| Đã trả 413 mà server vẫn ghi file tạm lớn | Handler chạy sau khi Starlette đã spool; chặn ở lớp 1 và ở reverse proxy |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Job nằm trong một <code>dict</code>: khởi động lại là mất hết, nhiều bản API không chia sẻ được. Kho đối tượng là một thư mục, không mã hóa, không quét virus. Không có xác thực (bài 34) hay giới hạn tốc độ, và ai biết mã job đều đọc được trạng thái. <code>MIN_SHARPNESS = 1000</code> chọn theo thẻ giả và phải đo lại trên ảnh thật; chất lượng ảnh chỉ là gợi ý cho người duyệt, không phải quyết định xác minh danh tính. Số liệu chạy trên một máy macOS, Python 3.14.7, fastapi 0.162.0, starlette 1.10.0, uvicorn 0.54.0, pillow 13.0.0, lúc load average khoảng 45 nên thời gian tuyệt đối chỉ để tham khảo.</p>
</div>

**Bài tập 1:** Client gửi một JPEG hợp lệ với tên `cccd.PNG` và `Content-Type: application/octet-stream`. Service xử lý thế nào, có cần sửa gì không?

**Bài tập 2:** Muốn job sống sót qua việc khởi động lại API mà đổi ít nhất, bạn đổi gì, và SSE đọc trạng thái từ đâu?

<details><summary>Xem lời giải</summary><p><b>1.</b> Nhận bình thường: lab trả <code>202</code> với khóa <code>kyc/DR-001/12ae24e6e6b1f448.jpg</code> và <code>format: "JPEG"</code>, vì không dòng code nào dùng <code>filename</code> hay <code>content_type</code> để quyết định; đuôi file đến từ định dạng Pillow đọc từ byte. Không cần sửa. <b>2.</b> Ghi job và sự kiện vào PostgreSQL (bài 24) hoặc Redis (bài 27) thay cho <code>JobRegistry</code>; handler ghi dòng job rồi đưa <code>job_id</code> vào hàng đợi của worker (bài 40) thay cho <code>add_task</code>; worker cập nhật trạng thái ở kho chung. SSE đọc cùng kho: hỏi lặp theo <code>Last-Event-ID</code> mỗi giây, hoặc nghe kênh pub/sub rồi đọc phần còn thiếu.</p></details>

Bài trước: [bài 34](/articles/python-fastapi-jwt-oauth2-authorization). Bài tiếp: [bài 36](/articles/python-fastapi-api-test-httpx).

## Tài liệu gốc

- [FastAPI: Request Files](https://fastapi.tiangolo.com/tutorial/request-files/).
- [FastAPI: Background Tasks](https://fastapi.tiangolo.com/tutorial/background-tasks/).
- [FastAPI: Server-Sent Events](https://fastapi.tiangolo.com/tutorial/server-sent-events/).
- [WHATWG HTML: Server-sent events](https://html.spec.whatwg.org/multipage/server-sent-events.html).
- [Pillow: Image module (open, verify, load)](https://pillow.readthedocs.io/en/stable/reference/Image.html).
- [Starlette: Requests, đọc form và file](https://starlette.dev/requests/).
