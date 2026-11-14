---
slug: python-redis-cache-rate-limit-geo
title: "Redis: cache, rate limit và GEO cho vị trí tài xế"
summary: "Bài 27: dùng Redis qua redis-py async cho Tới Nơi: cache-aside báo giá với TTL, token bucket bằng Lua, GEOADD/GEOSEARCH cho 25.000 tài xế, pipeline; Redis không phải nguồn sự thật. Lab chạy thật."
category: python-intermediate
level: intermediate
tags: [python, toi-noi, redis, cache, rate-limit]
language: vi
---

Giờ cao điểm của [Tới Nơi](/articles/du-an-mau-toi-noi) là khoảng 70 chuyến mỗi giây. Mỗi lần khách mở màn hình đặt xe, ứng dụng hỏi báo giá cho một cặp (dịch vụ, khoảng cách), và khách cùng khu vực hỏi các cặp gần giống nhau, vậy mà dịch vụ báo giá tính lại từng lần. Điều phối cần «5 tài xế gần nhất» trong 25.000 tài xế trực tuyến, và một khách bấm đặt liên tục không nên làm cạn tài nguyên của người khác. Cả ba việc cần một nơi trả lời trong cỡ một đến vài mili giây; Redis làm tốt cả ba.

Đây là bài 27 của lộ trình **Python trung bình qua Tới Nơi**. Bài dùng Redis 7 trong hạ tầng dùng chung (`infra/README.md`) qua `redis.asyncio`, nên cần quen `async`/`await` và `gather` ở bài 21. Mọi key của lab có tiền tố `lab27:` và bị xoá khi chạy xong; vị trí tài xế là dữ liệu tổng hợp quanh Hà Nội.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Cache-aside cho báo giá đã chạy ổn. Lúc cache nguội, 20 request cùng cặp (ride, 2,5 km) đến đúng một thời điểm. Dịch vụ báo giá bị gọi bao nhiêu lần?</p>
<details><summary>Xem đáp án</summary><p><b>20 lần.</b> Mỗi request đọc cache và thấy hụt trước khi có request nào kịp ghi kết quả vào, nên cả 20 cùng đi hỏi nguồn. Cache chỉ giúp những request đến <i>sau</i> khi đã có giá trị. Bài chạy đúng ca này rồi sửa bằng một khoá có hạn.</p></details>
</div>

## Redis giữ gì và không giữ gì

Redis là kho cấu trúc dữ liệu trong bộ nhớ: chuỗi, hash, sorted set và vài kiểu khác. Server thực thi lệnh lần lượt từng lệnh, nên mỗi lệnh là nguyên tử và một script Lua cũng chạy liền một mạch. Quy tắc đi kèm: **Redis không phải nguồn sự thật**. Mất toàn bộ dữ liệu của nó có thể làm hệ thống chậm đi hoặc kém chính xác tạm thời, nhưng không được làm sai nghiệp vụ hay mất thứ không dựng lại được.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Ứng dụng Python đọc và ghi Redis gồm ba nhóm key: báo giá có TTL, bộ đếm rate limit và chỉ mục GEO tài xế. Khi cache hụt, ứng dụng hỏi nguồn sự thật; khi Redis mất dữ liệu thì dựng lại từ nguồn sự thật."><defs><marker id="py27-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="110" width="210" height="90" rx="10" class="f-pk-soft s-pk"/><text x="36" y="145" class="t-b">Service Python</text><text x="36" y="170" class="t-small">redis.asyncio</text><rect x="330" y="20" width="330" height="150" rx="10" class="f-phys-soft s-phys"/><text x="346" y="44" class="t-b">Redis</text><rect x="346" y="56" width="298" height="30" rx="6" class="f-surface s-line"/><text x="358" y="76" class="t-small">lab27:quote:v1:... (TTL 30 giây)</text><rect x="346" y="94" width="298" height="30" rx="6" class="f-surface s-line"/><text x="358" y="114" class="t-small">lab27:rl:... (bộ đếm, có TTL)</text><rect x="346" y="132" width="298" height="30" rx="6" class="f-surface s-line"/><text x="358" y="152" class="t-small">lab27:geo:drivers:hanoi (chỉ mục)</text><rect x="330" y="210" width="330" height="72" rx="10" class="f-warn-soft s-warn"/><text x="346" y="236" class="t-b">Nguồn sự thật</text><text x="346" y="260" class="t-small">dịch vụ báo giá, vị trí gốc của tài xế</text><path d="M230 140 L330 95" class="f-none s-phys" stroke-width="2" marker-end="url(#py27-0-arrow)"/><path d="M230 175 L330 240" class="f-none s-phys" stroke-width="2" marker-end="url(#py27-0-arrow)"/><path d="M495 210 L495 172" class="f-none s-phys dash" stroke-width="2" marker-end="url(#py27-0-arrow)"/><text x="508" y="196" class="t-small">dựng lại, làm nóng</text><text x="690" y="60" class="t-b">Mất Redis</text><text x="690" y="84" class="t-small">chậm hơn, ít chính xác</text><text x="690" y="102" class="t-small">hơn, không sai nghiệp vụ</text></svg>
</div>
<figcaption>Hình 1. Redis đứng cạnh nguồn sự thật chứ không thay nó. Ứng dụng hỏi Redis trước; mất hoặc hụt thì quay về nguồn.</figcaption>
</figure>

