---
slug: python-ocr-service-fastapi-worker-timeout
title: "Đóng gói OCR thành service: FastAPI nhận ảnh, worker xử lý, timeout"
summary: "Bài 46: đưa OCR Tesseract thành service. FastAPI nhận ảnh, worker arq chạy OCR như tiến trình con có thể kill, timeout dừng việc thật, job idempotent theo hồ sơ và hash ảnh, tải thử p50/p95 bằng httpx."
category: python
level: advanced
tags: [python, toi-noi, fastapi, arq, ocr, timeout]
language: vi
---

Bài 41 đến 45 chạy OCR trong một script: đưa một tấm thẻ vào, nhận về các trường, mất khoảng 0,4 giây. Khi `kyc-ocr` của [Tới Nơi](/articles/du-an-mau-toi-noi) mở cho tài xế đăng ký thật, những thứ script không phải nghĩ tới đổ dồn vào cùng lúc: nhiều người gửi ảnh đồng thời, Tesseract là một tiến trình ngoài ăn hết một lõi CPU, và thỉnh thoảng có ảnh làm nó chạy lâu gấp nhiều lần. Nếu API gọi OCR thẳng, vài ảnh xấu là đủ hạ gục cả service.

Đây là bài 46 của lộ trình **Python nâng cao qua Tới Nơi**. Bài dựa vào ba lớp kiểm khi nhận ảnh ở [bài 35](/articles/python-fastapi-upload-background-task-streaming), lỗi `application/problem+json` ở [bài 32](/articles/python-fastapi-validation-problem-details-openapi) và worker arq trên Redis ở [bài 40](/articles/python-worker-arq-scheduling-retry-dead-letter). Tách trường là bản rút gọn vì lab tự chứa và bài chỉ cần đủ trường để service có kết quả; [bài 44](/articles/python-hau-xu-ly-ocr-regex-parse-truong-checksum) đã làm đầy đủ. OCR dùng `vie+eng --psm 6` như bài 43, trên thẻ phẳng chưa qua tiền xử lý của bài 42. Ảnh là thẻ giả; kết quả OCR là đầu vào cho người duyệt, không tự động là quyết định xác minh.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Worker chạy OCR bằng <code>await asyncio.wait_for(asyncio.to_thread(chay_tesseract, anh), timeout=5)</code>. Đến giây thứ 5 ta nhận <code>TimeoutError</code> và đánh dấu job thất bại. Tiến trình Tesseract lúc đó còn chạy không, và yêu cầu kế tiếp có bị ảnh hưởng không?</p>
<details><summary>Xem đáp án</summary><p><b>Còn chạy.</b> <code>wait_for</code> chỉ ngừng <i>chờ</i>: thread không thể bị hủy từ ngoài, và tiến trình con mà thread đã sinh ra vẫn sống tới khi tự xong. Job đã «thất bại» trong sổ sách trong khi CPU vẫn phục vụ nó; với một ảnh treo thì nó phục vụ mãi. Bài chạy đúng thí nghiệm này: bỏ cuộc sau 0,15 giây mà Tesseract còn chạy thêm một lúc nữa. Yêu cầu kế tiếp bị ảnh hưởng gián tiếp: mỗi lần như vậy chiếm một thread của executor mặc định (tối đa <code>min(32, số lõi + 4)</code> theo tài liệu Python) và một lõi CPU tới khi Tesseract tự xong; đủ nhiều ảnh treo thì thread cạn và việc kế tiếp phải xếp hàng. Muốn timeout dừng việc thật thì tiến trình con phải bị kill.</p></details>
</div>

## Từ script sang service: API, hàng đợi, worker

