---
slug: python-frozen-dataclass-enum-immutability
title: "Frozen dataclass, Enum, equality và immutability: thiết kế Money, Quote"
summary: "Bài 13: StrEnum cho trạng thái chuyến, frozen dataclass cho Money và Quote có hạn, __eq__/__hash__ sinh sẵn, replace, so với NamedTuple; chỉ ra frozen là bản nông và không chặn người cố tình. Có output thật."
category: python
level: beginner
tags: [python, toi-noi, dataclass, enum, immutability]
language: vi
---

Khách mở ứng dụng [Tới Nơi](/articles/du-an-mau-toi-noi), thấy báo giá 14.250 VND cho 2,5 km, đọc điều khoản rồi mới bấm đặt, bảy phút sau. Hệ thống phải trả lời hai câu: giá đó có còn hiệu lực không, và trong lúc chờ có đoạn code nào lỡ sửa con số 14.250 không? Với trạng thái chuyến thì ngược lại: một lần gõ nhầm `"acepted"` trong chuỗi là đủ để đẩy chuyến vào ngõ cụt mà Python không báo gì. Cả hai vấn đề cùng một họ: giá trị đáng lẽ phải **cố định và có tên**, nhưng ta đang để chúng là chuỗi và số đổi được.

Đây là bài 13 của lộ trình **Python cơ bản qua Tới Nơi**. Bài 11 dựng `Trip` với trạng thái là chuỗi; bài này đưa trạng thái sang `Enum`, rồi thiết kế `Money` và `Quote` thành các **value object** bất biến. Lab 13 không có lớp `Trip`: bảng chuyển trạng thái đứng riêng, và `Trip` có thuộc tính `status` kiểu `TripStatus` xuất hiện lần đầu ở lab bài 16. Bạn nên đọc bài 11 trước, vì ta dùng lại `@dataclass`, `__post_init__` và bảng chuyển trạng thái.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p><code>Money(Decimal("12000")) == Money(Decimal("12000.0"))</code> cho <code>True</code> hay <code>False</code>? Và có dùng một đối tượng thay cho đối tượng kia làm khóa của dict được không?</p>
<details><summary>Xem đáp án</summary><p><b>Cả hai đều có.</b> Với frozen dataclass, Python sinh <code>__eq__</code> so từng field và <code>__hash__</code> tính từ chính các field đó. <code>Decimal("12000")</code> và <code>Decimal("12000.0")</code> bằng nhau và cùng hash, nên hai <code>Money</code> cũng vậy. Lab còn chuẩn hóa <code>12000.0</code> về <code>12000</code> ngay lúc tạo; mục «Frozen dataclass» chỉ cách làm khi field bị khóa.</p></details>
</div>

## Enum: đặt tên cho một tập giá trị đóng

Tám trạng thái chuyến là một **tập đóng**: không có trạng thái thứ chín xuất hiện giữa chừng. `Enum` biểu diễn đúng điều đó. Ta dùng `StrEnum` (có từ Python 3.11), loại Enum mà mỗi thành viên **cũng là một `str`** có giá trị cho trước:

```python
class TripStatus(StrEnum):
    """Định nghĩa chuẩn của 8 trạng thái chuyến (giá trị chữ thường)."""

    REQUESTED = "requested"
    SEARCHING = "searching"
    ACCEPTED = "accepted"
    ARRIVING = "arriving"
    PICKED_UP = "picked_up"
    IN_TRANSIT = "in_transit"
    COMPLETED = "completed"
    CANCELLED = "cancelled"

    # Enum cũng là class: gắn hành vi vào ngay tại trạng thái.
    def can_go_to(self, new_status: TripStatus) -> bool:
        return new_status in ALLOWED_TRANSITIONS[self]

    @property
    def is_final(self) -> bool:
        return not ALLOWED_TRANSITIONS[self]
```

Code trong `main.py` của lab 13 (các đoạn dưới đây cũng vậy, rút gọn). Có bốn điều cần nhớ:

