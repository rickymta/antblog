---
slug: python-list-dict-set-tuple
title: "List, dict, set, tuple: quản lý chuyến và tài xế trong bộ nhớ"
summary: "Bài 05: chọn list, tuple, dict, set, deque theo thao tác; hiểu mutability, alias, hashability; dùng Counter và defaultdict để đếm, gom chuyến Tới Nơi. Có lab chạy được và --verify."
category: python-beginner
level: beginner
tags: [python, toi-noi, collections, dict]
language: vi
---

Điều phối của [Tới Nơi](/articles/du-an-mau-toi-noi) cần trả lời bốn câu hỏi nhỏ trong vài mili-giây: chuyến nào đang chờ, tài xế `DR-002` là ai, tài xế nào vừa đang online vừa đủ điều kiện nhận chuyến, và chuyến nào được gán trước. Cả bốn đều là "một đống phần tử", nhưng mỗi câu cần một cấu trúc dữ liệu khác nhau.

Đây là bài 05 của lộ trình **Python cơ bản qua Tới Nơi**. Bạn đã biết hàm, vòng lặp và điều kiện từ bài 03; hôm nay ta gom dữ liệu vào bốn cấu trúc dựng sẵn của Python, nhìn rõ cái giá của từng cấu trúc, rồi dừng lại ở một cái bẫy kinh điển: hai tên cùng trỏ vào một list. Ai đã đọc bài C# về [generics và collections](/articles/csharp-generics-collections) sẽ thấy bốn lựa chọn quen thuộc (`List`, `Dictionary`, `HashSet`, `Queue`), nhưng cách Python làm khác ở vài chỗ quan trọng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Sau <code>b = a</code> với <code>a</code> là một list, rồi <code>b.append("TN-0003")</code>, list <code>a</code> có chứa <code>"TN-0003"</code> không?</p>
<details><summary>Xem đáp án</summary><p><b>Có.</b> <code>b = a</code> không sao chép list, nó chỉ gắn thêm một tên cho cùng một đối tượng. Muốn bản sao độc lập phải gọi <code>a.copy()</code> hoặc <code>copy.deepcopy(a)</code>.</p></details>
</div>

## Bốn thao tác, bốn cấu trúc

Chọn cấu trúc bắt đầu từ thao tác chính, không phải từ thói quen. Cần giữ thứ tự và lấy theo vị trí: **list**. Cần tìm theo mã: **dict**. Cần biết «đã có chưa» và loại trùng: **set**. Cần lấy từ đầu hàng ra: **deque**. Còn **tuple** là bản ghi nhỏ bất biến, dùng làm khoá ghép hoặc gói vài giá trị đi cùng nhau; nó không phải thao tác thứ năm mà là cách đóng gói dữ liệu cho bốn cấu trúc kia. Cho cả bốn dùng chung list thì ở 6 phần tử vẫn chạy đúng, nhưng với 25.000 tài xế online thì mỗi lần «tìm tài xế theo mã» quét cả list sẽ chậm rõ rệt.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Bốn cấu trúc song song theo thao tác: list lấy theo vị trí, dict tra theo khoá, set kiểm tra thành viên, deque lấy từ đầu hàng."><rect x="25" y="25" width="395" height="85" rx="10" class="f-phys-soft s-phys"/><text x="41" y="55" class="t-b">list</text><text x="41" y="78" class="t-small">Thứ tự, lấy theo vị trí</text><text x="41" y="98" class="t-mono">ids[2]  ids.append(x)</text><rect x="450" y="25" width="395" height="85" rx="10" class="f-pk-soft s-pk"/><text x="466" y="55" class="t-b">dict</text><text x="466" y="78" class="t-small">Tra cứu theo khoá</text><text x="466" y="98" class="t-mono">drivers["DR-001"]</text><rect x="25" y="140" width="395" height="85" rx="10" class="f-pk-soft s-pk"/><text x="41" y="170" class="t-b">set</text><text x="41" y="193" class="t-small">Thành viên duy nhất, phép tập hợp</text><text x="41" y="213" class="t-mono">"DR-002" in online   a &amp; b</text><rect x="450" y="140" width="395" height="85" rx="10" class="f-phys-soft s-phys"/><text x="466" y="170" class="t-b">deque</text><text x="466" y="193" class="t-small">Vào cuối, ra đầu (FIFO)</text><text x="466" y="213" class="t-mono">q.append(x)  q.popleft()</text></svg>
</div>
<figcaption>Hình 1. Bốn cấu trúc chạy song song theo bốn thao tác chính, không phải một chuỗi biến đổi từ cấu trúc này sang cấu trúc khác.</figcaption>
</figure>