Service có ba phần. **API** nhận ảnh, kiểm, lưu vào kho, xếp một job vào hàng đợi rồi trả `202`. **Worker** lấy job, chạy OCR, ghi kết quả. **Redis** nằm giữa, giữ hàng đợi, trạng thái job và kết quả. API không chạy OCR và không giữ trạng thái nào trong RAM.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Ứng dụng tài xế gửi POST tới API. API lưu ảnh vào kho ảnh, xếp job vào Redis rồi trả 202. Worker arq lấy job từ Redis, đọc ảnh từ kho, chạy tesseract như tiến trình con và ghi kết quả vào Redis. Yêu cầu GET đọc trạng thái trực tiếp từ Redis, ở bất kỳ bản API nào."><defs><marker id="py46-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="95" width="130" height="80" rx="10" class="f-pk-soft s-pk"/><text x="34" y="122" class="t-b">Ứng dụng tài xế</text><text x="34" y="144" class="t-small">POST ảnh</text><text x="34" y="161" class="t-small">GET job_id</text><path d="M150 125 L196 125" class="f-none s-phys" stroke-width="2" marker-end="url(#py46-0-arrow)"/><rect x="198" y="30" width="190" height="110" rx="10" class="f-pk-soft s-pk"/><text x="212" y="56" class="t-b">API (uvicorn) x N</text><text x="212" y="80" class="t-small">kiểm ảnh, giới hạn 10 MiB</text><text x="212" y="98" class="t-small">enqueue_job(_job_id=…)</text><text x="212" y="116" class="t-small">không chạy OCR</text><path d="M388 70 L440 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py46-0-arrow)"/><rect x="442" y="50" width="190" height="110" rx="10" class="f-phys-soft s-phys"/><text x="456" y="76" class="t-b">Redis</text><text x="456" y="100" class="t-small">hàng đợi lab46:queue</text><text x="456" y="118" class="t-small">khóa job, khóa đang chạy</text><text x="456" y="136" class="t-small">khóa kết quả (15 phút)</text><path d="M442 135 L390 125" class="f-none s-phys dash" stroke-width="2" marker-end="url(#py46-0-arrow)"/><text x="396" y="178" class="t-small">GET đọc ở Redis</text><path d="M632 95 L674 80" class="f-none s-phys" stroke-width="2" marker-end="url(#py46-0-arrow)"/><rect x="676" y="30" width="190" height="100" rx="10" class="f-pk-soft s-pk"/><text x="690" y="56" class="t-b">Worker arq x M</text><text x="690" y="80" class="t-small">max_jobs = 4 mỗi worker</text><text x="690" y="98" class="t-small">ghi kết quả về Redis</text><path d="M770 130 L770 188" class="f-none s-phys" stroke-width="2" marker-end="url(#py46-0-arrow)"/><rect x="676" y="190" width="190" height="80" rx="10" class="f-warn-soft s-warn"/><text x="690" y="216" class="t-b">tesseract (tiến trình con)</text><text x="690" y="238" class="t-small">ảnh vào stdin, TSV ra stdout</text><text x="690" y="256" class="t-small">hết hạn thì bị kill</text><rect x="430" y="205" width="210" height="70" rx="10" class="f-phys-soft s-phys"/><text x="444" y="230" class="t-b">Kho ảnh</text><text x="444" y="252" class="t-small">put ở API, get ở worker</text><path d="M290 140 L428 235" class="f-none s-phys" stroke-width="2" marker-end="url(#py46-0-arrow)"/><text x="300" y="210" class="t-small">put ảnh</text><path d="M692 130 L642 225" class="f-none s-phys dash" stroke-width="2" marker-end="url(#py46-0-arrow)"/><text x="548" y="196" class="t-small">worker get ảnh</text></svg>
</div>
<figcaption>Hình 1. Ba phần của service. Mọi trạng thái nằm ở Redis và kho ảnh, nên thêm bản API hay thêm worker không làm đổi câu trả lời.</figcaption>
</figure>

Lý do tách là điều bài 35 đã chạm tới: `BackgroundTasks` giữ job trong dict của tiến trình API, nên khởi động lại là mất và nhiều bản API cho `GET` job vừa tạo trả `404`; bản đầu của lab 35 còn đánh mã `JOB-0001` bằng bộ đếm riêng nên hai tiến trình phát trùng mã. Lab này không dict, không bộ đếm: mã job là **UUID v5** suy từ `(kyc_id, loại giấy tờ, sha256 của ảnh)`, nên mọi bản API tự tính ra cùng một mã.

Ta chọn **arq** như bài 40: có sẵn hàng đợi bền, retry và `job_timeout`. `ProcessPoolExecutor` đặt ngay trong API đơn giản hơn, nhưng pool chết cùng API, trạng thái nằm trong RAM, và hủy một việc đang chạy trong pool không dừng được nó (phần timeout). `kill_workers()` và `terminate_workers()` của Python 3.14 thì dừng cả pool, tức mọi yêu cầu khác. arq đang ở chế độ chỉ bảo trì (bài 40 đã nói); khái niệm giữ nguyên nếu đổi thư viện. Một ràng buộc phiên bản: theo bài 43, PaddleOCR chỉ có wheel đến Python 3.13, nên nếu `kyc-ocr` dùng nó thì worker chạy image Python 3.13 riêng còn API vẫn 3.14; job serialize bằng JSON nên hai phía chỉ chia sẻ Redis và kho ảnh. Lab chỉ chạy Tesseract nên không thử cấu hình đó.

Worker không chạy Tesseract trong thread hay pool mà **sinh nó thành tiến trình con bằng `asyncio.create_subprocess_exec`**. Coroutine của job chỉ chờ I/O, nên một worker với `max_jobs = 4` điều phối bốn lần OCR song song mà vòng lặp sự kiện vẫn rảnh, và mỗi lần OCR là một tiến trình của hệ điều hành, tức là kill được.

## Cửa vào: nhận ảnh và từ chối sớm

Handler `POST /v1/kyc/ocr-jobs` dùng lại ba lớp kiểm của bài 35: middleware nhìn `Content-Length`, `read_limited` đếm byte thật (giới hạn 10 MiB như bài 35), và Pillow xác định định dạng từ byte đầu file (JPEG, MPO hoặc PNG), đếm điểm ảnh từ header (trần 50 triệu) rồi gọi `verify()`. Tên file và `Content-Type` do client khai nên không được tin. Mọi lỗi theo đúng một dạng Problem Details của bài 32; lỗi 422 có `errors[{field, code, message}]` mà không chép lại giá trị đã gửi.

