---
slug: python-dieu-kien-vong-lap-ham
title: "Điều kiện, vòng lặp và hàm: tách quy tắc tính cước"
summary: "Bài 03: dùng if/elif/else, for, while, tham số mặc định và keyword-only để tách calculate_fare(distance_km, *, service) cho ride, food, parcel; từ chối ngoài miền bằng ValueError, kiểm bằng --verify."
category: programming
level: beginner
tags: [python, toi-noi, control-flow, functions]
language: vi
---

Khi thêm dịch vụ giao đồ ăn, bộ phận vận hành của [Tới Nơi](/articles/du-an-mau-toi-noi) hỏi một câu tưởng đơn giản: «Công thức này có đang nhận cả đơn đồ ăn và bưu kiện không, hay chỉ tính cho xe?». Ở hai bài trước ta đã có hàm tính cước, nhưng nó còn mỏng: chưa kiểm miền khoảng cách, chưa có docstring ghi hợp đồng, chỉ biết một loại chuyến. Hôm nay ta bổ sung rẽ nhánh, vòng lặp, tham số có tên và một bộ kiểm tra để hàm ấy đủ tin cậy làm lõi cho các bài sau.

Đây là bài 03 của lộ trình **Python cơ bản qua Tới Nơi**. Bài tiếp nối [kiểu dữ liệu và Decimal](/articles/python-bien-kieu-du-lieu-decimal); ta vẫn dùng `Decimal` cho tiền và khoảng cách.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trong Python, <code>0 &lt; km &lt;= 100</code> có hợp lệ không, và nó có nghĩa giống <code>(0 &lt; km) &lt;= 100</code> không?</p>
<details><summary>Xem đáp án</summary><p>Hợp lệ và <b>không</b> giống. Python hiểu chuỗi so sánh như <code>0 &lt; km and km &lt;= 100</code>, với <code>km</code> chỉ được đánh giá một lần. Dạng có ngoặc so sánh kết quả <code>bool</code> của phép đầu với 100, thường không phải điều ta muốn.</p></details>
</div>

## Rẽ nhánh với if, elif, else

Khối lệnh trong Python được xác định bằng **thụt lề**, không có dấu `{}`. Sau dòng kết thúc bằng `:`, mọi dòng thụt vào cùng mức thuộc về cùng khối. Thụt sai mức gây `IndentationError`; trộn tab với khoảng trắng trong cùng khối gây `TabError` (một lớp con của `IndentationError`). Hãy để editor chèn 4 khoảng trắng cho mỗi mức.

```python
def classify_fare(total: Decimal) -> str:
    """if/elif/else: gắn nhãn mức cước để minh họa rẽ nhánh nhiều bậc."""
    if total <= BASE_FARE:
        return "cơ bản"
    elif total <= Decimal("50000"):
        return "trung bình"
    else:
        return "đường dài"
```

Các nhánh được thử từ trên xuống; nhánh đầu tiên có điều kiện đúng chạy và các nhánh sau bị bỏ qua. Vì vậy thứ tự `elif` quan trọng: nếu đặt điều kiện rộng lên trước, nhánh hẹp phía sau sẽ không bao giờ chạy.

Điều kiện ghép dùng `and`, `or`, `not`, đọc gần như tiếng Anh. `and` và `or` có short-circuit: vế phải chỉ được đánh giá khi cần. Python còn cho phép **chained comparison** để viết miền như toán học:

```python
if not MIN_KM <= distance_km <= MAX_KM:
    raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
```

`MIN_KM <= distance_km <= MAX_KM` là miền đóng 0 đến 100 km. Ta chủ động từ chối giá trị ngoài miền thay vì sửa lén thành 0 hay 100: một báo giá cho -1 km cần được người gọi biết là sai, không được trở thành một chuyến miễn phí.

Câu `if` ở trên là một **guard clause**: kiểm đầu vào ngay đầu hàm và thoát sớm bằng `raise` hoặc `return`, để phần còn lại của hàm chỉ phải nghĩ tới dữ liệu hợp lệ. Cách viết này tránh được chuỗi `if` lồng nhau ba bốn tầng thụt lề, khó đọc và khó kiểm thử.

