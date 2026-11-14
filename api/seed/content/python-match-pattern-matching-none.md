---
slug: python-match-pattern-matching-none
title: "Pattern matching và xử lý None: chuyển trạng thái chuyến rõ ràng"
summary: "Bài 16: dùng match/case với class, sequence và mapping pattern, guard để mô tả chuyến và định tuyến sự kiện; tách None, chuỗi rỗng và khoá không có trong dict; sentinel, TypeIs và mypy kiểm tra đủ trường hợp. Có kết quả chạy thật."
category: python-beginner
level: beginner
tags: [python, toi-noi, pattern-matching, none]
language: vi
---

Màn hình theo dõi chuyến của [Tới Nơi](/articles/du-an-mau-toi-noi) từng hiện dòng «Tài xế: None» cho khách đang chờ ghép tài xế. Bản sửa nhanh là `if trip.driver_id:`, nhưng bản sửa này che luôn một lỗi khác: có chuyến đã nhận mà `driver_id` là chuỗi rỗng, và màn hình cũng ghi «chưa có tài xế» như không có gì xảy ra. Ba tình huống khác nghĩa (chưa có tài xế, thiếu tài xế do dữ liệu lỗi, có tài xế) bị gộp vào một nhánh `if`.

Đây là bài 16 của lộ trình **Python cơ bản qua Tới Nơi**. Bài dùng `TripStatus(StrEnum)` từ bài 13 và `dataclass` từ bài 11, và giới thiệu `match/case` cùng cách xử lý `None` để mỗi tình huống có một nhánh riêng, có tên.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trong <code>match trip.status:</code>, dòng <code>case ACCEPTED:</code> có so trạng thái với <code>TripStatus.ACCEPTED</code> không? Chuyện gì xảy ra nếu bên dưới còn các <code>case</code> khác?</p>
<details><summary>Xem đáp án</summary><p><b>Không so gì cả.</b> Tên trần trong <code>case</code> là <b>capture</b>: nó khớp với mọi giá trị và gán giá trị đó vào biến tên <code>ACCEPTED</code>. Vì vậy mọi <code>case</code> phía dưới không bao giờ chạy, và Python báo <code>SyntaxError: name capture 'ACCEPTED' makes remaining patterns unreachable</code> nếu còn <code>case</code> sau nó. Muốn so với hằng, hãy viết tên có dấu chấm: <code>case TripStatus.ACCEPTED:</code>. Bài sẽ chạy đúng trường hợp này.</p></details>
</div>

## match/case: so khớp cấu trúc, không chỉ so bằng

`match` (từ Python 3.10, PEP 634) thường bị gọi là «switch của Python». Gọi vậy hơi sai: `switch` ở nhiều ngôn ngữ chỉ so một giá trị với các hằng, còn `match` là **structural pattern matching**: mỗi `case` là một **pattern** mô tả hình dạng dữ liệu, và khi khớp, pattern có thể gán luôn các phần của dữ liệu vào biến. Ba quy tắc cần thuộc trước khi đọc code:

- Các `case` được thử **từ trên xuống**; case đầu tiên khớp thắng, các case sau bị bỏ qua. Thứ tự là một phần của logic.
- Sau pattern có thể có **guard** `if điều_kiện`; guard chỉ chạy khi pattern đã khớp, và nếu guard sai thì Python thử case kế tiếp.
- `case _:` khớp mọi thứ và không gán biến. Case này thường đứng cuối để bắt «phần còn lại».

`match` và `case` là **soft keyword**: chỉ có nghĩa đặc biệt ở đầu câu lệnh `match`, nên code cũ đặt tên biến `match` vẫn chạy.

## Class pattern và guard: mô tả chuyến

**Class pattern** viết như gọi constructor: `Trip(status=TripStatus.SEARCHING, driver_id=None)`. Python kiểm tra `isinstance(subject, Trip)` rồi đọc từng thuộc tính theo tên và so với pattern con. Với `dataclass`, `__match_args__` gồm các field tham gia `__init__`, theo thứ tự khai báo. `Trip` của lab này khai báo đủ bảy trường theo từ điển miền nên dạng theo vị trí `Trip(trip_id, _, None)` hợp lệ; nhưng `Trip` của bài 11 đặt `driver_id` và `status` là `init=False`, nên `__match_args__` của nó chỉ còn năm tên (`trip_id, customer_id, service, distance_km, created_at`) và vị trí thứ ba là `service`. Vì thế dạng có tên an toàn hơn. Hàm `describe` trong `matching.py` biến mọi chuyến thành một câu cho người dùng, kể cả dữ liệu lỗi:

