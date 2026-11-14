---
slug: python-real-time-scoring-feature-store-redis-latency
title: "Chấm điểm thời gian thực: feature store trên Redis, ngân sách độ trễ, fallback"
summary: "Bài 59: đưa bộ chấm điểm gian lận vào phục vụ: 30 đặc trưng trong Redis, ngân sách 50 ms bằng asyncio.timeout, điểm mặc định kèm cờ degraded, đo p50 và p99, cầu dao và từ chối bớt tải. Lab kiểm bằng độ trễ tiêm vào."
category: python-advanced
level: advanced
tags: [python, toi-noi, fraud, redis, latency]
language: vi
---

Mô hình ở [bài 57](/articles/python-scikit-learn-lightgbm-imbalanced-cross-validation) chạy tốt trên một bảng 66.704 chuyến. Nhưng mỗi chuyến của [Tới Nơi](/articles/du-an-mau-toi-noi) chỉ có một khoảnh khắc: lúc thu tiền, luồng thanh toán và Dispatch hỏi service fraud-scoring «chuyến này có đáng để người duyệt xem không» rồi đi tiếp. Ở cao điểm khoảng 70 chuyến mỗi giây, không ai chờ một service chậm hay kẹt hẳn. Để chấm, service phải biết khách này đã đi mấy chuyến trong giờ qua, tài xế vừa kết thúc chuyến trước ở đâu: những con số nằm ở một kho đọc nhanh, và kho đó có thể chậm, mất dữ liệu hay hỏng.

Đây là bài 59 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần quen 30 đặc trưng ở [bài 55](/articles/python-feature-engineering-tu-event-kafka), Redis ở [bài 27](/articles/python-redis-cache-rate-limit-geo) và deadline gRPC ở [bài 37](/articles/python-grpc-fraud-scoring-service). Điểm rủi ro là đầu vào cho người duyệt, không phải quyết định khoá tài khoản. «Không chặn Dispatch» ở đây có nghĩa cụ thể: người gọi (luồng thu tiền hay Dispatch) đặt hạn 50 ms; hết hạn mà chưa có điểm thì nó đi tiếp với kết quả `degraded` thay vì chờ. Mô hình của loạt bài chấm lúc thu tiền (`PaymentCaptured`); điểm lúc khách mới đặt chuyến cần mô hình khác, chỉ dùng đặc trưng biết lúc đặt, và lab không làm.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Service bọc việc chấm điểm trong <code>asyncio.timeout</code> với ngân sách ngắn. Một bản mô hình chạy thẳng trong event loop và mất 1 giây vì tính toán thuần CPU. Người gọi có nhận được câu trả lời đúng hạn không? Nếu chuyển mô hình sang một luồng (<code>to_thread</code>) thì sao, và luồng bị bỏ lại có dừng không?</p>
<details><summary>Xem đáp án</summary><p><b>Không, nếu chạy thẳng trong loop.</b> Timeout chỉ ngắt được ở một <code>await</code>; khi coroutine đang tính CPU, loop không có cơ hội xử lý hạn chót, nên lời gọi kết thúc bình thường sau cả giây, không có lỗi nào. Chuyển sang luồng thì người gọi nhận câu trả lời đúng hạn, nhưng <b>luồng không bị huỷ</b>: nó chạy hết 1 giây rồi mới xong. Bài chạy cả hai và in số thật ở mục «Khi hạ tầng hỏng».</p></details>
</div>

## Online và offline: cùng 30 đặc trưng, hai nơi tính

