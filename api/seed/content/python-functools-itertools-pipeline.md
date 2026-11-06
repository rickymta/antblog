---
slug: python-functools-itertools-pipeline
title: "functools, itertools và pipeline hàm: xử lý lô chuyến"
summary: "Bài 18: partial, reduce, lru_cache, groupby phải sort trước, chain, batched, accumulate; pipeline generator lọc, tính cước, nhóm theo thành phố cho lô 10.000 chuyến giả; đối chiếu LINQ, không dùng pandas."
category: programming
level: beginner
tags: [python, toi-noi, functools, itertools]
language: vi
---

Cuối mỗi khung giờ, bộ phận vận hành của [Tới Nơi](/articles/du-an-mau-toi-noi) nhận một lô chuyến và hỏi: mỗi thành phố có bao nhiêu chuyến đã hoàn thành, doanh thu bao nhiêu, cước trung bình thế nào? Cách quen tay là ba vòng `for` nối tiếp, mỗi vòng dựng một list: lọc chuyến, tính cước, rồi gom nhóm. Với 10.000 chuyến thì chạy được, nhưng mỗi list trung gian giữ cả lô trong bộ nhớ, và ba vòng lặp trộn ba mối quan tâm vào nhau. Bài này dựng cùng việc đó thành một **pipeline** các bước nhỏ nối bằng generator, dùng những mảnh ghép của `functools` và `itertools`.

Đây là bài 18 của lộ trình **Python cơ bản qua Tới Nơi**. Bạn cần quen generator và `itertools` ở bài 06, và decorator ở bài 17, vì `lru_cache` là một decorator. Ta không dùng pandas: loại công cụ đó thuộc bài 29.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Lô có 8.365 chuyến tính cước được, thuộc 6 thành phố, đang ở thứ tự ngẫu nhiên. Gọi <code>itertools.groupby</code> theo thành phố mà chưa sắp xếp thì được bao nhiêu nhóm?</p>
<details><summary>Xem đáp án</summary><p><b>Hàng nghìn nhóm, không phải 6.</b> <code>groupby</code> chỉ gom các phần tử <em>liền nhau</em> có cùng khóa. Trên lô của lab, kết quả là 6.438 nhóm; sau khi <code>sorted</code> theo cùng khóa mới còn đúng 6.</p></details>
</div>

## Lô chuyến giả và các mảnh ghép

Code của bài nằm ở `examples/toi-noi-python/18-functools-itertools-pipeline/main.py`. Lô gồm 10.000 chuyến do generator sinh bằng `Random(seed)` với seed cố định, nên mỗi lần chạy ra đúng cùng dữ liệu. Thành phố lấy trong 6 thành phố của Tới Nơi (TP.HCM và Hà Nội đông nhất), dịch vụ là `ServiceKind` (`ride`, `food`, `parcel`), khoảng cách tính bằng `Decimal` theo bước 0,1 km. Khoảng 1,2% chuyến mang khoảng cách hỏng (trên 100 km, giả lập lỗi nhập hay GPS nhảy). Trạng thái dùng `TripStatus` quen thuộc. `Trip` ở đây là bản ghi lô gọn gồm năm trường, không phải `Trip` của bài 11.

```python
class Trip(NamedTuple):
    trip_id: str
    city: str
    service: str
    distance_km: Decimal
    status: TripStatus

def is_billable(t: Trip) -> bool:
    """Tính cước được: chuyến đã hoàn thành và khoảng cách trong miền 0..100 km."""
    return t.status is TripStatus.COMPLETED and Decimal("0") <= t.distance_km <= MAX_KM
```

Với seed của lab, 10.000 chuyến gồm 8.466 `completed`, 1.224 `cancelled`, 310 `in_transit`. Trong 8.466 chuyến hoàn thành có 101 chuyến khoảng cách ngoài miền 0..100 km, bị `is_billable` loại, còn lại **8.365** chuyến tính cước được. Hàm `calculate_fare` được rút từ bài 10 và 13 (tham số `service` là `ServiceKind`) và giữ nguyên quy tắc: cước 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, làm tròn `ROUND_HALF_UP`, ném `ValueError` khi ngoài miền. Không có phụ phí hay giảm giá.