Một lưu ý về «truthiness»: `Decimal("0")` được coi là sai trong điều kiện, như `0`, chuỗi rỗng hay list rỗng. Vì vậy `if not distance_km:` sẽ lẫn «chưa nhập» với «0 km hợp lệ». Khi cần phân biệt, hãy so sánh tường minh (`is None`, `== 0`) thay vì dựa vào truthiness.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Luồng rẽ nhánh của calculate_fare: kiểm tra khoảng cách trong miền 0 đến 100 km và dịch vụ hợp lệ, nếu sai thì raise ValueError; nếu đúng thì chọn extra bằng 0 hoặc tính phần vượt, rồi làm tròn và trả về.">
<defs>
<marker id="py03-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="py03-0-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<rect x="20" y="95" width="130" height="72" rx="10" class="f-phys-soft s-phys"/>
<text x="32" y="123" class="t-b">Nhận đầu vào</text>
<text x="32" y="145" class="t-small t-mono">km, service</text>
<rect x="185" y="95" width="170" height="72" rx="10" class="f-pk-soft s-pk"/>
<text x="197" y="118" class="t-b">Đầu vào hợp lệ?</text>
<text x="197" y="138" class="t-small t-mono">0 ≤ km ≤ 100</text>
<text x="197" y="154" class="t-small t-mono">service thuộc SERVICES</text>
<rect x="390" y="95" width="145" height="72" rx="10" class="f-pk-soft s-pk"/>
<text x="402" y="123" class="t-b">km ≤ 2 ?</text>
<text x="402" y="145" class="t-small">đã gồm 2 km</text>
<rect x="185" y="215" width="170" height="56" rx="10" class="f-warn-soft s-warn"/>
<text x="197" y="239" class="t-b t-warn">raise ValueError</text>
<text x="197" y="257" class="t-small">không trả giá</text>
<rect x="580" y="30" width="240" height="50" rx="10" class="f-phys-soft s-phys"/>
<text x="592" y="52" class="t-b">extra = 0</text>
<text x="592" y="68" class="t-small">đúng: chỉ thu giá cơ bản</text>
<rect x="580" y="106" width="240" height="50" rx="10" class="f-phys-soft s-phys"/>
<text x="592" y="128" class="t-b">extra = (km − 2) × 4.500</text>
<text x="592" y="144" class="t-small">sai: tính phần vượt</text>
<rect x="580" y="205" width="240" height="60" rx="10" class="f-phys-soft s-phys"/>
<text x="592" y="230" class="t-b">làm tròn rồi return</text>
<text x="592" y="248" class="t-small">12.000 + extra, HALF_UP về 1 đồng</text>
<line x1="150" y1="131" x2="183" y2="131" class="s-pk f-none" stroke-width="1.6" marker-end="url(#py03-0-arrow)"/>
<line x1="355" y1="131" x2="388" y2="131" class="s-pk f-none" stroke-width="1.6" marker-end="url(#py03-0-arrow)"/>
<text x="360" y="123" class="t-small">đúng</text>
<line x1="270" y1="167" x2="270" y2="213" class="s-warn f-none dash" stroke-width="1.6" marker-end="url(#py03-0-warn)"/>
<text x="280" y="196" class="t-small t-warn">sai</text>
<path d="M462,95 L462,55 L578,55" class="s-pk f-none" stroke-width="1.6" marker-end="url(#py03-0-arrow)"/>
<line x1="535" y1="131" x2="578" y2="131" class="s-pk f-none" stroke-width="1.6" marker-end="url(#py03-0-arrow)"/>
<path d="M820,55 L850,55 L850,235 L822,235" class="s-pk f-none" stroke-width="1.6" marker-end="url(#py03-0-arrow)"/>
<path d="M820,131 L850,131" class="s-pk f-none" stroke-width="1.6"/>
</svg>
</div>
<figcaption>Hình 1. Luồng rẽ nhánh tính cước. Kiểm tra đầu vào nằm trước mọi phép tính; nhánh lỗi dừng bằng <code>raise</code>, không đi tiếp tới công thức.</figcaption>
</figure>

## Vòng lặp: for, range, enumerate và while

`for` trong Python lặp qua một **iterable** (list, chuỗi, `range`, khóa của dict, ...), không có bộ đếm ba phần như ở C#. Muốn lặp N lần, ta dùng `range`: `range(3)` cho `0, 1, 2`; `range(0, 10, 5)` cho `0, 5`. Điểm cuối luôn bị loại trừ.

Khi cần cả vị trí lẫn giá trị, dùng `enumerate` thay vì tự cộng biến đếm; tham số `start=1` đánh số từ 1 cho người đọc:

```python
def print_fare_table() -> None:
    """Vòng for lồng nhau với enumerate: in bảng cước, bỏ qua dòng ngoài miền."""
    distances = ["-1", "0", "2", "2.5", "5", "100", "100.5"]
    for service in SERVICE_SURCHARGE:
        print(f"[{service}]")
        for i, text in enumerate(distances, start=1):
            km = Decimal(text)
            if not MIN_KM <= km <= MAX_KM:
                print(f"  {i}. {text:>5} km  TỪ CHỐI (ngoài miền)")
                continue  # bỏ qua phần còn lại của lần lặp này
            total = calculate_fare(km, service=service)
            print(f"  {i}. {text:>5} km  {format_vnd(total):>8} VND  {classify_fare(total)}")
```

`format_vnd` là hàm nhỏ đổi `12000` thành `12.000` để dễ đọc; bài 04 sẽ giải thích kỹ vì sao nó phải làm tròn HALF_UP trước (spec `.0f` của f-string làm tròn HALF_EVEN) và vì sao không dùng `locale`. Ở bài này cứ xem nó là hàm định dạng.

`continue` bỏ phần còn lại của lần lặp hiện tại để sang phần tử kế tiếp; `break` thoát cả vòng; `return` thoát cả hàm. Một yêu cầu sai khoảng cách chỉ làm dòng đó bị từ chối, các dòng còn lại vẫn được tính. Đừng nhầm `continue` với `break`: nhầm lẫn này sẽ khiến một dòng lỗi làm mất cả lô.

`while` lặp khi điều kiện còn đúng và hợp với bài toán không biết trước số bước. Ví dụ: một khách có ngân sách 30.000 VND hỏi đi được xa nhất bao nhiêu km, ta tăng từng 0,5 km tới khi bước kế tiếp vượt ngân sách:

```python
def max_distance_for_budget(budget: Decimal, *, service: str = "ride") -> Decimal:
    """Vòng while: tăng 0,5 km một lần tới khi vượt ngân sách hoặc hết miền."""
    step = Decimal("0.5")
    distance = Decimal("0")
    while distance + step <= MAX_KM:
        if calculate_fare(distance + step, service=service) > budget:
            break  # bước kế tiếp vượt ngân sách: dừng, giữ khoảng cách hiện tại
        distance += step
    return distance
```

Mỗi vòng `while` phải làm điều kiện tiến dần về sai (ở đây `distance` tăng, hoặc `break` thoát). Quên cập nhật biến là nguyên nhân thường gặp nhất của vòng lặp vô hạn. Với ngân sách 30.000 VND, khách đi tối đa 6 km (12.000 + 4 × 4.500 = 30.000); với 20.000 VND chỉ đi được tối đa 3,5 km (4 km đã tốn 21.000). Vì bước nhảy là 0,5 km, kết quả luôn là bội của 0,5.

## Hàm: nhận tham số, trả kết quả, không in

Hàm được khai báo bằng `def`, kết quả đi ra qua `return`. Một hàm không có `return` sẽ trả `None`. Quy tắc tính cước nên là một hàm **thuần**: cùng đầu vào cho cùng kết quả, không `print`, không đọc file, không hỏi người dùng. Lúc nào cần hiển thị, người gọi quyết định in ra console, trả JSON hay vẽ lên app.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hàm calculate_fare như một hộp nhận distance_km và service, trả về Decimal, hoặc ném ValueError; việc in nằm ở phía người gọi.">
<defs>
<marker id="py03-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="py03-1-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<text x="30" y="68" class="t-mono t-b">distance_km</text>
<text x="30" y="128" class="t-mono t-b">service=</text>
<line x1="130" y1="64" x2="298" y2="92" class="s-pk f-none" stroke-width="1.6" marker-end="url(#py03-1-arrow)"/>
<line x1="130" y1="124" x2="298" y2="112" class="s-pk f-none" stroke-width="1.6" marker-end="url(#py03-1-arrow)"/>
<rect x="300" y="55" width="260" height="100" rx="12" class="f-phys-soft s-phys"/>
<text x="316" y="88" class="t-b t-mono">calculate_fare()</text>
<text x="316" y="110" class="t-small">tính từ đầu vào, rồi return</text>
<text x="316" y="130" class="t-small">không print, không đọc file</text>
<line x1="560" y1="95" x2="688" y2="95" class="s-pk f-none" stroke-width="1.6" marker-end="url(#py03-1-arrow)"/>
<text x="600" y="85" class="t-small">return</text>
<rect x="690" y="70" width="170" height="50" rx="10" class="f-pk-soft s-pk"/>
<text x="702" y="92" class="t-b t-mono">Decimal</text>
<text x="702" y="108" class="t-small">tiền đã làm tròn</text>
<line x1="430" y1="155" x2="430" y2="188" class="s-warn f-none dash" stroke-width="1.6" marker-end="url(#py03-1-warn)"/>
<rect x="320" y="190" width="220" height="34" rx="10" class="f-warn-soft s-warn"/>
<text x="332" y="212" class="t-b t-warn t-mono">ValueError (ngoài miền)</text>
<text x="690" y="170" class="t-small">Người gọi mới quyết định in,</text>
<text x="690" y="186" class="t-small">trả JSON hoặc ghi log</text>
</svg>
</div>
<figcaption>Hình 2. Hàm là hộp nhận tham số và trả kết quả; ngoài miền thì báo lỗi bằng exception. Việc hiển thị thuộc về người gọi.</figcaption>
</figure>

