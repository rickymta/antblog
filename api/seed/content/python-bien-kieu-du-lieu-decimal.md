---
slug: python-bien-kieu-du-lieu-decimal
title: "Biến, kiểu dữ liệu và Decimal: biểu diễn tiền và khoảng cách"
summary: "Bài 02: biến là tên gắn với đối tượng, dynamic typing, int, float và 0.1 + 0.2, Decimal với quantize và ROUND_HALF_UP, chuyển đổi kiểu, bool, None. Có bảng kiểm tay và lab chạy được."
category: python-beginner
level: beginner
tags: [python, toi-noi, decimal]
language: vi
---

Một khách đi 2,501 km. Cước thô là 14.254,5 VND; phải thu 14.254 hay 14.255? Đó là quyết định nghiệp vụ, nhưng trước khi quyết định, chương trình phải giữ được đúng con số đang tính. Trong [bài 01](/articles/python-va-uv-tu-source-code-den-chuong-trinh), ta đã chạy công thức tính cước của [Tới Nơi](/articles/du-an-mau-toi-noi) bằng `Decimal` mà chưa giải thích. Đây là bài 02 của lộ trình **Python cơ bản qua Tới Nơi**: ta xem kỹ dữ liệu nằm trong các biến và chốt quy tắc làm tròn tiền.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trong Python, biểu thức <code>0.1 + 0.2 == 0.3</code> cho <code>True</code> hay <code>False</code>? Và <code>Decimal(0.1)</code> có bằng <code>Decimal("0.1")</code> không?</p>
<details><summary>Xem đáp án</summary><p><b>False</b>, và <b>không</b>. Literal <code>0.1</code> là float nhị phân, đã xấp xỉ ngay khi viết. <code>Decimal(0.1)</code> sao chép nguyên giá trị xấp xỉ đó, còn <code>Decimal("0.1")</code> đọc từ chữ số thập phân bạn gõ nên đúng bằng 0,1.</p></details>
</div>

## Biến là tên gắn với đối tượng

Trong C#, `decimal distanceKm = 2.5m;` khai báo một biến có kiểu cố định. Python hoạt động khác: mọi giá trị là một **đối tượng** có kiểu của riêng nó, và biến chỉ là **tên** gắn vào đối tượng đó. Dấu `=` gắn tên, không khai báo kiểu.

```python
trip_id = "TN-0001"      # tên trip_id gắn với một đối tượng str
value = 5                # value gắn với một int
value = "5"              # gắn lại tên này vào một str: hợp lệ
a = [1, 2]
b = a                    # b là tên thứ hai của cùng một list, không phải bản sao
b.append(3)
```

Hàm `type()` cho biết kiểu của đối tượng mà một tên đang gắn tới. Với mã trên, lab in:

```text
1. TN-0001 -> str
2. value = 5 -> int
3. value = '5' -> str
4. a=[1, 2, 3], a is b: True
```

Đó là **dynamic typing**: kiểu được kiểm tra khi chương trình chạy, và cùng một tên có thể gắn với đối tượng khác kiểu ở các thời điểm khác nhau. Dòng 4 cho thấy hệ quả: sửa qua tên `b` làm đổi cả những gì `a` nhìn thấy, vì chỉ có một list. Toán tử `is` hỏi hai tên có cùng một đối tượng hay không; `==` hỏi hai giá trị có bằng nhau không. Danh sách và các kiểu có thể thay đổi sẽ quay lại ở bài 05.

Tự do này có giá: Python không báo lỗi khi bạn gắn nhầm chuỗi vào chỗ cần số, cho đến lúc dòng đó chạy. Cách giảm rủi ro là đặt tên có đơn vị (`distance_km`) và thêm **type hint**, chú thích kiểu mà công cụ như mypy đọc được:

```python
def raw_fare(distance_km: Decimal) -> Decimal:
    ...
```

Interpreter bỏ qua type hint khi chạy; type hint là hợp đồng cho người đọc và công cụ kiểm tra. Từ giờ ta dùng ở mức vừa phải, và bài 12 sẽ đi sâu vào type hint, mypy và Protocol.

## Số nguyên, phép chia và float

`int` trong Python **không có giới hạn 32 hay 64 bit**: giá trị lớn tới đâu thì dùng bộ nhớ tới đó. Phép chia có hai dạng, khác với điều nhiều người đã quen từ C#:

```python
print(2**100)       # 1267650600228229401496703205376
print(5 / 2, 5 // 2)  # 2.5 2
```