Ba việc của bài đều theo quy tắc ấy. Báo giá: nguồn sự thật là dịch vụ báo giá, mất cache thì nó bị gọi nhiều hơn. Rate limit: bộ đếm vốn tạm thời, mất thì đếm lại từ đầu. Vị trí tài xế: vị trí gốc nằm ở service tiếp nhận vị trí, mất chỉ mục thì phải dựng lại.

Một quyết định đi kèm: khi Redis không trả lời, rate limit cho qua (fail-open) hay chặn (fail-closed)? API tra cứu và API thanh toán trả lời khác nhau; hãy ghi lại từ trước.

## Cache-aside cho báo giá

**Cache-aside** là mẫu đơn giản nhất: ứng dụng tự đọc cache, hụt thì hỏi nguồn rồi ghi vào cache. Dịch vụ báo giá của lab là `QuoteSource` chờ 50 ms rồi gọi `calculate_fare` (sao chép từ bài 10), có bộ đếm số lần bị gọi; `service` là `ServiceKind` (`StrEnum`: `ride`, `food`, `parcel`). Trích `main.py`:

```python
def normalize_km(distance_km: Decimal) -> Decimal:
    """Làm tròn km về mét TRƯỚC khi vào key và TRƯỚC khi tính giá."""
    return distance_km.quantize(Decimal("0.001"), rounding=ROUND_HALF_UP)


def quote_key(service: ServiceKind, distance_km: Decimal) -> str:
    # km đã chuẩn hóa: 2.5 và 2.50 ra cùng một key. Phiên bản chính sách (v1) nằm trong key:
    # đổi bảng giá thì đổi v2, key cũ tự chết theo TTL.
    return f"{PREFIX}quote:v1:{service.value}:{normalize_km(distance_km)}"


async def get_quote(r, source, service, distance_km, *, ttl_ms=30_000):
    """Cache-aside: đọc cache; hụt thì hỏi nguồn rồi ghi cache kèm TTL."""
    km = normalize_km(distance_km)  # chuẩn hóa một lần, dùng cho cả key lẫn nguồn
    key = quote_key(service, km)
    cached = await r.get(key)
    if cached is not None:
        return json.loads(cached), "hit"
    quote = await source.fetch(service, km)
    await r.set(key, json.dumps(quote), px=ttl_ms)
    return quote, "miss"
```

