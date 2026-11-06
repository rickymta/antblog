---
slug: python-worker-arq-scheduling-retry-dead-letter
title: "Worker nền với arq/taskiq: lịch, retry, dead-letter"
summary: "Bài 40: worker arq trên Redis chấm điểm chuyến với retry backoff, dead-letter, job hoãn và cron, job idempotent theo trip_id, graceful shutdown khi SIGTERM. Chạy trên Python 3.14; tổng kết P03 và P04."
category: programming
level: intermediate
tags: [python, toi-noi, arq, redis, retry, dead-letter]
language: vi
---

Mỗi chuyến vừa hoàn thành của [Tới Nơi](/articles/du-an-mau-toi-noi) cần một điểm chấm gian lận để người duyệt xem sau. Sự kiện `trip.completed.v1` đến consumer Python qua Kafka (bài 38 và 39), nhưng consumer không nên gọi dịch vụ chấm điểm ngay trong vòng đọc: dịch vụ chậm thì partition kẹt. Cách quen thuộc là consumer chỉ **xếp một job** vào hàng đợi rồi đọc tiếp, còn một **worker** riêng làm việc đó.

Việc đẩy sang worker tạo ba bài toán. Dịch vụ chấm điểm có thể lỗi tạm thời, nên job phải **thử lại có kiểm soát** và có chỗ cất khi hết lượt (dead-letter). Sự kiện có thể trùng (bài 39), nên hiệu ứng của job phải **idempotent**. Mỗi lần triển khai, worker nhận SIGTERM giữa lúc chạy job, nên phải **dừng êm**. Đây là bài 40 của lộ trình **Python trung bình qua Tới Nơi**, và là bài tổng hợp của P03 và P04. Bạn cần nắm Redis ở [bài 27](/articles/python-redis-cache-rate-limit-geo) và outbox, inbox ở [bài 39](/articles/python-outbox-inbox-idempotent-consumer).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Job <code>score_trip</code> gọi dịch vụ chấm điểm đang quá tải và nhận 503. Worker bắt lỗi rồi thử lại ngay lập tức, không chờ, và cứ thế vô hạn. Chuyện gì xảy ra với dịch vụ chấm điểm, và cần đổi những gì?</p>
<details><summary>Xem đáp án</summary><p>Mỗi worker biến một lỗi thành hàng nghìn yêu cầu mỗi giây dội vào dịch vụ đang quá tải, làm nó khó hồi phục hơn (retry storm). Cần ba thay đổi: <b>chờ lâu dần</b> giữa các lần thử (backoff, thường có thêm jitter), <b>giới hạn số lần</b> thử, và khi hết lượt thì <b>đưa job vào dead-letter</b> thay vì thử mãi hay bỏ lặng lẽ.</p></details>
</div>

## Vì sao cần worker, và chọn arq hay taskiq

BackgroundTasks của FastAPI (bài 35) chạy việc trong chính tiến trình API: tiến trình chết là việc biến mất, không có retry, không có lịch, không có chỗ xem việc nào thất bại. Worker với hàng đợi bền giải các vấn đề đó: job nằm ở Redis, sống sót qua lần khởi động lại của API, và nhiều worker cùng lấy việc. Đổi lại, hàng đợi chỉ bảo đảm **ít nhất một lần**: job có thể chạy lại vì retry, bị hủy khi worker dừng, hoặc worker chết giữa chừng.

Lab dùng **arq**: thư viện job queue bất đồng bộ, nhỏ, nói chuyện với Redis qua redis-py. Hai điều cần nói rõ, kiểm ngày 6/11/2026. Một: arq 0.28.0 (phát hành 16/4/2026) khai báo hỗ trợ Python 3.14 và lab chạy đủ cả bài trên 3.14.7. Hai: README của dự án ghi arq đang ở **chế độ chỉ bảo trì**; nó vẫn chạy tốt nhưng cần cân nhắc cho dự án dài hạn. Hướng thay thế là **taskiq** với `taskiq-redis`; lab này **không chạy taskiq**, nên bài không đưa mã của nó, còn khái niệm thì áp dụng nguyên.

arq mặc định đóng gói job bằng pickle, và giải pickle dữ liệu đọc từ Redis là chạy mã của bất kỳ ai ghi được vào Redis đó. Lab truyền `job_serializer` và `job_deserializer` bằng JSON (`app/settings.py`): job chỉ mang chuỗi, đọc được bằng `redis-cli`, không có đường thực thi mã.

## Job đi qua hàng đợi, retry có backoff