Bảng sau tóm tắt cái giá của từng thao tác. Các độ phức tạp là trung bình theo tài liệu và cách cài đặt phổ biến của CPython, đủ để chọn đúng cấu trúc; không phải cam kết cho mọi phiên bản hay mọi dữ liệu.

| Cấu trúc | Thao tác phù hợp | Cái giá |
| --- | --- | --- |
| `list` | Lấy `x[i]`, thêm cuối `append`, lấy cuối `pop()` | O(1) cho ba thao tác này (`append` là O(1) khấu hao: thỉnh thoảng list phải cấp phát lại, nhưng trung bình mỗi lần thêm vẫn O(1)); `x in lst` là O(n); `insert(0, x)` và `pop(0)` O(n) vì phải dịch cả list; `sort` O(n log n) |
| `tuple` | Bản ghi nhỏ bất biến, khoá ghép | Không sửa được; nếu mọi phần tử hash được thì cả tuple hash được |
| `dict` | Tra, thêm, xoá theo khoá | O(1) trung bình; tốn thêm RAM cho bảng băm; khoá phải hash được |
| `set` | `in`, `add`, `remove`, phép giao/hợp | O(1) trung bình cho `in`/`add`; không có thứ tự, không có chỉ số |
| `deque` | `append`, `popleft` hai đầu | O(1) hai đầu; lấy phần tử giữa O(n) |

## List và tuple: thứ tự, sửa được và bất biến

Toàn bộ code của bài nằm ở `examples/toi-noi-python/05-list-dict-set-tuple/main.py`. Dữ liệu gồm sáu chuyến `TN-0001` đến `TN-0006`, mỗi chuyến là một **tuple có tên** (`NamedTuple`): vẫn là tuple bất biến, nhưng đọc được bằng `trip.city` thay vì `trip[1]`. Ba tài xế `DR-001` đến `DR-003` nằm trong một dict, sáu thành phố trong một dict khác.

```python
class Trip(NamedTuple):
    id: str
    city: str
    service: str
    driver: str | None  # None = chưa có tài xế nhận
    fare: Decimal
    status: str

TRIPS = [
    Trip("TN-0001", "SG", "ride", "DR-001", Decimal("25500"), "completed"),
    Trip("TN-0002", "HN", "food", "DR-002", Decimal("12000"), "completed"),
    # ... bốn chuyến còn lại; TN-0005 và TN-0006 chưa có tài xế
]
```

List giữ thứ tự chèn và sửa được. Cắt lát `ids[:2]` tạo một list mới; chỉ số âm đếm từ cuối. `insert(0, x)` và `pop(0)` làm việc ở đầu list và dịch mọi phần tử còn lại, nên với hàng dài ta sẽ đổi sang `deque` ở mục sau.

```python
ids = [t.id for t in TRIPS]
first_two = ids[:2]           # ['TN-0001', 'TN-0002']
last = ids[-1]                # 'TN-0006'
queue_view = ids.copy()
queue_view.append("TN-0007")  # thêm cuối
queue_view.insert(0, "TN-0000")  # chèn đầu: dịch cả list
by_fare = [t.id for t in sorted(TRIPS, key=lambda t: (-t.fare, t.id))]
```

`sorted` trả về list mới, còn `list.sort()` sắp xếp tại chỗ và trả `None`. Tham số `key` nhận một hàm trả về giá trị để so sánh. Ở đây khoá `(-t.fare, t.id)` là một tuple: cước giảm dần, hai chuyến cùng cước thì mã tăng dần. Kết quả thật của lab:

```text
theo cước  : ['TN-0001', 'TN-0004', 'TN-0003', 'TN-0006', 'TN-0002', 'TN-0005']
```

Tuple khác list ở chỗ không thêm, bớt hay gán lại phần tử được. Gán `pickup[0] = 0.0` cho tuple ném `TypeError`. Bất biến này đem lại ba lợi ích: một bản ghi không bị sửa nhầm ở nơi khác, tuple hash được nên dùng làm khoá dict, và đọc code biết ngay đây là "một bộ giá trị cố định". Unpack giúp đọc tuple gọn:

```python
trip_id, city, *_rest = TRIPS[0]   # 'TN-0001', 'SG'; *_rest nhận phần còn lại
lat, lng = (10.7769, 106.7009)

by_city_service = {}
for t in TRIPS:
    key = (t.city, t.service)      # tuple làm khóa ghép
    by_city_service[key] = by_city_service.get(key, 0) + 1
```