## `partial` và `reduce`: cố định tham số, gộp dần

`functools.partial` tạo một hàm mới từ hàm cũ bằng cách **đóng sẵn** một số đối số. Hai chính sách cước (chuẩn và thử nghiệm 5.000 VND/km vượt) và ba dịch vụ cho ta cả loạt biến thể mà không phải viết lại hàm:

```python
fare_food = partial(calculate_fare, service=ServiceKind.FOOD)
fare_5000 = partial(calculate_fare, policy=POLICY_5000)
fare_food(Decimal("5"))            # 25500
fare_5000(Decimal("5"))            # 27000
fare_food.func is calculate_fare   # True
fare_food.keywords                 # {'service': <ServiceKind.FOOD: 'food'>}
per_service = [partial(calculate_fare, service=s) for s in ServiceKind]
```

Đối tượng `partial` giữ lại `func`, `args`, `keywords` để soi được, và không có `__name__` (lab kiểm `hasattr(fare_food, "__name__")` ra `False`), nên nếu cần log tên thì tự đặt. CPython cũng làm phẳng `partial` của `partial` thành một lớp (lab kiểm `nested.func is calculate_fare`), nhưng đó là chi tiết cài đặt, đừng dựa vào. Từ Python 3.14, `functools.Placeholder` cho phép cố định đối số vị trí ở giữa (`partial(pow, Placeholder, 2)` là hàm bình phương). So với `lambda`, `partial` hơn ở chỗ **chốt giá trị ngay lúc tạo**, tránh lỗi late binding của bài 17: dòng `per_service` ở trên cho ba hàm với `service` lần lượt là `ride`, `food`, `parcel`, đúng như mong đợi, trong khi `[lambda: s for s in ServiceKind]` sẽ cho cả ba cùng `parcel`.

`functools.reduce(f, iterable, initial)` gộp dần một dãy thành một giá trị bằng cách áp `f(acc, x)` lặp lại. Lab kiểm: `reduce(add, fares, Decimal(0))` bằng `sum(fares, Decimal(0))`, và `reduce(max, fares)` bằng `max(fares)`. Vì `sum`, `max`, `min`, `any`, `all` đã bao các phép gộp phổ biến và đọc dễ hơn, `reduce` hiếm khi cần. Nó hợp lý khi phép gộp là hàm tự định nghĩa, và khi dựng một chuỗi hàm:

```python
def compose(*steps):
    """compose(f, g, h)(x) == h(g(f(x))): bước đầu chạy trước."""
    return reduce(lambda f, g: lambda x: g(f(x)), steps)

compose(lambda x: x + 1, lambda x: x * 10)(1)   # 20, tức (1 + 1) * 10
```

Hai điều cần nhớ về `reduce`: gọi trên dãy rỗng mà không có `initial` ném `TypeError: reduce() of empty iterable with no initial value` (lab kiểm đúng thông báo đó), và phép gộp phải được đọc theo thứ tự trái sang phải.

## `lru_cache` và `cache`: nhớ kết quả của hàm thuần

Trong lô, nhiều chuyến có cùng khoảng cách và dịch vụ, nên `calculate_fare` bị gọi lại với cùng đối số. `functools.lru_cache` là decorator ghi nhớ kết quả theo đối số; `functools.cache` là `lru_cache(maxsize=None)`, không giới hạn số khóa. Lab dùng dạng gọi hàm của decorator (nhắc lại `@` chỉ là cú pháp):

```python
fare_cached = lru_cache(maxsize=1024)(calculate_fare)

for t in billable_trips:
    fare_cached(t.distance_km, service=t.service)
fare_cached.cache_info()
```

Kết quả thật khi chạy cho 8.365 chuyến, rồi so với `set` các cặp (khoảng cách, dịch vụ):

