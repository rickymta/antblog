---
slug: python-comprehension-generator-iterator
title: "Comprehension, iterator và generator: dòng vị trí tài xế không cần nạp hết"
summary: "Bài 06: viết comprehension đúng chỗ, hiểu iterator và vòng for, dùng generator với yield và itertools để lọc 1 triệu điểm GPS giả; đo bộ nhớ list và generator bằng tracemalloc."
category: programming
level: beginner
tags: [python, toi-noi, generator, itertools]
language: vi
---

Lúc cao điểm, 25.000 tài xế online của [Tới Nơi](/articles/du-an-mau-toi-noi) mỗi người gửi vị trí một lần mỗi 4 giây: khoảng **6.250 dòng mỗi giây**. Một phút là 375.000 dòng. Nếu muốn quét một khoảng dữ liệu để tìm các điểm có tốc độ phi lý (vài trăm km/h do GPS nhảy), cách đầu tiên ta nghĩ tới là đọc hết vào một list rồi lọc. Bài này cho thấy cách đó tốn bao nhiêu RAM và cách để chỉ giữ một điểm trong bộ nhớ tại mỗi thời điểm.

Đây là bài 06 của lộ trình **Python cơ bản qua Tới Nơi**. Ta đi từ comprehension (cách viết gọn một vòng lặp tạo list, dict, set), qua giao thức iterator đứng sau mọi vòng `for`, tới generator và `yield`, rồi áp dụng vào bài toán 1 triệu điểm GPS giả với số liệu thật. Người đã đọc bài C# về [LINQ và deferred execution](/articles/csharp-linq-deferred-execution) sẽ nhận ra ý tưởng "chưa chạy cho tới khi có người duyệt"; Python làm điều đó bằng generator, và khác ở chỗ generator chỉ duyệt được một lần.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Gọi một hàm có chứa <code>yield</code> đã chạy dòng nào trong thân hàm chưa?</p>
<details><summary>Xem đáp án</summary><p><b>Chưa dòng nào.</b> Lời gọi chỉ tạo ra một generator object. Thân hàm bắt đầu chạy ở lần <code>next()</code> đầu tiên và dừng lại ở mỗi <code>yield</code>.</p></details>
</div>

## Comprehension: gọn khi một ý, rối khi lồng

Dữ liệu dùng lại sáu chuyến của bài 05. Code của bài nằm ở `examples/toi-noi-python/06-comprehension-generator/main.py`.

```python
expensive = [t.id for t in TRIPS if t.fare >= Decimal("14250")]   # list comprehension
fare_by_id = {t.id: t.fare for t in TRIPS}                         # dict comprehension
cities = {t.city for t in TRIPS}                                   # set comprehension
```

Kết quả thật: `['TN-0001', 'TN-0003', 'TN-0004', 'TN-0006']` cho list, `['DN', 'HN', 'HP', 'SG']` cho set (đã `sorted` để in). Ba dòng trên đọc như một câu: «lấy mã của mỗi chuyến mà cước từ 14.250». Biến vòng lặp `t` nằm trong phạm vi riêng của comprehension, không rò ra ngoài.

Comprehension hợp khi mỗi phần tử có một biểu thức ngắn, tối đa một điều kiện. Nó xấu đi khi lồng nhiều tầng. Lab có hai cách tính cùng một thứ: gom chuyến có cước từ 14.250 theo từng tài xế.

```python
# Cách 1: một dòng lồng nhau. Quét TRIPS một lần cho mỗi tài xế, khó đọc.
one_liner = {
    d: [t.id for t in TRIPS if t.driver == d and t.fare >= Decimal("14250")]
    for d in {t.driver for t in TRIPS if t.driver}
}
# Cách 2: vòng for thường. Dài hơn nhưng đọc từ trên xuống và quét một lần.
loop = defaultdict(list)
for t in TRIPS:
    if t.driver and t.fare >= Decimal("14250"):
        loop[t.driver].append(t.id)
```