Lúc huấn luyện, bài 55 tính đặc trưng theo lô bằng polars trên cả bảng. Lúc phục vụ, từng chuyến đến một mình và cần cùng 30 con số đó trong vài mili giây. Nếu hai nơi tính khác nhau (training-serving skew), điểm kém đi mà không có lỗi nào; lab bắt hai nơi so với nhau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 370" role="img" aria-label="Đường ghi: ba topic Kafka vào Updater, Updater gom ba event thành một chuyến rồi ghi bằng một script Lua vào Redis gồm sorted set cửa sổ, hash trạng thái và mốc applied_until. Đường đọc: Dispatch hoặc luồng thu tiền gọi Scorer với hạn 50 mili giây; Scorer đọc Redis bằng một pipeline chín lệnh rồi chạy mô hình trong một luồng. Phía offline là bảng polars của bài 55 cùng 30 cột, dùng để huấn luyện và đối chiếu với Redis."><defs><marker id="py59-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="22" class="t-b">Đường ghi, từ event</text><rect x="20" y="36" width="200" height="104" rx="10" class="f-surface s-line"/><text x="36" y="62" class="t-b">Kafka</text><text x="36" y="86" class="t-small">booking.created.v1</text><text x="36" y="104" class="t-small">trip.completed.v1</text><text x="36" y="122" class="t-small">payment.captured.v1</text><path d="M220 88 L286 88" class="f-none s-phys" stroke-width="2" marker-end="url(#py59-0-arrow)"/><rect x="290" y="36" width="200" height="104" rx="10" class="f-pk-soft s-pk"/><text x="306" y="62" class="t-b">Updater</text><text x="306" y="86" class="t-small">TripAssembler: ba event</text><text x="306" y="104" class="t-small">thành một ClosedTrip</text><text x="306" y="122" class="t-small">record_trip: 1 script Lua</text><path d="M490 88 L556 88" class="f-none s-phys" stroke-width="2" marker-end="url(#py59-0-arrow)"/><rect x="560" y="36" width="300" height="184" rx="10" class="f-phys-soft s-phys"/><text x="576" y="62" class="t-b">Redis, tiền tố lab59:</text><rect x="576" y="74" width="268" height="30" rx="6" class="f-surface s-line"/><text x="588" y="94" class="t-small">win:dev, cust, drv, pair (sorted set)</text><rect x="576" y="112" width="268" height="30" rx="6" class="f-surface s-line"/><text x="588" y="132" class="t-small">feat:CU-..., DR-... (hash)</text><rect x="576" y="150" width="268" height="30" rx="6" class="f-surface s-line"/><text x="588" y="170" class="t-small">meta (mốc đã ghi)</text><text x="576" y="206" class="t-small">đọc theo [t - W, t), loại chính chuyến</text><text x="20" y="252" class="t-b">Đường đọc, khi chấm điểm</text><rect x="20" y="266" width="200" height="90" rx="10" class="f-pk-soft s-pk"/><text x="36" y="292" class="t-b">Người gọi</text><text x="36" y="316" class="t-small">Dispatch hoặc luồng thu tiền,</text><text x="36" y="334" class="t-small">chờ tối đa 50 ms</text><path d="M220 311 L286 311" class="f-none s-phys" stroke-width="2" marker-end="url(#py59-0-arrow)"/><rect x="290" y="266" width="200" height="90" rx="10" class="f-warn-soft s-warn"/><text x="306" y="292" class="t-b">Scorer</text><text x="306" y="316" class="t-small">asyncio.timeout 50 ms</text><text x="306" y="334" class="t-small">mô hình trong to_thread</text><path d="M490 296 L592 224" class="f-none s-phys" stroke-width="2" marker-end="url(#py59-0-arrow)"/><text x="590" y="246" class="t-small">1 pipeline, 9 lệnh</text><rect x="560" y="266" width="300" height="90" rx="10" class="f-surface s-line dash"/><text x="576" y="292" class="t-b">Offline, bài 55</text><text x="576" y="316" class="t-small">polars theo lô, cùng 30 cột</text><text x="576" y="334" class="t-small">dùng để huấn luyện và đối chiếu</text><path d="M740 266 L740 224" class="f-none s-muted dash" stroke-width="2"/><text x="752" y="250" class="t-small">đối chiếu</text></svg>
</div>
<figcaption>Hình 1. Hai đường đi qua cùng một kho. Đường ghi nuôi Redis từ event; đường đọc có hạn chót và phải trả lời dù Redis hay mô hình có chậm. Bảng theo lô ở phía offline là mốc để kiểm đường online.</figcaption>
</figure>

Redis là kho đặc trưng online, và quy tắc của bài 27 vẫn đúng: nó không phải nguồn sự thật. Mất dữ liệu thì dựng lại bằng cách phát lại event theo thứ tự `paid_at`. Bộ tính online của bài 55 (`online.py`) chỉ phủ 14 trong 30 cột và thiếu `cust_promo_share_30d`; feature store của bài này phủ đủ 30 cột và lab in thẳng phạm vi đó:

```text
feature store phủ 30/30 cột: 13 từ chính các event của chuyến, 14 từ trạng thái Redis (đúng bộ online.py của bài 55), 3 bổ sung ở bài này (cust_promo_share_30d, impossible_speed, impossible_jump); cột mô hình cần mà store thiếu: []
```

Mô hình đòi cột mà store không phủ thì `FeatureStore.missing_for` trả tên cột và `Scorer` từ chối khởi tạo (lab thử ở bước 5). Điền 0 cho cột thiếu là cách sai kín đáo nhất: điểm vẫn ra, nhưng là của một chuyến không có thật.

## Mô hình dữ liệu trong Redis

Một đặc trưng như `cust_trips_1h` phụ thuộc thời điểm hỏi, nên lưu nó thành một con số trong hash là sai sau một phút. Cách đúng là lưu **thời điểm** của từng chuyến rồi đếm lúc đọc: mỗi khoá thiết bị, khách, tài xế và cặp khách-tài xế là một sorted set có score là `paid_at` tính bằng micro giây từ epoch. Phần không phụ thuộc thời điểm (số chuyến của khách, chuyến cuối của tài xế) nằm trong hash `lab59:feat:{CU-...|DR-...}`. Đọc là chín lệnh trong một pipeline, một vòng mạng (`store.py`, trích):