```text
hits                      : 7554
misses                    : 811
currsize                  : 811
calls                     : 8365
distinct_keys             : 811
8365 lần tính cước: không cache 27.7 ms | có cache (kể cả 811 lần miss) 2.1 ms
```

`hits + misses` bằng số lần gọi, và `misses` bằng đúng số khóa khác nhau: mỗi khóa tính một lần, 7.554 lần còn lại lấy từ cache. Về thời gian, hai con số cuối là của một lần chạy trên máy tác giả (lab lấy giá trị nhỏ nhất của 5 lần đo) và thay đổi theo máy; điều đáng nhớ là tỷ lệ và chi phí tuyệt đối. Với một hàm chỉ tính số học như `calculate_fare`, cache tiết kiệm cỡ vài chục mili giây cho cả lô. Cache đáng giá hơn nhiều khi hàm đắt (gọi mạng, đọc đĩa, tính nặng), và nó luôn tốn bộ nhớ cho các khóa đã nhớ; phần bộ nhớ đó được đo riêng ở mục cuối.

`lru_cache` chỉ đúng với **hàm thuần**: kết quả chỉ phụ thuộc vào đối số. Bẫy thật là hàm đọc thêm biến nằm ngoài khóa:

```python
@cache
def quote_active(km):                       # SAI: phụ thuộc ACTIVE_POLICY, không nằm trong khóa
    return calculate_fare(km, ACTIVE_POLICY["policy"])

@cache
def quote_with_policy(km, policy):          # ĐÚNG: mọi đầu vào đều là đối số
    return calculate_fare(km, policy)
```

Đổi `ACTIVE_POLICY` từ chính sách chuẩn sang 5.000 VND/km giữa chừng, `quote_active(Decimal("5"))` vẫn trả **25.500** (giá cũ, lấy từ cache) dù đáng lẽ 27.000, còn `quote_with_policy` trả đúng 25.500 và 27.000 cho hai chính sách. Ở production đây là lỗi «đổi bảng giá mà giá cũ vẫn hiện». Các lưu ý còn lại của `lru_cache`: đối số phải hashable (`fare_cached([Decimal("5")])` ném `TypeError: unhashable type: 'list'`; `Decimal` và `NamedTuple` chứa `Decimal` thì hashable), lời gọi bằng vị trí và bằng từ khóa được coi là các khóa khác nhau, hàm trả về đối tượng khả biến thì mọi người gọi dùng chung một đối tượng, và dùng trên phương thức sẽ giữ tham chiếu tới `self` suốt đời cache.

## Pipeline generator: lọc, tính cước, nhóm

Giờ ghép các bước. Mỗi bước nhận một iterable và trả một iterator, nên không bước nào dựng list. `filter` và `map` đã là loại bước này; `partial` đóng sẵn đối số, và `compose` nối chúng:

```python
def price_one(fare_fn, t):
    return PricedTrip(t.trip_id, t.city, t.service, t.distance_km,
                      fare_fn(t.distance_km, service=t.service))

def make_pipeline(fare_fn=fare_cached):
    return compose(
        partial(filter, is_billable),                # Iterable[Trip] -> Iterator[Trip]
        partial(map, partial(price_one, fare_fn)),   # Iterator[Trip] -> Iterator[PricedTrip]
    )
```

Gọi `make_pipeline()` chưa chạy gì. Chỉ khi có bên tiêu thụ kéo phần tử thì dữ liệu mới chảy. Lab bọc nguồn bằng một generator đếm để thấy rõ điều đó: lấy 3 chuyến đầu của pipeline bằng `islice` chỉ kéo **4** chuyến từ nguồn 10.000 chuyến (chuyến thứ hai là `cancelled` nên bị lọc):