### Tham số mặc định và cái bẫy mutable default

Tham số có giá trị mặc định (`service: str = "ride"`) cho phép bỏ qua khi gọi. Giá trị mặc định được tính **một lần**, vào lúc `def` chạy, rồi dùng chung cho mọi lần gọi. Với số, chuỗi, `None` (bất biến) điều này vô hại. Với `list`, `dict`, `set` thì là cái bẫy:

```python
def append_bad(item: str, bucket: list[str] = []) -> list[str]:
    bucket.append(item)
    return bucket


def append_good(item: str, bucket: list[str] | None = None) -> list[str]:
    if bucket is None:
        bucket = []
    bucket.append(item)
    return bucket
```

Kết quả chạy thật trong lab:

```text
Mặc định list bị dùng chung:
  append_bad('a')  -> ['a']
  append_bad('b')  -> ['a', 'b']
  append_good('a') -> ['a']
  append_good('b') -> ['b']
```

Lần gọi thứ hai của `append_bad` thấy lại `'a'` của lần đầu vì cả hai dùng cùng một list. Quy ước an toàn: mặc định là `None`, tạo đối tượng mới trong thân hàm. Trong một service đặt chuyến chạy lâu ngày, đây là kiểu lỗi làm «đơn của khách này lẫn vào đơn của khách trước» và chỉ lộ ra sau nhiều giờ chạy.

### Keyword-only với dấu `*`

Trong chữ ký `calculate_fare(distance_km, *, service="ride")`, dấu `*` nghĩa là mọi tham số đứng sau nó **chỉ** được truyền theo tên. Ta dùng nó cho `service` vì ba giá trị `ride`, `food`, `parcel` là chuỗi, dễ truyền nhầm vị trí nếu sau này hàm có thêm tham số:

```python
calculate_fare(Decimal("5"), service="food")   # đúng
calculate_fare(Decimal("5"), "food")           # TypeError
```

Lời gọi thứ hai báo `TypeError: calculate_fare() takes 1 positional argument but 2 were given`. Lỗi rõ ràng này tốt hơn việc âm thầm chạy sai.

### Trả về nhiều giá trị và docstring

Python cho phép trả về một **tuple**, rồi người gọi «bung» ra nhiều biến. `fare_breakdown` trả hai phần: giá cơ bản và tiền phần vượt. Nó cũng giữ miền cho mọi nhánh: ngoài 0 đến 100 km hoặc dịch vụ lạ đều ném `ValueError`.

```python
def fare_breakdown(
    distance_km: Decimal, *, service: str = "ride"
) -> tuple[Decimal, Decimal]:
    """Trả về (giá cơ bản, tiền phần vượt), chưa làm tròn.

    Hàm chỉ tính và trả kết quả, không in gì ra màn hình.
    Ném ValueError khi khoảng cách hoặc dịch vụ nằm ngoài miền cho phép.
    """
    if not MIN_KM <= distance_km <= MAX_KM:
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    if service not in SERVICES:
        raise ValueError(f"service không hợp lệ: {service!r}")

    if distance_km <= INCLUDED_KM:
        extra = Decimal("0")
    else:
        extra = (distance_km - INCLUDED_KM) * PRICE_PER_EXTRA_KM
    return BASE_FARE, extra
```

Chuỗi ba dấu nháy ngay dưới dòng `def` là **docstring**: nó đi theo hàm, đọc được bằng `help(fare_breakdown)` hay `fare_breakdown.__doc__`, và là nơi tốt nhất để ghi hợp đồng của hàm (nhận gì, trả gì, khi nào lỗi). Kiểu gợi ý `-> tuple[Decimal, Decimal]` chỉ là tài liệu cho người đọc và công cụ; bài 12 sẽ đi sâu vào type hints.