- **Mỗi thành viên là một singleton.** `TripStatus("requested")` không tạo đối tượng mới mà trả đúng `TripStatus.REQUESTED`, nên so bằng `is` hoặc `==` đều được. Giá trị lạ như `"flying"` bị từ chối bằng `ValueError: 'flying' is not a valid TripStatus`. Đó chính là chỗ gõ nhầm bị bắt: một lần, ở biên (khi đọc chuỗi từ JSON hoặc database), thay vì rải rác khắp nơi.
- **`StrEnum` hòa hợp với dữ liệu ngoài.** `TripStatus.PICKED_UP == "picked_up"` là `True`; `f"{TripStatus.PICKED_UP}"` ra `picked_up`; `json.dumps({"status": TripStatus.REQUESTED})` ra `{"status": "requested"}`. Enum thường (`PlainService(Enum)` trong lab) thì không: `PlainService.RIDE == "ride"` là `False`, và đưa nó cho JSON sẽ lỗi. Dữ liệu Tới Nơi luôn đi qua JSON, Kafka, database nên `StrEnum` là lựa chọn mặc định hợp lý; phiên bản `IntEnum` chỉ dùng khi giá trị thật sự là số.
- **Thành viên không gán lại được**: `setattr(TripStatus, "REQUESTED", "x")` ném `AttributeError: cannot reassign member 'REQUESTED'`.
- **Enum là class thường**, nên có phương thức và property. Thay vì rải điều kiện `status in ("completed", "cancelled")`, ta hỏi `status.is_final`. Bảng chuyển của bài 11 giữ nguyên hình dạng, chỉ đổi khóa và giá trị từ chuỗi sang thành viên:

```python
ALLOWED_TRANSITIONS: dict[TripStatus, tuple[TripStatus, ...]] = {
    TripStatus.REQUESTED: (TripStatus.SEARCHING, TripStatus.CANCELLED),
    TripStatus.SEARCHING: (TripStatus.ACCEPTED, TripStatus.CANCELLED),
    TripStatus.ACCEPTED: (TripStatus.ARRIVING, TripStatus.CANCELLED),
    TripStatus.ARRIVING: (TripStatus.PICKED_UP, TripStatus.CANCELLED),
    TripStatus.PICKED_UP: (TripStatus.IN_TRANSIT,),
    TripStatus.IN_TRANSIT: (TripStatus.COMPLETED,),
    TripStatus.COMPLETED: (),
    TripStatus.CANCELLED: (),
}
```

Trình soạn code giờ gợi ý `TripStatus.` và báo ngay khi bạn gõ sai tên thành viên. Công cụ kiểm tra kiểu của bài 12 còn bắt được việc truyền chuỗi bất kỳ vào chỗ cần `TripStatus`. Lab có thêm `ServiceKind(StrEnum)` với `ride`, `food`, `parcel`, và `calculate_fare` nhận `ServiceKind`: dịch vụ lạ bị chặn bằng chính `ServiceKind(service)`.

## Frozen dataclass: Money là giá trị, không phải đối tượng có đời sống

Chuyến có **định danh**: `TN-0001` vẫn là nó dù trạng thái đổi. Số tiền thì khác: 14.250 VND chỉ là 14.250 VND, không có «đời sống» hay lịch sử. Hai khoản tiền cùng số và cùng loại tiền là **một giá trị**. Kiểu như vậy gọi là **value object**: so sánh theo nội dung, và nên **bất biến**, vì nếu `Money` đổi được thì cùng một đối tượng được chia sẻ giữa báo giá và hóa đơn có thể bị sửa từ chỗ này làm sai chỗ kia.

`@dataclass(frozen=True)` đặt `__setattr__` và `__delattr__` thành lỗi, nên không gán lại được field sau khi tạo:

```python
@dataclass(frozen=True)
class Money:
    """Số tiền + loại tiền. Bất biến; bằng nhau khi cùng amount và currency."""

    amount: Decimal
    currency: str = "VND"

    def __post_init__(self) -> None:
        if not isinstance(self.amount, Decimal):
            raise TypeError(f"amount phải là Decimal, nhận {type(self.amount).__name__}")
        if not self.amount.is_finite():
            raise ValueError(f"amount phải hữu hạn: {self.amount}")
        if self.amount < 0:
            raise ValueError(f"amount không được âm: {self.amount}")
        if len(self.currency) != 3 or not (self.currency.isascii() and self.currency.isalpha() and self.currency.isupper()):
            raise ValueError(f"currency phải gồm 3 chữ in hoa: {self.currency!r}")
        if self.currency == "VND":
            if self.amount != self.amount.to_integral_value():
                raise ValueError(f"VND không có đồng lẻ: {self.amount}")
            # Chuẩn hóa 12000.0 -> 12000. Frozen cấm self.amount = ... kể cả trong
            # __post_init__, nên ta gán qua object.__setattr__.
            object.__setattr__(self, "amount", self.amount.quantize(Decimal("1")))
```