```python
pipe = self.r.pipeline(transaction=False)
pipe.zrangebyscore(key_dev(t.device_id), s - KEEP_DEV, s, withscores=True)
pipe.zrangebyscore(key_cust(t.customer_id), s - KEEP_CUST, before, withscores=True)
pipe.zcount(key_drv(t.driver_id), s - US_H, before)
pipe.zcount(key_drv(t.driver_id), s - US_D, before)
pipe.zcount(key_pair(t.customer_id, t.driver_id), s - 7 * US_D, before)
pipe.hmget(key_feat(t.customer_id), "prior_trips", "last_us")
pipe.hmget(key_feat(t.driver_id), "paid_us", "done_us", "end_lat", "end_lng", "prev_done_us", "prev_end_lat", "prev_end_lng")
pipe.hmget(KEY_META, "applied_until", "history_from")
pipe.zscore(key_cust(t.customer_id), SEP.join((t.trip_id, str(int(t.promo_code is not None)))))
```

`before` là `f"({s}"`: dấu ngoặc của Redis đánh dấu cận loại trừ, nên cửa sổ là `[s - W, s)`, đúng với `closed="left"` của bản polars. Nhưng cận loại trừ chỉ loại chính chuyến khỏi các **sorted set**. Hash `prior_trips` và chuyến cuối của tài xế là trạng thái mới nhất: nếu Updater đã ghi chuyến trước khi nó được chấm, chuyến thấy chính mình (`cust_prior_trips` thêm 1, nhảy GPS và tốc độ sai). `read_features` xử lý ba trạng thái (trích):

```python
if (cust_last is not None and cust_last > s) or (drv_paid is not None and drv_paid > s):
    state = "ahead"  # đã có chuyến sau: hash không khôi phục được, người gọi phải bỏ kết quả
elif mine is not None:  # Updater đã ghi chính chuyến này và nó là chuyến mới nhất: trừ nó khỏi hash
    state = "latest"
    prior_n -= 1
    if drv_paid == s:  # chuyến cuối của tài xế chính là chuyến này: dùng chuyến liền trước nó
        last = drv[4:7]
```

Để làm được điều đó, script ghi giữ chuyến cuối cũ làm `prev_*`. Kafka giao ít nhất một lần nên cùng chuyến có thể đến hai lần, và một chuyến phải đổi sáu khoá cùng lúc. Một script Lua giải cả hai: Redis chạy nó liền một mạch, và `ZADD` đầu tiên vào sorted set khách trả 0 nếu mã chuyến đã có, khi đó script thoát (`store.py`, trích):

```python
_RECORD_LUA = """
local score = ARGV[1]
if redis.call('ZADD', KEYS[2], score, ARGV[3]) == 0 then return 0 end   -- đã ghi chuyến này rồi
redis.call('ZADD', KEYS[1], score, ARGV[2])
...
redis.call('HINCRBY', KEYS[5], 'prior_trips', 1)
...
  if last then                                                          -- giữ chuyến cuối cũ làm «chuyến liền trước»
...
"""
```

Điểm số và mốc cắt đi vào script dưới dạng chuỗi, không phải số, vì Lua của Redis đổi số thành chuỗi bằng `%.14g` còn một mốc micro giây có 16 chữ số. Lab thử ngay trên Redis đang chạy (bước 6):

```text
Lua của Redis: tostring(tonumber('1773000000123456') - 5) = 1.7730000001235e+15; chuỗi nối '(' .. ARGV[1] = (1773000000123456
```

Số thứ nhất đã mất chữ số (đáng ra là 1773000000123451). Lab phát lại 600.000 envelope thành 200.000 chuyến và ghi vào Redis theo thứ tự thời gian. Từ ngày 40, cứ 20 chuyến thì lấy một chuyến: đọc đặc trưng **trước khi ghi nó**, so với bảng polars của bài 55, chấm điểm và so với điểm theo lô của cùng mô hình; sau đó ghi chuyến và đọc lại, kết quả phải y hệt:

```text
ghi 133,296 chuyến trước ngày 40: 15.5 giây (8,577 chuyến/giây, 500 chuyến mỗi pipeline, 1 script Lua mỗi chuyến)
đối chiếu đặc trưng online (Redis) với theo lô (polars): 3,336 chuyến x 30 đặc trưng = 100,080 giá trị, 0 giá trị lệch (rel_tol 1e-9)
đối chiếu điểm online với điểm theo lô của cùng mô hình: 3,336 chuyến, chênh lớn nhất 0.00e+00
ghi chuyến rồi đọc lại ngay: 3,336/3,336 chuyến cho đặc trưng y hệt lần đọc trước khi ghi; trạng thái {'latest': 3336}
Redis sau khi phát lại: 302,194 key, used_memory tăng 47.5 MiB (máy dùng chung, số xấp xỉ)
```