Hàng đợi của arq là một sorted set trong Redis: mỗi job là một `job_id` với điểm số là thời điểm được phép chạy. Worker định kỳ lấy các job có điểm số đã tới, và mỗi lần lấy tăng bộ đếm `job_try` của job. Retry vì thế đơn giản là đặt lại điểm số của job về một thời điểm sau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Vòng đời job. Producer enqueue vào sorted set lab40:queue. Worker arq lấy job. Kết quả có ba nhánh: thành công thì ghi điểm bằng SET NX; lỗi tạm thời thì Retry với defer theo backoff và quay lại hàng đợi; lỗi vĩnh viễn, lỗi chưa phân loại hoặc hết bốn lần thì vào dead-letter lab40:dlq."><defs><marker id="py40-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py40-0-arrow-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="110" width="150" height="80" rx="10" class="f-pk-soft s-pk"/><text x="34" y="138" class="t-b">Producer</text><text x="34" y="158" class="t-small">enqueue_job</text><text x="34" y="176" class="t-small">_job_id theo trip_id</text><path d="M170 150 L208 150" class="f-none s-phys" stroke-width="2" marker-end="url(#py40-0-arrow)"/><rect x="210" y="100" width="200" height="100" rx="10" class="f-phys-soft s-phys"/><text x="226" y="128" class="t-b">Redis</text><text x="226" y="148" class="t-small">lab40:queue</text><text x="226" y="166" class="t-small">sorted set: job_id</text><text x="226" y="184" class="t-small">điểm = giờ được chạy</text><path d="M410 150 L448 150" class="f-none s-phys" stroke-width="2" marker-end="url(#py40-0-arrow)"/><rect x="450" y="100" width="170" height="100" rx="10" class="f-pk-soft s-pk"/><text x="466" y="128" class="t-b">Worker arq</text><text x="466" y="150" class="t-small">max_jobs = 5</text><text x="466" y="168" class="t-small">job_try tăng mỗi lần</text><text x="466" y="186" class="t-small">job_timeout = 10 giây</text><path d="M620 150 L645 150 L645 50 L668 50" class="f-none s-phys" stroke-width="2" marker-end="url(#py40-0-arrow)"/><path d="M620 150 L668 150" class="f-none s-phys" stroke-width="2" marker-end="url(#py40-0-arrow)"/><path d="M620 150 L645 150 L645 250 L668 250" class="f-none s-phys" stroke-width="2" marker-end="url(#py40-0-arrow)"/><rect x="670" y="15" width="190" height="70" rx="10" class="f-phys-soft s-phys"/><text x="686" y="40" class="t-b">Thành công</text><text x="686" y="60" class="t-small">SET NX điểm, lưu kết quả</text><text x="686" y="76" class="t-small">hiệu ứng đúng một lần</text><rect x="670" y="115" width="190" height="70" rx="10" class="f-warn-soft s-warn"/><text x="686" y="140" class="t-b">Lỗi tạm thời</text><text x="686" y="160" class="t-small">raise Retry(defer=...)</text><text x="686" y="176" class="t-small">tối đa 4 lần chạy</text><rect x="670" y="215" width="190" height="70" rx="10" class="f-bg s-dead"/><text x="686" y="240" class="t-b">Dead-letter</text><text x="686" y="260" class="t-small">lab40:dlq (list Redis)</text><text x="686" y="276" class="t-small">vĩnh viễn, lạ, hoặc lần 4</text><path d="M535 202 L535 262 L310 262 L310 204" class="f-none s-warn dash" stroke-width="1.5" marker-end="url(#py40-0-arrow-w)"/><text x="334" y="282" class="t-small">Retry: vào lại hàng đợi sau backoff</text></svg>
</div>
<figcaption>Hình 1. Một job có ba kết cục. Chỉ lỗi tạm thời mới quay lại hàng đợi, và nó quay lại với giờ chạy muộn hơn; lỗi vĩnh viễn, lỗi chưa phân loại và lượt thử cuối đi thẳng vào dead-letter.</figcaption>
</figure>

Điều đầu tiên cần biết về arq: **chỉ `Retry` mới được thử lại**. Exception nào khác thoát khỏi hàm job làm job thất bại, không còn lần thử nào (chuyến có khoảng cách `abc`: hàm chỉ được gọi 1 lần). Vì vậy việc phân loại lỗi thuộc về code của ta, với hai loại trong `app/scoring.py`: `TransientScoringError` (backend 503, thử lại có thể thành công) và `InvalidTripError` (đầu vào sai, thử lại vô ích). Khoảng chờ tính bằng một hàm thuần (`app/jobs.py`):

```python
def backoff_delay(job_try: int, *, rng: Random | None = None) -> float:
    delay = min(BACKOFF_CAP_S, BACKOFF_BASE_S * 2 ** (job_try - 1))
    return rng.uniform(0, delay) if rng else delay
```

Lần thất bại thứ `n` chờ `base × 2^(n-1)`, có trần. Khi truyền `rng`, hàm dùng «full jitter» (ngẫu nhiên từ 0 đến độ trễ) để nhiều job lỗi cùng lúc không quay lại cùng một thời điểm; mặc định không jitter để lab kiểm được con số. Lab dùng base 0,2 giây cho nhanh; production tính bằng giây đến phút. Hàm job (`score_trip`, trích) ghép phân loại lỗi, backoff và giới hạn lần:

```python
try:
    result = await call_backend(redis, trip_id, distance_km)
except InvalidTripError as exc:
    await dead_letter(redis, ctx, trip_id, distance_km, "permanent", exc)  # không thử lại: vô ích
    raise
except TransientScoringError as exc:
    if job_try >= MAX_TRIES:
        # Lần cuối: arq sẽ không gọi lại hàm này, nên đây là cơ hội duy nhất ghi dead-letter.
        await dead_letter(redis, ctx, trip_id, distance_km, "max_tries", exc)
        raise
    raise Retry(defer=backoff_delay(job_try)) from exc
except Exception as exc:
    # Lỗi chưa phân loại: arq không thử lại và không báo ai, nên ta tự ghi dead-letter.
    await dead_letter(redis, ctx, trip_id, distance_km, "unexpected", exc)
    raise
```