Ba quyết định nhỏ. **Key mang đủ biến số** ảnh hưởng tới giá: dịch vụ, khoảng cách đã chuẩn hoá (không thì `2.5` và `2.50` thành hai key) và phiên bản chính sách. Chuẩn hoá còn phải đi đúng chỗ: `get_quote` hỏi nguồn bằng **km đã chuẩn hoá**. Nếu key dùng 2,500 mà nguồn tính theo 2,5004 hay 2,5001, hai khách cùng key nhận giá của người đến trước (14.252 và 14.250 đồng) và cache đã đổi giá. Làm tròn đến mét là quyết định nghiệp vụ và phép tính cước thật phải dùng cùng quy tắc; sản phẩm thật thường gom km thành bậc lớn hơn (ví dụ 100 m) để tăng tỉ lệ trúng cache. **Giá trị là JSON với tiền là chuỗi**, vì `Decimal` không có trong JSON. **Mọi key cache đều có TTL** (`px` là mili giây), không dài hơn thời hạn hiệu lực của báo giá (`Quote.expires_at`, bài 13).

```text
== 1. Cache-aside cho báo giá ==
lần 1: miss   51.8 ms  -> {'service': 'ride', 'distance_km': '2.500', 'fare': '14250', 'currency': 'VND', 'policy': 'v1'}
lần 2: hit     0.5 ms  (nguồn bị gọi 1 lần)
key = lab27:quote:v1:ride:2.500
PTTL = 29999 ms (còn lại trước khi key tự xóa)
TTL 150 ms: ngay sau ghi -> hit; sau 200 ms -> miss
4,0004 km -> 21000 (4.000); 4,0001 km -> 21000 (hit); 4,0006 km -> 21005 (4.001, miss)
```

Lần đầu hụt, tốn 50 ms giả lập của nguồn; lần hai trúng sau chưa đến một mili giây và nguồn không bị gọi lại. `PTTL` là thời gian còn lại của key. Số mili giây phụ thuộc máy (xem dòng máy đo ở mục «Chạy và kiểm chứng»); chỉ tỷ lệ giữa hai lần là bài học. Với TTL 150 ms, sau 200 ms key biến mất và lần đọc kế là hụt: cache tự cập nhật khi bảng giá đổi theo cách đó. Dòng cuối cho thấy chuẩn hoá hoạt động: 4,0004 và 4,0001 km cùng key, cùng giá 21.000 đồng; 4,0006 km làm tròn lên 4,001, là key và giá khác (21.005).

## Cache nguội và đám đông

Câu hỏi mở bài có đáp án trong output của lab. Với 20 yêu cầu đồng thời vào một key nguội (hai dòng đầu), rồi với nguồn cố ý chậm hơn khoá và hơn thời gian chịu chờ (dòng cuối):

```text
20 yêu cầu đồng thời, key nguội, cache-aside thường: nguồn bị gọi 20 lần
20 yêu cầu đồng thời, có khóa single-flight:          nguồn bị gọi 1 lần
nguồn chậm 0,3 s, khóa 0,1 s, chờ tối đa 0,15 s: nguồn bị gọi 2 lần, 2 caller có giá, 18 caller nhận QuoteUnavailable
```

Đó là **cache stampede**: đúng lúc một key phổ biến hết hạn, mọi request cùng thấy hụt và cùng đổ về nguồn. Cách sửa là **single-flight**: chỉ một caller được hỏi nguồn, số còn lại chờ rồi đọc cache. Khoá là một key khác, giành bằng `SET ... NX PX`: chỉ thành công nếu key chưa tồn tại, và tự hết hạn nếu người giữ khoá chết.

```python
# (trích get_quote_single_flight)
token = uuid4().hex
if await r.set(lock_key, token, nx=True, px=lock_ms):  # SET NX PX = giành khóa có hạn
    try:
        # Kiểm lại cache sau khi giành khóa: người giữ khóa trước có thể vừa ghi xong
        # (và nhả khóa) giữa lúc ta đọc hụt và lúc ta giành được khóa.
        cached = await r.get(key)
        if cached is not None:
            return json.loads(cached), "hit"
        quote = await source.fetch(service, km)
        await r.set(key, json.dumps(quote), px=ttl_ms)
        return quote, "miss"
    finally:
        await r.eval(UNLOCK_LUA, 1, lock_key, token)
if time.monotonic() > give_up_at:   # chờ quá wait_ms: báo lỗi, KHÔNG tự gọi nguồn
    raise QuoteUnavailable(f"chờ quá {wait_ms} ms: {key}")
await asyncio.sleep(0.01)           # rồi quay lại đọc cache
```