Cửa không bắt được mọi thứ, và lab kiểm đúng ranh giới đó. PNG cắt đôi bị `verify()` bắt ngay (`400 image-corrupt`) vì PNG có checksum. JPEG cắt đôi thì header vẫn đọc được và `verify()` gần như không kiểm gì, nên nó qua cửa với `202` và chỉ lộ khi worker giải nén thật: job `failed` với `error.code = image_unreadable`. Worker tự giải nén ảnh trong thread trước khi gọi Tesseract để lỗi này có tên, thay vì thành một thông báo lạ của Tesseract.

Phần cuối của handler phân biệt nhận với xử lý. Ghi ảnh trước, xếp job sau, để job không bao giờ trỏ vào object chưa có (`app/main.py`, trích):

```python
digest = sha256_hex(data)
job_id = job_id_for(kyc_id, document_type.value, digest)   # UUID v5
key = object_key(kyc_id, digest, info.ext)
await asyncio.to_thread(store.put, key, data)
enqueued = await pool.enqueue_job(
    "ocr_document", kyc_id, key, document_type.value,
    _job_id=arq_id(job_id), _queue_name=settings.queue_name)
out = await describe_job(pool, settings, job_id)
if enqueued is None:  # arq từ chối vì job_id đã có
    response.status_code = 200
    response.headers["Idempotent-Replayed"] = "true"
else:
    response.headers["Location"] = f"{UPLOAD_PATH}/{job_id}"
return out
```

## Idempotency theo hồ sơ và hash ảnh

Tài xế bấm «Gửi» hai lần vì mạng chập chờn, hoặc app tự gửi lại. Cùng một ảnh của cùng một hồ sơ không được sinh hai job. Khóa idempotency của lab là **`(kyc_id, loại giấy tờ, sha256 của ảnh)`**. Mã job suy ra được từ ba giá trị ấy, nên `GET` job phải kiểm quyền người gọi với hồ sơ (bài 34), điều lab chưa làm.

arq cho sẵn cơ chế này: `enqueue_job(..., _job_id=...)` trả `None` nếu khóa job hoặc khóa kết quả của mã đó đã tồn tại, và phép kiểm nằm trong một transaction `WATCH` nên hai yêu cầu đồng thời không cùng thắng (theo mã nguồn arq 0.28.0). Lab đo: mười `POST` đồng thời cùng một thẻ cho đúng một `202`, chín `200` và một job trong hàng đợi; cùng hồ sơ ảnh khác, hoặc cùng ảnh hồ sơ khác, đều cho job mới.

```text
  lặp lại đúng thẻ       -> 200  job 35f01a2d…  (job cũ)  done
  cùng hồ sơ, thẻ khác   -> 202  job 78ed94a5…  (job mới)  queued
  cùng thẻ, hồ sơ khác   -> 202  job 5c84897d…  (job mới)  queued
  10 POST đồng thời cùng thẻ: [200, 200, 200, 200, 200, 200, 200, 200, 200, 202] -> 1 job
```

Cửa sổ idempotency bằng thời gian giữ kết quả, `result_ttl_s = 900` giây; đó cũng là thời hạn số giấy tờ rõ nằm trong Redis nên ta giữ ngắn, lưu lâu dài là việc của bài 47. Job `failed` chia hai loại. `image_unreadable` được «nhớ»: gửi lại đúng ảnh hỏng trả `200` với job cũ, vì chạy lại chỉ cho lỗi cũ; người dùng chụp ảnh khác (hash khác, job mới). `ocr_timeout` có thể do worker quá tải nên được thử lại: gửi lại đúng ảnh xóa khóa kết quả rồi xếp lại, và nếu hai yêu cầu cùng thử lại thì chỉ một `enqueue_job` thành công. `ocr_failed` và `internal_error` cũng bị nhớ trong cửa sổ mà lab chưa có endpoint thử lại: đó là giới hạn.

Bốn trạng thái trả về (`queued`, `running`, `done`, `failed`) được dựng từ ba khóa của arq. Lần chạy `--verify` đầu tiên bắt được một lỗi đua: ca quan sát trạng thái báo `['queued', 'running', 404, 'done']`, tức một `GET` đúng lúc job vừa xong nhận `404` cho job có thật. Bản đầu gọi `Job.status()` rồi `Job.info()` của arq, hai lượt đọc; job xong giữa hai lượt thì lượt sau không còn thấy khóa job. Cách sửa là đọc cả ba khóa trong **một** transaction (`describe_job`, trích):

```python
async with pool.pipeline(transaction=True) as tr:
    tr.get(result_key_prefix + aid)
    tr.get(job_key_prefix + aid)
    tr.exists(in_progress_key_prefix + aid)
    result_raw, job_raw, in_progress = await tr.execute()
if result_raw:    # xong: done, hoặc failed (có error.code) nếu hàm job trả {"ok": False, ...}
    ...
if job_raw is None:
    return None   # chưa từng có, hoặc đã quá thời gian lưu kết quả
return JobOut(..., status=JobState.RUNNING if in_progress else JobState.QUEUED)
```