Hai cách ra cùng kết quả `{'DR-001': ['TN-0001', 'TN-0003'], 'DR-003': ['TN-0004']}` sau khi bỏ khóa có list rỗng (cách 1 còn tạo khóa `DR-002` với list rỗng, cách 2 không), nhưng cách 1 có độ phức tạp tỷ lệ tích số tài xế và số chuyến. Với 25.000 tài xế và vài triệu chuyến, đây là lỗi hiệu năng ẩn trong một dòng "thông minh". Dấu hiệu nên chuyển về vòng `for`: phải cuộn ngang, có `if` lồng nhau, cần log hay bắt exception từng phần tử, hoặc đang dùng comprehension chỉ để gọi hàm có side effect (`[print(x) for x in xs]`).

## Iterator: vòng `for` thật sự làm gì

Một **iterable** là thứ có thể đưa cho `iter()` để lấy ra một **iterator**; iterator là đối tượng có `__next__()` trả từng phần tử và ném `StopIteration` khi hết, đồng thời có `__iter__()` trả về chính nó (nhờ vậy iterator cũng dùng được trong vòng `for`). List, tuple, dict, set, chuỗi, file đều là iterable.

```python
ids = ["TN-0001", "TN-0002"]
it = iter(ids)           # iterable -> iterator
next(it)                 # 'TN-0001'
next(it)                 # 'TN-0002'
next(it)                 # StopIteration

it2 = iter(ids)
while True:              # đây là điều vòng for làm ngầm
    try:
        item = next(it2)
    except StopIteration:
        break
```

Vòng `for item in ids:` gọi `iter(ids)` một lần, rồi `next()` lặp lại cho tới khi gặp `StopIteration` và kết thúc êm, không có lỗi nào lan ra ngoài. Lab kiểm ra ba điều: `iter(ids) is ids` là `False` (list là iterable, không phải iterator), `iter(it2) is it2` là `True` (iterator trả về chính nó, nên dùng trong `for` được), và vòng `while` thủ công cho kết quả bằng vòng `for`. Một iterator đã đi qua thì không quay lại: muốn duyệt lại, ta phải lấy iterator mới từ iterable gốc.

## Generator và `yield`: hàm chạy từng đoạn

Hàm thường chạy một mạch rồi `return`. Hàm có `yield` là **generator function**: mỗi lần được hỏi, nó chạy tới `yield`, đưa một giá trị ra và đứng yên cùng toàn bộ biến cục bộ, chờ lần sau.

```python
def trip_ids(log):
    for t in TRIPS[:3]:
        log.append(f"yield {t.id}")
        yield t.id

log = []
gen = trip_ids(log)      # chưa chạy dòng nào của thân hàm: log == []
next(gen)                # 'TN-0001'; log == ['yield TN-0001']
list(gen)                # ['TN-0002', 'TN-0003']: kéo nốt phần còn lại
```

Đây là **lazy evaluation**: giá trị chỉ được tính khi có người kéo. Generator object cũng là iterator, nên `for`, `list()`, `sum()`, `max()` đều dùng được với nó. Biểu thức generator có cú pháp giống comprehension nhưng dùng ngoặc tròn và không tạo list:

```python
squares = (n * n for n in range(1, 4))
sum(squares)   # 14
sum(squares)   # 0: đã cạn, chỉ tiêu thụ được một lần
```