```text
3 chuyến đầu được tính cước: [('TN-0001', 'Cần Thơ', '12000'), ('TN-0003', 'Cần Thơ', '32700'), ('TN-0004', 'Đà Nẵng', '42600')]
số chuyến nguồn bị kéo để lấy 3 chuyến đó: 4
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Pipeline xử lý lô chuyến: nguồn generate_trips sinh 10.000 chuyến, bước filter is_billable cho qua 8.365 chuyến và loại 1.635 chuyến, bước map price_one tính cước với cache có 811 khóa, bước gom nhóm theo thành phố ra 6 nhóm và tổng doanh thu 338.740.950 VND. Mũi tên đứt chỉ nhánh bị loại gồm 1.224 hủy, 310 đang đi, 101 khoảng cách ngoài miền."><defs><marker id="py18-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py18-0-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="30" width="180" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="56" class="t-b">nguồn</text><text x="34" y="78" class="t-mono">generate_trips()</text><text x="34" y="98" class="t-small">10.000 chuyến giả</text><rect x="250" y="30" width="180" height="90" rx="10" class="f-pk-soft s-pk"/><text x="264" y="56" class="t-b">lọc</text><text x="264" y="78" class="t-mono">is_billable(t)</text><text x="264" y="98" class="t-small">8.365 qua, 1.635 loại</text><rect x="480" y="30" width="180" height="90" rx="10" class="f-pk-soft s-pk"/><text x="494" y="56" class="t-b">tính cước</text><text x="494" y="78" class="t-mono">price_one(t)</text><text x="494" y="98" class="t-small">cache: 811 khóa</text><rect x="710" y="30" width="150" height="90" rx="10" class="f-phys-soft s-phys"/><text x="724" y="56" class="t-b">gom nhóm</text><text x="724" y="78" class="t-mono">theo city</text><text x="724" y="98" class="t-small">6 nhóm</text><path d="M200 75 L248 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py18-0-arrow)"/><path d="M430 75 L478 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py18-0-arrow)"/><path d="M660 75 L708 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py18-0-arrow)"/><path d="M340 120 L340 158" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py18-0-warn)"/><rect x="190" y="160" width="400" height="62" rx="10" class="f-warn-soft s-warn"/><text x="204" y="184" class="t-b">bị loại, không tính cước</text><text x="204" y="204" class="t-small">1.224 cancelled, 310 in_transit, 101 khoảng cách ngoài miền</text><text x="624" y="190" class="t-small">đầu ra: 338.740.950 VND</text><text x="624" y="208" class="t-small">tổng doanh thu 8.365 chuyến</text></svg>
</div>
<figcaption>Hình 1. Pipeline chạy theo kiểu kéo: mỗi chuyến đi qua cả ba bước rồi mới tới chuyến sau, không bước nào giữ cả lô; nhánh đứt là những chuyến bị loại ở bước lọc.</figcaption>
</figure>

Xử lý theo **lô** là nhu cầu thường gặp: ghi database 1.000 dòng một lần hiệu quả hơn từng dòng. `itertools.batched(iterable, n)` (Python 3.12 trở lên) cắt dãy thành các tuple dài tối đa `n`; `chain.from_iterable` trải phẳng các lô trở lại một dãy; `accumulate` cho tổng cộng dồn:

```python
batches = list(batched(generate_trips(), BATCH))        # 10 lô, mỗi lô 1.000 chuyến
per_batch = [process_batch(b) for b in batches]         # production: ghi database mỗi lô một lần
revenues = [sum((p.fare for p in rows), Decimal(0)) for rows in per_batch]
running = list(accumulate(revenues))                    # doanh thu cộng dồn qua từng lô
flattened = list(chain.from_iterable(per_batch))        # == chạy thẳng cả lô
```

Kết quả thật: `số chuyến mỗi lô: [1000, 1000, ...]` (10 lô), doanh thu ba lô đầu `['31.828.800', '32.506.650', '33.819.450']` và cộng dồn `['31.828.800', '64.335.450', '98.154.900']`. Số cuối của `running` bằng tổng doanh thu, và `flattened` bằng đúng kết quả chạy thẳng pipeline, tức là cắt lô không làm đổi đáp số. `batched` có thêm tham số `strict=True` (từ Python 3.13): lô cuối ngắn hơn `n` sẽ ném `ValueError` thay vì trả lô ngắn.

## `groupby` và hai cách gom nhóm

Bước cuối là nhóm theo thành phố. `itertools.groupby(iterable, key)` trả các cặp (khóa, nhóm) nhưng chỉ gom các phần tử **liền nhau** cùng khóa, khác hẳn `GROUP BY` của SQL hay `GroupBy` của LINQ. Muốn mỗi khóa một nhóm phải `sorted` theo cùng khóa trước. `operator.attrgetter("city")` tạo hàm khóa nhanh và rõ hơn `lambda`:

```python
def by_city_groupby(priced):
    ordered = sorted(priced, key=attrgetter("city"))    # bắt buộc, và nạp cả dãy vào list
    return {city: stats_of(list(rows)) for city, rows in groupby(ordered, key=attrgetter("city"))}
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="So sánh groupby trên tám chuyến minh họa. Chưa sắp xếp: Hà Nội, Hà Nội, TP.HCM, Hà Nội, Đà Nẵng, TP.HCM, TP.HCM, Hà Nội cho sáu nhóm vì chỉ các phần tử liền nhau cùng khóa mới gom. Sau khi sắp xếp theo thành phố: Đà Nẵng, bốn Hà Nội, ba TP.HCM cho ba nhóm."><text x="20" y="22" class="t-small">mỗi ô là một chuyến, chữ là mã thành phố (HN, SG, DN); gạch dưới là một nhóm do groupby trả về</text><text x="20" y="70" class="t-b">chưa sort</text><rect x="170" y="44" width="56" height="40" rx="6" class="f-phys-soft s-phys"/><text x="190.0" y="69" class="t-mono">HN</text><rect x="234" y="44" width="56" height="40" rx="6" class="f-phys-soft s-phys"/><text x="254.0" y="69" class="t-mono">HN</text><rect x="298" y="44" width="56" height="40" rx="6" class="f-pk-soft s-pk"/><text x="318.0" y="69" class="t-mono">SG</text><rect x="362" y="44" width="56" height="40" rx="6" class="f-phys-soft s-phys"/><text x="382.0" y="69" class="t-mono">HN</text><rect x="426" y="44" width="56" height="40" rx="6" class="f-warn-soft s-warn"/><text x="446.0" y="69" class="t-mono">DN</text><rect x="490" y="44" width="56" height="40" rx="6" class="f-pk-soft s-pk"/><text x="510.0" y="69" class="t-mono">SG</text><rect x="554" y="44" width="56" height="40" rx="6" class="f-pk-soft s-pk"/><text x="574.0" y="69" class="t-mono">SG</text><rect x="618" y="44" width="56" height="40" rx="6" class="f-phys-soft s-phys"/><text x="638.0" y="69" class="t-mono">HN</text><path d="M170 96 L290 96" class="f-none s-muted" stroke-width="2"/><path d="M298 96 L354 96" class="f-none s-muted" stroke-width="2"/><path d="M362 96 L418 96" class="f-none s-muted" stroke-width="2"/><path d="M426 96 L482 96" class="f-none s-muted" stroke-width="2"/><path d="M490 96 L610 96" class="f-none s-muted" stroke-width="2"/><path d="M618 96 L674 96" class="f-none s-muted" stroke-width="2"/><text x="702" y="64" class="t-b">6 nhóm</text><text x="702" y="82" class="t-small">HN xuất hiện 3 nhóm riêng</text><text x="20" y="170" class="t-b">đã sort</text><rect x="170" y="144" width="56" height="40" rx="6" class="f-warn-soft s-warn"/><text x="190.0" y="169" class="t-mono">DN</text><rect x="234" y="144" width="56" height="40" rx="6" class="f-phys-soft s-phys"/><text x="254.0" y="169" class="t-mono">HN</text><rect x="298" y="144" width="56" height="40" rx="6" class="f-phys-soft s-phys"/><text x="318.0" y="169" class="t-mono">HN</text><rect x="362" y="144" width="56" height="40" rx="6" class="f-phys-soft s-phys"/><text x="382.0" y="169" class="t-mono">HN</text><rect x="426" y="144" width="56" height="40" rx="6" class="f-phys-soft s-phys"/><text x="446.0" y="169" class="t-mono">HN</text><rect x="490" y="144" width="56" height="40" rx="6" class="f-pk-soft s-pk"/><text x="510.0" y="169" class="t-mono">SG</text><rect x="554" y="144" width="56" height="40" rx="6" class="f-pk-soft s-pk"/><text x="574.0" y="169" class="t-mono">SG</text><rect x="618" y="144" width="56" height="40" rx="6" class="f-pk-soft s-pk"/><text x="638.0" y="169" class="t-mono">SG</text><path d="M170 196 L226 196" class="f-none s-muted" stroke-width="2"/><path d="M234 196 L482 196" class="f-none s-muted" stroke-width="2"/><path d="M490 196 L674 196" class="f-none s-muted" stroke-width="2"/><text x="702" y="164" class="t-b">3 nhóm</text><text x="702" y="182" class="t-small">mỗi khóa một nhóm</text><text x="20" y="236" class="t-small">Trên lô 8.365 chuyến thật của lab: 6.438 nhóm khi chưa sort, 6 nhóm sau khi sort.</text></svg>
</div>
<figcaption>Hình 2. groupby chỉ gom các chuyến liền nhau cùng thành phố: dữ liệu chưa sắp xếp tạo ra nhiều nhóm trùng khóa, sắp xếp trước thì mỗi khóa đúng một nhóm (tám chuyến minh họa, không phải dữ liệu của lab).</figcaption>
</figure>