Chú thích ở nhánh cuối là chỗ dễ sai. Theo mã nguồn arq 0.28.0 (`worker.py`), khi `job_try` vượt `max_tries` worker ghi «max retries exceeded» mà **không gọi hàm của ta**; hàm cứ `raise Retry` ở lượt cuối thì không còn dịp ghi dead-letter. Vì thế `score_trip` tự đếm và ghi dead-letter ở lần chạy thứ `MAX_TRIES` trước khi ném lỗi. `TN-0002` có backend lỗi hai lần đầu rồi hồi phục, `TN-0003` lỗi mãi (log arq lấy từ lần chạy thật, lược các dòng khác):

```text
  arq:   0.01s → lab40:score:TN-0002:score_trip('TN-0002', '5')
  arq:   0.00s ↻ lab40:score:TN-0002:score_trip retrying job in 0.20s
  arq:   0.31s → lab40:score:TN-0002:score_trip('TN-0002', '5') try=2
  arq:   0.00s ↻ lab40:score:TN-0002:score_trip retrying job in 0.40s
  arq:   0.82s → lab40:score:TN-0002:score_trip('TN-0002', '5') try=3
  arq:   0.00s ← lab40:score:TN-0002:score_trip ● {'score': '0.090', 'reasons': [], 'model_version': 'rules-2026.11.1'}
TN-0002: lỗi 2 lần rồi thành công: success=True, lần thử 3, hàm được gọi 3 lần, tổng thời gian 0.92 giây (chờ tối thiểu 0,2 + 0,4)
```

Số đứng trước chữ `→` là thời gian tính từ lúc **enqueue**, cộng dồn, không phải khoảng giữa hai lần chạy. Với `TN-0002`, lần chạy thứ hai bắt đầu ở 0,31 giây và thứ ba ở 0,82 giây, nên khoảng thật giữa các lần chạy là 0,30 và 0,51 giây: lớn hơn backoff 0,2 và 0,4 giây vì còn thời gian chạy hàm và chu kỳ thăm dò 0,1 giây của worker. `--verify` chỉ kiểm cận dưới (tổng thời gian không nhỏ hơn 0,6 giây).

Hai nhánh còn lại chặn hai cách job «biến mất» khỏi dead-letter. Thứ nhất, backend **treo**: để nguyên thì job chạm `job_timeout` của arq, thất bại, không retry và không qua code dead-letter của ta. Nên `call_backend` đặt hạn chót riêng 2 giây bằng `asyncio.timeout` (bài 21), ngắn hơn `job_timeout` 10 giây, và đổi `TimeoutError` thành `TransientScoringError`; `TN-0011` treo ở lần chạy đầu rồi thành công ở lần hai. Thứ hai, exception **chưa phân loại**: arq không thử lại, nên ta ghi dead-letter `unexpected` rồi ném lại. Đầu vào cũng cần kiểm kỹ: `Decimal("NaN")` là chuỗi hợp lệ nhưng so sánh với 0 ném `InvalidOperation` ngoài vùng bắt lỗi, nên `parse_distance` kiểm `is_finite()` trước.

```text
TN-0013: distance_km = 'NaN': hàm gọi 1 lần, lý do dead-letter: permanent
TN-0012: lỗi không lường trước: success=False, hàm gọi 1 lần, lý do dead-letter: unexpected
  arq:   0.01s → lab40:score:TN-0011:score_trip('TN-0011', '3')
  arq:   2.00s ↻ lab40:score:TN-0011:score_trip retrying job in 0.20s
  arq:   2.25s → lab40:score:TN-0011:score_trip('TN-0011', '3') try=2
  arq:   0.01s ← lab40:score:TN-0011:score_trip ● {'score': '0.074', 'reasons': [], 'model_version': 'rules-2026.11.1'}
TN-0011: backend treo ở lần 1: success=True, lần thử 2, hiệu ứng = 3
```

## Dead-letter: nơi job hết lượt được xếp lại

Job hết lượt thử không được biến mất. **Dead-letter queue** (DLQ) giữ chúng lại để người vận hành xem, sửa nguyên nhân, rồi đưa lại. arq không có DLQ dựng sẵn, nên lab tự làm đơn giản nhất: một list Redis `lab40:dlq`, mỗi phần tử là một JSON mang đủ để chẩn đoán và đưa lại (`trip_id`, `distance_km`, `job_id`, `job_try`, `reason`, `error`, thời điểm). `reason` có ba giá trị: `permanent` cho lỗi đầu vào (vào ngay từ lần đầu, kể cả `NaN`), `max_tries` cho lỗi tạm thời hết lượt, `unexpected` cho exception chưa phân loại. Chỉ redrive loại `max_tries` sau khi backend sống lại; hai loại kia phải sửa dữ liệu hoặc code trước.