```python
def describe(trip: Trip | None) -> str:
    match trip:
        case None:  # `is None`: chuyến không tồn tại
            return "Không tìm thấy chuyến"
        case Trip(status=TripStatus.CANCELLED):
            return "Chuyến đã hủy"
        case Trip(status=TripStatus.REQUESTED | TripStatus.SEARCHING, driver_id=None):
            return "Đang tìm tài xế"
        case Trip(status=TripStatus.REQUESTED | TripStatus.SEARCHING, driver_id=str()):
            return "Dữ liệu lỗi: chưa nhận chuyến mà đã có tài xế"
        case Trip(driver_id=None):  # tới đây trạng thái từ accepted trở đi
            return "Dữ liệu lỗi: chuyến đã nhận mà thiếu tài xế"
        case Trip(driver_id=str() as driver) if not driver.strip():
            return "Dữ liệu lỗi: driver_id trống"
        case Trip(status=TripStatus.IN_TRANSIT, distance_km=km, driver_id=driver) if km > 20:
            return f"Đang chạy đường dài {km} km, tài xế {driver}"
        case Trip(status=status, driver_id=driver):
            return f"Tài xế {driver}: {status.value}"
        case _:  # mypy biết trip đã hết khả năng: assert_never báo nếu thêm trường hợp mà quên xử lý
            assert_never(trip)
```

Mỗi thành phần có một việc:

- `case None` dùng so sánh `is None`; PEP 634 quy định `None`, `True`, `False` được so bằng `is`, còn các literal khác so bằng `==`.
- `TripStatus.REQUESTED | TripStatus.SEARCHING` là **or-pattern**: một trong hai trạng thái. Tên có dấu chấm là **value pattern**, so bằng `==` với hằng enum. Vì `StrEnum` bằng chuỗi cùng giá trị, chuỗi `"accepted"` cũng khớp `case TripStatus.ACCEPTED` (lab có test case kiểm tra điều này); đó là lý do dữ liệu từ ngoài nên đi qua bước validate ở bài 15 trước khi vào `match`.
- `str()` là class pattern không có đối số: khớp mọi `str`, kể cả chuỗi rỗng. `str() as driver` vừa kiểm kiểu vừa đặt tên cho giá trị.
- `driver_id=str()` ở case thứ tư tách «có tài xế» khỏi «chưa có» mà không cần so với một giá trị cụ thể.
- Hai case có guard (`if not driver.strip()`, `if km > 20`) dùng giá trị vừa bắt được để quyết định thêm.

Thứ tự ở đây có chủ đích: chuyến `cancelled` được xử lý trước, vì chuyến bị huỷ khi đang tìm tài xế có `driver_id` là `None` mà hoàn toàn hợp lệ. Nếu đặt `case Trip(driver_id=None)` lên trước case `CANCELLED`, chuyến huỷ đó sẽ bị báo lỗi dữ liệu. Kết quả chạy trên mười chuyến mẫu:

```text
không có chuyến            -> Không tìm thấy chuyến
requested, chưa tài xế     -> Đang tìm tài xế
searching, chưa tài xế     -> Đang tìm tài xế
requested mà có tài xế     -> Dữ liệu lỗi: chưa nhận chuyến mà đã có tài xế
accepted, DR-001           -> Tài xế DR-001: accepted
accepted, thiếu tài xế     -> Dữ liệu lỗi: chuyến đã nhận mà thiếu tài xế
accepted, driver_id trống  -> Dữ liệu lỗi: driver_id trống
in_transit 25 km           -> Đang chạy đường dài 25 km, tài xế DR-001
in_transit 5 km            -> Tài xế DR-001: in_transit
cancelled                  -> Chuyến đã hủy
```