Phép đối chiếu này chứng minh hai nơi dùng **cùng định nghĩa** khi trạng thái trong Redis đúng; nó không chứng minh sự khớp ở production, nơi event đến muộn, trùng hay mất. Tốc độ dựng lại dao động mạnh theo tải: lần này 8.577 chuyến mỗi giây, các lần chạy khác của lab trong lúc viết bài cho từ 2.502 đến 12.936 (không dán đủ output). Với 60 triệu chuyến của cửa sổ 30 ngày ở 2 triệu chuyến mỗi ngày, khoảng đó ứng với 1,3 đến 6,7 giờ trên một tiến trình; chia song song lab chưa đo.

Dựng lại còn có hai cái bẫy. Store trống (Redis khởi động lại mất dữ liệu) làm mọi đặc trưng thành 0: mốc `meta` bắt ca này và `Scorer` trả `store_cold`. Store đang dựng từ lịch sử ngắn thì cửa sổ 30 ngày còn thiếu: `history_from` ghi chuyến đầu tiên store thấy, và dưới 30 ngày lịch sử `Scorer` trả `store_building`. Ngưỡng này chỉ bảo vệ các cửa sổ; `prior_trips` đúng khi phát lại từ đầu hoặc nạp bù từ bảng offline.

## Ngân sách 50 ms: trả lời đúng hạn, kể cả khi không chấm được

Ngân sách là một hợp đồng, nên nó nằm trong code ở một chỗ (`scorer.py`, trích):

```python
async with asyncio.timeout(budget):  # chỉ ngắt được tại một `await`
    got = await self.store.read_features(trip)
    store_answered = True
    ...
    if got.applied_until_us is None:  # store trống: mọi đặc trưng sẽ là 0, điểm thấp giả
        self._store_ok()
        return self._degraded(trip, "store_cold", t0, read_ms)
```

`Scorer.score()` không ném lỗi lên người gọi. Khi hết hạn, mất kết nối, store trống hay mới dựng, chuyến cũ đã có chuyến sau, mô hình từ chối đầu vào hay quá tải, nó trả `ScoreResult` với `degraded=True`, một `reason` ổn định và điểm mặc định: tỉ lệ gian lận nền của mô hình (0,0171), kèm `flagged=False`. Đây là quyết định «mở cửa khi lỗi» (fail-open); như bài 27 nhắc cho rate limit, hãy ghi nó ra từ trước. Hệ quả: `degraded=True` nghĩa là **chưa có điểm**, không phải «chuyến sạch»; bên nhận cần xếp những chuyến này vào đợt chấm bù theo lô (đường polars của bài 55) và đếm tỉ lệ `degraded` như một chỉ số sức khoẻ. Khi chấm được, `flagged` theo quy tắc chung của loạt bài (`model.flag`): p × (tiền chuyến + 20.000) > 8.000; ngưỡng toàn cục 0,149 chỉ để so sánh.

Người gọi còn có hạn của riêng mình: `score(trip, deadline_s=...)` lấy `min(ngân sách, deadline_s)`; với gRPC, `context.time_remaining()` (bài 37) là giá trị tự nhiên cho tham số này.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Trục thời gian từ 0 đến 160 mili giây. Bốn thanh là p50 3,08 ms, p90 7,21 ms, p99 28,10 ms và max 137,24 ms của tổng thời gian trong score() cho mọi chuyến trong một lần đo. Một vạch ở 50 mili giây là ngân sách: p50, p90 và p99 nằm trước vạch, max nằm sau vạch."><text x="20" y="30" class="t-b">Thời gian trong score(), mọi chuyến, một chuyến mỗi lần (bước 4 của lab)</text><text x="20" y="72" class="t-small">p50</text><rect x="120" y="56" width="14" height="24" rx="4" class="f-phys-soft s-phys"/><text x="144" y="73" class="t-small">3,08 ms</text><text x="20" y="112" class="t-small">p90</text><rect x="120" y="96" width="32" height="24" rx="4" class="f-phys-soft s-phys"/><text x="162" y="113" class="t-small">7,21 ms</text><text x="20" y="152" class="t-small">p99</text><rect x="120" y="136" width="127" height="24" rx="4" class="f-phys-soft s-phys"/><text x="257" y="153" class="t-small">28,10 ms</text><text x="20" y="192" class="t-small">max</text><rect x="120" y="176" width="618" height="24" rx="4" class="f-warn-soft s-warn"/><text x="748" y="193" class="t-small">137,24 ms</text><path d="M345 44 L345 214" class="f-none s-warn" stroke-width="2"/><text x="353" y="212" class="t-small t-warn">50 ms: ngân sách</text><path d="M120 224 L840 224" class="f-none s-muted"/><text x="113" y="244" class="t-mono">0</text><text x="293" y="244" class="t-mono">40</text><text x="473" y="244" class="t-mono">80</text><text x="653" y="244" class="t-mono">120</text><text x="833" y="244" class="t-mono">160</text><text x="20" y="278" class="t-small">Chuyến vượt vạch 50 ms bị huỷ và trả degraded; max là một chuyến timeout vẫn đến sau hạn.</text></svg>
</div>
<figcaption>Hình 2. Phân phối thời gian của một lần chấm điểm trong lần chạy bài. Phần lớn chuyến xong rất sớm; đuôi dài mới là thứ ngân sách phải cắt.</figcaption>
</figure>