```text
  arq:   0.01s → lab40:score:TN-0003:score_trip('TN-0003', '10')
  arq:   0.00s ↻ lab40:score:TN-0003:score_trip retrying job in 0.20s
  arq:   0.31s → lab40:score:TN-0003:score_trip('TN-0003', '10') try=2
  arq:   0.00s ↻ lab40:score:TN-0003:score_trip retrying job in 0.40s
  arq:   0.82s → lab40:score:TN-0003:score_trip('TN-0003', '10') try=3
  arq:   0.00s ↻ lab40:score:TN-0003:score_trip retrying job in 0.80s
  arq:   1.63s → lab40:score:TN-0003:score_trip('TN-0003', '10') try=4
  arq:   0.00s ! lab40:score:TN-0003:score_trip failed, TransientScoringError: backend 503 ở lần chạy 4
TN-0003: backend luôn lỗi: success=False, lần thử 4, dead-letter 1 phần tử: max_tries / TransientScoringError: backend 503 ở lần chạy 4
TN-0004: dữ liệu sai: hàm gọi 1 lần, dead-letter 2 phần tử, lý do mới nhất: permanent
```

`TN-0003` được chạy đúng 4 lần với ba khoảng chờ 0,2, 0,4 và 0,8 giây rồi vào dead-letter; `TN-0004` (khoảng cách `abc`) chạy một lần rồi vào dead-letter ngay. Phần đưa lại (`app/dlq.py`, trích) chỉ lấy phần tử `max_tries` ra khỏi DLQ **sau khi** enqueue được:

```python
for raw in await redis.lrange(DLQ, 0, -1):
    entry = json.loads(raw)
    if entry["reason"] != "max_tries":
        continue
    job = await redis.enqueue_job(
        "score_trip", entry["trip_id"], entry["distance_km"], _job_id=f"{entry['job_id']}:redrive"
    )
    if job is not None:
        await redis.lrem(DLQ, 1, raw)  # chỉ gỡ khỏi dead-letter khi đã enqueue được
```

Hai chi tiết. Job đưa lại mang `job_id` **mới** (hậu tố `:redrive`), vì arq từ chối enqueue khi còn khóa job hoặc khóa kết quả của `job_id` cũ. Thứ tự «enqueue rồi mới gỡ» chọn trùng hơn mất: chết giữa hai bước thì job có thể được đưa lại hai lần, và ta nhờ tính idempotent ở phần dưới. Lab xóa cờ lỗi của backend giả rồi redrive: `dead-letter còn ['TN-0012', 'TN-0013', 'TN-0004']`, không còn `TN-0003`. Một job cron ghi độ dài DLQ vào Redis mỗi 30 giây để cảnh báo «DLQ khác 0» có chỗ đọc.

## Lịch, job hoãn và job idempotent

arq có hai kiểu «lịch». Kiểu thứ nhất là **job hoãn**: `enqueue_job(..., _defer_by=1.0)` đặt điểm số cách bây giờ 1 giây. Lab enqueue `TN-0005` kiểu này và đo từ lúc enqueue đến lúc bắt đầu chạy: 1,03 giây, không bao giờ nhỏ hơn 1 giây. Kiểu thứ hai là **cron**, khai báo trong `WorkerSettings` (`app/worker.py`, trích):

```python
class WorkerSettings:
    functions = [func(score_trip, max_tries=MAX_TRIES)]
    # Theo lịch: mỗi phút vào giây 0 và 30, cộng một lần ngay khi worker khởi động.
    cron_jobs = [cron(report_queue_health, name="lab40:report_queue_health", second={0, 30}, run_at_startup=True)]
    redis_settings = redis_settings()
    queue_name = QUEUE
    max_tries = MAX_TRIES
    max_jobs = 5  # tối đa 5 job chạy đồng thời trong một worker
    job_timeout = 10  # giây; quá hạn thì job bị hủy
    keep_result = 600  # giữ kết quả 10 phút
    poll_delay = 0.1
    job_completion_wait = int(os.environ.get("LAB40_GRACE_S", "10"))
```

Cron của arq mặc định `unique=True`: có nhiều worker thì mỗi mốc giờ chỉ một worker chạy job. Cron hợp với dọn dẹp, tổng hợp, báo cáo; retry thuộc về `Retry(defer=...)` của từng job. Lab khởi động một worker thật, chờ cron chạy lúc khởi động (`run_at_startup=True`) rồi đọc kết quả: `queued=1, dead_letters=3`.

Phần còn lại của mục là **idempotency**, vì hàng đợi chỉ giao ít nhất một lần. Lab có hai lớp, mỗi lớp chống một loại trùng. **Lớp 1** ở lúc enqueue: `_job_id` đặt theo `trip_id` (`lab40:score:TN-0001`), arq trả `None` khi job cùng id đã tồn tại hoặc kết quả còn được giữ. Lớp này rẻ nhưng ngắn hạn: nó mất khi kết quả hết hạn (`keep_result`), và không chặn được hai producer dùng hai id khác nhau. **Lớp 2** nằm trong job và bền hơn: (2a) thấy kết quả đã có thì trả lại, không gọi backend; (2b) ghi kết quả bằng `SET key value NX`, chỉ một lần ghi được, và bộ đếm hiệu ứng chỉ tăng khi ghi được (`score_trip`, trích):