`token` ngẫu nhiên cùng script `UNLOCK_LUA` (chỉ `DEL` nếu giá trị vẫn là token của ta) tránh việc xoá nhầm khoá của người khác sau khi khoá của ta đã hết hạn. **Kiểm lại cache sau khi giành khoá** là phần bản đầu của lab thiếu: chạy `--verify` nhiều lần, thỉnh thoảng nguồn bị gọi 2 lần, vì một caller đọc hụt trước lúc người giữ khoá ghi xong rồi giành khoá ngay sau khi khoá được nhả. Đó là lỗi «kiểm tra rồi hành động», cùng họ với lỗi nhận trùng chuyến ở [bài 23](/articles/python-concurrency-lock-queue-semaphore), và chỉ lộ khi chạy nhiều lần.

Thời gian chờ có giới hạn riêng: `wait_ms` (caller chịu chờ bao lâu) khác `lock_ms` (tuổi thọ khoá). Bản đầu của lab cho caller hết kiên nhẫn tự gọi nguồn, và đó là cái bẫy: khi nguồn chậm hơn khoá, mọi caller đã chờ cùng hết hạn một lúc rồi cùng đổ về nguồn, tự tạo đúng stampede mà khoá định chặn. Giờ caller hết `wait_ms` ném `QuoteUnavailable`; tầng API trả 503 kèm `Retry-After`, hoặc giá cũ nếu đã giữ bản cũ. `lock_ms` nên lớn hơn thời gian nguồn trả lời lúc chậm; nếu không, mỗi tuổi thọ khoá chỉ thêm một caller chứ không phải tất cả (dòng cuối khối output trên: 2 lần gọi nguồn, 18 caller nhận lỗi).

Khoá này đủ cho một Redis đơn lẻ và việc chỉ cần «thường thì một lần», không đủ để bảo vệ tiền hay trạng thái nghiệp vụ (xem phần cảnh báo về distributed locks trong tài liệu Redis); bảo đảm đúng-sai phải nằm ở database như bài 25.

## Rate limit: cửa sổ cố định và token bucket

Cách đơn giản nhất là **cửa sổ cố định**: mỗi khách một bộ đếm theo từng cửa sổ thời gian, vượt giới hạn thì chặn. Lab dùng `INCR` rồi `EXPIRE ... NX` (TTL chỉ đặt lần đầu của cửa sổ, cần Redis 7.0) trong cùng một MULTI/EXEC; tách ra mà tiến trình chết giữa chừng thì bộ đếm không bao giờ hết hạn. `now` được truyền vào để ca thử lặp lại được:

```python
# (trích)
async def fixed_window_allow(r, client_id, *, limit, window_s, now) -> bool:
    window_id = int(now // window_s)
    key = f"{PREFIX}rl:fw:{client_id}:{window_id}"
    async with r.pipeline(transaction=True) as pipe:
        pipe.incr(key)
        pipe.expire(key, window_s, nx=True)  # chỉ đặt TTL lần đầu của cửa sổ
        count, _ = await pipe.execute()
    return count <= limit
```