Hệ quả thực tế: không gọi được `len()` trên generator, không lấy phần tử bằng chỉ số, và nếu truyền cùng một generator cho hai nơi thì nơi thứ hai nhận phần còn lại (có thể là rỗng). Cần dùng nhiều lần thì tạo lại generator, hoặc chấp nhận `list()` để lưu kết quả.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Pipeline generator: nguồn sinh điểm GPS, bộ lọc tốc độ phi lý, bước map định dạng, rồi bên tiêu thụ sum hoặc islice kéo từng phần tử từ phải sang trái."><defs><marker id="py06-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py06-0-pull" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><text x="250" y="22" class="t-small">consumer kéo: mỗi next() đi ngược về nguồn, một phần tử đi xuôi</text><g transform="translate(0,30)"><rect x="20" y="50" width="185" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="82" class="t-b">source</text><text x="34" y="104" class="t-mono">gps_stream(n)</text><text x="34" y="124" class="t-small">yield 1 điểm GPS</text><rect x="245" y="50" width="185" height="90" rx="10" class="f-phys-soft s-phys"/><text x="259" y="82" class="t-b">filter</text><text x="259" y="104" class="t-mono">is_absurd(p)</text><text x="259" y="124" class="t-small">giữ tốc độ &gt; 120</text><rect x="470" y="50" width="185" height="90" rx="10" class="f-phys-soft s-phys"/><text x="484" y="82" class="t-b">map</text><text x="484" y="104" class="t-mono">(seq, km/h)</text><text x="484" y="124" class="t-small">chọn trường cần</text><rect x="695" y="50" width="165" height="90" rx="10" class="f-warn-soft s-warn"/><text x="709" y="82" class="t-b">consumer</text><text x="709" y="104" class="t-mono">sum / islice</text><text x="709" y="124" class="t-small">gọi next()</text><path d="M205 95 L243 95" class="f-none s-phys" stroke-width="2" marker-end="url(#py06-0-arrow)"/><path d="M430 95 L468 95" class="f-none s-phys" stroke-width="2" marker-end="url(#py06-0-arrow)"/><path d="M655 95 L693 95" class="f-none s-phys" stroke-width="2" marker-end="url(#py06-0-arrow)"/><path d="M777 50 C 777 10, 112 10, 112 48" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py06-0-pull)"/><text x="20" y="190" class="t-small">Tại mỗi thời điểm chỉ có một điểm đang đi qua đường ống, không có list trung gian.</text></g></svg>
</div>
<figcaption>Hình 1. Pipeline generator: bên tiêu thụ kéo từng phần tử qua nguồn, bộ lọc và bước biến đổi. Đường đứt màu vàng là yêu cầu kéo, mũi tên liền là dữ liệu đi xuôi; chưa ai kéo thì nguồn chưa sinh gì.</figcaption>
</figure>

## 1 triệu điểm GPS mà không nạp hết vào list

Lab giả lập luồng vị trí bằng một generator. Mỗi điểm gồm số thứ tự, mã tài xế (1 đến 25.000), tốc độ, vĩ độ, kinh độ. Dữ liệu sinh bằng `random.Random(seed)` với seed cố định, nên mỗi lần chạy ra đúng cùng một dòng dữ liệu; ta chỉ dùng `random()` vì tài liệu của module `random` cam kết: cùng seed thì `random()` cho cùng một chuỗi số giữa các phiên bản Python, còn các phương thức khác (như `gauss` hay `choice`) có thể đổi thuật toán. Khoảng 0,1% điểm là «lỗi GPS» với tốc độ 150 đến 400 km/h; còn lại 0 đến 60 km/h. Ngưỡng phi lý là trên 120 km/h.

```python
def gps_stream(n, seed=SEED):
    rng = Random(seed)
    for seq in range(n):
        driver_no = 1 + int(rng.random() * DRIVERS_ONLINE)
        if rng.random() < 0.001:
            speed = 150 + rng.random() * 250
        else:
            speed = rng.random() * 60
        lat = 10.70 + rng.random() * 0.20
        lng = 106.60 + rng.random() * 0.20
        yield GpsPoint(seq, driver_no, speed, lat, lng)

def count_absurd_stream(n=POINTS):
    return sum(1 for p in gps_stream(n) if is_absurd(p))      # chỉ giữ 1 điểm

def absurd_via_list(n=POINTS):
    points = list(gps_stream(n))                              # nạp hết n điểm
    return len([p for p in points if is_absurd(p)])
```

Hai hàm cho cùng đáp án. Muốn thấy vài điểm đầu mà không duyệt hết, ta ghép `filter` với `islice`:

```python
[(p.seq, p.driver_no, round(p.speed_kmh, 1))
 for p in islice(filter(is_absurd, gps_stream(POINTS)), 3)]
```

Kết quả thật: `[(2306, 14312, 342.2), (2769, 3135, 160.5), (4002, 10478, 156.1)]`. Một triệu điểm tương đương 1.000.000 ÷ 6.250 = 160 giây dữ liệu ở mức tải cao điểm. Lab đo bằng `tracemalloc`, ghi lại **đỉnh** bộ nhớ trong lúc chạy:

```python
def peak_memory(fn, *args):
    tracemalloc.start()
    try:
        result = fn(*args)
        _, peak = tracemalloc.get_traced_memory()
    finally:
        tracemalloc.stop()
    return result, peak
```

Số liệu từ một lần chạy trên máy tác giả:

| Cách làm | Điểm phi lý | Đỉnh bộ nhớ (`tracemalloc`) | Thời gian (đo riêng, không bật `tracemalloc`) |
| --- | --- | --- | --- |
| Generator, duyệt từng điểm | 972 | 3,8 KiB | 0,43 s |
| `list(...)` nạp hết rồi lọc | 972 | 229,0 MiB | 0,70 s |

Cùng 972 điểm phi lý, nhưng list giữ cả triệu điểm cùng lúc nên đỉnh bộ nhớ gấp khoảng 62 nghìn lần. Số điểm phi lý cố định theo seed; thời gian và số byte thay đổi theo máy và theo phiên bản Python, chỉ nên so sánh tỷ lệ. Một service phải xử lý nhiều luồng song song sẽ nhân con số 229 MiB lên theo số luồng; còn mô hình generator thì bộ nhớ không đổi dù luồng dài bao nhiêu.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="So sánh: list giữ một triệu điểm trong bộ nhớ cùng lúc, khoảng 229 MiB; generator chỉ giữ một điểm đang xử lý, khoảng vài KiB."><text x="20" y="28" class="t-b">list: nạp hết rồi mới lọc</text><rect x="20" y="40" width="840" height="86" rx="10" class="f-warn-soft s-warn"/><rect x="38" y="62" width="44" height="42" rx="4" class="f-bg s-warn"/><rect x="92" y="62" width="44" height="42" rx="4" class="f-bg s-warn"/><rect x="146" y="62" width="44" height="42" rx="4" class="f-bg s-warn"/><rect x="200" y="62" width="44" height="42" rx="4" class="f-bg s-warn"/><rect x="254" y="62" width="44" height="42" rx="4" class="f-bg s-warn"/><rect x="308" y="62" width="44" height="42" rx="4" class="f-bg s-warn"/><rect x="362" y="62" width="44" height="42" rx="4" class="f-bg s-warn"/><rect x="416" y="62" width="44" height="42" rx="4" class="f-bg s-warn"/><text x="476" y="90" class="t-b">...</text><text x="520" y="80" class="t-small">1.000.000 điểm cùng ở trong RAM</text><text x="520" y="100" class="t-b t-warn">đỉnh khoảng 229 MiB</text><text x="20" y="162" class="t-b">generator: một điểm tại một thời điểm</text><rect x="20" y="174" width="840" height="86" rx="10" class="f-phys-soft s-phys"/><rect x="38" y="196" width="44" height="42" rx="4" class="f-none s-phys dash"/><rect x="92" y="196" width="44" height="42" rx="4" class="f-none s-phys dash"/><rect x="146" y="196" width="44" height="42" rx="4" class="f-pk s-pk"/><rect x="200" y="196" width="44" height="42" rx="4" class="f-none s-phys dash"/><rect x="254" y="196" width="44" height="42" rx="4" class="f-none s-phys dash"/><text x="320" y="214" class="t-small">đã xử lý: bị thu hồi    |    chưa sinh: chưa tồn tại</text><text x="520" y="234" class="t-b t-phys">đỉnh khoảng 3,8 KiB</text></svg>
</div>
<figcaption>Hình 2. list nạp hết giữ mọi phần tử cùng lúc; generator giữ đúng một phần tử đang xử lý (ô đặc), các ô nét đứt đã được thu hồi hoặc chưa được sinh ra.</figcaption>
</figure>

`sys.getsizeof` dễ gây hiểu nhầm ở đây. Với list triệu điểm nó báo 8.448.728 byte, chỉ khoảng 8 MiB, vì `getsizeof` đo «vỏ ngoài» (mảng con trỏ), không cộng các tuple và số thực mà list trỏ tới. Với generator nó báo 296 byte và con số này không đổi dù nguồn dài bao nhiêu. Muốn biết tổng bộ nhớ Python cấp phát cho các đối tượng, dùng `tracemalloc` như trên. Lưu ý đó không phải bộ nhớ tiến trình (RSS) mà hệ điều hành báo.

## `itertools` và các hàm tổng hợp

Module `itertools` cung cấp các mảnh ghép để xây pipeline mà không materialize (tạo hẳn list trong bộ nhớ). Lab dùng bốn thứ:

```python
head = [p.seq for p in islice(gps_stream(POINTS), 3)]   # [0, 1, 2]: lấy 3 điểm đầu của 1 triệu
list(chain(["TN-0001"], ["TN-0002", "TN-0003"]))       # nối nhiều iterable
list(batched((t.id for t in TRIPS), 4))                 # từng nhóm 4 phần tử, nhóm cuối có thể ngắn hơn
```

Kết quả `batched` là `[('TN-0001', 'TN-0002', 'TN-0003', 'TN-0004'), ('TN-0005', 'TN-0006')]`. `itertools.batched` có sẵn trong thư viện chuẩn từ Python 3.12, không cần cài thêm gói; hữu ích khi ghi 1 triệu điểm xuống database từng lô 1.000 dòng.

`groupby` là hàm đặt bẫy người mới. Nó chỉ gom các phần tử **liền nhau** có cùng khóa, không như `GROUP BY` của SQL. Cùng sáu chuyến, khi chưa sắp xếp, ta nhận sáu nhóm:

```text
unsorted_groups : [('SG', 1), ('HN', 1), ('SG', 1), ('DN', 1), ('SG', 1), ('HP', 1)]
sorted_groups   : [('DN', 1), ('HN', 1), ('HP', 1), ('SG', 3)]
```

Muốn mỗi thành phố một nhóm, phải `sorted(..., key=...)` theo cùng khóa trước. Lưu ý `sorted` trả về list nên nó phá tính lười của pipeline: dữ liệu phải nạp hết để sắp xếp. Với luồng rất lớn, `Counter` hoặc `defaultdict(list)` ở bài 05 thường hợp lý hơn `groupby`.

Các hàm tổng hợp `sum`, `max`, `min`, `any`, `all` nhận thẳng generator. `any` và `all` dừng ngay khi biết đáp án (short-circuit). Trên luồng giả lập, `any(is_absurd(p) for p in gps_stream(POINTS))` trả `True` sau khi mới kéo **2.307** điểm trong số 1 triệu; lab đếm số lần kéo bằng một generator bọc. Với 100.000 điểm đầu, `max(p.speed_kmh ...)` ra 398,81 km/h và `all(...)` xác nhận mọi vĩ độ nằm trong khung thành phố. Cả hai đều chạy mà không lập list.

## Đối chiếu LINQ, và giới hạn của ví dụ

Trong C#, `Where` và `Select` của LINQ to Objects cũng trì hoãn: khai báo query chưa chạy predicate, chỉ khi `foreach` hoặc `ToList` mới chạy. Generator expression của Python gần với một query LINQ, còn `list(...)` tương ứng với `ToList()`. Khác biệt quan trọng: query LINQ duyệt lại sẽ chạy lại từ nguồn (bài C# cho thấy duyệt lần hai thấy cả dòng mới thêm), còn generator đã cạn thì duyệt lại ra rỗng. Muốn nhiều lần duyệt trong Python, bạn tạo lại generator từ iterable gốc, hoặc lưu bằng `list`.

Cả hai thế giới có chung một cạm bẫy: lazy không đồng nghĩa mọi bước đều streaming. `sorted` và `groupby` sau khi sort cần toàn bộ dữ liệu; còn lỗi trong generator chỉ lộ ra tại thời điểm kéo, không phải lúc khai báo, nên traceback có thể trỏ tới nơi bạn không nghĩ.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn so với production</span>
<p>Dữ liệu ở đây do generator tự sinh trong cùng một process, còn dữ liệu thật đến từ Kafka, qua mạng, có độ trễ, thứ tự đảo và điểm thiếu. Tốc độ là một trường có sẵn trong điểm giả; thực tế phải suy ra từ khoảng cách giữa hai điểm liên tiếp và chênh lệch thời gian, sau đó mới lọc nhiễu. Ngưỡng 120 km/h là giả định để học. Lab chạy một luồng đơn, không xử lý lỗi từng điểm, và số đo bộ nhớ là của một lần chạy trên một máy.</p>
</div>

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Generator trả rỗng ở lần duyệt thứ hai | Đã bị tiêu thụ ở lần đầu; tạo lại generator hoặc lưu bằng `list` |
| `TypeError: object of type 'generator' has no len()` | Generator không biết độ dài; đếm bằng `sum(1 for _ in g)` hoặc lưu list |
| `groupby` cho nhiều nhóm trùng khóa | Chưa `sorted` theo cùng khóa; `groupby` chỉ gom phần tử liền nhau |
| RAM tăng vọt khi xử lý file/luồng lớn | Có `list(...)` hoặc `[... for ...]` ở giữa pipeline không; đổi sang generator expression |
| Lỗi chỉ xuất hiện khi chạy một nơi xa chỗ khai báo | Lazy: lỗi bung ra khi có người kéo; kiểm tra chỗ tiêu thụ |

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/06-comprehension-generator examples/toi-noi-python/06-comprehension-generator/main.py
uv run --project examples/toi-noi-python/06-comprehension-generator examples/toi-noi-python/06-comprehension-generator/main.py --verify
```

Lệnh đầu chạy vài giây và in các khối kết quả như trên, gồm hai dòng so sánh generator và list. Lệnh thứ hai kiểm 15 kết quả: comprehension, iterator, generator lười và dùng một lần, `islice`, `chain`, `batched`, `groupby`, số điểm phi lý bằng 972, generator và list cho cùng đáp án, và đỉnh bộ nhớ generator nhỏ hơn rất nhiều so với list. Kết quả cuối:

```text
OK 15/15
```

Sai kiểm tra nào, chương trình in `FAIL` kèm tên và thoát với mã 1.

**Bài tập:** (1) Viết generator expression tính tổng tốc độ của các điểm phi lý trong 1 triệu điểm mà không tạo list. (2) Vì sao `groupby(gps_stream(1000), key=lambda p: p.driver_no)` hầu như cho mỗi điểm một nhóm riêng?

<details><summary>Xem lời giải</summary><p>(1) <code>sum(p.speed_kmh for p in gps_stream(POINTS) if is_absurd(p))</code>: generator expression truyền thẳng vào <code>sum</code>, bộ nhớ không đổi, và <code>sum</code> chỉ kéo từng điểm một. (2) Điểm của cùng một tài xế nằm rải rác trong luồng (mã tài xế được sinh ngẫu nhiên trong 25.000 mã), còn <code>groupby</code> chỉ gom các phần tử liền kề cùng khóa; muốn gom theo tài xế phải dùng <code>defaultdict(list)</code> hoặc sắp xếp theo <code>driver_no</code> trước (và chấp nhận nạp hết dữ liệu để sort).</p></details>

Bài trước: [bài 05](/articles/python-list-dict-set-tuple). Bài tiếp: [bài 07](/articles/python-module-package-import).

## Tài liệu gốc

- [Python tutorial: Iterators](https://docs.python.org/3.14/tutorial/classes.html#iterators).
- [Python tutorial: Generators](https://docs.python.org/3.14/tutorial/classes.html#generators).
- [Python tutorial: List comprehensions](https://docs.python.org/3.14/tutorial/datastructures.html#list-comprehensions).
- [itertools: functions creating iterators for efficient looping](https://docs.python.org/3.14/library/itertools.html).
- [tracemalloc: trace memory allocations](https://docs.python.org/3.14/library/tracemalloc.html).