Có ba điều nên biết. Một, `sorted` trả về list nên bước này **phá tính lười**: pipeline phải nạp hết dữ liệu để sắp xếp. Hai, nhóm con `rows` dùng chung iterator với `groupby`: khi `groupby` đã đi sang nhóm kế tiếp, nhóm cũ cạn. Lab chứng minh bằng `held = list(groupby(...))` rồi `list(held[0][1])` ra `[]`; vì vậy phải dùng `rows` ngay trong vòng lặp, như `list(rows)` ở trên. Ba, cách gom một lượt không cần sort thường hợp hơn với luồng lớn: giữ số chuyến và doanh thu theo khóa trong `defaultdict`, bộ nhớ tỷ lệ với số khóa (6), không phải số chuyến.

```python
def by_city_one_pass(priced):
    count = defaultdict(int)
    revenue = defaultdict(Decimal)
    for p in priced:
        count[p.city] += 1
        revenue[p.city] += p.fare
    return {city: CityStat(count[city], revenue[city], round_vnd(revenue[city] / count[city]))
            for city in count}
```

(Rút gọn: `round_vnd` là `quantize(Decimal("1"), ROUND_HALF_UP)` viết tắt.) Hai cách cho kết quả bằng nhau (lab kiểm `grouped == one_pass`), đây là bảng thật của lô 8.365 chuyến:

```text
thành phố   số chuyến      doanh thu   cước TB
Cần Thơ           672     27.739.350    41.279
Hà Nội           2141     87.217.350    40.737
Hải Phòng         822     32.555.250    39.605
Nha Trang         829     34.147.200    41.191
TP.HCM           2938    118.510.500    40.337
Đà Nẵng           963     38.571.300    40.053
tổng doanh thu: 338.740.950 VND
```

Cộng sáu dòng ra đúng 338.740.950 VND và 8.365 chuyến.

## Đối chiếu LINQ, bộ nhớ và giới hạn

Pipeline này cùng tinh thần với LINQ to Objects, đã nói ở bài [LINQ và deferred execution](/articles/csharp-linq-deferred-execution) của loạt C#: khai báo các bước, chưa chạy cho tới khi có người duyệt. Bảng dưới là ánh xạ gần đúng; cột phải ghi chỗ khác nhau cần cẩn thận.

| LINQ | Python | Khác biệt đáng nhớ |
| --- | --- | --- |
| `Where` | `filter(pred, xs)` hoặc `(x for x in xs if pred(x))` | Cùng lười |
| `Select` | `map(f, xs)` hoặc `(f(x) for x in xs)` | Cùng lười |
| `SelectMany` | `chain.from_iterable(map(f, xs))` | Trải phẳng một cấp |
| `GroupBy` | `sorted` + `groupby`, hoặc `defaultdict` | `groupby` của Python cần dữ liệu đã sort theo khóa |
| `Aggregate` / `Sum` | `reduce` / `sum` | `reduce` rỗng cần `initial` |
| `Take(n)` / `Skip(n)` | `islice(xs, n)` / `islice(xs, n, None)` | `islice` không nhận số âm (ném `ValueError`) |
| `Chunk(n)` | `batched(xs, n)` | Python trả tuple |
| `Concat` | `chain(a, b)` | Cùng lười |
| `OrderBy` | `sorted(xs, key=...)` | Trả list, không lười; `sorted` ổn định |
| `First()` / `FirstOrDefault()` | `next(iter(xs))` / `next(iter(xs), None)` | Rỗng ném `StopIteration`, không phải `InvalidOperationException` |
| `ToList()` | `list(xs)` | Tương đương |