```text
== 2. Rate limit ==
cửa sổ cố định, 5 yêu cầu/60 s, t=59,9 s:  [True, True, True, True, True, False]
cửa sổ cố định, 5 yêu cầu/60 s, t=60,1 s:  [True, True, True, True, True, False]
-> 10 yêu cầu được nhận trong 0,2 giây quanh ranh giới cửa sổ (giới hạn danh nghĩa 5/60 s)
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hai cửa sổ liên tiếp, mỗi cửa sổ cho phép tối đa năm yêu cầu. Năm yêu cầu cuối cửa sổ thứ nhất và năm yêu cầu đầu cửa sổ thứ hai cách nhau 0,2 giây vẫn đều được nhận, tổng mười yêu cầu."><rect x="40" y="70" width="400" height="100" rx="8" class="f-phys-soft s-phys"/><rect x="440" y="70" width="400" height="100" rx="8" class="f-pk-soft s-pk"/><path d="M440 50 L440 190" class="f-none s-warn dash" stroke-width="2"/><text x="56" y="92" class="t-small">cửa sổ [0, 60) giây: tối đa 5</text><text x="456" y="92" class="t-small">cửa sổ [60, 120) giây: tối đa 5</text><text x="340" y="38" class="t-warn t-b">10 yêu cầu trong 0,2 giây</text><circle cx="380" cy="130" r="6" class="f-warn s-warn"/><circle cx="392" cy="130" r="6" class="f-warn s-warn"/><circle cx="404" cy="130" r="6" class="f-warn s-warn"/><circle cx="416" cy="130" r="6" class="f-warn s-warn"/><circle cx="428" cy="130" r="6" class="f-warn s-warn"/><circle cx="452" cy="130" r="6" class="f-warn s-warn"/><circle cx="464" cy="130" r="6" class="f-warn s-warn"/><circle cx="476" cy="130" r="6" class="f-warn s-warn"/><circle cx="488" cy="130" r="6" class="f-warn s-warn"/><circle cx="500" cy="130" r="6" class="f-warn s-warn"/><text x="40" y="190" class="t-small">0 s</text><text x="426" y="190" class="t-small">60 s</text><text x="812" y="190" class="t-small">120 s</text><text x="290" y="214" class="t-small">t = 59,9 s: 5 yêu cầu qua</text><text x="450" y="214" class="t-small">t = 60,1 s: 5 yêu cầu qua</text></svg>
</div>
<figcaption>Hình 2. Điểm yếu của cửa sổ cố định: bộ đếm về 0 ở ranh giới, nên đúng gấp đôi giới hạn (10 thay vì 5) lọt qua trong một khoảnh khắc.</figcaption>
</figure>

Cửa sổ cố định đủ cho việc chống lạm dụng thô. Muốn mượt hơn, dùng **token bucket**: xô chứa tối đa `capacity` token, nạp lại với tốc độ `refill_per_s`, mỗi yêu cầu lấy một token. Bước «đọc, nạp lại, trừ, ghi» phải nguyên tử nên nằm trong một script Lua (trích):

```lua
-- (trích)
local elapsed = math.max(0, now_ms - ts)
ts = math.max(ts, now_ms)  -- đồng hồ client lùi: không ghi lùi ts
milli = math.min(capacity, milli + elapsed * refill_per_s)
local allowed = 0
if milli >= cost then
    milli = milli - cost
    allowed = 1
end
redis.call('HSET', KEYS[1], 'milli', milli, 'ts', ts)
redis.call('PEXPIRE', KEYS[1], math.ceil(capacity / refill_per_s) + 1000)
return {allowed, tostring(milli)}
```

Token tính bằng **mili-token** (1 token = 1000): với tốc độ nạp nguyên, phép tính là số nguyên và state lưu gọn (`'0'`, `'1000'`); số trong Lua vẫn là double nên tốc độ nạp lẻ có sai số nhỏ. Script trả `tostring(milli)` vì Lua cắt số thực thành nguyên khi trả về client; `register_script` nạp script một lần rồi gọi qua `EVALSHA`. Với dung lượng 5, nạp 1 token mỗi giây, mô phỏng ba mốc thời gian:

```text
token bucket (dung lượng 5, nạp 1 token/s), mô phỏng thời gian:
  t=+ 0.0 s: 8 yêu cầu -> 5 qua, 3 chặn, còn 0 token
  t=+ 2.0 s: 3 yêu cầu -> 2 qua, 1 chặn, còn 0 token
  t=+12.0 s: 7 yêu cầu -> 5 qua, 2 chặn, còn 0 token