Hàm job trả thất bại có phân loại bằng giá trị trả về (`{"ok": False, "error": {...}}`) chứ không `raise`, để arq coi job là «chạy xong» và không tự thử lại. Thông báo gửi ra client là câu cố định theo mã lỗi; stderr của Tesseract và thông báo của Pillow chỉ vào log. Ngoại lệ không lường trước thoát ra và API trả `failed` với `internal_error`. Worker đặt `log_results = False` vì arq mặc định in kết quả job ra log, mà kết quả có số giấy tờ rõ.

## Timeout: dừng việc thật, không chỉ ngừng chờ

Timeout có ba tầng và phải xếp theo thứ tự: một lần gọi Tesseract (`ocr_timeout_s`, 20 giây) ngắn hơn timeout cả job (`job_timeout`, 30 giây), cả hai ngắn hơn timeout phía client. Nhờ vậy lỗi do ta phân loại (`ocr_timeout`) luôn thấy trước lỗi chung của framework. arq thực thi `job_timeout` bằng `asyncio.wait_for` bọc quanh job, tức **hủy coroutine**; điều đó chỉ hữu ích nếu hủy coroutine dẫn tới dừng việc thật.

Lab đặt ba cách cạnh nhau trên cùng một thẻ với timeout 0,15 giây (output của `main.py`; máy đang rất bận, load average ghi ở dòng đầu của output đầy đủ bên dưới):

```text
  wait_for(to_thread):   bỏ cuộc sau 0.15 s; Tesseract còn chạy: True; thread thật sự xong sau 0.77 s
  ProcessPoolExecutor:   (làm ấm pool 2.24 s) Tesseract bắt đầu trước timeout: True; cancel() = False; việc cũ xong sau 1.05 s; yêu cầu kế tiếp phải chờ 1.05 s mới được chạy
  kill tiến trình con:   bỏ cuộc sau 0.18 s; Tesseract còn chạy: False; việc kế tiếp chạy ngay (0.91 s cho một thẻ)
  hủy coroutine từ ngoài (arq quá job_timeout): tiến trình con còn chạy: [False]
  độ trễ lớn nhất của vòng lặp sự kiện trong lúc OCR (1086 ms): subprocess.run trong coroutine 1084 ms, asyncio subprocess 8 ms
  một lần OCR (trung bình 5 lần, tuần tự): mặc định 1.15 s thực, 0.43 s CPU; OMP_THREAD_LIMIT=1 1.76 s thực, 0.43 s CPU
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Ba làn thời gian cùng một mốc timeout. Làn một, wait_for với to_thread: coroutine bỏ cuộc ở mốc timeout nhưng Tesseract chạy tiếp tới khi tự xong. Làn hai, ProcessPoolExecutor: cancel trả False, việc cũ chạy tiếp và việc kế tiếp phải chờ. Làn ba, kill tiến trình con: Tesseract dừng ngay ở mốc timeout và việc kế tiếp bắt đầu liền sau đó."><defs><marker id="py46-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><path d="M300 28 L300 250" class="f-none s-warn dash" stroke-width="2"/><text x="308" y="22" class="t-small">mốc timeout</text><text x="20" y="62" class="t-b">wait_for(to_thread)</text><rect x="200" y="44" width="100" height="26" rx="6" class="f-phys-soft s-phys"/><text x="212" y="62" class="t-small">Tesseract</text><rect x="300" y="44" width="300" height="26" rx="6" class="f-warn-soft s-warn"/><text x="312" y="62" class="t-small">vẫn chạy, không ai chờ kết quả</text><text x="620" y="62" class="t-small">tự xong muộn</text><text x="20" y="152" class="t-b">ProcessPoolExecutor</text><rect x="200" y="122" width="100" height="26" rx="6" class="f-phys-soft s-phys"/><text x="212" y="140" class="t-small">Tesseract</text><rect x="300" y="122" width="440" height="26" rx="6" class="f-warn-soft s-warn"/><text x="312" y="140" class="t-small">cancel() = False, vẫn chạy tiếp</text><rect x="300" y="154" width="440" height="26" rx="6" class="f-none s-dead dash"/><text x="312" y="172" class="t-small">việc kế tiếp phải chờ worker duy nhất</text><rect x="740" y="154" width="100" height="26" rx="6" class="f-pk-soft s-pk"/><text x="752" y="172" class="t-small">chạy</text><text x="20" y="232" class="t-b">kill tiến trình con</text><rect x="200" y="214" width="100" height="26" rx="6" class="f-phys-soft s-phys"/><text x="212" y="232" class="t-small">Tesseract</text><path d="M296 227 L316 227" class="f-none s-warn" stroke-width="3"/><text x="322" y="231" class="t-small t-warn">kill</text><rect x="360" y="214" width="120" height="26" rx="6" class="f-pk-soft s-pk"/><text x="372" y="232" class="t-small">việc kế tiếp</text></svg>
</div>
<figcaption>Hình 2. Cùng một mốc timeout, ba kết cục. Chỉ cách cuối trả lại tài nguyên đúng lúc. Các đoạn không vẽ theo tỉ lệ thời gian.</figcaption>
</figure>

Cách thứ nhất là lỗi của câu hỏi đầu bài. Cách thứ hai: `future.result(timeout)` báo `TimeoutError` nhưng `cancel()` trả `False` vì việc đã chạy, nên với pool một worker, yêu cầu kế tiếp phải chờ tới khi việc cũ tự xong. Bản đầu của lab đo sai chỗ này: pool chưa được làm ấm, worker (spawn trên macOS) còn đang nạp lại `main.py` khi hết timeout, Tesseract chỉ bắt đầu sau timeout, và phần lớn «chờ» là khởi động worker. Giờ lab làm ấm pool bằng một việc rỗng trước khi đo (2,24 giây trong lần chạy trên) và kiểm Tesseract đã bắt đầu *trước* timeout; chỉ khi đó câu «việc đã chạy nên `cancel()` trả `False`» mới đúng. Cách thứ ba là `run_tesseract` của lab (`app/ocr.py`, trích):

```python
proc = await asyncio.create_subprocess_exec(
    binary, "stdin", "stdout", "-l", "vie+eng", "--psm", "6", "tsv",
    stdin=PIPE, stdout=PIPE, stderr=PIPE, env=ocr_env())