Python không ép kiểu phần tử như `List<string>` của C#: một list có thể trộn `str` và `int`. Gợi ý kiểu như `list[str]` giúp trình kiểm tra tĩnh (mypy, ta học ở bài 12) và người đọc, nhưng interpreter không chặn khi bạn thêm sai kiểu lúc chạy.

## Dict và set: tìm theo khoá, hỏi «có không»

Dict ánh xạ khoá sang giá trị. Từ Python 3.7, thứ tự chèn là một phần của ngôn ngữ, nên `list(DRIVERS)` luôn ra `['DR-001', 'DR-002', 'DR-003']`. Có hai cách tra khoá vắng mặt: `d[key]` ném `KeyError`, còn `d.get(key)` trả `None` (hoặc giá trị mặc định bạn đưa vào). Đây là chỗ Python gần với `TryGetValue` của C#.

```python
DRIVERS["DR-001"]["name"]        # 'Nguyễn Văn An'
DRIVERS.get("DR-999")            # None
DRIVERS.get("DR-999", {"name": "(không có)"})["name"]   # '(không có)'
# DRIVERS["DR-999"] -> KeyError

shifts = {}
shifts.setdefault("DR-001", []).append("sáng")
shifts.setdefault("DR-001", []).append("chiều")   # khóa đã có: không tạo list mới
pairs = [(code, d["city"]) for code, d in DRIVERS.items()]
```

`setdefault` lấy giá trị nếu khoá có, còn chưa có thì chèn giá trị mặc định rồi trả về. Chú ý rằng đối số mặc định `[]` được tạo mỗi lần gọi dù khoá đã có; với giá trị đắt, dùng `defaultdict` ở mục sau. Lab in:

```text
setdefault : {'DR-001': ['sáng', 'chiều']} | thứ tự: ['DR-001', 'DR-002', 'DR-003']
items      : [('DR-001', 'SG'), ('DR-002', 'HN'), ('DR-003', 'DN')]
```

Set là tập hợp không trùng, không thứ tự. Câu hỏi «tài xế nào vừa online vừa đủ điều kiện» chính là giao của hai tập:

```python
online = {"DR-001", "DR-002", "DR-002"}   # trùng bị loại: len == 2
eligible = {"DR-002", "DR-003"}
sorted(online & eligible)   # ['DR-002']                  giao
sorted(online | eligible)   # ['DR-001', 'DR-002', 'DR-003']  hợp
sorted(online - eligible)   # ['DR-001']                  online nhưng chưa đủ điều kiện
sorted(online ^ eligible)   # ['DR-001', 'DR-003']        đúng một trong hai
```

Ta luôn bọc kết quả bằng `sorted()` khi in, vì thứ tự duyệt set không được ngôn ngữ đảm bảo (với chuỗi, nó còn thay đổi giữa các lần chạy do băm chuỗi được xáo ngẫu nhiên). Đừng viết logic phụ thuộc thứ tự một set. Lợi thế lớn của set là `x in s` trung bình O(1). Lab có chế độ `--bench`; trên máy tác giả, tìm phần tử cuối của 20.000 mã 500 lần mất khoảng 83 ms với list và khoảng 0,016 ms với set (số đo thay đổi theo máy, chỉ nên đọc theo tỷ lệ).

## Mutability, alias và hashability

Hai khái niệm này giải thích phần lớn lỗi «sao dữ liệu tự đổi?» và «sao không dùng được làm khoá?».

**Mutability.** List, dict, set sửa được tại chỗ. Số, chuỗi, tuple không sửa được: mọi phép «đổi» thực ra tạo đối tượng mới. **Alias.** Phép gán `b = a` không bao giờ sao chép; nó gắn thêm tên. `a is b` trả `True` vì hai tên cùng trỏ một đối tượng, nên `b.append(...)` hiện ra ở cả `a`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hai tên a và b cùng trỏ vào một đối tượng list chứa TN-0001, TN-0002 và TN-0003; không có list thứ hai."><defs><marker id="py05-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="40" y="35" width="170" height="64" rx="10" class="f-pk-soft s-pk"/><text x="58" y="62" class="t-b t-mono">a</text><text x="58" y="82" class="t-small">tên (biến)</text><rect x="40" y="145" width="170" height="64" rx="10" class="f-pk-soft s-pk"/><text x="58" y="172" class="t-b t-mono">b = a</text><text x="58" y="192" class="t-small">chỉ thêm một tên</text><path d="M210 67 L450 105" class="f-none s-pk" stroke-width="2" marker-end="url(#py05-1-arrow)"/><path d="M210 177 L450 140" class="f-none s-pk" stroke-width="2" marker-end="url(#py05-1-arrow)"/><rect x="455" y="70" width="380" height="100" rx="10" class="f-warn-soft s-warn"/><text x="473" y="98" class="t-b">một đối tượng list</text><text x="473" y="124" class="t-mono">["TN-0001", "TN-0002", "TN-0003"]</text><text x="473" y="150" class="t-small">b.append("TN-0003") làm đổi cái nhìn qua a</text><text x="455" y="205" class="t-small">Bản sao thật cần: a.copy() hoặc copy.deepcopy(a)</text></svg>
</div>
<figcaption>Hình 2. Alias: hai tên, một list. Hộp có viền đậm đánh dấu chỗ phải trả giá, nơi một sửa đổi ảnh hưởng mọi tên đang trỏ tới.</figcaption>
</figure>