trạng thái trong Redis: {'milli': '0', 'ts': '1012000'} | PTTL 5999 ms
đồng hồ lùi 1 s: gọi lúc t-1 s -> False; gọi lúc t+1 s -> [True, False] (chỉ 1 token mới)
```

Xô đầy cho 5 qua ngay; hai giây sau chỉ có 2 token mới; nghỉ 12 giây thì xô chỉ đầy tới dung lượng 5. Key có TTL bằng thời gian nạp đầy cộng một giây, nên khách nghỉ lâu thì state biến mất và xô mới cũng đầy.

Vì sao phải Lua? 50 coroutine cùng xin token khi xô có 5; bản «`GET` rồi `SET`» (giữa hai lệnh nhường quyền một lần để mô phỏng việc xử lý) và bản Lua cho kết quả:

```text
50 yêu cầu đồng thời, dung lượng 5: GET rồi SET cho qua 50, Lua cho qua 5
```

Đây là lỗi mất cập nhật cùng họ với bài 25: cả 50 cùng đọc «còn 5» rồi cùng trừ; Lua chạy nguyên tử trên server nên đúng 5 qua. Hai hạn chế. Lab truyền `now_ms` từ client để ca thử lặp lại được; production nhiều instance có đồng hồ lệch nên nên đọc `TIME` của Redis trong script, và nếu vẫn nhận giờ từ client thì script không được ghi lùi `ts` (dòng cuối của output: instance chậm 1 giây không làm xô nạp thừa token). Và khoá theo `client_id` chỉ có nghĩa nếu định danh đã được xác thực.

## GEO cho 25.000 tài xế

Redis lưu chỉ mục địa lý ngay trong **sorted set**: `GEOADD` mã hoá (kinh độ, vĩ độ) thành số nguyên 52 bit (geohash) làm điểm số của thành viên, và `GEOSEARCH` quét các khoảng điểm số lân cận rồi lọc theo khoảng cách thật; `ZCARD`, `ZREM` dùng được nguyên xi. Lab sinh 25.000 tài xế `DR-00001` đến `DR-25000` (25.000 tài xế cần 5 chữ số, dài hơn `DR-001` của từ điển miền) trong đĩa bán kính 15 km quanh Hồ Gươm, rồi tìm 5 tài xế gần nhất của một khách trong 3 km:

```python
# (trích)
async def find_nearest(r, lat, lon, *, radius_km, count):
    found = await r.geosearch(GEO_KEY, longitude=lon, latitude=lat, radius=radius_km, unit="km",
                              sort="ASC", count=count, withdist=True)
    return [(member, float(dist)) for member, dist in found]
```

Lab đối chiếu với haversine tính tay trên cả 25.000 điểm.

```text
== 3. GEO: 25.000 tài xế quanh Hà Nội ==
kiểu key: zset | ZCARD = 25000
điểm (score) của DR-00001 trong sorted set: 3973586506935826 (geohash 52 bit)
vị trí gốc DR-00001: (105.743332, 20.995858); Redis trả: (105.743330, 20.995857)

GEOSEARCH 5 tài xế gần nhất trong 3 km (1.2 ms):
  DR-02542  0.0566 km   (tính tay haversine: 0.0565 km)
  DR-03814  0.0706 km   (tính tay haversine: 0.0705 km)
  DR-21827  0.1158 km   (tính tay haversine: 0.1158 km)
  DR-09761  0.1462 km   (tính tay haversine: 0.1462 km)
  DR-03482  0.1745 km   (tính tay haversine: 0.1745 km)