```python
existing = await redis.get(SCORE + trip_id)
if existing is not None:
    return {**json.loads(existing), "duplicate": True}
# ... gọi backend ...
if await redis.set(SCORE + trip_id, json.dumps(result), nx=True):
    await redis.incr(EFFECTS)
```

Đoạn đầu trả kết quả cũ cho job đến sau, không gọi backend; đoạn sau bảo vệ chính cái cửa sổ giữa «đã đọc, chưa có» và «ghi», khi hai job cùng chuyến chạy đồng thời. Lab kiểm cả ba tình huống:

```text
Enqueue lại cùng job_id của TN-0001: None
Job khác id, cùng chuyến TN-0001: success=True, duplicate=True, hiệu ứng = 3
TN-0006 có hai job chạy đồng thời: hàm gọi 2 lần, hiệu ứng = 4
```

Dòng đầu là lớp 1, dòng hai là bước 2a. Dòng ba là bước 2b, ca khó nhất: hai job cùng chuyến chạy song song, cả hai cùng qua bước đọc, hàm được gọi 2 lần, nhưng chỉ một lần ghi thắng nên bộ đếm hiệu ứng tăng đúng 1 (từ 3 lên 4). Cùng ý với inbox ở bài 39: quyết định cuối thuộc về một thao tác nguyên tử của kho dữ liệu, không thuộc về câu kiểm tra «đã có chưa» đứng trước.

## Graceful shutdown: SIGTERM không được làm mất job

Mỗi lần triển khai, hệ thống điều phối gửi SIGTERM cho worker. Mặc định arq **hủy ngay** các job đang chạy. `job_completion_wait` đổi hành vi đó: sau tín hiệu, worker ngừng nhận job mới và cho job đang chạy tối đa ngần ấy giây, hết hạn mới hủy. Tài liệu arq gợi ý đặt nó cùng `terminationGracePeriodSeconds` của Kubernetes: hệ thống chỉ cho sống 30 giây thì chờ 60 giây là vô nghĩa. Lab đọc giá trị từ `LAB40_GRACE_S` (mặc định 10) và khởi động worker thật bằng đúng lệnh `arq app.worker.WorkerSettings` hai lần.

Lần một chờ 10 giây. Job `TN-0008` được cấu hình chậm 1,5 giây; ngay khi nó bắt đầu chạy, lab gửi SIGTERM, đợi dòng log shutdown, rồi enqueue thêm `TN-0010` (đến **sau** tín hiệu):

```text
SIGTERM khi TN-0008 đang chạy (chờ tối đa 10 giây): worker thoát mã 0 sau 1.57 giây
  log worker: shutdown on SIGTERM ◆ 1 jobs complete ◆ 0 failed ◆ 0 retries ◆ 1 to be completed
  log worker: shutdown on SIGTERM, wait complete ◆ 2 jobs complete ◆ 0 failed ◆ 0 retries ◆ 0 ongoing to cancel
  TN-0008: success=True, lần thử 1; TN-0010 gửi sau SIGTERM: runs=None
```

Job đang chạy hoàn thành trong 1,57 giây rồi worker thoát với mã 0. `TN-0010` không được nhận (`runs=None`): nó ở lại hàng đợi cho worker khác. Lần hai đặt `LAB40_GRACE_S=0`, tức là hủy ngay:

```text
SIGTERM không chờ: worker thoát mã 0; job TN-0009 xếp lại hàng đợi=True, chưa có kết quả=True, chưa có điểm=True
  log worker: shutdown on SIGTERM ◆ 2 jobs complete ◆ 0 failed ◆ 0 retries ◆ 1 ongoing to cancel
  log worker: 0.02s ↻ lab40:score:TN-0009:score_trip cancelled, will be run again
Worker mới: TN-0009 success=True, lần thử 2; TN-0010 success=True; hiệu ứng tổng = 9
```

Job bị hủy giữa chừng không mất: arq xếp nó lại hàng đợi (`will be run again`), và worker khởi động sau chạy nó ở lần thử 2. Việc này an toàn nhờ idempotent: lần bị hủy chưa kịp ghi điểm (`SET NX` ở cuối hàm), nên lần chạy lại chỉ tạo một hiệu ứng.

## Tổng kết P03 và P04