Thứ tự kiểm tra phản ánh bài 02: từ chối `float` (mất chính xác), `NaN` và `Infinity`, số âm (bản học tập này chưa mô hình hóa hoàn tiền), và với VND không cho đồng lẻ. Vì đối tượng bất biến, **validation chỉ cần chạy một lần lúc tạo**: nếu `Money(...)` tạo được thì mọi nơi nhận được nó đều có thể tin nó hợp lệ. Dòng `object.__setattr__` là cách chính thức để gán trong `__post_init__` khi lớp bị khóa; nó dùng để chuẩn hóa chứ không để lách quy tắc.

Từ `@dataclass(frozen=True)`, Python sinh sẵn hai phương thức đi cùng nhau. `__eq__` so từng field và chỉ coi hai đối tượng bằng nhau khi **cùng lớp** và các field bằng nhau. `__hash__` tính từ chính các field đó. Hai phương thức phải nhất quán: **hai đối tượng bằng nhau thì hash phải bằng nhau**, nếu không `dict` và `set` hoạt động sai. Vì đối tượng không đổi nên hash không đổi theo thời gian, và ta dùng được `Money` làm khóa dict hoặc phần tử set. Đây là điểm đối lập với dataclass thường ở bài 11, nơi `eq=True` mà không `frozen` làm `__hash__` thành `None`.

`__add__` định nghĩa phép `+`, kiểm tra cùng loại tiền, và trả `Money` **mới** thay vì sửa tại chỗ. Trả `NotImplemented` khi `other` không phải `Money` để Python báo `TypeError` đúng chuẩn thay vì ta tự ném lỗi. Còn muốn «sửa» một giá trị, ta tạo bản mới bằng `dataclasses.replace`, và quan trọng là nó chạy lại `__post_init__`, nên bản mới cũng được kiểm tra:

```python
replace(m, amount=Decimal("20000"))   # Money mới; m không đổi
replace(m, amount=Decimal("-5"))      # __post_init__ chạy lại, nên bị từ chối
```

Kết quả của hai dòng này (từ `demo()` trong lab) nằm ở mục «Chạy và kiểm chứng».

## Quote có hạn

Một báo giá là **ảnh chụp tại một thời điểm**: khoảng cách, giá, và thời hạn. Nó không nên sửa được: nếu giá đã báo cho khách thay đổi âm thầm thì ta không còn gì để đối chiếu khi có khiếu nại. Muốn gia hạn hoặc đổi giá thì phát hành báo giá **mới**.

```python
@dataclass(frozen=True)
class Quote:
    """Báo giá tại một thời điểm. Hết hạn thì phải báo lại, không sửa báo giá cũ."""

    trip_distance_km: Decimal
    fare: Money
    expires_at: datetime  # aware

    def is_valid(self, now: datetime) -> bool:
        """Còn hiệu lực khi now < expires_at. `now` truyền vào: bài 14 sẽ thay bằng Clock."""
        if now.tzinfo is None:
            raise ValueError("now phải là datetime aware")
        return now < self.expires_at

    @classmethod
    def issue(cls, distance_km: Decimal, *, issued_at: datetime,
              service: ServiceKind = ServiceKind.RIDE, ttl: timedelta = QUOTE_TTL) -> Quote:
        return cls(distance_km, Money(calculate_fare(distance_km, service=service)), issued_at + ttl)
```

(`__post_init__` của `Quote` kiểm `expires_at` phải aware và khoảng cách trong miền, bị lược ở đây.) Chi tiết đáng chú ý:

- **`now` là tham số**, không đọc `datetime.now()` bên trong. Nhờ vậy test dựng được mọi thời điểm mà không cần chờ hay can thiệp đồng hồ hệ thống; bài 14 sẽ gói việc này bằng `Clock`.
- **`expires_at` aware, `now` aware** (bài 04). Trộn naive với aware là lỗi `TypeError` khi so sánh; ta chủ động báo `ValueError` rõ nghĩa hơn.
- **Biên hạn là loại trừ**: còn hiệu lực khi `now < expires_at`, nên đúng mốc 5 phút là hết. Cách chốt biên nào cũng được miễn là viết thành test; lab kiểm ba điểm 4 phút 59 giây, 5 phút đúng, 5 phút 1 giây.
- **`-> Quote` không cần ngoặc kép.** Từ Python 3.14, annotation là **annotation lười** (PEP 649/749): chúng không bị tính ngay lúc định nghĩa, nên tên lớp đang định nghĩa dùng được ngay trong annotation của chính nó (bài 12 giải thích). Vì thế `new_status: TripStatus` trong chính lớp `TripStatus` và `__add__(...) -> Money` trong lớp `Money` cũng không cần ngoặc kép.
- **Equality theo thời điểm.** `expires_at` bằng nhau khi là cùng một khoảnh khắc dù khác múi giờ, và hash cũng khớp, nên một báo giá lưu theo UTC và một báo giá hiển thị giờ Việt Nam vẫn là cùng giá trị.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Trục thời gian của một Quote: từ lúc phát hành T0 đến expires_at cách T0 năm phút là vùng is_valid trả True, từ expires_at trở đi là vùng trả False; Quote không sửa, muốn gia hạn thì tạo Quote mới."><defs><marker id="py13-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="60" y="38" class="t-small">Quote bất biến suốt quãng này; muốn hạn mới thì phát hành Quote mới bằng Quote.issue.</text><rect x="60" y="60" width="500" height="56" rx="8" class="f-phys-soft s-phys"/><text x="76" y="95" class="t-b">is_valid(now) = True</text><rect x="560" y="60" width="260" height="56" rx="8" class="f-warn-soft s-warn"/><text x="576" y="95" class="t-b">is_valid(now) = False</text><path d="M560 50 L560 140" class="f-none s-line dash" stroke-width="2"/><text x="60" y="140" class="t-mono">T0 phát hành</text><text x="60" y="160" class="t-small">01:30 UTC = 08:30 giờ Việt Nam</text><text x="572" y="140" class="t-mono">expires_at = T0 + 5 phút</text><text x="572" y="160" class="t-small">now &lt; expires_at mới còn hạn</text><path d="M60 180 L820 180" class="f-none s-phys" stroke-width="2" marker-end="url(#py13-0-arrow)"/></svg>
</div>
<figcaption>Hình 1. Một Quote chỉ có hai trạng thái theo thời gian. Biên 5 phút đúng thuộc vùng hết hạn.</figcaption>
</figure>

## NamedTuple hay frozen dataclass

Ở bài 10, `FarePolicy` là `NamedTuple`: ba con số gói thành một giá trị bất biến có tên. Giờ ta có hai công cụ cho «giá trị nhỏ, bất biến», nên cần một tiêu chí chọn. Khác biệt lớn nhất nằm ở **equality**: `NamedTuple` *là* một tuple, nên so như tuple.