số tài xế trong 2 km: Redis 432, tính tay 432
```

Cùng năm mã tài xế, cùng thứ tự, khoảng cách lệch dưới 1 mét; số tài xế trong 2 km cũng khớp (432). Sai lệch nhỏ đến từ việc toạ độ bị lượng tử hoá khi lưu: `GEOPOS` trả `105.743330` thay vì `105.743332`. Khoảng cách GEO là **đường chim bay trên mô hình hình cầu**, không phải quãng đường đi: tài xế cách 100 m bên kia sông có thể xa hơn tài xế cách 400 m cùng bờ, nên điều phối thật dùng GEO để lọc ứng viên rồi mới tính lộ trình. Thời gian tìm kiếm dao động mạnh giữa các lần chạy (lần này 1,2 ms, một lần chạy khác đo được 16,3 ms), nên đừng rút kết luận từ một lần đo.

### Pipeline: bớt vòng đi-về

Nạp 25.000 tài xế cho thấy nút thắt thường không ở server mà ở **vòng đi-về mạng**. Lab nạp ba cách trên cùng máy (Docker Desktop):

```text
nạp 25000 tài xế, mỗi lệnh một vòng đi-về :    11938 ms
nạp 25000 tài xế, pipeline lô 5.000 lệnh   :      249 ms
nạp 25000 tài xế, 5 lệnh GEOADD x 5.000    :       95 ms
```

Cách đầu `await` từng lệnh: 25.000 vòng đi-về, mỗi vòng cỡ nửa mili giây. **Pipeline** gửi cả lô rồi đọc phản hồi một lượt: vẫn 25.000 lệnh nhưng chỉ 5 vòng đi-về, nhanh hơn hàng chục lần ở đây. Cách thứ ba dùng việc `GEOADD` nhận nhiều bộ ba, nên server chỉ nhận 5 lệnh. Lưu ý `pipeline(transaction=False)` chỉ gom lệnh, **không** nguyên tử; muốn các lệnh liền nhau không bị chen thì dùng `transaction=True` (MULTI/EXEC) như ở `fixed_window_allow`.

```python
# (trích)
async def load_drivers_pipeline(r, key, drivers, chunk=5_000):
    """GEOADD mỗi tài xế một lệnh nhưng gửi theo lô qua pipeline."""
    for start in range(0, len(drivers), chunk):
        async with r.pipeline(transaction=False) as pipe:
            for d in drivers[start:start + chunk]:
                pipe.geoadd(key, (d.lon, d.lat, d.driver_id))
            await pipe.execute()
```

### Di chuyển, offline và dựng lại

Tài xế di chuyển thì `GEOADD` lại cùng thành viên là cập nhật tại chỗ. Tài xế tắt app thì phải `ZREM`: sorted set không có TTL riêng cho từng thành viên, nên production cần cơ chế dọn (ví dụ một sorted set thứ hai theo thời điểm cập nhật cuối). Cuối cùng, nguyên tắc đầu bài được thử bằng cách xoá hẳn key:

```text
DR-02542 đi xa: top 5 mới bắt đầu bằng DR-03814; ZCARD vẫn 25000
mất key: tìm thấy 0 tài xế; dựng lại từ nguồn gốc mất 252 ms, top 5 trùng ban đầu: True
```

Mất key thì không tìm được ai; dựng lại từ danh sách gốc (đóng vai database vị trí) mất 252 ms và cho đúng kết quả cũ. Redis chỉ an toàn làm chỉ mục khi mọi thứ trong key tái tạo được từ nơi khác.

## Chạy và kiểm chứng

Cần hạ tầng dùng chung (`docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait`). Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/27-redis-cache-rate-limit-geo examples/toi-noi-python/27-redis-cache-rate-limit-geo/main.py
uv run --project examples/toi-noi-python/27-redis-cache-rate-limit-geo examples/toi-noi-python/27-redis-cache-rate-limit-geo/main.py --verify
```

Lệnh đầu in các khối output ở trên (trích từng phần, bỏ dòng đầu mô tả máy đo). `--verify` chạy cùng kịch bản, so 44 kết quả kỳ vọng rồi xoá mọi key `lab27:*`:

```text
OK 44/44
```