So với tình huống mở đầu, màn hình không còn ghi «Tài xế: None»: «chưa có tài xế» và «dữ liệu lỗi» là hai dòng khác nhau. Bài C# [nullable và pattern matching](/articles/csharp-nullable-pattern-matching) dùng switch expression cho đúng ba tình huống này; cú pháp khác nhưng tư duy giống nhau.

## Sequence và or-pattern: chuyển trạng thái

Chuyển trạng thái cần xét **hai thứ cùng lúc**: trạng thái hiện tại và lệnh. Ta ghép chúng thành một tuple trong câu lệnh `match`, rồi viết **sequence pattern** có or-pattern và guard:

```python
def apply_command(trip: Trip, command: tuple[object, ...]) -> Trip:
    """Trả Trip mới (không sửa tại chỗ). Lệnh không khớp trạng thái -> InvalidTransition."""
    match trip.status, command:
        case TripStatus.REQUESTED, ("search",):
            return replace(trip, status=TripStatus.SEARCHING)
        case TripStatus.SEARCHING, ("assign", str() as driver_id) if driver_id.strip():
            return replace(trip, status=TripStatus.ACCEPTED, driver_id=driver_id)
        case TripStatus.ACCEPTED, ("arrive",):
            return replace(trip, status=TripStatus.ARRIVING)
        # ... pickup, depart, complete theo cùng khuôn (xem matching.py)
        case (
            TripStatus.REQUESTED
            | TripStatus.SEARCHING
            | TripStatus.ACCEPTED
            | TripStatus.ARRIVING
        ), ("cancel",):
            return replace(trip, status=TripStatus.CANCELLED)
        case _:
            raise InvalidTransition(f"{trip.status.value} không nhận lệnh {command!r}")
```

Mỗi `case` đọc như một dòng của bảng chuyển trạng thái: «đang ở `searching` và nhận lệnh `assign` kèm một tài xế không trống thì thành `accepted`». Không có nhánh nào cho các cặp còn lại, và `case _` biến chúng thành `InvalidTransition`, nên không thể chuyển sai mà không báo lỗi. Hàm trả `Trip` mới bằng `dataclasses.replace` thay vì sửa chuyến cũ; test dễ viết hơn, và lab kiểm tra chuyến gốc không đổi.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Bảy trạng thái đi tuần tự từ requested tới completed; từ bốn trạng thái đầu có thể chuyển sang cancelled."><defs><marker id="py16-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py16-0-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="40" width="104" height="50" rx="8" class="f-pk-soft s-pk"/><text x="30" y="70" class="t-mono">requested</text><path d="M124 65 L144 65" class="f-none s-phys" stroke-width="2" marker-end="url(#py16-0-arrow)"/><rect x="144" y="40" width="104" height="50" rx="8" class="f-pk-soft s-pk"/><text x="154" y="70" class="t-mono">searching</text><path d="M248 65 L268 65" class="f-none s-phys" stroke-width="2" marker-end="url(#py16-0-arrow)"/><rect x="268" y="40" width="104" height="50" rx="8" class="f-pk-soft s-pk"/><text x="278" y="70" class="t-mono">accepted</text><path d="M372 65 L392 65" class="f-none s-phys" stroke-width="2" marker-end="url(#py16-0-arrow)"/><rect x="392" y="40" width="104" height="50" rx="8" class="f-pk-soft s-pk"/><text x="402" y="70" class="t-mono">arriving</text><path d="M496 65 L516 65" class="f-none s-phys" stroke-width="2" marker-end="url(#py16-0-arrow)"/><rect x="516" y="40" width="104" height="50" rx="8" class="f-phys-soft s-phys"/><text x="526" y="70" class="t-mono">picked_up</text><path d="M620 65 L640 65" class="f-none s-phys" stroke-width="2" marker-end="url(#py16-0-arrow)"/><rect x="640" y="40" width="104" height="50" rx="8" class="f-phys-soft s-phys"/><text x="650" y="70" class="t-mono">in_transit</text><path d="M744 65 L764 65" class="f-none s-phys" stroke-width="2" marker-end="url(#py16-0-arrow)"/><rect x="764" y="40" width="104" height="50" rx="8" class="f-phys-soft s-phys"/><text x="774" y="70" class="t-mono">completed</text><path d="M72 90 L72 125 L444 125" class="f-none s-warn dash" stroke-width="2"/><path d="M196 90 L196 125" class="f-none s-warn dash" stroke-width="2"/><path d="M320 90 L320 125" class="f-none s-warn dash" stroke-width="2"/><path d="M444 90 L444 125" class="f-none s-warn dash" stroke-width="2"/><path d="M140 125 L140 160" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py16-0-warn)"/><rect x="20" y="160" width="240" height="50" rx="8" class="f-warn-soft s-warn"/><text x="30" y="190" class="t-mono">cancelled</text><text x="290" y="180" class="t-small">cancel hợp lệ từ bốn trạng thái đầu</text><text x="290" y="202" class="t-small">assign chỉ nhận driver_id không trống</text></svg>
</div>
<figcaption>Hình 1. Bảy trạng thái đi tuần tự và nhánh huỷ. Mỗi mũi tên là một case trong apply_command; cặp (trạng thái, lệnh) không có case nào thì rơi vào case cuối và bị từ chối.</figcaption>
</figure>