Khác biệt lớn nhất không nằm trong bảng: query LINQ duyệt lại sẽ chạy lại từ nguồn, còn generator của Python đã cạn thì duyệt lại ra rỗng; muốn duyệt nhiều lần phải tạo lại generator hoặc `list`.

Lợi ích bộ nhớ của pipeline, đo bằng `tracemalloc` với cùng việc «gom doanh thu theo thành phố» trên 10.000 chuyến. Phép đo phải tách bộ nhớ của cache ra, vì `lru_cache` giữ khóa và kết quả suốt đời process: nếu đo khi cache đã ấm hoặc không dọn, con số của pipeline sẽ đẹp giả tạo. Lab đo hai chế độ, ở chế độ có cache thì gọi `cache_clear()` trước mỗi lần đo:

```text
đỉnh bộ nhớ (tracemalloc), 10.000 chuyến:
  không cache : pipeline 5.7 KiB | list từng bước 4,560 KiB | tỷ lệ 799 lần
  cache lạnh  : pipeline 276 KiB | list từng bước 3,755 KiB | tỷ lệ 14 lần
  chi phí riêng của cache (811 khóa): khoảng 270 KiB
```

Không cache, pipeline chỉ giữ vài KiB cho các dict gom nhóm, còn bản dựng `list` cho từng bước (`trips`, `billable`, `priced`) giữ cả lô. Có cache lạnh, 811 khóa nhớ thêm khoảng 270 KiB, nên tỷ lệ chỉ còn cỡ 14 lần; cache đổi bộ nhớ lấy thời gian. Con số cụ thể thay đổi theo máy và phiên bản Python; ở 10.000 dòng chưa đáng kể, nhưng lợi thế của streaming là thứ nhân lên khi lô lớn lên, còn kích thước cache bị chặn bởi số khóa khác nhau (hoặc `maxsize`).

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `groupby` cho hàng nghìn nhóm trùng khóa | Chưa `sorted` theo cùng khóa, hoặc dùng `defaultdict` một lượt |
| Nhóm con của `groupby` rỗng | Đã đi tiếp sang nhóm sau; dùng `list(rows)` ngay trong vòng lặp |
| Giá cũ vẫn hiện sau khi đổi chính sách | Hàm `@cache` đọc biến ngoài khóa; đưa mọi đầu vào thành đối số |
| `TypeError: unhashable type` ở hàm có cache | Đối số là list hoặc dict; dùng tuple hoặc `frozenset` |
| RAM tăng khi xử lý lô lớn | Có `list(...)`, `sorted(...)` hay `[... for ...]` ở giữa pipeline không |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn so với production</span>
<p>Lô 10.000 chuyến do generator tự sinh trong cùng process; dữ liệu thật đến từ Kafka hoặc database với thứ tự đảo, trùng lặp và dòng lỗi mà ta mới chỉ loại bằng một bộ lọc. Pipeline chạy trong một luồng, không retry, không đánh dấu chuyến bị loại để điều tra, và không nằm trong transaction (bài 17 bàn phần đó). <code>lru_cache</code> chỉ nhớ trong bộ nhớ của một process, không chia sẻ giữa nhiều worker. Hai cách gom nhóm chỉ đúng khi mọi khóa vừa trong bộ nhớ; với 2 triệu chuyến mỗi ngày, việc tổng hợp thường làm ở database hoặc công cụ chuyên cho bảng dữ liệu (bài 29). Số đo thời gian và bộ nhớ là của một lần chạy trên một máy.</p>
</div>

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/18-functools-itertools-pipeline examples/toi-noi-python/18-functools-itertools-pipeline/main.py
uv run --project examples/toi-noi-python/18-functools-itertools-pipeline examples/toi-noi-python/18-functools-itertools-pipeline/main.py --verify
```

Lệnh đầu in các nhóm kết quả mà bài đã trích (`partial`, `reduce`, cache, pipeline, bảng thành phố, đo bộ nhớ) và chạy trong vài giây (phần đo thời gian lặp lại 5 lần). Lệnh thứ hai kiểm 21 ca: dữ liệu giả lặp lại được theo seed, `partial` (kể cả trong comprehension), `reduce` và `compose`, pipeline bằng với viết tay và lười, ba bẫy của cache, `groupby` cần sort và nhóm con cạn, hai cách gom nhóm bằng nhau, tổng doanh thu, `batched`, `accumulate`, `chain.from_iterable`, và bộ nhớ của pipeline streaming so với list từng bước, có và không có cache. Kết quả cuối:

```text
OK 21/21
```

Sai ca nào, chương trình in tên ca đó cùng `FAIL n/21` (n là số ca đạt) và thoát với mã 1.

**Bài tập 1:** Viết biểu thức tính doanh thu riêng của dịch vụ `food` trên cả lô mà không dựng list nào. **Bài tập 2:** Thêm một bước vào `make_pipeline` để chỉ giữ chuyến trên 10 km, dùng `compose` và `partial`, không sửa các bước cũ. Có cần đổi `by_city_one_pass` không?

<details><summary>Xem lời giải</summary><p>(1) <code>sum((p.fare for p in make_pipeline()(generate_trips()) if p.service is ServiceKind.FOOD), Decimal(0))</code>: generator expression truyền thẳng vào <code>sum</code>, pipeline không dựng list, và <code>Decimal(0)</code> làm giá trị đầu để kết quả vẫn là <code>Decimal</code> kể cả khi không có chuyến nào. (2) Thêm bước nhận <code>Iterator[PricedTrip]</code>: <code>partial(filter, lambda p: p.distance_km &gt; Decimal("10"))</code> đặt cuối <code>compose</code>; vì <code>filter</code> nhận một iterable ở vị trí thứ hai nên <code>partial</code> chỉ cần đóng sẵn hàm điều kiện. <code>by_city_one_pass</code> không phải đổi gì, vì nó chỉ cần một iterable các <code>PricedTrip</code>; đó là lợi ích của việc mỗi bước chỉ biết đầu vào và đầu ra của mình.</p></details>

Bài trước: [bài 17](/articles/python-decorator-context-manager). Bài tiếp: [bài 19](/articles/python-design-pattern-strategy-state-adapter).

## Tài liệu gốc

- [functools: higher-order functions and operations on callable objects](https://docs.python.org/3.14/library/functools.html) (`partial`, `reduce`, `lru_cache`, `cache`).
- [itertools: functions creating iterators for efficient looping](https://docs.python.org/3.14/library/itertools.html) (`groupby`, `chain`, `batched`, `accumulate`, `islice`).
- [operator: standard operators as functions](https://docs.python.org/3.14/library/operator.html) (`attrgetter`, `add`).
- [Functional Programming HOWTO](https://docs.python.org/3.14/howto/functional.html).
- [tracemalloc: trace memory allocations](https://docs.python.org/3.14/library/tracemalloc.html).