`copy()` làm bản sao nông: lớp ngoài mới, phần tử bên trong vẫn dùng chung. Với dict chứa list, bản sao nông và bản gốc vẫn chia sẻ list bên trong; `copy.deepcopy` sao chép toàn bộ đồ thị đối tượng.

```python
a = ["TN-0001", "TN-0002"]
b = a
b.append("TN-0003")                # a cũng đổi; a is b == True

original = {"driver": "DR-001", "trips": ["TN-0001"]}
shallow = original.copy()
deep = copy.deepcopy(original)
shallow["trips"].append("TN-0003")  # original["trips"] cũng đổi
deep["trips"].append("TN-0009")     # original không đổi
```

Đầu ra thật: `original_trips: ['TN-0001', 'TN-0003']`, `shallow_shares: True`, `deep_shares: False`. Một cái bẫy cùng họ là tham số mặc định mutable như `def f(x=[])`: list mặc định được tạo một lần và dùng chung cho mọi lần gọi. C# cũng có alias với kiểu tham chiếu như `List<T>`; Python làm điều đó với mọi đối tượng, kể cả khi bạn không để ý.

**Hashability.** Dict và set cần băm khoá để tìm trong O(1). Một đối tượng hash được khi giá trị băm không đổi trong suốt đời nó; vì vậy list, dict, set (sửa được) không hash được, còn str, số, tuple chứa toàn phần tử hash được thì có. Ngay cả tuple cũng thất bại khi bên trong có list:

```text
{'tuple': True, 'str': True, 'list': False, 'tuple_with_list': False}
```

`TypeError: unhashable type: 'list'` khi dùng list làm khoá là cách Python bảo vệ bạn: nếu khoá đổi sau khi đã chèn vào bảng băm, dict sẽ không bao giờ tìm lại được nó.

## `collections`: đếm, gom và xếp hàng

Ba công cụ trong module `collections` giải bài toán nào cũng gặp ở Tới Nơi. **`Counter`** đếm phần tử, khoá vắng trả `0` thay vì `KeyError`. **`defaultdict(list)`** tự tạo list rỗng khi gặp khoá mới, thay cho mẫu `setdefault`. **`deque`** là hàng đợi hai đầu, `popleft()` O(1).

```python
per_city = Counter(t.city for t in TRIPS)     # đếm chuyến theo thành phố
per_city.most_common(1)                       # [('SG', 3)]
{code: per_city[code] for code in CITIES}     # CT và NT -> 0, đủ 6 thành phố

by_driver = defaultdict(list)
waiting = deque()
for t in TRIPS:
    if t.driver is None:
        waiting.append(t.id)                  # chưa có tài xế: vào hàng điều phối
    else:
        by_driver[t.driver].append(t.id)      # khóa mới tự có list rỗng
served = []
while waiting:
    served.append(waiting.popleft())          # FIFO: TN-0005 trước, TN-0006 sau
```

Kết quả thật:

```text
per_city_all: {'SG': 3, 'HN': 1, 'DN': 1, 'HP': 1, 'CT': 0, 'NT': 0}
by_driver   : {'DR-001': ['TN-0001', 'TN-0003'], 'DR-002': ['TN-0002'], 'DR-003': ['TN-0004']}
served      : ['TN-0005', 'TN-0006']
```

`defaultdict` có một tác dụng phụ: chỉ cần *đọc* `by_driver["DR-999"]` cũng chèn khoá đó. Khi muốn tra mà không chèn, dùng `.get()`. Bản `deque` tương ứng với `Queue<T>` của C#, nhưng đừng nhầm với `queue.Queue` của thư viện chuẩn: cái đó dành cho giao tiếp giữa các thread và có khoá riêng, ta để dành cho bài về concurrency.