P03 và P04 dựng dần các mảnh mà nhóm service Python của [Tới Nơi](/articles/du-an-mau-toi-noi) cần, theo năm lớp (hình 2). Nền bất đồng bộ: [bài 21](/articles/python-asyncio-coroutine-task-timeout-cancellation) cho `asyncio.timeout` mà bài này dùng, bài 22 và 23 cho GIL, `Lock`, `Queue`. Dữ liệu: PostgreSQL (24, 25), MongoDB (26), Redis (27; hàng đợi job là một cách dùng mới của nó), test trên database thật (28), polars và numpy (29, 30; chưa bài nào của P04 dùng tới). API: hợp đồng và lỗi (31, 32), dependency (33), JWT (34), upload và `BackgroundTasks` (35; dẫn tới câu hỏi «khi nào cần worker»), test httpx (36), gRPC (37). Thông điệp: Kafka (38), outbox và inbox (39). Cuối cùng là worker của bài này.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Năm lớp của P03 và P04 từ trái sang phải: nền bất đồng bộ bài 21 đến 23, dữ liệu bài 24 đến 30, API bài 31 đến 37, thông điệp bài 38 và 39, việc nền bài 40. Dưới mỗi lớp là các chủ đề của lớp đó."><defs><marker id="py40-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="10" y="20" width="160" height="44" rx="10" class="f-bg s-line"/><text x="24" y="47" class="t-b">Nền async (21-23)</text><path d="M170 42 L183 42" class="f-none s-phys" stroke-width="2" marker-end="url(#py40-1-arrow)"/><rect x="10" y="80" width="160" height="150" rx="10" class="f-bg s-line"/><text x="24" y="108" class="t-small">asyncio, timeout (21)</text><text x="24" y="134" class="t-small">GIL, thread (22)</text><text x="24" y="160" class="t-small">Lock, Queue (23)</text><rect x="185" y="20" width="160" height="44" rx="10" class="f-phys-soft s-phys"/><text x="199" y="47" class="t-b">Dữ liệu (24-30)</text><path d="M345 42 L358 42" class="f-none s-phys" stroke-width="2" marker-end="url(#py40-1-arrow)"/><rect x="185" y="80" width="160" height="150" rx="10" class="f-bg s-line"/><text x="199" y="108" class="t-small">PostgreSQL (24, 25)</text><text x="199" y="134" class="t-small">MongoDB (26)</text><text x="199" y="160" class="t-small">Redis (27)</text><text x="199" y="186" class="t-small">test thật (28)</text><text x="199" y="212" class="t-small">polars, numpy (29-30)</text><rect x="360" y="20" width="160" height="44" rx="10" class="f-pk-soft s-pk"/><text x="374" y="47" class="t-b">API (31-37)</text><path d="M520 42 L533 42" class="f-none s-phys" stroke-width="2" marker-end="url(#py40-1-arrow)"/><rect x="360" y="80" width="160" height="150" rx="10" class="f-bg s-line"/><text x="374" y="108" class="t-small">hợp đồng API (31-32)</text><text x="374" y="134" class="t-small">dependency (33)</text><text x="374" y="160" class="t-small">JWT (34), test (36)</text><text x="374" y="186" class="t-small">upload (35)</text><text x="374" y="212" class="t-small">gRPC (37)</text><rect x="535" y="20" width="160" height="44" rx="10" class="f-phys-soft s-phys"/><text x="549" y="47" class="t-b">Thông điệp (38-39)</text><path d="M695 42 L708 42" class="f-none s-phys" stroke-width="2" marker-end="url(#py40-1-arrow)"/><rect x="535" y="80" width="160" height="150" rx="10" class="f-bg s-line"/><text x="549" y="108" class="t-small">Kafka consumer (38)</text><text x="549" y="134" class="t-small">commit sau xử lý (38)</text><text x="549" y="160" class="t-small">outbox, lease (39)</text><text x="549" y="186" class="t-small">inbox khử trùng (39)</text><rect x="710" y="20" width="160" height="44" rx="10" class="f-pk-soft s-pk"/><text x="724" y="47" class="t-b">Việc nền (40)</text><rect x="710" y="80" width="160" height="150" rx="10" class="f-bg s-line"/><text x="724" y="108" class="t-small">arq trên Redis</text><text x="724" y="134" class="t-small">backoff, dead-letter</text><text x="724" y="160" class="t-small">SET NX theo trip_id</text><text x="724" y="186" class="t-small">SIGTERM không mất job</text></svg>
</div>
<figcaption>Hình 2. P03 và P04 theo năm lớp, từ nền bất đồng bộ đến việc nền; số trong ngoặc là số bài. Các lab bài 24 đến 28 và 38 đến 40 cần dịch vụ ngoài (PostgreSQL, MongoDB, Redis, Kafka); các lab còn lại chạy trong tiến trình hoặc trên file cục bộ.</figcaption>
</figure>

Có một sợi chỉ chung: **mỗi chỗ giao việc cho hệ thống khác đều có thể trùng hoặc mất, và cách chữa là ghi ý định hoặc dấu «đã làm» vào nơi bền, trong cùng một thao tác nguyên tử với hiệu ứng.** Ở PostgreSQL đó là UNIQUE và cột `version` (bài 25), ở ranh giới database và Kafka là outbox và inbox (bài 39), ở worker là `SET NX` theo `trip_id` (bài này).

Hai điều chưa nối với nhau: `score_trip` gọi backend chấm điểm **giả** chứ không gọi service gRPC của bài 37, và consumer bài 39 chưa enqueue job vào hàng đợi bài này. Lộ trình nâng cao dùng đúng các mảnh này: bài 41 mở P05 với bài toán KYC tài xế, bài 46 đóng gói OCR thành service có API và worker. Điểm chấm gian lận và kết quả OCR luôn là đầu vào cho người duyệt.

## Chạy và kiểm chứng