Hai chi tiết của sequence pattern đáng nhớ. Pattern `("search",)` khớp **mọi sequence** có đúng một phần tử `"search"`, nên danh sách `["search"]` cũng khớp. Ngược lại, **chuỗi `str` không bao giờ được coi là sequence** trong `match`, nên `match "cancel"` không khớp `("cancel",)`; nếu không, mỗi chuỗi sẽ thành dãy ký tự và gây ra những lần khớp rất khó hiểu. Chạy thử đường đi hợp lệ và bốn lệnh bị từ chối:

```text
('search',)            -> searching  driver=None
('assign', 'DR-001')   -> accepted   driver=DR-001
('arrive',)            -> arriving   driver=DR-001
('pickup',)            -> picked_up  driver=DR-001
('depart',)            -> in_transit driver=DR-001
('complete',)          -> completed  driver=DR-001
từ chối: completed không nhận lệnh ('cancel',)
từ chối: searching không nhận lệnh ('assign', '')
từ chối: requested không nhận lệnh ('assign', 'DR-001')
từ chối: picked_up không nhận lệnh ('cancel',)
```

Còn một việc mà `match` giúp công cụ làm hộ: nhắc ta khi thiếu trường hợp. Hàm `next_step_label(status)` duyệt đủ tám trạng thái và kết thúc bằng `case _: assert_never(status)`. Khi kiểu của `status` đã bị loại hết thành viên, mypy thấy đối số là `Never` và không báo gì. Nếu ta bỏ sót một trạng thái, mypy báo lỗi ngay lúc kiểm kiểu, không đợi tới lúc chạy. `unsafe_example.py` trong lab cố ý chỉ xử lý hai trạng thái:

```text
examples/toi-noi-python/16-match-pattern-none/unsafe_example.py:23: error: Argument 1 to "assert_never" has incompatible type "Literal[TripStatus.SEARCHING, TripStatus.ACCEPTED, TripStatus.ARRIVING, TripStatus.PICKED_UP, TripStatus.IN_TRANSIT, TripStatus.CANCELLED]"; expected "Never"  [arg-type]
```

Thông báo liệt kê đúng sáu trạng thái còn thiếu. Đây là lý do nên thêm trạng thái mới vào `TripStatus` rồi để mypy chỉ ra mọi `match` cần sửa. bài 12 đã nói về mypy; còn `assert_never` được giới thiệu ngay ở bài này.

## Mapping pattern: định tuyến sự kiện

Sự kiện trong Tới Nơi là envelope JSON (`event_id`, `event_type`, `version`, `payload`). Sau khi parse, Python có một `dict`, và **mapping pattern** khớp trực tiếp hình dạng của nó:

```python
def handle_event(envelope: dict[str, Any]) -> str:
    match envelope:
        case {
            "event_type": "BookingCreated",
            "version": 1,
            "payload": {"trip_id": str() as trip_id, "service": "ride" | "food" | "parcel" as service},
        }:
            return f"tạo chuyến {trip_id} ({service})"
        case {
            "event_type": "TripCompleted",
            "version": 1,
            "payload": {"trip_id": str() as trip_id, "driver_id": None},
        }:  # khóa CÓ mặt và là null: khác với khóa vắng mặt
            return f"cách ly {trip_id}: TripCompleted có driver_id null"
        case {"event_type": str() as event_type, "version": int() as version} if version > 1:
            return f"bỏ qua {event_type} v{version}: phiên bản mới hơn consumer hiểu"
        case {"event_type": str() as event_type, **rest}:  # **rest gom các khóa còn lại
            return f"cách ly {event_type}: payload sai hợp đồng (khóa: {sorted(rest)})"
        case _:
            return "cách ly: không phải envelope"
```

(Đoạn trích bỏ case `TripCompleted` bình thường; xem `matching.py`.) Quy tắc của mapping pattern: dict phải **có đủ các khoá** được nêu, giá trị khớp pattern con, và **khoá thừa bị bỏ qua**. Vì vậy `note` trong payload không làm hỏng case đầu. Muốn gom phần thừa thì dùng `**rest`. Tám envelope mẫu cho kết quả:

```text
BookingCreated v1    -> tạo chuyến TN-0001 (food)
TripCompleted v1     -> chuyến TN-0001 hoàn tất, tài xế DR-001
driver_id là null    -> cách ly TN-0002: TripCompleted có driver_id null
driver_id vắng mặt   -> cách ly TripCompleted: payload sai hợp đồng (khóa: ['payload', 'version'])
version 2            -> bỏ qua TripCompleted v2: phiên bản mới hơn consumer hiểu
service lạ           -> cách ly BookingCreated: payload sai hợp đồng (khóa: ['payload', 'version'])
version là True      -> tạo chuyến TN-0005 (ride)
không có event_type  -> cách ly: không phải envelope
```

Hai dòng đáng đọc kỹ. Dòng «driver_id là null» và «driver_id vắng mặt» đi hai nhánh khác nhau, vì mapping pattern phân biệt khoá có giá trị `None` với khoá không có mặt. Dòng «version là True» là một cái bẫy: literal `1` trong pattern được so bằng `==`, mà `True == 1`, nên `"version": True` đi qua như phiên bản 1. Lab chạy thêm `match True: case 1:` và `match 1.0: case 1:`: cả hai đều khớp.

Bẫy này nhắc ta nhớ đúng phạm vi của `match`: nó **định tuyến** theo hình dạng, không thay việc **kiểm tra** dữ liệu. Envelope phải được validate trước (bài 15 giới thiệu validate và strict mode ở biên); `match` sau đó chỉ chọn nhánh xử lý. Đừng viết hàng chục `case` để thay cho một model.

## None: ba trạng thái, sentinel và thu hẹp kiểu

`None` là giá trị duy nhất của kiểu `NoneType`, và Python dùng nó cho mọi nghĩa của «không có». Nghĩa nào cũng thành `None` thì code khó đoán. `driver_id` có bốn khả năng (Hình 2): khoá vắng mặt trong payload, `None`, chuỗi trống, và mã hợp lệ. Cách viết dựa vào truthiness (giá trị đúng/sai của đối tượng trong điều kiện) quen thuộc sẽ gộp ba ô đầu:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Bốn trường hợp của driver_id: khoá vắng mặt, None, chuỗi trống và mã hợp lệ có ý nghĩa khác nhau; truthiness gộp ba trường hợp đầu thành một nhánh."><defs><marker id="py16-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="190" height="110" rx="10" class="f-warn-soft s-warn"/><text x="34" y="76" class="t-b">Khoá vắng mặt</text><text x="34" y="104" class="t-small">payload thiếu driver_id</text><text x="34" y="124" class="t-small">hợp đồng bị vi phạm</text><rect x="230" y="40" width="190" height="110" rx="10" class="f-phys-soft s-phys"/><text x="244" y="76" class="t-b">None</text><text x="244" y="104" class="t-small">chưa có tài xế</text><text x="244" y="124" class="t-small">trạng thái hợp lệ</text><rect x="440" y="40" width="190" height="110" rx="10" class="f-warn-soft s-warn"/><text x="454" y="76" class="t-b">Chuỗi trống</text><text x="454" y="104" class="t-small">dữ liệu lỗi cần ghi</text><text x="454" y="124" class="t-small">không phải «chưa có»</text><rect x="650" y="40" width="190" height="110" rx="10" class="f-phys-soft s-phys"/><text x="664" y="76" class="t-b">DR-001</text><text x="664" y="104" class="t-small">tài xế hợp lệ</text><text x="664" y="124" class="t-small">hiển thị bình thường</text><rect x="20" y="180" width="610" height="44" rx="10" class="f-surface s-line dash"/><text x="34" y="207" class="t-small">driver_id or "chưa có": một nhánh duy nhất cho ba ô đầu, mất ranh giới giữa chúng</text></svg>
</div>
<figcaption>Hình 2. Bốn trường hợp của driver_id. Chỉ ô thứ hai và thứ tư là trạng thái hợp lệ; truthiness không phân biệt được ba ô đầu.</figcaption>
</figure>