## Đo p50 và p99

Bước 3 của lab chấm 3.336 chuyến (một trong mỗi 20 chuyến của giai đoạn valid), mỗi lần một chuyến, và bước 4 in thời gian từng giai đoạn. Load average lúc đo khoảng 90 đến 125 trên 10 lõi, nên đây là các số của một máy ồn, không phải máy rảnh:

```text
giai đoạn (ms)                                  mẫu      p50      p90      p99       max
PING, client riêng (không bao giờ bị hủy)     3,336     0.57     1.57    10.61     49.03
PING trên pool chấm điểm, sau lời gọi ok      3,327     0.56     1.59    10.05     31.94
PING trên pool chấm điểm, sau lời gọi timeout      9    13.62    19.52    21.49     21.71
độ trễ event loop (ngủ 5 ms, thức trễ)       10,366     0.39     2.66    10.70    133.60
đọc Redis + dựng đặc trưng (chỉ chuyến ok)    3,327     0.87     2.33    14.17     39.66
mô hình + chuyển luồng (chỉ chuyến ok)        3,327     2.18     4.06    13.93     30.73
tổng score(), mọi chuyến                      3,336     3.08     7.21    28.10    137.24
tổng score(), chuyến ok                       3,327     3.08     7.03    24.38     47.11
tổng score(), chuyến timeout                      9    52.84    91.43   132.66    137.24
kết quả: {'ok': 3327, 'timeout': 9}; load average lúc đo 88.52 125.56 194.86
```

Ba điều đọc ra. Một: lần chấm điển hình mất 3,08 ms; hai dòng giai đoạn chỉ gồm chuyến `ok` nên bị cắt bởi ngân sách, và phân vị không cộng được thành phân vị của tổng. Hai: 9 trong 3.336 chuyến (0,27 %) chạm ngân sách, mất 52,84 ms ở trung vị và tới 137,24 ms. Event loop cũng có lúc thức dậy trễ tới 133,60 ms; hai con số cùng cỡ gợi ý hạn chót đến muộn vì loop không được cấp CPU, nhưng lab chưa chứng minh. Ba: PING trên client riêng (p50 0,57 ms) khác PING ngay sau một lời gọi timeout trên pool chấm điểm (p50 13,62 ms), nhưng nhóm sau chỉ có 9 mẫu nên chỉ là gợi ý rằng kết nối phải nối lại; mã nguồn `read_response` của redis-py 8.1.0 đóng kết nối khi bị huỷ giữa lúc chờ phản hồi.

Thứ ít phụ thuộc tải hơn thời gian trôi qua là CPU tiêu thụ. Lab đo 500 chuyến mới tuần tự với ngân sách nới, rồi gửi 70 chuyến mỗi giây trong 10 giây theo lịch cố định, tính độ trễ **từ lịch** (không phải từ lúc coroutine bắt đầu) để không giấu việc người gọi bị trễ lịch:

```text
CPU mỗi chuyến, một tiến trình, 500 chuyến tuần tự, ngân sách nới 5 giây: 3.27 ms CPU (3.66 ms thời gian trôi qua); CPU chiếm 89% thời gian trôi qua, phần còn lại chưa tách (chờ I/O, chờ được cấp CPU); nếu chỉ tính CPU thì tối đa khoảng 306 chuyến/giây
giai đoạn (ms)                                  mẫu      p50      p90      p99       max
từ lịch đến xong, mọi chuyến                    700    17.47    47.91    93.31    130.40
lệch lịch lúc bắt đầu gọi                       700     2.05     9.12    37.11     68.50
tốc độ phát thực tế 69.6 chuyến/giây (đặt 70); kết quả: {'ok': 671, 'timeout': 29}; load average sau đo 86.27 123.84 193.43
```

Mỗi chuyến tốn 3,27 ms CPU, nên 70 chuyến mỗi giây dùng cỡ 23 % một lõi nếu chỉ tính CPU; vậy mà 29 trên 700 lời gọi (4,1 %) bị `degraded` và p99 từ lịch tới xong là 93,31 ms. Khi nhiều lời gọi chồng nhau, mỗi lời gọi chờ thêm: trong 50 ms một tiến trình chỉ phục vụ nổi cỡ 15 chuyến chồng nhau. Lab thử đợt 200 chuyến đến cùng một lúc; mỗi cấu hình dùng client Redis mới, đổi một thứ so với A, và in số kết nối của pool trước và sau:

```text
200 chuyến đến cùng một lúc, mỗi cấu hình dùng client Redis mới (đổi một thứ so với cấu hình đầu), load average 86.27 123.84 193.43:
  A. pool 16 kết nối, mở sẵn 16
    pool trước: 16 kết nối, 16 đang nối, tối đa 16; sau: 16 kết nối, 0 đang nối, tối đa 16
    xong sau 64 ms; p50 57.6, p99 59.8 ms; {'ok': 4, 'timeout': 196}
  B. như A nhưng chưa mở sẵn kết nối nào
    pool trước: 0 kết nối, 0 đang nối, tối đa 16; sau: 16 kết nối, 7 đang nối, tối đa 16
    xong sau 90 ms; p50 55.6, p99 65.3 ms; {'timeout': 199, 'ok': 1}
  C. như A, thêm tối đa 8 lời gọi cùng lúc
    pool trước: 16 kết nối, 16 đang nối, tối đa 16; sau: 16 kết nối, 16 đang nối, tối đa 16
    xong sau 22 ms; p50 0.0, p99 20.2 ms; {'ok': 8, 'overloaded': 192}
  D. pool mặc định của redis-py, chưa mở sẵn
    pool trước: 0 kết nối, 0 đang nối, tối đa 100; sau: 100 kết nối, 0 đang nối, tối đa 100
    xong sau 119 ms; p50 26.2, p99 69.7 ms; {'timeout': 100, 'store_error': 100}; lỗi store: {'MaxConnectionsError': 100}
```

Đọc từng cặp. A so với B chỉ khác việc mở sẵn kết nối: cả hai gần như toàn timeout, nên mở sẵn không cứu được đợt 200 lời gọi. A so với C chỉ khác giới hạn 8 lời gọi cùng lúc: C trả 8 điểm thật trong 22 ms và từ chối 192 lời gọi ngay, pool giữ nguyên 16 kết nối vì không lời gọi nào bị huỷ giữa chừng; ở A pool kết thúc với 0 kết nối đang nối, khớp với việc huỷ giữa chừng đóng kết nối (lab không tách huỷ ở pha đọc Redis với pha mô hình, nơi kết nối đã trả về pool). D khác B ở hai thứ (loại pool và cỡ): pool mặc định của redis-py 8.1.0 giới hạn 100 kết nối chứ không «vô hạn», và 100 lời gọi vượt giới hạn nhận `MaxConnectionsError`. Kết luận chỉ đúng trong bốn cấu hình đã thử; lời giải thật cho đợt tăng tải là gom lô (bài 60 chấm 33.416 chuyến trong một lô mất 0,26 giây) hoặc thêm bản service.

## Khi hạ tầng hỏng: độ trễ tiêm vào

Fallback chỉ có giá trị nếu chạy đúng khi cần. Bước 5 tiêm lỗi vào store và mô hình (`faults.py`) nên kết quả không phụ thuộc tốc độ máy: store «kẹt vĩnh viễn» là một `asyncio.Event().wait()` không bao giờ được set, và `FakeClock` thay `time.monotonic` trong cầu dao:

```text
store kẹt vĩnh viễn:       score=0.017110 degraded=True reason=timeout sau 91 ms; lời gọi bị hủy: 1
chậm 300 ms, ngân sách 50 ms: degraded=True reason=timeout; ngân sách 2 s: degraded=False, score=0.003386
người gọi chỉ còn 5 ms (deadline_s) so với không đặt: degraded=True và False
Redis không nghe ở cổng:   degraded=True reason=store_error sau 6 ms
cầu dao (3 lần hỏng thì mở, chờ 2 giây, đồng hồ giả): ['timeout', 'timeout', 'timeout'] -> breaker_open (store không bị gọi: True)
  sau 2,1 giây: thăm dò -> timeout; ngay sau đó -> breaker_open; store hồi phục, sau 2,1 giây nữa -> ok, rồi ok
store khỏe, mô hình chậm 1,2 s, ngân sách 0,4 s, 5 lời gọi: ['timeout', 'timeout', 'timeout', 'timeout', 'timeout']
```

(trích). Store chậm 300 ms thì `degraded` với ngân sách 50 ms nhưng ra điểm thật với ngân sách 2 giây: ngân sách là chính sách, không phải lỗi của store. Cầu dao (`Breaker`) giải bài toán khác: mỗi lời gọi vào một store treo đốt trọn 50 ms của người gọi. Sau ba lần hỏng liên tiếp nó mở hai giây và trả `breaker_open` mà không đụng store; hết hạn, đúng một lời gọi thăm dò đi qua. Cầu dao chỉ đếm lời gọi mà **store** không kịp trả lời: dòng cuối cho thấy store khoẻ mà mô hình chậm thì năm lời gọi đều `timeout`, không lần nào mở cầu dao.