try:
    async with asyncio.timeout(timeout):
        out, err = await proc.communicate(image)
except TimeoutError:
    raise OcrTimeout(timeout, proc.pid) from None
finally:
    if proc.returncode is None:      # còn chạy: hết hạn HOẶC bị cancel từ ngoài
        proc.kill()
        await asyncio.shield(proc.wait())   # thu xác, không để zombie
```

`finally` là chỗ quan trọng: nó chạy cả khi hết `timeout` của chính ta lẫn khi arq hủy coroutine vì quá `job_timeout`. Lab kiểm cả hai: sau `OcrTimeout` tiến trình con không còn (`os.kill(pid, 0)` báo không tồn tại), và khi task bao `run_tesseract` bị `cancel()`, tiến trình `tesseract` con cũng biến mất. Ảnh vào qua `stdin` nên không có file tạm để dọn.

Còn một lý do để không chạy `subprocess.run` ngay trong coroutine. Lab đo độ trễ lớn nhất của vòng lặp sự kiện (một «nhịp tim» 10 ms) trong lúc OCR: `subprocess.run` trong `async def` làm vòng lặp đứng gần suốt thời gian OCR, còn `asyncio` subprocess chỉ chậm 8 ms (so với 1.084 ms trong lần chạy trên). Trong service thật, mọi yêu cầu khác của cùng tiến trình, kể cả `/healthz`, ngừng trả lời.

Về số luồng: FAQ của Tesseract khuyên `OMP_THREAD_LIMIT=1` khi chạy mỗi lõi một tiến trình, và lab đặt nó. Nhưng binary Tesseract 5.5.4 của Homebrew trên máy này không dùng OpenMP (`tesseract --version` không in «Found OpenMP», `otool -L` không thấy thư viện OpenMP), nên biến không có tác dụng ở đây: dòng cuối output trên cho CPU bằng nhau (0,43 giây) có và không có nó. Ta giữ biến cho build có OpenMP. Khi máy bận, CPU tin được hơn thời gian thực.

## Tải thử: p50 và p95

`--loadtest` gửi thẻ khác nhau bằng `httpx` tới hai bản API (client xoay vòng và hỏi kết quả ở bản **kia**, nên mỗi lần đo cũng là một lần đọc xuyên tiến trình), với hai worker arq, mỗi worker tối đa 4 job đồng thời. Mỗi mức gửi 40 thẻ JPEG, bằng `main.py --loadtest 40 --concurrency 2,8,32`; client hỏi trạng thái mỗi 100 ms nên số «job done» có sai số tới 100 ms.

```text
macOS 26.6.2, arm64, 10 lõi, Python 3.14.7
trước khi đo: load average 1/5/15 phút: 34.06 87.52 98.95
2 uvicorn (client xoay vòng, hỏi kết quả ở bản kia), 2 arq (mỗi worker tối đa 4 job đồng thời), 40 thẻ mỗi mức
sau khi đo:   load average 1/5/15 phút: 36.17 82.79 96.82
40 thẻ, 2 yêu cầu đồng thời: done 40, failed 0, 202 nhận 40; tổng 13.2 giây (3.0 job/giây)
  POST tới khi nhận 202      p50       16 ms   p95       43 ms
  POST tới khi job done      p50      580 ms   p95      906 ms
    trong đó chờ trong hàng  p50       57 ms   p95      105 ms
    trong đó Tesseract chạy  p50      440 ms   p95      724 ms
40 thẻ, 8 yêu cầu đồng thời: done 40, failed 0, 202 nhận 40; tổng 5.3 giây (7.6 job/giây)
  POST tới khi nhận 202      p50       43 ms   p95      107 ms
  POST tới khi job done      p50     1017 ms   p95     1224 ms
    trong đó chờ trong hàng  p50      168 ms   p95      330 ms
    trong đó Tesseract chạy  p50      690 ms   p95      856 ms