```python
def driver_label_truthy(trip: Trip) -> str:
    """Cách viết hay gặp nhưng sai: truthiness gộp None và chuỗi rỗng."""
    return trip.driver_id or "chưa có tài xế"


def driver_label(trip: Trip) -> str:
    """Tách rõ: None (chưa có), "" hoặc toàn khoảng trắng (dữ liệu lỗi), chuỗi hợp lệ."""
    match trip.driver_id:
        case None:
            return "chưa có tài xế"
        case str() as driver if not driver.strip():
            return "dữ liệu lỗi"
        case str() as driver:
            return driver
```

```text
None        truthy='chưa có tài xế'     match='chưa có tài xế'
chuỗi rỗng  truthy='chưa có tài xế'     match='dữ liệu lỗi'
DR-001      truthy='DR-001'             match='DR-001'
```

Dòng thứ hai là điểm khác: truthiness che mất dữ liệu lỗi. Ở bài 03 ta đã gặp cùng cái bẫy với `Decimal("0")`; nguyên tắc vẫn là so sánh tường minh khi các giá trị rơi vào cùng nhánh có nghĩa khác nhau.

Có vài điều về kiểu cần nói thêm. `X | None` (PEP 604, Python 3.10) là cách viết hiện đại của `Optional[X]`; ở Python 3.14, `Optional[int] == (int | None)` là `True`. Chữ «Optional» dễ gây hiểu nhầm: nó **không** có nghĩa «tham số tuỳ chọn». Một tham số có thể bỏ trống vẫn cần giá trị mặc định, và kiểu của nó chỉ chứa `None` khi `None` là giá trị hợp lệ. Còn khi so sánh với `None`, hãy viết `is None` thay vì `== None`: `is` kiểm tra đúng đối tượng duy nhất đó và không bị `__eq__` do lớp khác tự định nghĩa can thiệp.

### Khi None đã mang nghĩa: sentinel

Khi `None` là giá trị hợp lệ, nó không thể đồng thời đóng vai «không truyền gì». Hàm `patch_trip` cập nhật từng phần một chuyến: `driver_id=None` nghĩa là **bỏ tài xế** (tài xế huỷ, chuyến quay về tìm người khác), còn không truyền `driver_id` nghĩa là **giữ nguyên**. Ta cần một giá trị thứ ba, gọi là **sentinel**:

```python
class _Unset(Enum):
    UNSET = "UNSET"


# Sentinel: khác với None. `driver_id=None` nghĩa là "bỏ tài xế", bỏ trống tham số nghĩa là "giữ nguyên".
UNSET: Final = _Unset.UNSET


def patch_trip(
    trip: Trip,
    *,
    driver_id: str | None | _Unset = UNSET,
    status: TripStatus | _Unset = UNSET,
) -> Trip:
    changes: dict[str, Any] = {}
    if driver_id is not UNSET:
        changes["driver_id"] = driver_id
    if status is not UNSET:
        changes["status"] = status
    return replace(trip, **changes)
```

```text
patch không tham số     : DR-001
patch driver_id=None    : None
patch driver_id='DR-002': DR-002
```

Sentinel ở đây là Enum một thành viên: mypy hiểu `is not UNSET` để thu hẹp kiểu, còn một `object()` trần thì không. Dùng khi hàm thực sự cần phân biệt «không truyền» với `None`; còn lại, mặc định `None` là đủ và đơn giản hơn.