Rồi đến câu hỏi đầu bài, đo thật, cùng với việc luồng bị bỏ lại:

```text
mô hình chặn 1.000 ms, ngân sách 250 ms: chạy thẳng trong loop -> degraded=False sau 1019 ms; to_thread -> degraded=True sau 250 ms (luồng vẫn chạy hết: True)
max_in_flight=1, mô hình chặn 1,2 s, ngân sách 0,4 s, 5 lời gọi nối tiếp: ['timeout', 'overloaded', 'overloaded', 'overloaded', 'overloaded']; luồng bị bỏ lại 1, mô hình được bắt đầu 1 lần; sau khi luồng xong: 0
```

Chạy thẳng, lời gọi kéo dài 1.019 ms và không có lỗi nào: đồng hồ hỏng mà không ai biết. Qua luồng, người gọi nhận câu trả lời ở 250 ms, nhưng luồng bị bỏ lại vẫn chạy hết (Python không huỷ được luồng): timeout bảo vệ **người gọi**, không giải phóng CPU. Vì thế giới hạn lời gọi cùng lúc phải tính cả luồng còn chạy: `Scorer` đếm chúng (`_stray`) tới khi xong, nên lời gọi thứ hai bị từ chối thay vì thêm một luồng nữa.

Ba kiểu hỏng khác không gây lỗi nào. Store trống hay mới dựng đã nói ở trên. Chấm lại chuyến cũ khi hash đã đi quá nó, mà sorted set không còn thấy chuyến (đã quá cửa sổ): `last_us` của khách và `paid_us` của tài xế bắt ca đó và `Scorer` trả `state_ahead` thay vì một điểm sai. Cuối cùng, nguồn đặc trưng trả NaN hay thiếu cột: mô hình từ chối, và `Scorer` biến lỗi thành `degraded` kèm tên cột:

```text
chấm lại một chuyến cũ (khách đã có chuyến sau): degraded=True reason=state_ahead
đặc trưng speed_kmh là NaN: degraded=True reason=model_error
nguồn thiếu cột cust_promo_share_30d: degraded=True reason=model_error (ModelInputError: thiếu đặc trưng: cust_promo_share_30d)
mô hình đòi thêm cột dev_fraud_known: Scorer từ chối khởi tạo: feature store không phủ các cột mô hình cần: ['dev_fraud_known']
store chỉ có 1 giờ lịch sử: degraded=True reason=store_building; sau 31 ngày lịch sử: degraded=False reason=None
ghi trước, đọc sau: cust_prior_trips 1 -> 1, drv_jump_km 27.14 -> 27.14; trạng thái before -> latest; sau khi có chuyến sau nó: ahead
```

Lab không dùng độ cũ của mốc `applied_until` để phát hiện updater chậm: bước 2 in ra rằng khoảng cách lớn nhất giữa hai lần thu tiền liên tiếp là 94 phút (58 khoảng dài hơn 10 phút), nên «mốc cũ» không phân biệt được ban đêm vắng khách với updater tắc. Độ trễ consumer của Kafka mới là chỉ số đúng.

## Chạy và kiểm chứng

Cần uv, Python 3.14, Redis trong hạ tầng chung của loạt bài (`docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait`) và, trên macOS, `brew install libomp` cho LightGBM. Lần đầu cần mạng, dựng bảng đặc trưng và huấn luyện mô hình (khoảng một phút, lưu vào `out/`). Mọi key có tiền tố `lab59:` và bị xoá khi chạy xong. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/59-real-time-scoring-redis examples/toi-noi-python/59-real-time-scoring-redis/main.py
uv run --project examples/toi-noi-python/59-real-time-scoring-redis examples/toi-noi-python/59-real-time-scoring-redis/main.py --verify
```

Dòng cuối của `--verify` (lần chạy này mất 1 phút 56 giây khi load average khoảng 90 đến 140 trên 10 lõi; máy bận hơn thì mất 9 đến 10 phút):

```text
Dọn: đã xóa 0 key `lab59:*`, còn lại 0