Một chi tiết về **phạm vi** (scope): biến `extra`, `base` tạo trong hàm chỉ sống trong hàm đó và biến mất khi hàm trả về. Hằng số `BASE_FARE` ở đầu file thì đọc được từ trong hàm. Nhờ vậy hai lần gọi hàm không ảnh hưởng nhau, trừ khi ta chủ động dùng đối tượng mutable dùng chung, như cái bẫy ở trên.

`calculate_fare` bọc ngoài, cộng hai phần rồi làm tròn **một lần ở cuối**:

```python
def calculate_fare(distance_km: Decimal, *, service: str = "ride") -> Decimal:
    """Tính cước (VND), làm tròn ROUND_HALF_UP về 1 đồng."""
    base, extra = fare_breakdown(distance_km, service=service)
    return (base + extra).quantize(Decimal("1"), rounding=ROUND_HALF_UP)
```

## Ba dịch vụ, một công thức

Tới Nơi có ba dịch vụ `ride`, `food`, `parcel`. **Ở mô hình học tập này, cả ba dùng cùng công thức giá**: 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, không phụ phí, thuế, giảm giá hay hệ số giờ cao điểm. Đây là giả định để học, không phải bảng giá thật. Tham số `service` vẫn có mặt vì ba lý do: hợp đồng của hàm đã nói rõ loại chuyến, ta kiểm được giá trị lạ như `"taxi"` ngay từ bây giờ, và khi sản phẩm đổi quy tắc theo dịch vụ thì chữ ký hàm không phải đổi. Dấu `*` còn buộc người gọi viết `service="food"`, nên đọc lời gọi là biết loại chuyến.

Ném `ValueError` là cách báo «đầu vào sai hợp đồng». Khác với trả `0` hay `None`, người gọi không thể vô tình xem đó là một báo giá hợp lệ. Bài 08 sẽ đào sâu `try/except`, exception tùy biến và traceback; hôm nay ta chỉ cần biết exception dừng luồng thông thường và mang theo một thông báo.

Chạy lab từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/03-dieu-kien-vong-lap-ham examples/toi-noi-python/03-dieu-kien-vong-lap-ham/main.py
```

Kết quả (trích: bảng của `ride` và `food`; bảng `parcel` giống hệt):

```text
Bảng cước (giả định học tập):
[ride]
  1.    -1 km  TỪ CHỐI (ngoài miền)
  2.     0 km    12.000 VND  cơ bản
  3.     2 km    12.000 VND  cơ bản
  4.   2.5 km    14.250 VND  trung bình
  5.     5 km    25.500 VND  trung bình
  6.   100 km   453.000 VND  đường dài
  7. 100.5 km  TỪ CHỐI (ngoài miền)
[food]
  1.    -1 km  TỪ CHỐI (ngoài miền)
  2.     0 km    12.000 VND  cơ bản
  3.     2 km    12.000 VND  cơ bản
  4.   2.5 km    14.250 VND  trung bình
  5.     5 km    25.500 VND  trung bình
  6.   100 km   453.000 VND  đường dài
  7. 100.5 km  TỪ CHỐI (ngoài miền)