```python
STANDARD == (Decimal("12000"), Decimal("2"), Decimal("4500"))   # True
PointNT(1, 2) == SizeNT(1, 2)                                     # True: hai kiểu khác nghĩa mà bằng nhau
PointDC(1, 2) == SizeDC(1, 2)                                     # False: frozen dataclass so theo kiểu
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Bên trái NamedTuple: PointNT(1, 2) bằng SizeNT(1, 2) vì so như tuple. Bên phải frozen dataclass: PointDC(1, 2) không bằng SizeDC(1, 2) vì khác kiểu."><rect x="20" y="20" width="410" height="190" rx="10" class="f-pk-soft s-pk"/><text x="40" y="52" class="t-b">NamedTuple</text><text x="40" y="88" class="t-mono">PointNT(1, 2) == SizeNT(1, 2)</text><text x="40" y="116" class="t-b">True: so như tuple (1, 2)</text><text x="40" y="154" class="t-small">Cũng bằng tuple thường (1, 2)</text><text x="40" y="176" class="t-small">Lặp được, giải nén được, truy cập theo chỉ số</text><rect x="450" y="20" width="410" height="190" rx="10" class="f-phys-soft s-phys"/><text x="470" y="52" class="t-b">frozen dataclass</text><text x="470" y="88" class="t-mono">PointDC(1, 2) == SizeDC(1, 2)</text><text x="470" y="116" class="t-b">False: khác kiểu thì khác</text><text x="470" y="154" class="t-small">Không bằng tuple thường, không lặp được</text><text x="470" y="176" class="t-small">Có __post_init__ để kiểm tra dữ liệu</text></svg>
</div>
<figcaption>Hình 2. Cùng hai con số, hai cách so khác nhau. Chọn theo việc bạn có muốn «hai kiểu khác nghĩa» bị coi là bằng nhau hay không.</figcaption>
</figure>

| Tiêu chí | `NamedTuple` | `@dataclass(frozen=True)` |
| --- | --- | --- |
| Equality | Như tuple, kể cả khác kiểu | Chỉ khi cùng lớp |
| Lặp, giải nén, `[0]` | Có | Không (`TypeError: 'Money' object is not iterable`) |
| Kiểm tra dữ liệu lúc tạo | Không có `__post_init__` | Có `__post_init__` |
| Phương thức thêm vào | Được, nhưng ít khi thêm | Tự nhiên (`__add__`, `is_valid`) |
| Sao chép có sửa | `_replace(...)` | `dataclasses.replace(...)` |
| Dùng khi | Bản ghi nhỏ, vị trí cố định, cần tương thích tuple | Value object có quy tắc hợp lệ và hành vi |

Với `FarePolicy`, ba tham số không có quy tắc phụ và ta muốn giải nén `base, included, per_km = policy`, nên giữ `NamedTuple`. Với `Money` và `Quote`, ta cần validation và không muốn `Money(Decimal("1"), "VND")` bằng một tuple `(1, "VND")` vô tình, nên chọn frozen dataclass. Khi cần cả bất biến lẫn tiết kiệm bộ nhớ như ở bài 11, có thể viết `@dataclass(frozen=True, slots=True)`.

Người đọc loạt C# sẽ nhận ra đây là chỗ `record` chiếm vị trí: [bài C# về record, struct và immutability](/articles/csharp-record-struct-money-quote) làm đúng việc này, với `with` tương ứng `dataclasses.replace`, và cùng giới hạn «sao chép nông» sẽ nói ngay dưới đây.

## Giới hạn của frozen

**Frozen chỉ khóa chính các field, không khóa những gì bên trong.** Nếu field là list thì list vẫn sửa được, và hash bị hỏng:

```python
@dataclass(frozen=True)
class LabelledQuote:
    quote: Quote
    labels: list[str]
```

`lq.labels.append("priority")` chạy bình thường, `hash(lq)` ném `TypeError: unhashable type: 'list'` (hash của đối tượng cần hash của mọi field), và `replace(lq, quote=q).labels is lq.labels` là `True`: `replace` không sao chép sâu, bản mới dùng chung đúng list đó. Cách sửa là chọn kiểu bất biến cho field, như `labels: tuple[str, ...]`. `Money` và `Quote` an toàn vì mọi field của chúng (`Decimal`, `str`, `datetime`, `Money`) đều bất biến.

**Frozen cũng không chống người cố tình.** `object.__setattr__(victim, "amount", Decimal("-1"))` vòng qua `__setattr__` bị khóa và để lại `Money` âm, bỏ qua mọi kiểm tra của `__post_init__`. Frozen ngăn **lỡ tay**, như một dòng `quote.fare = ...` ở nơi xa, chứ không phải rào bảo mật. Đó là cùng bài học với dấu gạch dưới ở bài 11: bất biến trong Python là thỏa thuận được cấu trúc hỗ trợ, và code review cùng test phải giữ phần còn lại.

## Chạy và kiểm chứng

Từ thư mục gốc repository (không cần mạng, không thư viện ngoài):

```bash
uv run --project examples/toi-noi-python/13-frozen-dataclass-enum examples/toi-noi-python/13-frozen-dataclass-enum/main.py
```

Output (một lần chạy thật, Python 3.14.7):

```text
== 1. Enum ==
TripStatus('requested') is TripStatus.REQUESTED: True
TripStatus('flying'): ValueError: 'flying' is not a valid TripStatus
TripStatus.PICKED_UP == 'picked_up': True
PlainService.RIDE == 'ride': False
f-string: picked_up | name: PICKED_UP
json.dumps: {"status": "requested", "service": "food"}
số trạng thái: 8 | trạng thái cuối: ['completed', 'cancelled']
ACCEPTED -> ARRIVING: True | PICKED_UP -> CANCELLED: False
gán lại thành viên: AttributeError: cannot reassign member 'REQUESTED'