Toán tử `/` luôn trả số thực (`2.5`), còn `//` là phép chia lấy phần nguyên (`2`). Cái bẫy chia hai số int ra số nguyên (`5 / 2` ra 2 trong C#) không có ở Python 3, nhưng đổi lại có một bẫy khác: `5 / 2` trả `float`, và `float` không dành cho tiền.

`float` biểu diễn số theo cơ số 2. Như ở mọi ngôn ngữ dùng chuẩn IEEE 754, nhiều phân số thập phân như 0,1 không có biểu diễn nhị phân hữu hạn, nên chương trình lưu giá trị gần đúng:

```python
print(0.1 + 0.2)           # 0.30000000000000004
print(0.1 + 0.2 == 0.3)    # False
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 210" role="img" aria-label="So sánh: float lưu xấp xỉ nhị phân nên 0.1 cộng 0.2 ra 0.30000000000000004; Decimal từ chuỗi giữ chữ số thập phân nên ra 0.3."><rect x="20" y="30" width="410" height="150" rx="10" class="f-warn-soft s-warn"/><text x="36" y="66" class="t-b">float: nhị phân xấp xỉ</text><text x="36" y="102" class="t-mono">0.1  →  0.1000000000000000055...</text><text x="36" y="128" class="t-mono">0.1 + 0.2  →  0.30000000000000004</text><text x="36" y="160" class="t-small">== 0.3 cho False</text><rect x="450" y="30" width="410" height="150" rx="10" class="f-phys-soft s-phys"/><text x="466" y="66" class="t-b">Decimal: chữ số thập phân</text><text x="466" y="102" class="t-mono">Decimal("0.1")  →  0.1</text><text x="466" y="128" class="t-mono">0.1 + 0.2  →  0.3</text><text x="466" y="160" class="t-small">== 0.3 cho True</text></svg>
</div>
<figcaption>Hình 1. Cùng phép cộng, hai cách biểu diễn. Decimal chỉ giữ đúng chữ số thập phân khi được tạo từ chuỗi; tạo từ float sẽ mang theo xấp xỉ của ô bên trái.</figcaption>
</figure>

Đây là hệ quả của cách biểu diễn số, không phải lỗi của Python. float vẫn phù hợp cho đo đạc, toạ độ GPS, điểm số của mô hình; bạn so sánh bằng ngưỡng sai số (`math.isclose`) thay vì `==`. Nhưng với tiền và khoảng cách cộng dồn vào tiền, ta cần kiểu thập phân.

## Decimal: tạo từ chuỗi, không từ float

```python
from decimal import Decimal

Decimal("0.1") + Decimal("0.2") == Decimal("0.3")   # True
print(Decimal(0.1))   # 0.1000000000000000055511151231257827021181583404541015625
print(Decimal("0.1")) # 0.1
```

Kết quả của hai dòng `print` này chính là dòng 9 và 10 của lab. `Decimal(0.1)` không sai: nó cho bạn thấy đúng giá trị mà float đang giữ. Vì vậy quy tắc cả loạt là: **tạo `Decimal` từ chuỗi** (`Decimal("2.501")`), hoặc từ `int` (`Decimal(5)`), không bao giờ từ literal float. Dòng lệnh, form và CSV đưa dữ liệu vào dưới dạng chuỗi, nên quy tắc này khớp với cách dữ liệu đến. JSON thì khác: `json.loads` đổi số như `2.5` thành float, trừ khi ta yêu cầu dùng `Decimal` ngay lúc đọc (`parse_float=Decimal`); bài 09 sẽ quay lại chuyện này.

Vài tính chất cần biết:

- `Decimal` cộng, trừ, nhân được với `int` (`Decimal("2.5") * 4500` ra `11250.0`) nhưng **ném `TypeError` khi cộng, trừ, nhân, chia với float**. Python cố tình từ chối, thay vì để độ chính xác bị hỏng mà không báo gì. Riêng phép so sánh thì không báo lỗi mà so đúng giá trị: `Decimal("0.1") == 0.1` cho `False`, vì float `0.1` là giá trị xấp xỉ khác.
- Kết quả giữ số chữ số thập phân theo phép tính: `Decimal("2.501") - 2` cho `0.501`, và `0.501 * 4500` cho `2254.500`. Đây là lý do bài 01 in `14250.0` cho 2,5 km.
- Độ chính xác do **context** của module `decimal` quy định: mặc định 28 chữ số có nghĩa, chế độ làm tròn mặc định là `ROUND_HALF_EVEN`. Khác với `decimal` của C# vốn là kiểu 128 bit cố định (khoảng 28 đến 29 chữ số), ở Python độ chính xác là cấu hình có thể đổi. Ví dụ chia `Decimal(1) / Decimal(3)` ra 28 chữ số 3 và vẫn là xấp xỉ.

Hai bên giống nhau ở ý chính: cả hai là kiểu thập phân, đều mặc định làm tròn điểm giữa về số chẵn (`Math.Round` mặc định `ToEven` trong C#), và đều không thay thế validation hay quy tắc tiền tệ.

## Làm tròn tiền bằng quantize

Giữ nguyên quy tắc bài 01: 12.000 VND gồm 2 km đầu, phần vượt 4.500 VND/km. Bổ sung một quy tắc **chỉ dùng cho mô hình minh hoạ**: làm tròn tiền cuối cùng đến 1 VND, điểm giữa làm tròn ra xa số 0 (`ROUND_HALF_UP`). Mô hình có ba giá trị khác nhau: khoảng cách, cước thô và tiền phải trả.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 175" role="img" aria-label="Khoảng cách 2,501 km cho cước thô 14.254,500 VND; làm tròn một lần ở cuối thành 14.255 VND phải trả."><rect x="20" y="35" width="258" height="106" rx="10" class="f-phys-soft s-phys"/><text x="32" y="73" class="t-b">Khoảng cách</text><text x="32" y="108" class="t-small">Decimal("2.501") km</text><text x="281" y="93" class="t-b">→</text><rect x="300" y="35" width="258" height="106" rx="10" class="f-phys-soft s-phys"/><text x="312" y="73" class="t-b">Cước thô</text><text x="312" y="108" class="t-small">14254.500 VND</text><text x="561" y="93" class="t-b">→</text><rect x="580" y="35" width="258" height="106" rx="10" class="f-pk-soft s-pk"/><text x="592" y="73" class="t-b">Tiền phải trả</text><text x="592" y="108" class="t-small">quantize: 14255 VND</text></svg>
</div>
<figcaption>Hình 2. Giữ độ chính xác trong tính toán, chỉ làm tròn một lần tại ranh giới trả tiền theo chính sách đã chọn.</figcaption>
</figure>

Phương thức `quantize` ép một `Decimal` về cùng số chữ số thập phân với mẫu; mẫu `Decimal("1")` nghĩa là 0 chữ số sau dấu thập phân:

```python
from decimal import ROUND_HALF_UP, Decimal

ONE_VND = Decimal("1")
half = Decimal("14254.5")
half.quantize(ONE_VND)                              # 14254, theo mặc định HALF_EVEN
half.quantize(ONE_VND, rounding=ROUND_HALF_UP)      # 14255
```

Hai dòng này trong lab (dòng 11 và 12) in `14254` rồi `14255`. Bài học: khi hợp đồng nghiệp vụ đã chọn quy tắc làm tròn, **ghi rõ `rounding=`**, đừng dựa vào mặc định của context. Hàm có sẵn `round(Decimal("14254.5"))` cũng làm tròn điểm giữa về số chẵn và trả 14254, một lý do khác để không dùng nó cho tiền. Với giá trị không phải điểm giữa, `ROUND_HALF_UP` vẫn làm tròn về số gần nhất.

```python
# examples/toi-noi-python/02-bien-kieu-du-lieu-decimal/main.py (rút gọn)
def raw_fare(distance_km: Decimal) -> Decimal:
    """Cước thô: giữ nguyên độ chính xác, chưa làm tròn."""
    extra_km = max(Decimal("0"), distance_km - INCLUDED_KM)
    return BASE_FARE + extra_km * PRICE_PER_EXTRA_KM


def calculate_fare(distance_km: Decimal) -> Decimal:
    """Tiền phải trả: làm tròn đến 1 VND, điểm giữa làm tròn ra xa số 0."""
    return raw_fare(distance_km).quantize(ONE_VND, rounding=ROUND_HALF_UP)
```

Với 2,501 km: phần vượt 0,501 km, tiền vượt 2.254,5 VND, cước thô `14254.500`, `calculate_fare` trả `14255`. Ta không làm tròn khoảng cách trước khi tính, vì điều đó có thể đổi báo giá. Định dạng khi in (ví dụ `f"{total:,}"` chèn dấu phẩy ngăn cách nghìn theo kiểu Anh-Mỹ) chỉ đổi cách hiển thị; muốn các bước sau dùng giá đã làm tròn, hãy lưu kết quả `quantize` vào biến. Hiển thị cho người Việt sẽ học ở bài 04.

## Chuyển đổi kiểu cần một quyết định

Dữ liệu từ ngoài như dòng lệnh, form hay CSV đến dưới dạng chuỗi (với JSON, số đã được `json.loads` đổi thành float, xem bài 09). Chuyển đổi là chỗ chương trình gặp dữ liệu xấu đầu tiên:

```python
int("5")             # 5
Decimal("4.5")       # Decimal('4.5')
int("2.5")           # ValueError
Decimal("abc")       # decimal.InvalidOperation
Decimal("1.5") + 0.5 # TypeError
int(Decimal("2.9"))  # 2: cắt về 0, không làm tròn
```

Lab in thông báo thật của từng trường hợp (dòng 13 tới 17), ví dụ `int('2.5') -> ValueError: invalid literal for int() with base 10: '2.5'`. Có ba điểm cần chú ý.

Thứ nhất, `int()` không nhận chuỗi `"2.5"`, và `int(Decimal("2.9"))` cắt phần thập phân chứ không làm tròn; đừng ép khoảng cách về `int` trước khi tính phần vượt, vì 2,5 km sẽ thành 2 và giá còn 12.000 thay vì 14.250.

Thứ hai, `decimal.InvalidOperation` **không phải** `ValueError`: nó kế thừa `ArithmeticError`, nên `except ValueError` không bắt được lỗi parse `Decimal`. Dòng 26 của lab xác nhận điều này.

Thứ ba, `Decimal("NaN")` và `Decimal("Infinity")` được chấp nhận như số hợp lệ. Với dữ liệu từ ngoài, lab gói việc đó vào một hàm nhỏ:

```python
def parse_decimal(text: str) -> Decimal:
    """Đổi chuỗi ngoài hệ thống thành Decimal hữu hạn; mọi lỗi quy về ValueError."""
    try:
        value = Decimal(text)
    except InvalidOperation as exc:
        raise ValueError(f"không phải số thập phân: {text!r}") from exc
    if not value.is_finite():
        raise ValueError(f"số không hữu hạn: {text!r}")
    return value
```

Hàm này mới kiểm tra «có phải số hữu hạn không», chưa kiểm tra miền giá trị. Khoảng cách âm vẫn lọt qua; bài 03 thêm ranh giới đầu vào còn bài 08 xử lý exception có cấu trúc.

## bool, truthiness và None

`bool` là kiểu con của `int` (`True + True` bằng 2), nhưng ta dùng nó như cờ đúng/sai. Python còn cho phép mọi đối tượng đứng ở chỗ điều kiện: các giá trị «rỗng» như `0`, `Decimal("0")`, chuỗi rỗng, list rỗng và `None` được coi là sai; giá trị khác là đúng. Đó là **truthiness**.

```python
[bool(0), bool(Decimal("0")), bool(""), bool([]), bool(None)]   # tất cả False
bool("False")    # True: chuỗi không rỗng
```

Dòng 18 và 19 của lab xác nhận cả hai. Cái bẫy kinh điển: chuỗi `"False"` đọc từ file cấu hình là **đúng**, vì nó không rỗng; cần so sánh chuỗi rõ ràng chứ không gọi `bool()`.

`None` là giá trị duy nhất của kiểu `NoneType`, biểu diễn «chưa có giá trị». Một chuyến chưa có tài xế sẽ có `driver_id = None`. Kiểm tra bằng `is None`, không phải `== None`, vì `None` là đối tượng duy nhất. Đừng dùng truthiness để thay cho kiểm tra `None` khi `0` hoặc chuỗi rỗng cũng là giá trị hợp lệ: số 0 km không đồng nghĩa với «không có khoảng cách». Bài 16 quay lại xử lý `None` bằng pattern matching.

## Chạy và kiểm chứng

```bash
uv run --project examples/toi-noi-python/02-bien-kieu-du-lieu-decimal examples/toi-noi-python/02-bien-kieu-du-lieu-decimal/main.py
```

Kết quả chạy thật (trích; chương trình in 30 dòng đánh số):

```text
1. TN-0001 -> str
...
7. float: 0.1 + 0.2 = 0.30000000000000004; == 0.3 là False
8. Decimal: 0.1 + 0.2 = 0.3; == 0.3 là True
9. Decimal(0.1) = 0.1000000000000000055511151231257827021181583404541015625
10. Decimal('0.1') = 0.1
11. mặc định (HALF_EVEN): 14254
12. ROUND_HALF_UP: 14255
...
21. 2.501 km: raw=14254.500, fare=14255
22. 0 km -> 12000 VND
23. 2 km -> 12000 VND
24. 2.5 km -> 14250 VND
25. 5 km -> 25500 VND
26. InvalidOperation là ValueError: False
27. parse_decimal('2.5') = 2.5
28. parse_decimal('abc') -> ValueError: không phải số thập phân: 'abc'
29. parse_decimal('NaN') -> ValueError: số không hữu hạn: 'NaN'
30. 2.499 km: raw=14245.500, fare=14246
```

Chế độ kiểm tra tự động so từng dòng với kết quả kỳ vọng:

```bash
uv run --project examples/toi-noi-python/02-bien-kieu-du-lieu-decimal examples/toi-noi-python/02-bien-kieu-du-lieu-decimal/main.py --verify
```

```text
OK 30/30
```

Bảng kiểm tra bằng tay (dòng 22 tới 25), sau `quantize`:

| Khoảng cách | Phần vượt | Tiền phải trả |
| --- | --- | --- |
| 0 km | 0 km | 12000 VND theo giả định hiện tại |
| 2 km | 0 km | 12000 VND |
| 2,5 km | 0,5 km | 14250 VND |
| 5 km | 3 km | 25500 VND |

Khác với bài 01, 2,5 km giờ in `14250` chứ không còn `14250.0`, vì `quantize` đã chốt số chữ số thập phân. 0 km vẫn tính giá cơ bản là lựa chọn của mô hình này, không phải quy luật bắt buộc.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `0.1 + 0.2 != 0.3` | Đang dùng float; chuyển sang `Decimal` tạo từ chuỗi |
| `Decimal` ra chuỗi số dài lạ như `0.1000000000000000055...` | Có tạo từ literal float không; viết `Decimal("0.1")` |
| `TypeError` khi cộng `Decimal` với số | Một toán hạng là float; đổi sang chuỗi hoặc `Decimal` |
| `except ValueError` không bắt lỗi `Decimal("abc")` | `InvalidOperation` là `ArithmeticError`; bắt riêng hoặc dùng hàm bọc |
| Tiền ra 14254 thay vì 14255 | Quên `rounding=ROUND_HALF_UP`; mặc định là `ROUND_HALF_EVEN` |

Giới hạn của ví dụ: giá trị gán sẵn hoặc tạo từ chuỗi cố định; hằng số bảng giá đặt trong code thay vì cấu hình có hiệu lực theo thời điểm; chỉ một tiền tệ (VND, không có đơn vị nhỏ hơn 1 đồng); chưa có kiểu `Money`, chưa kiểm tra miền giá trị, chưa xử lý ngoại lệ cho người dùng. Đây là mô hình để học, không phải lõi tính tiền của sản phẩm.

**Bài tập:**

1. Với đơn giá 4.500, đổi khoảng cách thành 2,499 km. Cước thô và tiền phải trả là bao nhiêu?
2. Vì sao `Decimal(2.501)` không nên dùng để thay cho `Decimal("2.501")`?
3. Một người sửa code thành `int(distance_km)` trước khi tính. Hãy tìm một đầu vào làm lộ ra sai lệch.

<details><summary>Xem lời giải</summary><p>1. Phần vượt 0,499 km, tiền vượt 2.245,5 VND, cước thô <code>14245.500</code>, phải trả <b>14.246 VND</b> theo <code>ROUND_HALF_UP</code>. <code>HALF_EVEN</code> cũng ra 14.246 vì 14.246 là số chẵn, nên bài này chưa phân biệt được hai chế độ; cần điểm giữa như 14.254,5 của 2,501 km. Dòng 30 của lab kiểm lại đáp án này. 2. Literal <code>2.501</code> là float nhị phân đã xấp xỉ, nên <code>Decimal(2.501)</code> mang theo cả dãy chữ số thừa; chuỗi giữ đúng những gì bạn gõ. 3. Ví dụ 2,5 km: ép thành 2 khiến giá còn 12.000 thay vì 14.250 VND.</p></details>

Nhận xét thêm: chương trình vẫn nhận khoảng cách âm nếu ta tự gán, và `parse_decimal` chưa chặn nó. Bài 03 thêm ranh giới đầu vào, tính nhiều báo giá bằng vòng lặp và tách công thức thành hàm có thể kiểm thử.

Bài trước: [bài 01](/articles/python-va-uv-tu-source-code-den-chuong-trinh). Bài tiếp: [bài 03](/articles/python-dieu-kien-vong-lap-ham).

Code bài này nằm trong repository tại `examples/toi-noi-python/02-bien-kieu-du-lieu-decimal`.

## Tài liệu gốc

- [Module decimal](https://docs.python.org/3.14/library/decimal.html).
- [Số thực dấu phẩy động: vấn đề và giới hạn](https://docs.python.org/3.14/tutorial/floatingpoint.html).
- [Mô hình dữ liệu: đối tượng, giá trị và kiểu](https://docs.python.org/3.14/reference/datamodel.html).
- [PEP 484: Type Hints](https://peps.python.org/pep-0484/).