### Thu hẹp kiểu với TypeIs và TypeGuard

Sau `if driver_id is not None`, mypy biết `driver_id` là `str`: đó là việc **thu hẹp kiểu** (narrowing). Khi điều kiện nằm trong một hàm riêng, ta nói cho mypy biết hàm đó thu hẹp gì bằng `TypeIs` (PEP 742, Python 3.13) hoặc `TypeGuard`:

```python
def is_assigned(driver_id: str | None) -> TypeIs[str]:
    """TypeIs hợp lệ vì kết quả đúng khi và chỉ khi giá trị là str (thu hẹp cả nhánh else)."""
    return driver_id is not None


def has_usable_driver(driver_id: str | None) -> TypeGuard[str]:
    """Phải là TypeGuard: "" cũng là str nhưng trả False, nên nhánh else KHÔNG được thu hẹp thành None."""
    return driver_id is not None and driver_id.strip() != ""
```

Điểm khác nhau là nhánh `else`. `TypeIs[str]` hứa «trả `True` khi và chỉ khi giá trị là `str`», nên mypy suy ra nhánh `else` là phần còn lại (`None`). `has_usable_driver("")` trả `False` dù `""` vẫn là `str`, nên không thể hứa như vậy; `TypeGuard` chỉ thu hẹp nhánh đúng. Nếu nhầm dùng `TypeIs` cho hàm thứ hai, mypy sẽ tin rằng `driver_id` là `None` ở nhánh `else` và có thể bỏ qua chuỗi trống thật. `reveal_type` trong `unsafe_example.py` cho thấy rõ sự khác nhau (trích, rút gọn đường dẫn):

```text
unsafe_example.py:28: note: Revealed type is "str"
unsafe_example.py:30: note: Revealed type is "None"
unsafe_example.py:32: note: Revealed type is "str"
unsafe_example.py:34: note: Revealed type is "str | None"
```

Bốn dòng lần lượt là nhánh đúng và nhánh `else` của `is_assigned`, rồi nhánh đúng và nhánh `else` của `has_usable_driver`. File này cũng chứa trường hợp `trip.driver_id[0]` mà không kiểm tra `None`, và mypy báo `Value of type "str | None" is not indexable`: lỗi `TypeError` vốn sẽ xảy ra ở runtime nay được bắt sớm lúc kiểm kiểu.

## Chạy và kiểm chứng