== 2. Money (frozen dataclass) ==
repr: Money(amount=Decimal('14250'), currency='VND') | str: 14.250 VND
m.amount = 1: FrozenInstanceError: cannot assign to field 'amount'
12000 == 12000.0: True | cùng hash: True | repr b: Money(amount=Decimal('12000'), currency='VND')
is: False | dùng làm khóa dict: cơ bản
cộng: 14.250 VND
sum: 26.250 VND
VND + USD: ValueError: không cộng được VND với USD
âm: ValueError: amount không được âm: -1
đồng lẻ: ValueError: VND không có đồng lẻ: 1.5
float: TypeError: amount phải là Decimal, nhận float
replace hợp lệ: 20.000 VND | m vẫn là 14.250 VND
replace sai: ValueError: amount không được âm: -5

== 3. Quote có hạn ==
quote: 14.250 VND đến 08:35:00 +0700
is_valid(+4 phút 59 giây): True
is_valid(+5 phút đúng): False
is_valid(+5 phút 1 giây): False
now naive: ValueError: now phải là datetime aware
replace chỉ tạo bản sao, giá vẫn là giá cũ: 14.250 VND | hết hạn 01:40 | bản gốc 01:35
cùng thời điểm, khác múi giờ: True | cùng hash: True

== 4. NamedTuple hay frozen dataclass ==
giải nén: 12000 2 4500
STANDARD == tuple thường: True
PointNT(1, 2) == SizeNT(1, 2): True
PointDC(1, 2) == SizeDC(1, 2): False
STANDARD.base_fare = 1: AttributeError: can't set attribute
_replace: 5000
tuple(Money): TypeError: 'Money' object is not iterable