40 thẻ, 32 yêu cầu đồng thời: done 40, failed 0, 202 nhận 40; tổng 9.3 giây (4.3 job/giây)
  POST tới khi nhận 202      p50      111 ms   p95      505 ms
  POST tới khi job done      p50     3473 ms   p95     7594 ms
    trong đó chờ trong hàng  p50     2325 ms   p95     4858 ms
    trong đó Tesseract chạy  p50      994 ms   p95     2763 ms
```

(Trích, bỏ dòng độ chính xác. Lần đo này chạy trước các sửa sau review (khóa idempotency thêm loại giấy tờ, `image_sha256` trong kết quả), vốn không nằm trên đường đo, nhưng không phải lần chạy cuối. Load average khoảng 35 là mức thấp nhất máy có trong buổi đo; một lần chạy sau đó, load 156 rồi 268, cho Tesseract p50 3.738 ms ngay ở mức 2 đồng thời.) Thông lượng tăng từ 3,0 lên 7,6 job/giây khi đồng thời tăng từ 2 lên 8, rồi **giảm** còn 4,3 ở mức 32: chờ trong hàng vọt lên 2,3 giây và Tesseract cũng chậm hơn (994 ms so với 690 ms); lab không đo nguyên nhân.

«POST tới khi nhận 202» là thứ người dùng thấy ngay: kiểm ảnh, ghi kho, xếp hàng, chưa có OCR. «Chờ trong hàng» tăng khi số yêu cầu đồng thời vượt số chỗ của worker (2 worker nhân 4 job); quá tải thì tăng worker hoặc từ chối sớm bằng giới hạn tốc độ, không tăng timeout. Số giấy tờ đúng cả 40 thẻ ở mỗi mức, ngày sinh 38 đến 40 trên 40; họ tên chỉ đúng nguyên văn rất ít vì Tesseract đọc «NGUYỄN VĂN A» thành «NGUYEN VAN A», nhưng đúng khi bỏ dấu ở cả 40 thẻ. Đo và giảm lỗi đó là việc của bài 44 và 45.

## Chạy và kiểm chứng

Cần Redis của hạ tầng dùng chung và Tesseract có ngôn ngữ `vie`. Từ gốc repository:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/46-ocr-service-fastapi-worker examples/toi-noi-python/46-ocr-service-fastapi-worker/main.py
```

Demo dựng API trong tiến trình, worker chạy theo lượt. Một job đi hết đường (trích; demo cố ý chờ thêm 0,5 giây trong job để kịp thấy `running`):

```text
macOS 26.6.2, arm64, 10 lõi, Python 3.14.7; load average 1/5/15 phút: 193.34 228.00 199.33

POST /v1/kyc/ocr-jobs -> 202  Location: /v1/kyc/ocr-jobs/35f01a2d-12c2-5692-acf6-d0fc2bccc8c3
GET khi chưa có worker -> 200 queued  Retry-After: 1
trạng thái quan sát được: queued -> running -> done
so với đáp án do hàm sinh thẻ trả về:
  id_number      OCR '099090159083'     đáp án '099090159083'     đúng
  date_of_birth  OCR '13/03/1990'       đáp án '13/03/1990'       đúng
  full_name      OCR 'NGUYEN VAN A'     đáp án 'NGUYỄN VĂN A'     khác
  sex            OCR 'Nam'              đáp án 'Nam'              đúng
  hometown_city  OCR 'Cần Thơ'          đáp án 'Cần Thơ'          đúng
```

Cửa vào, cùng lần chạy:

```text
  file chữ đội lốt .jpg              -> 415 application/problem+json  unsupported-image
  ảnh 15.9 MB (Content-Length)       -> 413 application/problem+json  upload-too-large
  cùng ảnh, chunked                  -> 413 application/problem+json  upload-too-large
  PNG nhỏ, 64 triệu điểm ảnh         -> 400 application/problem+json  image-too-many-pixels
  PNG cắt cụt                        -> 400 application/problem+json  image-corrupt
  kyc_id sai dạng, loại gplx         -> 422 application/problem+json  validation-error
```

Với tiến trình thật, lab khởi động hai uvicorn trên hai cổng cùng hai arq, gửi ở API này và hỏi ở API kia. Bài 35 thấy `404` ở đúng chỗ này; ở đây cả hai trả `200` vì trạng thái ở Redis chứ không ở RAM:

```text
  POST ở API pid 94631 -> 202 queued
  GET ở API A (pid 94631) -> 200 done, worker pid 94629
  GET ở API B (pid 94632) -> 200 done, worker pid 94629
```

Kiểm chứng đầy đủ là chế độ `--verify`: 42 ca, gồm 29 ca gọi app trong tiến trình qua `httpx.ASGITransport` với worker arq chạy theo lượt, 6 ca về bộ tách trường và timeout, 7 ca trên tiến trình thật (output nguyên văn của lần chạy cuối; dòng đầu ghi máy và load average lúc chạy):

```bash
uv run --project examples/toi-noi-python/46-ocr-service-fastapi-worker examples/toi-noi-python/46-ocr-service-fastapi-worker/main.py --verify
```