```

Gần cuối, lab in ba dòng tách chuyến và ngân sách, rồi phần minh họa mặc định `list` đã trích ở trên:

```text
Tách 5 km food: cơ bản=12000, phần vượt=13500
Ngân sách 30.000 VND đi tối đa 6.0 km (ride)
Ngân sách 20.000 VND đi tối đa 3.5 km (ride)
```

## Kiểm chứng với --verify

Một vài lần chạy bằng mắt chưa đủ để tin một quy tắc tiền. Lab có chế độ `--verify`: hàm `verify()` so kết quả thực với giá trị kỳ vọng, đếm số ca, in `OK n/n` khi đúng hết; nếu có ca sai, nó in tên ca, in `FAIL n/m` và thoát với mã 1. Các ca được chọn sát biên:

| Đầu vào | Kỳ vọng | Vì sao chọn |
| --- | --- | --- |
| 0 km ride | 12000 | Cận dưới |
| 1,999 km ride | 12000 | Ngay dưới ngưỡng 2 km |
| 2 km ride | 12000 | Đúng ngưỡng giá cơ bản |
| 2,001 km ride | 12005 | Cước thô 12004,5; HALF_UP lên 12005 |
| 2,499 km ride | 14246 | Cước thô 14245,5 |
| 5 km ride / food / parcel | 25500 cả ba | Ba dịch vụ cùng giá |
| 100 km ride | 453000 | Cận trên |
| -0,001 km, 100,001 km, dịch vụ `taxi` | `ValueError` | Ngoài miền, kèm thông điệp lỗi |

Ngoài các ca tiền, `verify()` còn kiểm `TypeError` khi truyền `service` theo vị trí, tuple của `fare_breakdown`, hai lần gọi độc lập của `append_good`, việc `append_bad` trả về chính một list dùng chung (`['a', 'b']`), kết quả vòng `while` với hai ngân sách (6 km và 3,5 km) và `format_vnd(Decimal("12004.5"))` ra `12.005`.

```bash
uv run --project examples/toi-noi-python/03-dieu-kien-vong-lap-ham examples/toi-noi-python/03-dieu-kien-vong-lap-ham/main.py --verify
```

```text
OK 22/22
```

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `IndentationError` hoặc `TabError`, nhánh chạy sai chỗ | Thụt lề có đồng nhất 4 khoảng trắng, không lẫn tab không |
| `if not distance_km:` coi 0 km là thiếu | Dùng `is None` hoặc so sánh tường minh |
| List/dict mặc định «nhớ» dữ liệu giữa các lần gọi | Mặc định `None`, tạo mới trong thân hàm |
| `TypeError ... positional argument` | Tham số sau `*` phải truyền theo tên |
| Vòng `while` không dừng | Biến điều kiện có tiến dần về sai, có `break` không |

## Giới hạn của ví dụ

Đây là mô hình để học hàm và rẽ nhánh, không phải hệ thống giá. Khoảng cách được gõ sẵn dưới dạng chuỗi rồi đổi sang `Decimal`; ta chưa đọc đầu vào người dùng và chưa xử lý chuỗi sai định dạng (`Decimal("abc")` cũng ném lỗi, nhưng đó là việc của bài 08). Ba dịch vụ cùng giá là giả định, chưa có thuế, khuyến mãi, hệ số giờ cao điểm. Miền 0 đến 100 km là giới hạn để luyện kiểm tra, không phải giới hạn thật. Bài 10 sẽ chuyển các ca `--verify` thành `pytest` có cấu trúc.

**Bài tập 1 (thử nghiệm ngoài mô hình):** giả sử một đợt thử nghiệm cộng 5.000 VND phí lấy món cho `food` và 3.000 VND cho `parcel`. Bạn sửa ở đâu trong lab để thử, và `food` 5 km sẽ có giá bao nhiêu? Vì sao nên làm trên bản sao thay vì sửa thẳng lab?

**Bài tập 2:** vì sao `service: str = "ride"` an toàn còn `bucket: list[str] = []` thì không?

<details><summary>Xem lời giải</summary><p>(1) Thêm một dict như <code>SURCHARGE = {"food": Decimal("5000"), "parcel": Decimal("3000")}</code> và cộng <code>SURCHARGE.get(service, Decimal("0"))</code> vào tổng trước khi <code>quantize</code>. <code>food</code> 5 km: 12.000 + 13.500 + 5.000 = <b>30.500 VND</b>. Nên làm trên bản sao vì phụ phí nằm ngoài mô hình chung của cả loạt; các bài sau và bộ <code>--verify</code> đều giả định ba dịch vụ cùng giá. (2) Giá trị mặc định được tính một lần lúc <code>def</code>. Chuỗi bất biến nên không có cách nào sửa đối tượng <code>"ride"</code> đó: mọi lần «đổi» chỉ trỏ biến sang giá trị khác. List thì mutable: <code>append</code> sửa chính đối tượng duy nhất được tạo lúc <code>def</code>, nên các lần gọi sau nhìn thấy thay đổi của lần trước.</p></details>

Bài trước: [bài 02](/articles/python-bien-kieu-du-lieu-decimal). Bài tiếp: [bài 04](/articles/python-chuoi-f-string-ngay-gio-timezone).

## Tài liệu gốc

- [Tutorial: More Control Flow Tools (if, for, range, match, hàm, tham số)](https://docs.python.org/3.14/tutorial/controlflow.html).
- [Language reference: Compound statements](https://docs.python.org/3.14/reference/compound_stmts.html).
- [Language reference: Expressions (so sánh, chained comparison)](https://docs.python.org/3.14/reference/expressions.html#comparisons).
- [Thư viện chuẩn: decimal](https://docs.python.org/3.14/library/decimal.html).
- [PEP 8: Style Guide for Python Code](https://peps.python.org/pep-0008/).