Thời gian chạy tuỳ máy (cỡ 10 đến 25 giây ở đây) vì cách nạp từng lệnh 25.000 vòng đi-về là một phần của phép đo; `--verify` chỉ kiểm số đếm và bất biến, không kiểm số mili giây. Khi chưa bật Redis, `--verify` in `FAIL: không nối được Redis tại redis://localhost:63790/0 (docker compose …)` và thoát mã 1. Môi trường: Python 3.14.7, redis-py 8.1.0, Redis 7.4.12 trong container `redis:7`, macOS 26 trên Apple Silicon (10 CPU), load average 23,81 lúc chạy lần dán output do máy đang bận việc khác, nên số mili giây trong bài chỉ cho thứ bậc độ lớn. Các số đếm (20, 1, 5, 50, 432; 44 ca) lặp lại được.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Nguồn vẫn bị gọi nhiều lần khi cache nguội, hoặc caller đã chờ lại tự đổ về nguồn | Có khoá single-flight không; có kiểm lại cache sau khi giành khoá không; hết `wait_ms` có ném lỗi thay vì tự gọi nguồn không |
| Cache trả giá của khách khác hoặc giá cũ | Key đã đủ biến số chưa; nguồn có tính theo km đã chuẩn hoá không; TTL có dài hơn thời hạn báo giá không |
| Bộ đếm rate limit không bao giờ hết | `INCR` và `EXPIRE` có nằm trong cùng MULTI/EXEC không |
| `GEOSEARCH` trả rỗng | Thứ tự `(kinh độ, vĩ độ)` có bị đảo thành (vĩ độ, kinh độ) không; key có còn tồn tại không |
| Vài chục nghìn lệnh chạy chậm bất thường | Có `await` từng lệnh một không; dùng pipeline hoặc lệnh nhận nhiều thành viên |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Một Redis đơn lẻ, chưa nói gì về replica, failover hay cluster; khoá <code>SET NX</code> và bộ đếm chỉ đúng trong phạm vi một node. Compose của lab không đặt <code>maxmemory</code> hay chính sách eviction, production phải đặt. Thời gian 50 ms của dịch vụ báo giá là giả lập, và tài xế đứng yên chưa mô phỏng 6.250 dòng vị trí mỗi giây. Bảng giá, vị trí gốc và mọi bảo đảm về tiền nằm ở nơi khác, không phải ở Redis.</p>
</div>

**Bài tập:** Báo giá có thêm tham số `surge` (hệ số giờ cao điểm). `quote_key` cần đổi thế nào, và vì sao đổi phiên bản `v1` thành `v2` thường tốt hơn xoá key cũ?

<details><summary>Xem lời giải</summary><p>Thêm <code>surge</code> (đã chuẩn hoá số chữ số thập phân) vào key, ví dụ <code>quote:v2:{service}:{km}:{surge}</code>; thiếu nó, hai khách có hệ số khác nhau dùng chung một giá. Đổi phiên bản tốt hơn xoá vì xoá hàng loạt cần quét key (tốn, dễ sót), còn key cũ không bao giờ được đọc nữa và tự chết theo TTL; instance cũ và mới lúc deploy dần cũng không giẫm lên giá trị của nhau. Giá trị <code>surge</code> cũng phải được chuẩn hoá trước khi vào key và trước khi tính giá, đúng như bài học của <code>normalize_km</code>.</p></details>

Bài trước: [bài 26](/articles/python-mongodb-pymongo-motor-document-ocr). Bài tiếp: [bài 28](/articles/python-integration-test-testcontainers-fixture).

## Tài liệu gốc

- [redis-py: tài liệu](https://redis.readthedocs.io/en/stable/).
- [redis-py: ví dụ asyncio](https://redis.readthedocs.io/en/stable/examples/asyncio_examples.html).
- [Redis: GEOSEARCH](https://redis.io/docs/latest/commands/geosearch/).
- [Redis: SET (tuỳ chọn NX, PX)](https://redis.io/docs/latest/commands/set/).
- [Redis: Pipelining](https://redis.io/docs/latest/develop/using-commands/pipelining/).
- [Redis: Distributed Locks](https://redis.io/docs/latest/develop/use/patterns/distributed-locks/).