```text
macOS 26.6.2, arm64, 10 lõi, Python 3.14.7; load average 1/5/15 phút: 164.28 216.66 196.34
Nhóm 1: API trong tiến trình (ASGITransport) và worker arq chạy theo lượt
  đạt  POST thẻ hợp lệ: 202 queued, Location trỏ tới job
  đạt  job_id là UUID v5 suy từ (kyc_id, loại giấy tờ, sha256 ảnh), không có bộ đếm
  đạt  chưa có worker: GET trả queued kèm Retry-After
  đạt  cùng hồ sơ, cùng ảnh: 200, Idempotent-Replayed, đúng job cũ
  đạt  10 POST đồng thời cùng (hồ sơ, ảnh): đúng một 202, chín 200, một job trong hàng
  đạt  cùng hồ sơ, ảnh khác: job mới
  đạt  cùng ảnh, hồ sơ khác: job mới
  đạt  JPEG cắt cụt vẫn qua cửa (header đọc được): 202
  đạt  job xong: status done, result có đủ 5 trường
  đạt  số giấy tờ, ngày sinh, giới tính, quê quán đọc đúng từng ký tự
  đạt  họ tên chỉ đúng khi bỏ dấu (Tesseract vie+eng đọc chữ hoa có dấu kém)
  đạt  độ tin cậy mỗi trường trong [0,1]; confidence = nhỏ nhất; needs_review theo ngưỡng 0,8
  đạt  kết quả có queue_ms, run_ms, không lộ raw_text
  đạt  nối lại sau khi xong: 200, vẫn đúng job và kết quả cũ, không chạy lại
  đạt  JPEG cắt cụt: failed, error.code image_unreadable (lỗi lộ ở worker, không ở cửa)
  đạt  gửi lại đúng ảnh hỏng đó: 200, trả lại job failed cũ (chính sách: thấy lỗi, chụp ảnh khác)
  đạt  quan sát được queued -> running -> done
  đạt  OCR quá hạn: failed, error.code ocr_timeout
  đạt  lỗi gửi ra client là câu cố định, không có chi tiết của Tesseract hay Pillow
  đạt  sau job quá hạn, worker vẫn nhận việc kế tiếp và làm xong
  đạt  ocr_timeout là lỗi thử lại được: gửi lại đúng ảnh đó được 202, chạy thật và done (khác image_unreadable)
  đạt  file chữ đội lốt .jpg: 415 problem+json, không ghi object, không tạo job
  đạt  ảnh 15.9 MB có Content-Length: 413 ngay từ lớp 1
  đạt  cùng ảnh gửi chunked, không Content-Length: lớp 1 bỏ qua, lớp 2 vẫn trả 413
  đạt  PNG nhỏ, 64 triệu điểm ảnh: 400 image-too-many-pixels
  đạt  PNG cắt cụt: verify() bắt được ngay ở cửa, 400 image-corrupt (khác JPEG cắt cụt, chỉ lộ ở worker)
  đạt  kyc_id sai dạng và document_type chưa hỗ trợ: 422, errors[{field, code, message}], không chép lại giá trị
  đạt  job không tồn tại: 404 problem+json; job_id không phải UUID: 422
  đạt  /healthz: ok, có pid và độ sâu hàng đợi
Nhóm 2: bộ tách trường và ba cách đặt timeout
  đạt  chữ rác ở giới tính và quê quán: để trống (None, độ tin cậy 0), không nhận với độ tin cậy 0,95 của token rác
  đạt  wait_for(to_thread): sau timeout tiến trình Tesseract vẫn chạy, thread xong sau đó
  đạt  ProcessPoolExecutor đã làm ấm: Tesseract bắt đầu TRƯỚC timeout, cancel() trả False, yêu cầu thứ hai chờ việc cũ xong
  đạt  kill khi hết hạn: tiến trình con không còn, việc kế tiếp chạy được ngay
  đạt  coroutine bị hủy từ ngoài (arq quá job_timeout): tiến trình con cũng bị kill
  đạt  subprocess.run trong coroutine chặn vòng lặp gần suốt thời gian OCR; asyncio subprocess thì không
Nhóm 3: hai uvicorn và hai arq là tiến trình thật
  đạt  hai bản API là hai tiến trình khác nhau (pid 95990 và 95991), chung một Redis
  đạt  6 thẻ gửi xen kẽ hai bản API, hỏi kết quả ở bản kia, qua arq x2: cả sáu done
  đạt  số giấy tờ đọc đúng cả sáu
       (sáu job chạy ở 2 tiến trình worker khác nhau)
  đạt  40 GET cùng một job, một nửa ở mỗi bản API: không lần nào 404 (trạng thái ở Redis, không ở RAM)
  đạt  10 POST lặp, xen kẽ hai bản API: cùng một job_id, không lần nào 202
  đạt  tải thử nhỏ (8 thẻ, 4 đồng thời): done 8, failed 0, số giấy tờ đúng 8/8
  đạt  dọn xong: không còn key lab46 nào trong Redis
OK 42/42
```