Cần Redis của hạ tầng dùng chung đã chạy. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/40-worker-arq-retry-dead-letter examples/toi-noi-python/40-worker-arq-retry-dead-letter/main.py --verify
```

Muốn chạy riêng một worker, dùng đúng lệnh của arq. Lệnh `arq` tìm module `app` theo thư mục hiện tại, nên cần `--directory`:

```bash
uv run --directory examples/toi-noi-python/40-worker-arq-retry-dead-letter arq app.worker.WorkerSettings
```

arq không cho đổi tiền tố key nội bộ (`arq:job:`, `arq:result:`), nên lab đặt `job_id` có chữ `lab40` và dọn bằng hai mẫu `lab40:*` và `arq:*lab40*`. Output đầy đủ lần chạy cuối (các dòng `arq:` là log của arq trong ba job kể trên):

```text
Máy: load average 1/5/15 phút = 50.06 / 53.65 / 47.74
Backoff lần 1..5 (base 0,2 giây, trần 2 giây): [0.2, 0.4, 0.8, 1.6, 2.0]
TN-0001: success=True, lần thử 1, điểm 0.070, hiệu ứng = 1
  arq:   0.01s → lab40:score:TN-0002:score_trip('TN-0002', '5')
  arq:   0.00s ↻ lab40:score:TN-0002:score_trip retrying job in 0.20s
  arq:   0.31s → lab40:score:TN-0002:score_trip('TN-0002', '5') try=2
  arq:   0.00s ↻ lab40:score:TN-0002:score_trip retrying job in 0.40s
  arq:   0.82s → lab40:score:TN-0002:score_trip('TN-0002', '5') try=3
  arq:   0.00s ← lab40:score:TN-0002:score_trip ● {'score': '0.090', 'reasons': [], 'model_version': 'rules-2026.11.1'}
TN-0002: lỗi 2 lần rồi thành công: success=True, lần thử 3, hàm được gọi 3 lần, tổng thời gian 0.92 giây (chờ tối thiểu 0,2 + 0,4)
  arq:   0.01s → lab40:score:TN-0003:score_trip('TN-0003', '10')
  arq:   0.00s ↻ lab40:score:TN-0003:score_trip retrying job in 0.20s
  arq:   0.31s → lab40:score:TN-0003:score_trip('TN-0003', '10') try=2
  arq:   0.00s ↻ lab40:score:TN-0003:score_trip retrying job in 0.40s
  arq:   0.82s → lab40:score:TN-0003:score_trip('TN-0003', '10') try=3
  arq:   0.00s ↻ lab40:score:TN-0003:score_trip retrying job in 0.80s
  arq:   1.63s → lab40:score:TN-0003:score_trip('TN-0003', '10') try=4
  arq:   0.00s ! lab40:score:TN-0003:score_trip failed, TransientScoringError: backend 503 ở lần chạy 4
TN-0003: backend luôn lỗi: success=False, lần thử 4, dead-letter 1 phần tử: max_tries / TransientScoringError: backend 503 ở lần chạy 4
TN-0004: dữ liệu sai: hàm gọi 1 lần, dead-letter 2 phần tử, lý do mới nhất: permanent
TN-0013: distance_km = 'NaN': hàm gọi 1 lần, lý do dead-letter: permanent
TN-0012: lỗi không lường trước: success=False, hàm gọi 1 lần, lý do dead-letter: unexpected
  arq:   0.01s → lab40:score:TN-0011:score_trip('TN-0011', '3')
  arq:   2.00s ↻ lab40:score:TN-0011:score_trip retrying job in 0.20s
  arq:   2.25s → lab40:score:TN-0011:score_trip('TN-0011', '3') try=2
  arq:   0.01s ← lab40:score:TN-0011:score_trip ● {'score': '0.074', 'reasons': [], 'model_version': 'rules-2026.11.1'}
TN-0011: backend treo ở lần 1: success=True, lần thử 2, hiệu ứng = 3
Enqueue lại cùng job_id của TN-0001: None
Job khác id, cùng chuyến TN-0001: success=True, duplicate=True, hiệu ứng = 3
TN-0006 có hai job chạy đồng thời: hàm gọi 2 lần, hiệu ứng = 4
Redrive: đưa lại 1 job, success=True, lần thử 1; dead-letter còn ['TN-0012', 'TN-0013', 'TN-0004']
TN-0005 enqueue với _defer_by=1 giây: bắt đầu chạy sau 1.03 giây
Cron chạy ngay khi worker khởi động: queued=1, dead_letters=3
SIGTERM khi TN-0008 đang chạy (chờ tối đa 10 giây): worker thoát mã 0 sau 1.57 giây
  log worker: shutdown on SIGTERM ◆ 1 jobs complete ◆ 0 failed ◆ 0 retries ◆ 1 to be completed
  log worker: shutdown on SIGTERM, wait complete ◆ 2 jobs complete ◆ 0 failed ◆ 0 retries ◆ 0 ongoing to cancel
  TN-0008: success=True, lần thử 1; TN-0010 gửi sau SIGTERM: runs=None
SIGTERM không chờ: worker thoát mã 0; job TN-0009 xếp lại hàng đợi=True, chưa có kết quả=True, chưa có điểm=True
  log worker: shutdown on SIGTERM ◆ 2 jobs complete ◆ 0 failed ◆ 0 retries ◆ 1 ongoing to cancel
  log worker: 0.02s ↻ lab40:score:TN-0009:score_trip cancelled, will be run again