Kiểm chứng: OK 57/57
```

`--verify` kiểm 57 bất biến: đặc trưng và điểm online khớp bản theo lô, kể cả khi chuyến đã được ghi rồi mới đọc; cận cửa sổ; ghi trùng; chuyến đến muộn; store trống, đang dựng, chuyến cũ; mọi hành vi fallback bằng độ trễ tiêm vào và đồng hồ giả; luồng bị bỏ lại; dọn sạch key. Nó không kiểm tốc độ tuyệt đối và không ghim số p50 hay p99.

| Hiện tượng | Kiểm tra trước |
|---|---|
| Điểm đồng loạt thấp sau khi Redis khởi động lại | `reason=store_cold` hay `store_building`; phát lại event theo thứ tự `paid_at` |
| p99 vượt ngân sách dù p50 thấp | CPU mỗi chuyến, số lời gọi chồng nhau và độ trễ event loop; giới hạn `max_in_flight`, gom lô, thêm bản service |
| `store_error` với `MaxConnectionsError` khi tải tăng đột ngột | Cỡ và loại pool kết nối; pool mặc định giới hạn 100 |
| Chấm lại chuyến cũ cho `cust_prior_trips` hay `drv_jump_*` khác bản theo lô | Hash giữ trạng thái mới nhất; `state_ahead` nghĩa là không chấm lại bằng store này |
| `model_error` kèm `ModelInputError` | Cột thiếu hoặc NaN từ nguồn đặc trưng; sửa nguồn, không điền 0 |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Dữ liệu là tổng hợp (200.000 chuyến trong 60 ngày); mọi số đo chỉ nói về bộ này, một seed, một máy bận. Mô hình chạy trong chính tiến trình đo, không qua gRPC, nên không có độ trễ mạng giữa Dispatch và service. Redis là một nút; lab không đo failover, không cấu hình giữ dữ liệu hay loại key khi đầy bộ nhớ (một Redis tự xoá key khi đầy làm đặc trưng biến mất âm thầm mà <code>applied_until</code> không bắt được). Script Lua đụng nhiều key nên trên Redis Cluster phải đặt tên key lại cho cùng slot; updater giả định event đến theo thứ tự thời gian. Chi phí 8.000 và 20.000 VND là giả định của bài 54, kèm hai điều: gắn cờ được xem là chặn hết thiệt hại (người duyệt không sai), và số tiền chuyến đại diện cho thiệt hại (với promo_abuse, thiệt hại thật chỉ là phần giảm giá). Hai tạo tác của bộ sinh ảnh hưởng mọi số mô hình trong loạt bài: chuyến sạch dùng mã NEWUSER50 giảm từ 7,8 % xuống 0,9 % theo thời gian vì quần thể khách gần như cố định, và tài khoản gian lận không dùng mã nào ngoài NEWUSER50 của promo_abuse nên <code>cust_promo_share_30d &gt; 0</code> gần như tố «sạch». Đã chạy với Python 3.14.7, redis-py 8.1.0, Redis server 7.4.12, lightgbm 4.7.0, scikit-learn 1.9.1, numpy 2.5.4, polars 2.1.0.</p>
</div>

**Bài tập:** (1) Mỗi chuyến tốn 3,27 ms CPU và ngân sách là 50 ms. Tối đa bao nhiêu lời gọi chồng nhau, chạy nối tiếp trên một lõi, trước khi lời gọi cuối chắc chắn quá hạn? Ở 70 chuyến mỗi giây, một tiến trình dùng bao nhiêu phần trăm một lõi? (2) Bỏ `cust_promo_share_30d` thì sorted set của khách chỉ cần giữ 24 giờ thay vì 30 ngày. Số mục giảm bao nhiêu lần ở tốc độ chuyến không đổi, và `cust_prior_trips` còn tính được không?

<details><summary>Xem lời giải</summary><p>(1) 50 / 3,27 = 15,3, tức tối đa <b>15</b> lời gọi chồng nhau. Ở 70 chuyến mỗi giây: 70 × 3,27 = 229 ms CPU mỗi giây, khoảng <b>23 %</b> một lõi (chỉ tính CPU; lab vẫn thấy chuyến quá hạn vì chờ và độ ồn của máy). (2) 30 ngày / 24 giờ = <b>30 lần</b> ít hơn. <code>cust_prior_trips</code> <b>vẫn tính được</b> vì là bộ đếm trong hash; nhưng cờ chặn trùng của script dựa vào sorted set khách, nên một event giao lại sau hơn một ngày sẽ bị đếm hai lần.</p></details>

Bài trước: [bài 58](/articles/python-networkx-graph-cau-ket-tai-xe-khach). Bài tiếp: [bài 60](/articles/python-model-registry-ab-drift-retraining).

## Tài liệu gốc

- [Python: asyncio.timeout và asyncio.to_thread](https://docs.python.org/3.14/library/asyncio-task.html).
- [redis-py: ví dụ asyncio](https://redis.readthedocs.io/en/stable/examples/asyncio_examples.html).
- [Redis: sorted set](https://redis.io/docs/latest/develop/data-types/sorted-sets/).
- [Redis: ZRANGEBYSCORE và cận loại trừ](https://redis.io/docs/latest/commands/zrangebyscore/).
- [Redis: giới thiệu script Lua (EVAL)](https://redis.io/docs/latest/develop/interact/programmability/eval-intro/).
- [gRPC: Deadlines](https://grpc.io/docs/guides/deadlines/).