== 5. Giới hạn của frozen ==
labels sau append: ['ride', 'priority']
hash(LabelledQuote): TypeError: unhashable type: 'list'
replace dùng chung list: True
hash(LabelledQuoteSafe) chạy được: True
object.__setattr__ vòng qua frozen: Money(amount=Decimal('-1'), currency='VND')
```

Cách đọc vài dòng đáng chú ý. `is: False` cùng `dùng làm khóa dict: cơ bản` cho thấy `a` và `b` là hai đối tượng khác nhau (`is`) mà thay nhau làm khóa được, vì `==` và hash theo nội dung. `12000.0` được chuẩn hóa nên `repr b` in `Decimal('12000')`. Dòng `replace chỉ tạo bản sao` cho thấy `replace` đổi `expires_at` mà giá vẫn là giá cũ: nó là công cụ sao chép, không phải cách gia hạn báo giá. Gia hạn đúng nghĩa là phát hành báo giá mới bằng `Quote.issue`, để giá được tính lại. `đến 08:35:00 +0700` là `expires_at` (01:35 UTC) hiển thị theo giờ Việt Nam, và đúng 5 phút sau `T0` báo giá đã hết hạn.

Chế độ `--verify` so toàn bộ các kết quả này, gồm cả thông báo lỗi, ba ca biên của `Quote`, ca loại tiền có ký hiệu lạ như `U$D` và giá 0, 2, 2,5, 5 km (12.000, 12.000, 14.250, 25.500 như mọi bài trước):

```bash
uv run --project examples/toi-noi-python/13-frozen-dataclass-enum examples/toi-noi-python/13-frozen-dataclass-enum/main.py --verify
```

```text
OK 64/64
```

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `FrozenInstanceError: cannot assign to field ...` | Đúng thiết kế; tạo bản mới bằng `dataclasses.replace(obj, field=...)` |
| `ValueError: 'x' is not a valid TripStatus` | Chuỗi đầu vào sai tên hoặc sai chữ hoa chữ thường; kiểm nguồn dữ liệu trước khi chuẩn hóa |
| `TypeError: unhashable type: 'list'` khi hash một frozen dataclass | Field chứa list/dict/set; đổi sang `tuple` hoặc `frozenset` |
| `Money` này bằng `Money` kia nhưng `repr` khác | Equality theo giá trị `Decimal`; chuẩn hóa trong `__post_init__` nếu cần cùng dạng |
| Hai kiểu khác nghĩa lại `==` nhau | Có phải `NamedTuple`? So như tuple; đổi sang frozen dataclass nếu cần phân biệt kiểu |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Bài 13 chỉ nói về dữ liệu trong bộ nhớ. <code>Quote</code> ở đây không có mã định danh, không được lưu, và không có chữ ký: nếu client gửi lại «giá đã báo» thì server phải tra hoặc xác thực lại báo giá, không tin con số gửi lên. Thời hạn 5 phút là giả định học tập, và <code>is_valid(now)</code> chỉ đúng khi <code>now</code> đến từ một đồng hồ đáng tin cậy; lệch đồng hồ giữa các server sẽ làm biên hạn lệch theo. <code>Money</code> này chỉ có VND, không âm, không có hoàn tiền, phụ phí hay thuế, và <code>__add__</code> là tất cả phép toán có. Hệ thống thật cần thêm làm tròn theo loại tiền, phép trừ có chủ đích, và cách lưu số tiền trong database (<code>NUMERIC</code>, không dùng <code>float</code>). Enum cũng chỉ bảo vệ ở phía Python: một dòng lạ trong cột <code>status</code> của database vẫn có thể có, nên biên đọc dữ liệu phải gọi <code>TripStatus(value)</code> và xử lý <code>ValueError</code>.</p>
</div>

**Bài tập 1:** thêm phép trừ `Money.__sub__` với cùng quy tắc khác loại tiền như `__add__`. Khi kết quả âm thì sao, ta có phải viết thêm kiểm tra không?

**Bài tập 2:** dữ liệu từ một hệ thống cũ gửi `"Requested"` (viết hoa chữ đầu). `TripStatus("Requested")` cho kết quả gì, và bạn nên xử lý ở đâu?

<details><summary>Xem lời giải</summary><p>(1) Viết giống <code>__add__</code> nhưng dùng <code>self.amount - other.amount</code> rồi trả <code>Money(...)</code>. Không cần kiểm tra thêm: <code>Money.__post_init__</code> đã từ chối số âm, nên <code>Money(Decimal("2250")) - Money(Decimal("12000"))</code> tự ném <code>ValueError: amount không được âm: -9750</code>, còn <code>12000 - 2250</code> ra <code>9.750 VND</code>. Đây là lợi ích của việc đặt quy tắc ở constructor: mọi đường tạo <code>Money</code> đều đi qua nó. Nhưng hãy cân nhắc: «không âm» là quy tắc đúng với giá cước, chưa chắc đúng với số dư hay khoản hoàn tiền, và khi đó có thể cần kiểu khác thay vì nới lỏng <code>Money</code>. (2) <code>TripStatus("Requested")</code> ném <code>ValueError: 'Requested' is not a valid TripStatus</code>, vì Enum phân biệt chữ hoa chữ thường. Nên chuẩn hóa ở <b>biên tiếp nhận</b> (bộ chuyển đổi dữ liệu từ hệ thống cũ, ví dụ <code>TripStatus(raw.lower())</code>) và ghi lại nguồn gốc, chứ không nới lỏng chính <code>TripStatus</code> để nó nhận mọi dạng viết; nếu không, mọi nơi khác sẽ phải đoán dữ liệu nào đã được làm sạch.</p></details>

Bài trước: [bài 12](/articles/python-type-hints-mypy-protocol-generics). Bài tiếp: [bài 14](/articles/python-abc-protocol-composition-dependency-injection).

## Tài liệu gốc

- [Thư viện chuẩn: dataclasses (frozen, replace, `__post_init__`)](https://docs.python.org/3.14/library/dataclasses.html).
- [Thư viện chuẩn: enum (`StrEnum`)](https://docs.python.org/3.14/library/enum.html).
- [Enum HOWTO](https://docs.python.org/3.14/howto/enum.html).
- [typing.NamedTuple](https://docs.python.org/3.14/library/typing.html#typing.NamedTuple).
- [Data model: `object.__hash__`](https://docs.python.org/3.14/reference/datamodel.html#object.__hash__).
- [PEP 435: Adding an Enum type to the Python standard library](https://peps.python.org/pep-0435/).