Cần uv và Python 3.14. Lần đầu uv tải `mypy` nên cần mạng. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/16-match-pattern-none examples/toi-noi-python/16-match-pattern-none/main.py
uv run --project examples/toi-noi-python/16-match-pattern-none examples/toi-noi-python/16-match-pattern-none/main.py --verify
uv run --project examples/toi-noi-python/16-match-pattern-none mypy examples/toi-noi-python/16-match-pattern-none/main.py --strict
uv run --project examples/toi-noi-python/16-match-pattern-none mypy examples/toi-noi-python/16-match-pattern-none/unsafe_example.py --strict
```

Lệnh đầu in các demo trong bài (các đoạn kết quả ở trên là trích từ đó), kể cả phần «bẫy» chạy thật:

```text
tên trần trong case: name capture 'ACCEPTED' makes remaining patterns unreachable
tên trần khớp mọi giá trị: khớp
literal 1 khớp True
literal 1 khớp 1.0
chuỗi KHÔNG khớp sequence pattern
list khớp pattern viết bằng tuple
```

Lệnh `--verify` chạy 56 test case (10 mô tả chuyến, đường đi bảy trạng thái, bốn test case huỷ hợp lệ, chín test case bị từ chối, tám envelope, rồi các test case về bẫy cú pháp, `None` và sentinel) và kết thúc bằng:

```text
OK 56/56
```

`mypy main.py --strict` in `Success: no issues found in 1 source file`; lệnh cuối với `unsafe_example.py` báo hai lỗi và bốn dòng `Revealed type` như trên. Bản đã chạy dùng Python 3.14.7 và mypy 2.4.1 (xem `uv.lock`). Nếu một test case của `--verify` bị lệch, chương trình in `SAI: <tên ca>`, rồi `FAIL n/m` (n là số test case đạt, m là tổng) và thoát với mã 1.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `SyntaxError: name capture ... makes remaining patterns unreachable` | Có tên trần (không dấu chấm) trong `case`; viết `TripStatus.X` thay vì `X` |
| Mọi giá trị đều rơi vào một case bất ngờ | Tên trần đang là capture; case nhận tất cả đứng quá cao trong danh sách |
| `"version": True` hoặc `1.0` khớp `1` | Literal so bằng `==`; validate kiểu bằng model (bài 15) trước khi `match` |
| `match "abc"` không khớp pattern dạng `[a, b, c]` | `str` không phải sequence trong `match`; dùng `str()` hoặc `case "abc"` |
| «Chưa có tài xế» lẫn với dữ liệu lỗi | Có đang dùng `if not x` / `x or ...` trên giá trị có nhiều nghĩa không |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Ví dụ chỉ chạy trong bộ nhớ với dữ liệu giả: không có database, không có hai tiến trình cùng đổi trạng thái một chuyến. Production phải bảo đảm chuyển trạng thái nguyên tử (transaction, khoá lạc quan) và thường lưu lịch sử chuyển; <code>apply_command</code> trả một <code>Trip</code> mới không thay cho việc đó. <code>match</code> làm bảng chuyển dễ đọc nhưng không thay validation đầu vào, và danh sách lệnh dạng tuple chuỗi ở đây chỉ để minh hoạ: service thật nên có kiểu lệnh riêng. mypy chỉ kiểm tra khi ta chạy nó, và <code>assert_never</code> chỉ hữu ích nếu CI thật sự chạy mypy.</p>
</div>

**Bài tập 1:** Thêm trạng thái mới `EXPIRED = "expired"` vào `TripStatus`. Lệnh nào báo cho bạn những chỗ cần sửa, và chương trình có tự phát hiện lúc chạy `main.py` không?

**Bài tập 2:** Trong `describe`, đổi vị trí hai case: đưa `case Trip(driver_id=None)` lên ngay sau `case None`. Chuyến nào được mô tả sai?

<details><summary>Xem lời giải</summary><p><b>Bài 1.</b> <code>mypy --strict</code> báo <code>assert_never</code> nhận <code>Literal[TripStatus.EXPIRED]</code> thay vì <code>Never</code> ở <code>next_step_label</code> (và ở mọi <code>match</code> kết thúc bằng <code>assert_never</code> theo trạng thái). Khi chạy, <code>main.py</code> vẫn thoát mã 0 vì không gọi <code>next_step_label</code>; còn <code>main.py --verify</code> gọi hàm đó cho mọi trạng thái nên dừng bằng traceback <code>AssertionError</code> (không in <code>FAIL n/m</code>). Không chạy mypy thì lỗi chỉ lộ khi có dữ liệu thật đi vào nhánh đó. Với <code>describe</code>, chuyến mang trạng thái mới vẫn rơi vào case tổng quát <code>Trip(status=status, driver_id=driver)</code> nên không lỗi; hãy tự quyết xem trạng thái mới cần mô tả riêng không. <b>Bài 2.</b> Ba chuyến bị gắn nhãn «thiếu tài xế» sai: <code>requested</code> và <code>searching</code> chưa có tài xế (đúng ra là «Đang tìm tài xế»), và <code>cancelled</code> không có tài xế (đúng ra là «Chuyến đã huỷ»). Lý do là case rộng nằm trước các case hẹp, mà case đầu tiên khớp thắng. <code>--verify</code> báo đúng ba test case đó không đạt.</p></details>

Bài trước: [bài 15](/articles/python-pydantic-validation-settings). Bài tiếp: [bài 17](/articles/python-decorator-context-manager).

## Tài liệu gốc

- [PEP 634: Structural Pattern Matching: Specification](https://peps.python.org/pep-0634/).
- [PEP 636: Structural Pattern Matching: Tutorial](https://peps.python.org/pep-0636/).
- [Python reference: The match statement](https://docs.python.org/3.14/reference/compound_stmts.html#the-match-statement).
- [PEP 742: Narrowing types with TypeIs](https://peps.python.org/pep-0742/).
- [mypy: Type narrowing](https://mypy.readthedocs.io/en/stable/type_narrowing.html).