## Giới hạn: đây là bộ nhớ, không phải database

Mọi cấu trúc trong bài chỉ sống trong RAM của một process. Tắt chương trình là mất hết. Không có transaction: nếu chương trình dừng giữa lúc đã thêm chuyến vào `trips` nhưng chưa cập nhật `by_driver`, hai cấu trúc lệch nhau và không ai rollback. Hai process (hai worker) mỗi bên có tập `online` riêng và không thấy dữ liệu của nhau, nên có thể cùng giao một chuyến cho hai tài xế.

Vì thế `set` chống trùng trong bài chưa phải idempotency. Chống xử lý trùng cho thanh toán cần khoá lưu bền và ràng buộc unique của database (loạt PostgreSQL của blog). Hàng đợi `deque` cũng chưa có xác nhận (ack) hay giao lại sau khi crash; hệ thống thật dùng hàng đợi bền có ack như RabbitMQ, hoặc Kafka (một log commit theo offset, nơi consumer tự quản vị trí đọc). Ví dụ ở đây dùng 6 chuyến và một luồng thực thi để học cú pháp, không phải thiết kế kho dữ liệu cho 25.000 tài xế online.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Sửa một list mà list khác cũng đổi | Có phải `b = a`, hoặc `copy()` nông với list lồng nhau; đổi sang `deepcopy` hoặc dựng cấu trúc mới |
| `TypeError: unhashable type: 'list'` | Đang dùng list/dict/set làm khoá hoặc phần tử set; đổi sang tuple hoặc chuỗi |
| `KeyError` khi tra tài xế | Dùng `d[key]` với khoá vắng; chọn `.get()` nếu vắng là bình thường |
| Thứ tự in của set thay đổi giữa các lần chạy | Set không đảm bảo thứ tự; dùng `sorted(s)` khi cần in |
| Hàng đợi chậm khi dài | Có đang dùng `list.pop(0)` không; đổi sang `deque.popleft()` |

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/05-list-dict-set-tuple examples/toi-noi-python/05-list-dict-set-tuple/main.py
uv run --project examples/toi-noi-python/05-list-dict-set-tuple examples/toi-noi-python/05-list-dict-set-tuple/main.py --verify
```

Lệnh đầu in kết quả từng mục như các khối `text` ở trên. Lệnh thứ hai so từng kết quả với giá trị kỳ vọng và cuối cùng in:

```text
OK 17/17
```

Nếu có kiểm tra sai, chương trình in `FAIL` kèm tên kiểm tra và thoát với mã 1. Thêm `--bench` để tự đo so sánh list và set trên máy của bạn.

**Bài tập:** (1) Dùng `Counter` để tìm dịch vụ (`ride`, `food`, `parcel`) có nhiều chuyến nhất trong `TRIPS`. (2) Cho `a = [1, 2]`, `c = [a, a]`, rồi `c[0].append(3)`. In `c` ra là gì, và `copy.deepcopy(c)` có giữ cấu trúc «hai phần tử trỏ cùng một list» không?

<details><summary>Xem lời giải</summary><p>(1) <code>Counter(t.service for t in TRIPS).most_common(1)</code>. Theo dữ liệu lab, <code>ride</code> có 3 chuyến (TN-0001, TN-0004, TN-0006), <code>food</code> có 2, <code>parcel</code> có 1, nên kết quả là <code>[('ride', 3)]</code>. (2) <code>c</code> là <code>[[1, 2, 3], [1, 2, 3]]</code>: hai phần tử là cùng một list. <code>deepcopy</code> ghi nhớ các đối tượng đã sao chép, nên bản sao cũng có hai phần tử trỏ cùng <i>một</i> list mới (độc lập với <code>a</code>), tức giữ nguyên cấu trúc chia sẻ bên trong.</p></details>

Bài trước: [bài 04](/articles/python-chuoi-f-string-ngay-gio-timezone). Bài tiếp: [bài 06](/articles/python-comprehension-generator-iterator).

## Tài liệu gốc

- [Python tutorial: Data Structures](https://docs.python.org/3.14/tutorial/datastructures.html).
- [Built-in types: list, tuple, dict, set](https://docs.python.org/3.14/library/stdtypes.html).
- [collections: Counter, defaultdict, deque](https://docs.python.org/3.14/library/collections.html).
- [copy: shallow and deep copy operations](https://docs.python.org/3.14/library/copy.html).
- [Glossary: hashable](https://docs.python.org/3.14/glossary.html#term-hashable).