Worker mới: TN-0009 success=True, lần thử 2; TN-0010 success=True; hiệu ứng tổng = 9
Đã dọn 45 key (lab40:* và arq:*lab40*), còn lại 0
OK 26/26
```

`--verify` chạy 26 kiểm tra, in `OK 26/26` khi đạt; sai thì in tên ca, `FAIL n/m` (n là số ca đạt) và exit 1; không nối được Redis thì in `FAIL: không nối được Redis tại <url> ...` và exit 1. Thời gian phụ thuộc máy (macOS Apple silicon, Redis trong Docker); lab in load average ở dòng đầu, lần chạy cuối khoảng 50 trên máy dùng chung đang bận, nên `--verify` chỉ kiểm cận dưới. Ta chạy lab ba lần liên tiếp, cả ba `OK 26/26`. Thử phá bảy chỗ đều ra `FAIL`: bỏ `nx=True` (23/26), bỏ ghi dead-letter ở lượt cuối (7/8), bỏ `defer` của `Retry` (25/26), đặt thời gian chờ shutdown bằng 0 (21/22), bỏ `is_finite()` (24/26), bỏ hạn chót riêng của backend (21/26), bỏ nhánh `except Exception` (23/26); mẫu số nhỏ (8, 22) là do kịch bản dừng giữa chừng ở ca hỏng. Phiên bản: Python 3.14.7, arq 0.28.0, redis-py 5.3.1, Redis 7.4.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Job thất bại mà không được thử lại | Hàm có `raise Retry(...)` không; arq không thử lại exception thường |
| Job thất bại nhưng DLQ trống | Dead-letter chỉ được ghi ở code của ta; kiểm nhánh lần thử cuối, lỗi vĩnh viễn, exception lạ, và job chạm `job_timeout` |
| `enqueue_job` trả `None` | Còn khóa job hoặc kết quả của `job_id` đó (`keep_result`); dùng id mới khi đưa lại |
| Job chạy hai lần, hiệu ứng hai lần | Hiệu ứng có thao tác nguyên tử kiểu `SET NX` không |
| Sau khi deploy, job đang chạy bị hủy | `job_completion_wait` có lớn hơn job dài nhất không, và nhỏ hơn grace period của hệ thống không |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Lab chạy một Redis một node, tối đa hai worker, backend giả có lỗi theo kế hoạch, backoff tính bằng mili-giây. Dead-letter nằm trong Redis, vốn không phải nguồn sự thật (bài 27): Redis khởi động lại mà không lưu bền là mất DLQ, nên production nên giữ nó ở PostgreSQL hoặc Kafka. Ta đo riêng một lần, ngoài <code>--verify</code>, cho worker bị <code>kill -9</code> giữa job: job được nhận lại sau 20,1 giây (<code>job_timeout</code> 10 giây cộng 10 giây của arq) ở <code>job_try</code> 2. Theo mã nguồn arq, nếu lượt nhận lại vượt <code>max_tries</code> thì job ra «max N retries exceeded» mà không qua code dead-letter của ta (ca này chưa chạy; cách vá là cron quét kết quả thất bại, lab không làm). Job chạm <code>job_timeout</code> cũng không vào dead-letter; lab chặn bằng hạn chót riêng của backend. Chưa đo thông lượng ở 70 chuyến mỗi giây, chưa có giám sát, chưa thử Redis Cluster hay Sentinel, chưa chạy taskiq. Điểm chấm là dữ liệu giả.</</p>
</div>

**Bài tập:** (1) Vì sao `score_trip` tự ghi dead-letter ở lần chạy thứ `MAX_TRIES` thay vì cứ `raise Retry` rồi để arq báo hết lượt? (2) Hai producer cùng enqueue job cho `TN-0007` với hai `job_id` khác nhau, hai worker cùng nhận. Kết quả cuối cùng là gì và nhờ lớp nào?

<details><summary>Xem lời giải</summary><p>(1) Khi <code>job_try</code> vượt <code>max_tries</code>, arq ghi kết quả «max retries exceeded» mà không gọi hàm của ta, nên không còn dịp ghi dead-letter nào. Hàm phải tự đếm lần chạy và ghi dead-letter ở lần cuối, trước khi ném lỗi. (2) Lớp 1 không chặn được vì hai id khác nhau, nên cả hai job chạy, có thể song song. Kết quả đúng nhờ lớp 2: <code>SET NX</code> chỉ cho một job ghi điểm và tăng bộ đếm; job còn lại trả <code>duplicate</code> hoặc ghi thua. Cả hai job đều «thành công», hiệu ứng chỉ có một.</p></details>

Bài trước: [bài 39](/articles/python-outbox-inbox-idempotent-consumer). Bài tiếp: [bài 41](/articles/python-kyc-tai-xe-bai-toan-du-lieu-metric).

## Tài liệu gốc

- [arq: tài liệu chính thức](https://arq-docs.helpmanual.io/).
- [arq trên GitHub (README ghi chế độ chỉ bảo trì)](https://github.com/python-arq/arq).
- [taskiq: tài liệu chính thức](https://taskiq-python.github.io/).
- [Redis: lệnh SET (tùy chọn NX)](https://redis.io/docs/latest/commands/set/).
- [Redis: lệnh LPUSH](https://redis.io/docs/latest/commands/lpush/).
- [redis-py: tài liệu chính thức](https://redis.readthedocs.io/en/stable/).
- [Python 3.14: module signal](https://docs.python.org/3.14/library/signal.html).