Ca đo thời gian chỉ kiểm bất biến (thứ tự mốc, tiến trình còn hay không), không kiểm số giây tuyệt đối. Để biết `finally` có cần không, ta thay nó bằng `pass`. Hai ca đỏ:

```text
  SAI  kill khi hết hạn: tiến trình con không còn, việc kế tiếp chạy được ngay  [{'gave_up_s': 0.17123616696335375, 'child_alive_after_timeout': True, 'next_job_s': 1.583122249925509}]
  SAI  coroutine bị hủy từ ngoài (arq quá job_timeout): tiến trình con cũng bị kill  [{'children_seen': 3, 'alive_after_cancel': [True, True, True]}]
SAI: kill khi hết hạn: tiến trình con không còn, việc kế tiếp chạy được ngay
SAI: coroutine bị hủy từ ngoài (arq quá job_timeout): tiến trình con cũng bị kill
FAIL 40/42
```

`child_alive_after_timeout: True` là Tesseract còn sống sau khi ta báo timeout; `children_seen: 3` là ba tiến trình `tesseract` mồ côi chồng lên nhau.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `ocr_timeout` hàng loạt giờ cao điểm | Thời gian chờ trong hàng không tính vào timeout một lần OCR; so `max_jobs` nhân số worker với số lõi |
| `GET` job vừa tạo đôi khi `404` | Trạng thái đọc bằng nhiều lệnh Redis rời rạc; gom vào một transaction |
| Gửi lại đúng ảnh mà không có job mới | Cửa sổ idempotency (`result_ttl_s`) chưa hết; trừ `ocr_timeout`, job `failed` cũng được nhớ |
| Sau timeout CPU vẫn cao, `tesseract` còn trong `ps` | Timeout mới ngừng chờ, chưa kill; kiểm khối `finally` quanh `proc.kill()` |
| `/healthz` chậm đúng bằng một lần OCR | Có `subprocess.run` hoặc hàm đồng bộ nặng trong `async def` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Một Redis đơn lẻ, không replica: ví dụ không nói gì về mất job khi Redis chết. Kho ảnh là thư mục trên một máy; production là S3 hoặc GCS có mã hóa, vòng đời, quét virus. Worker bị kill giữa chừng thì theo mã nguồn arq, khóa «đang chạy» hết hạn sau <code>job_timeout</code> cộng 10 giây rồi job được lấy lại; lab không thử ca này. Không có xác thực hay phân quyền (bài 34) và giới hạn tốc độ theo tài xế. Số giấy tờ rõ nằm trong Redis 15 phút; bài 47 lưu lâu dài với mặt nạ PII. Tách trường chỉ là regex trên thẻ giả sạch; ảnh chụp thật cho kết quả khác hẳn. Số đo lấy trên macOS Apple M1 Max đang bận.</p>
</div>

**Bài tập 1:** Hai ảnh JPEG khác nhau về byte (một ảnh đã được app nén lại) nhưng cùng là mặt trước một thẻ, gửi cho cùng `KYC-0001`. Service tạo mấy job, và có đúng ý «một ảnh một lần OCR» không?

**Bài tập 2:** Ảnh JPEG cắt đôi cho job `failed` với `image_unreadable`; tài xế gửi lại đúng file đó trong 15 phút. Điều đó khác gì khi lỗi là `ocr_timeout`, và vì sao lab đối xử hai lỗi khác nhau?

<details><summary>Xem lời giải</summary><p><b>1.</b> Hai job: hash khác nên mã job khác. Khóa idempotency là <i>byte của ảnh</i>, không phải «nội dung thẻ», nên chỉ chặn việc gửi lại đúng một file. Chặn cả ảnh nén lại cần khóa theo thứ ổn định hơn, như một job đang chạy cho mỗi hồ sơ, và trả giá bằng phức tạp. <b>2.</b> Ảnh cắt đôi: <code>200</code> với job <code>failed</code> cũ, không chạy lại, vì chạy lại chỉ cho lỗi cũ. <code>ocr_timeout</code>: <code>202</code> và chạy lại thật, vì lỗi có thể do worker quá tải chứ không do ảnh. Đừng tắt idempotency cho mọi job lỗi: một ảnh hỏng sẽ bị chạy lại mỗi lần bấm.</p></details>

Bài trước: [bài 45](/articles/python-danh-gia-ocr-cer-wer-threshold). Bài tiếp: [bài 47](/articles/python-luu-ket-qua-ocr-mongodb-pii-retention).

## Tài liệu gốc

- [Python: asyncio subprocess](https://docs.python.org/3.14/library/asyncio-subprocess.html).
- [Python: asyncio.timeout và wait_for](https://docs.python.org/3.14/library/asyncio-task.html#timeouts).
- [Python: concurrent.futures, Future.cancel và result(timeout)](https://docs.python.org/3.14/library/concurrent.futures.html#future-objects).
- [arq: tài liệu](https://arq-docs.helpmanual.io/).
- [Tesseract: FAQ, OMP_THREAD_LIMIT](https://tesseract-ocr.github.io/tessdoc/FAQ.html).
- [RFC 9457: Problem Details for HTTP APIs](https://www.rfc-editor.org/rfc/rfc9457.html).
