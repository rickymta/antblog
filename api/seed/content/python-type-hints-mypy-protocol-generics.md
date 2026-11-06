---
slug: python-type-hints-mypy-protocol-generics
title: "Type hints, mypy/pyright, Protocol và generics: hợp đồng kiểu cho mã nghiệp vụ"
summary: "Bài 12: type hint không được Python kiểm lúc chạy, mypy mới đọc chúng. Dùng PEP 695, TypedDict, Protocol, annotation lười (PEP 649) trên mã Tới Nơi, chạy mypy --strict thật và xem 9 lỗi kiểu bị bắt."
category: python
level: beginner
tags: [python, toi-noi, type-hints, mypy]
language: vi
---

Một service phân tích của [Tới Nơi](/articles/du-an-mau-toi-noi) đọc sự kiện `BookingCreated` từ Kafka. Ở bài 11, ta đã bảo vệ `Trip` bằng dataclass và máy trạng thái, nhưng dữ liệu đi vào service vẫn là dict từ JSON. Một đêm, producer bên kia gửi khoảng cách dưới dạng số thực `2.5` thay vì chuỗi `"2.5"`, và một chuyến chưa có tài xế (`driver_id` là `None`) đi qua đoạn mã gọi `.upper()` trên mã tài xế. Cả hai lỗi chỉ lộ ra khi đúng dòng đó chạy trên đúng dữ liệu đó, có thể là hai giờ sáng. Python vốn không hỏi «giá trị này có đúng kiểu không» trước khi chạy.

Hai lỗi này khác bản chất. Lỗi `None` nằm trong **code của ta**: một công cụ đọc kiểu có thể thấy nó trước khi chương trình chạy. Lỗi số thực là **dữ liệu từ bên ngoài**: không công cụ tĩnh nào biết producer sẽ gửi gì, nên phải kiểm lúc chạy (bài 15 giới thiệu cách làm). Bài này đưa vào **type hint** và công cụ đọc chúng, **mypy**, để chặn loại lỗi thứ nhất và nói rõ ranh giới với loại thứ hai. Đây là bài 12 của lộ trình **Python cơ bản qua Tới Nơi**. Bạn cần quen dataclass (bài 11) và `Decimal` (bài 02).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hàm <code>km_label(distance: Decimal)</code> khai báo tham số là <code>Decimal</code>. Nếu gọi <code>km_label(2.5)</code> với số thực, Python 3.14 có ném lỗi không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Annotation chỉ được ghi vào hàm, interpreter không dùng nó để kiểm đối số. Lời gọi trên trả về <code>"2.5 km"</code> bình thường. Người bắt được lỗi là mypy, chạy trước khi chương trình khởi động. Bài sẽ chạy đúng dòng này ở cả hai phía.</p></details>
</div>

## Type hint là lời khai, ai đọc nó mới có tác dụng

Type hint (còn gọi annotation) là phần ghi thêm vào chữ ký hàm và khai báo field: `distance_km: Decimal`, `-> Decimal`, `driver_id: str | None`. PEP 484 đặt ra cú pháp này; từ đó các PEP sau bổ sung dần. Trong lab (`domain.py`), hàm tính cước và `Trip` trông như sau (trích):

```python
type TripId = str  # mã chuyến, ví dụ "TN-0001"
type ServiceName = Literal["ride", "food", "parcel"]  # bài 13 đổi thành ServiceKind(StrEnum)


def calculate_fare(distance_km: Decimal, *, service: ServiceName = "ride") -> Decimal:
    ...


@dataclass
class Trip:
    trip_id: TripId
    customer_id: str
    driver_id: str | None
    service: ServiceName
    distance_km: Decimal
    status: str
    created_at: datetime
```

`Trip` ở đây là bản rút gọn để tập trung vào kiểu: `status` chỉ là `str` và các field công khai, không mang máy trạng thái 8 trạng thái của bài 11. Ba chi tiết đáng đọc. `str | None` nói thẳng rằng chuyến có thể chưa có tài xế, nên mọi nơi dùng `driver_id` phải xử lý trường hợp `None`. `Literal["ride", "food", "parcel"]` thu hẹp `str` thành đúng ba giá trị hợp lệ. Câu lệnh `type TripId = str` (cú pháp từ Python 3.12) đặt tên cho một kiểu để mã đọc theo nghiệp vụ hơn.

Điều cần nhớ nhất: **interpreter lưu annotation nhưng không kiểm**. Hai đường đi của cùng một mã nguồn:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Mã nguồn có type hint đi hai đường: Python khi chạy chỉ lưu annotation và không kiểm đối số, còn mypy trước khi chạy đọc annotation và báo lỗi theo từng dòng."><defs><marker id="py12-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="75" width="250" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="108" class="t-b">Mã nguồn</text><text x="36" y="136" class="t-small">def km_label(d: Decimal)</text><text x="36" y="156" class="t-small">km_label(2.5)</text><path d="M270 110 L330 70" class="f-none s-phys" stroke-width="2" marker-end="url(#py12-0-arrow)"/><path d="M270 140 L330 180" class="f-none s-phys" stroke-width="2" marker-end="url(#py12-0-arrow)"/><rect x="330" y="20" width="530" height="100" rx="10" class="f-warn-soft s-warn"/><text x="346" y="53" class="t-b">Python 3.14, lúc chạy</text><text x="346" y="81" class="t-small">lưu annotation vào hàm, KHÔNG kiểm kiểu đối số</text><text x="346" y="101" class="t-small">km_label(2.5) vẫn trả "2.5 km"</text><rect x="330" y="130" width="530" height="100" rx="10" class="f-pk-soft s-pk"/><text x="346" y="163" class="t-b">mypy --strict, trước khi chạy</text><text x="346" y="191" class="t-small">đọc annotation, đối chiếu với nơi gọi</text><text x="346" y="211" class="t-small">báo: float không phải Decimal [arg-type]</text></svg>
</div>
<figcaption>Hình 1. Cùng một mã nguồn đi hai đường. Interpreter chỉ ghi lại annotation; công cụ kiểm tra tĩnh đọc chúng để đối chiếu và báo lỗi trước khi chương trình chạy.</figcaption>
</figure>

Vì vậy type hint có ba người đọc: mypy hoặc pyright (kiểm tra tĩnh), trình soạn thảo (gợi ý, đổi tên an toàn) và đồng nghiệp (chữ ký nói rõ hợp đồng). Từ bài 02, ta đã dùng hint nhẹ nhàng; bài này là lúc cho một công cụ thực sự dùng chúng.

## Chạy mypy: từ lỗi thật đến `--strict`

**mypy** là trình kiểm tra kiểu tĩnh phổ biến nhất của Python. Nó không thuộc bản cài Python mà là gói riêng; lab khai báo nó là phụ thuộc phát triển (`[dependency-groups] dev = ["mypy>=2.4"]`) và `uv.lock` ghi phiên bản đã chạy (2.4.1). Lab có file `broken.py` chứa 9 lỗi cố ý, mỗi dòng lỗi kết thúc bằng chú thích «E:». Chạy từ gốc repository:

```bash
uv run --project examples/toi-noi-python/12-type-hints-mypy-protocol mypy examples/toi-noi-python/12-type-hints-mypy-protocol/broken.py --strict
```

Kết quả (trích; đã rút tiền tố đường dẫn `examples/toi-noi-python/12-type-hints-mypy-protocol/` ở đầu mỗi dòng và bỏ các dòng `note`):

```text
broken.py:16: error: Argument 1 to "km_label" has incompatible type "float"; expected "Decimal"  [arg-type]
broken.py:17: error: Item "None" of "str | None" has no attribute "upper"  [union-attr]
broken.py:18: error: Argument 2 to "add" of "Registry" has incompatible type "Driver"; expected "Trip"  [arg-type]
broken.py:19: error: Missing positional arguments "customer_id", "driver_id", "service", "distance_km", "status", "created_at" in call to "Trip"  [call-arg]
broken.py:20: error: Argument 4 to "Trip" has incompatible type "Literal['taxi']"; expected "Literal['ride', 'food', 'parcel']"  [arg-type]
broken.py:21: error: Value of type variable "T" of "cheapest" cannot be "Driver"  [type-var]
broken.py:30: error: Argument 1 to "show_all" has incompatible type "list[Trip]"; expected "list[object]"  [arg-type]
broken.py:32: error: TypedDict "BookingCreatedPayload" has no key "distance"  [typeddict-item]
broken.py:35: error: Function is missing a type annotation  [no-untyped-def]
Found 9 errors in 1 file (checked 1 source file)
```

Hai kiểu lỗi mở đầu bài đều có mặt: dòng 16 là `km_label(2.5)` với số thực thay vì `Decimal`, dòng 17 là `.upper()` trên `driver_id` có thể là `None`. Mỗi lỗi mang một mã trong ngoặc vuông (`arg-type`, `union-attr`...); mã này dùng để tra tài liệu hoặc tắt đúng một loại lỗi khi thật sự cần.

Cờ `--strict` là gói nhiều tùy chọn nghiêm ngặt: nó bật, trong số khác, `--disallow-untyped-defs` (mọi hàm phải có annotation), `--warn-return-any` và `--warn-unused-ignores`. Lỗi ở dòng 35 chỉ xuất hiện nhờ nó: chạy `mypy broken.py` không có `--strict` chỉ báo 8 lỗi, hàm `helper(x)` không annotation lọt qua. Với mã mới, ta nên bật `--strict` ngay từ đầu; kéo một dự án cũ lên strict từng bước thì khó hơn nhiều.

**pyright** của Microsoft là công cụ kiểm tra thứ hai, cũng đọc các annotation này (Pylance trong VS Code dựa trên nó). Hai công cụ có thể không đồng ý ở một số ca biên. Bài này chỉ chạy mypy; lab không cài pyright, nên ta không đưa số liệu nào về nó.

Một điều dễ hiểu lầm: mypy xanh không có nghĩa chương trình đúng. Nó chứng minh các lời gọi **khớp với lời khai kiểu**, không chứng minh lời khai khớp với dữ liệu thật. Các phần sau sẽ quay lại điểm này.

## Generics với cú pháp PEP 695

Giả sử Tới Nơi cần một kho trong bộ nhớ cho chuyến, và một kho cho tài xế. Viết hai lớp giống nhau thì thừa; viết một lớp chứa `object` thì mất thông tin kiểu. **Generics** giải quyết bằng một **tham số kiểu**. Từ Python 3.12, PEP 695 cho viết tham số ngay sau tên, không cần khai báo `TypeVar` riêng như trước:

```python
class Registry[T]:
    """Kho trong bộ nhớ chứa đúng một loại đối tượng, tra theo mã."""

    def __init__(self, label: str) -> None:
        self._label = label
        self._items: dict[str, T] = {}

    def add(self, key: str, item: T) -> None:
        if key in self._items:
            raise ValueError(f"{self._label} đã có: {key}")
        self._items[key] = item

    def get(self, key: str) -> T | None:
        return self._items.get(key)
```

Hàm cũng có tham số kiểu: `def first[T](items: Sequence[T]) -> T` nói rằng kiểu trả về đi theo kiểu phần tử của danh sách truyền vào; `first(["TN-0001", "TN-0002"])` có kiểu `str`. Với `Registry[Trip]`, mypy hiểu `get` trả `Trip | None` và `add` chỉ nhận `Trip`; dòng 18 của `broken.py` thử `add` một `Driver` và bị chặn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Một định nghĩa Registry[T] sinh ra Registry[Trip] và Registry[Driver]; thêm Driver vào Registry[Trip] bị mypy báo lỗi arg-type."><defs><marker id="py12-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py12-1-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="60" width="240" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="95" class="t-b">Registry[T]</text><text x="36" y="123" class="t-small">một định nghĩa duy nhất</text><text x="36" y="143" class="t-small">T là tham số kiểu</text><path d="M260 95 L320 60" class="f-none s-phys" stroke-width="2" marker-end="url(#py12-1-arrow)"/><path d="M260 125 L320 160" class="f-none s-phys" stroke-width="2" marker-end="url(#py12-1-arrow)"/><rect x="320" y="20" width="250" height="80" rx="10" class="f-pk-soft s-pk"/><text x="336" y="53" class="t-b">Registry[Trip]</text><text x="336" y="80" class="t-small">add/get chỉ nhận Trip</text><rect x="320" y="120" width="250" height="80" rx="10" class="f-pk-soft s-pk"/><text x="336" y="153" class="t-b">Registry[Driver]</text><text x="336" y="180" class="t-small">add/get chỉ nhận Driver</text><path d="M570 60 L630 95" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py12-1-warn)"/><rect x="630" y="50" width="230" height="110" rx="10" class="f-warn-soft s-warn"/><text x="646" y="82" class="t-b">add(Driver)</text><text x="646" y="108" class="t-small">vào Registry[Trip]</text><text x="646" y="128" class="t-small">mypy: [arg-type]</text></svg>
</div>
<figcaption>Hình 2. Generics giữ thông tin kiểu qua một cấu trúc dùng chung. Kiểu đóng vai trò hợp đồng: thêm sai loại đối tượng bị báo ngay trên dòng gọi.</figcaption>
</figure>

Có hai điểm tinh tế. Thứ nhất là **cận trên** `T: SupportsFare` (ở phần Protocol bên dưới): T phải thỏa một hợp đồng nhưng kết quả vẫn mang đúng kiểu T, không bị «nới» thành kiểu của hợp đồng. Thứ hai là **variance**. Dòng 30 của `broken.py` truyền `list[Trip]` vào tham số `list[object]` và bị từ chối, vì `list` đổi được: nếu cho phép, hàm kia có thể nhét một `str` vào danh sách chuyến. mypy gợi ý dùng `Sequence`, kiểu chỉ đọc nên là covariant (hợp biến) và chấp nhận danh sách của kiểu con. Quy tắc thực hành: **tham số nhận kiểu rộng và chỉ đọc (`Sequence`), kết quả trả kiểu cụ thể (`list`)**. Với cú pháp PEP 695, trình kiểm tra tự suy ra variance từ cách T được dùng, nên không cần khai `covariant=True` bằng tay.

## Protocol: khớp theo cấu trúc, không cần kế thừa

Muốn tìm chuyến rẻ nhất, ta cần phần tử có thuộc tính `fare`. Không muốn buộc `Trip` và `QuoteLine` (dòng báo giá) cùng kế thừa một lớp cha. Python có sẵn duck typing; **Protocol** (PEP 544) cho phép khai báo duck typing đó để mypy kiểm:

```python
class SupportsFare(Protocol):
    """Bất cứ thứ gì có thuộc tính `fare` đọc được kiểu Decimal."""

    @property
    def fare(self) -> Decimal: ...


def cheapest[T: SupportsFare](items: Sequence[T]) -> T:
    return min(items, key=lambda item: item.fare)


class QuoteLine(NamedTuple):
    label: str
    fare: Decimal
```

`Trip` có `fare` là property, `QuoteLine` có `fare` là field của NamedTuple; không lớp nào nhắc tới `SupportsFare`, nhưng cả hai đều thỏa vì có thành viên đúng tên và đúng kiểu. `Driver` không có `fare`, nên `cheapest([driver])` bị báo `type-var` ở dòng 21. Sự khác biệt so với lớp cha thường: hợp đồng nằm ở **nơi dùng**, người viết `Trip` không cần biết nó sẽ được dùng theo cách nào. Bài 14 đi tiếp: so Protocol với lớp trừu tượng `abc.ABC` và dùng nó để tiêm phụ thuộc.

## TypedDict: hình dạng dict từ JSON, và giới hạn của lời khai

Sự kiện Kafka đến dưới dạng dict. `TypedDict` (PEP 589) mô tả khóa nào có, kiểu giá trị là gì. Kèm cú pháp generics, ta khai báo được bì thư dùng chung cho mọi loại sự kiện:

```python
class BookingCreatedPayload(TypedDict):
    trip_id: TripId
    customer_id: str
    service: ServiceName
    distance_km: str  # Decimal đi qua JSON dưới dạng chuỗi (bài 09)
    note: NotRequired[str]


class Envelope[P](TypedDict):
    event_id: str
    event_type: str
    occurred_at: str
    version: int
    payload: P


def parse_booking_created(raw: str) -> Envelope[BookingCreatedPayload]:
    event: Envelope[BookingCreatedPayload] = json.loads(raw)
    return event
```

Viết `event["payload"]["distance"]` thay vì `distance_km` bị mypy bắt (dòng 32, kèm gợi ý «Did you mean "distance_km"?»). Nhưng nhìn kỹ hàm `parse_booking_created`: `json.loads` trả `Any`, và gán `Any` cho biến có kiểu là hợp lệ. Ta đã **khai** rằng chuỗi này là `Envelope[BookingCreatedPayload]`; không có dòng nào kiểm điều đó lúc chạy. Lab thử bằng một sự kiện thiếu `distance_km` (trích output của `main.py`):

```text
BookingCreated v1: TN-0001 5 km
parse không lỗi, dùng mới lỗi: KeyError 'distance_km'
```

Đó là lý do mypy xanh không thay được việc kiểm dữ liệu tại biên hệ thống. Thử không gán biến trung gian mà trả thẳng `json.loads(raw)`, `--strict` sẽ báo `no-any-return`: tín hiệu nhắc rằng mypy không tự biết nội dung. Cách làm thật là kiểm dữ liệu ngay khi nhận, bằng thư viện chuyên dụng; bài 15 dùng pydantic cho việc đó, còn `TypedDict` vẫn hữu ích cho phần mã **sau** bước kiểm.

## Annotation lười trong Python 3.14

Python 3.14 đổi cách đánh giá annotation (PEP 649, bổ sung bởi PEP 749). Trước đây annotation của hàm được tính ngay khi `def` chạy, nên tham chiếu tới tên chưa khai báo (kể cả chính lớp đang định nghĩa) gây `NameError`, và người ta phải đặt tên kiểu trong dấu nháy. Giờ annotation được **đánh giá lười**: biểu thức chỉ được tính khi có ai đọc `__annotations__`. Trong `domain.py`, `Trip.with_driver` nhận `Driver` (khai báo sau `Trip`) và trả `Trip` (chính lớp đang định nghĩa), không có dấu nháy nào:

```python
    def with_driver(self, driver: Driver) -> Trip:
        return replace(self, driver_id=driver.driver_id)
```

File `forward_ref_probe.py` của lab chứa đúng hai lớp này và chạy trên cả hai phiên bản:

```bash
uv run --python 3.14 --no-project python examples/toi-noi-python/12-type-hints-mypy-protocol/forward_ref_probe.py
uv run --python 3.13 --no-project python examples/toi-noi-python/12-type-hints-mypy-protocol/forward_ref_probe.py
```

Lệnh thứ hai cần Python 3.13; nếu máy chưa có, uv sẽ tải về nên cần mạng.

```text
Python 3.14: định nghĩa xong, không lỗi
NameError: name 'Driver' is not defined
```

(Dòng thứ hai là dòng cuối của traceback khi chạy bằng Python 3.13.) Lười không có nghĩa tên không cần tồn tại: khi đọc mà tên vẫn chưa có thì `NameError` xuất hiện tại điểm đọc. Module `annotationlib` cho chọn dạng đọc. `main.py` dựng một hàm tham chiếu tên `Voucher` chưa từng khai báo bằng `exec` trên chuỗi cố định (để mypy không phàn nàn về chính nó), và kết quả:

```text
def apply_voucher(voucher: Voucher) chạy qua; đọc __annotations__ -> NameError: name 'Voucher' is not defined
Format.STRING không cần tên tồn tại: {'voucher': 'Voucher', 'return': 'None'}
khai báo class Voucher sau đó, Format.VALUE giải được: voucher -> Voucher, return -> None
```

Dòng cuối cho thấy lười còn có nghĩa là tên có thể được khai báo **sau** `def`: miễn là trước lúc đọc, `Format.VALUE` giải được ra lớp thật. `Format.STRING` trả các biểu thức dưới dạng chuỗi nên không bao giờ lỗi; `Format.FORWARDREF` trả đối tượng thật nếu tên đã có và `ForwardRef` nếu chưa. Đây là công cụ cho thư viện (dataclass, pydantic ở bài 15) đọc annotation an toàn; mã nghiệp vụ hiếm khi cần gọi trực tiếp. Còn `from __future__ import annotations` (PEP 563), cách cũ biến mọi annotation thành chuỗi, vẫn chạy như trước ở 3.14; PEP 749 vạch kế hoạch ngừng dần sau khi Python 3.13 hết vòng đời hỗ trợ. Mã mới nhắm tới 3.14 không cần dòng đó.

## Chạy và kiểm chứng

Từ gốc repository (lần đầu uv tải mypy nên cần mạng):

```bash
uv run --project examples/toi-noi-python/12-type-hints-mypy-protocol examples/toi-noi-python/12-type-hints-mypy-protocol/main.py
```

```text
== 1. Kiểu cơ bản và X | None ==
TN-0001: 5 km ride -> 25500 VND, driver_id=None
TN-0002: 1.5 km food -> 12000 VND, driver_id=None
with_driver -> DR-001; bản gốc vẫn None
== 2. Generics ==
first(['TN-0001', 'TN-0002']) = TN-0001
Registry[Trip] có 2 chuyến; get('TN-0009') = None
== 3. Protocol SupportsFare ==
cheapest(Trip) = TN-0002 (12000 VND)
cheapest(QuoteLine) = 1,5 km; total_fare(trips) = 37500
== 4. TypedDict qua JSON ==
BookingCreated v1: TN-0001 5 km
parse không lỗi, dùng mới lỗi: KeyError 'distance_km'
== 5. Annotation lười (PEP 649) ==
Trip.with_driver (Trip trả về chính nó, Driver khai báo sau): {'driver': 'Driver', 'return': 'Trip'}
def apply_voucher(voucher: Voucher) chạy qua; đọc __annotations__ -> NameError: name 'Voucher' is not defined
Format.STRING không cần tên tồn tại: {'voucher': 'Voucher', 'return': 'None'}
khai báo class Voucher sau đó, Format.VALUE giải được: voucher -> Voucher, return -> None
```

Mã trong `main.py` và `domain.py` phải qua `--strict` mà không báo gì:

```bash
uv run --project examples/toi-noi-python/12-type-hints-mypy-protocol mypy examples/toi-noi-python/12-type-hints-mypy-protocol/main.py --strict
```

```text
Success: no issues found in 1 source file
```

Lệnh chỉ nêu `main.py`, nhưng mypy đi theo `import domain` và kiểm cả `domain.py`; con số «1 source file» đếm file ta chỉ định, không đếm file được kéo vào qua import.

Chế độ `--verify` kiểm các kết quả trên (cước, `Registry`, `cheapest`, TypedDict thiếu khóa, annotation lười đọc bằng `Format.VALUE`, `km_label(2.5)` vẫn chạy được) và còn chạy mypy thật hai lần: `main.py` phải xanh, `broken.py` phải báo đúng 9 lỗi tại đúng các dòng đánh dấu:

```bash
uv run --project examples/toi-noi-python/12-type-hints-mypy-protocol examples/toi-noi-python/12-type-hints-mypy-protocol/main.py --verify
```

```text
OK 21/21
```

Lab chạy bằng CPython 3.14.7 và mypy 2.4.1.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `Library stubs not installed for "yaml"  [import-untyped]` | Thư viện chưa có kiểu: cài gói `types-...` mypy gợi ý, hoặc tắt riêng module đó trong `[tool.mypy]`, đừng tắt toàn cục |
| `Returning Any from function declared to return "str"  [no-any-return]` | Giá trị đến từ `json.loads` hoặc hàm không annotation: kiểm tra thật, hoặc gán vào biến có kiểu rồi mới trả |
| mypy xanh nhưng chạy vẫn `KeyError` | `TypedDict` và `Any` là lời khai: dữ liệu ngoài hệ thống phải được kiểm lúc chạy (bài 15) |
| `Incompatible return value type (got "str \| None", expected "str")` | Quên xử lý `None`: kiểm `is None` trước khi trả, hoặc đổi kiểu trả về cho đúng sự thật |
| `"list" is invariant` | Tham số chỉ đọc nên khai `Sequence[...]` thay vì `list[...]` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>mypy chỉ kiểm những chỗ có annotation và chỉ kiểm tính nhất quán giữa các lời khai; nó không thấy dữ liệu thật, không thay unit test và không bắt được lỗi nghiệp vụ như tính sai cước. <code>Any</code>, <code>cast</code> và <code># type: ignore</code> đều là lối thoát khỏi kiểm tra, nên mỗi chỗ dùng là một chỗ phải tự chịu trách nhiệm. Ví dụ ở đây dùng <code>Literal</code> và <code>str</code> cho dịch vụ, trạng thái; bài 13 thay bằng <code>StrEnum</code>. Lab chỉ có hai file nguồn nhỏ; dự án thật có thư viện bên thứ ba thiếu kiểu và cần cấu hình mypy trong <code>pyproject.toml</code> (bài 20). Phiên bản mypy trong lab là bản đã chạy; bản khác có thể đổi nội dung thông báo lỗi.</p>
</div>

**Bài tập 1:** trong `domain.py`, đổi `def first[T](items: Sequence[T]) -> T` thành `def first[T](items: list[T]) -> T`. Chạy mypy cho `main.py`: có gì thay đổi, và đó là lỗi của `first` hay của nơi gọi?

**Bài tập 2:** viết hàm `quote_summary(item: SupportsFare) -> str` trả chuỗi `"<fare> VND"`. Gọi nó với một `Trip` và với một `QuoteLine`; vì sao không cần sửa hai lớp này?

<details><summary>Xem lời giải</summary><p>(1) Không có gì thay đổi với các lời gọi hiện có: <code>main.py</code> chỉ gọi <code>first</code> với list. Chỉ khi truyền vào một tuple hoặc <code>Sequence</code> khác, mypy mới báo <code>arg-type</code> tại nơi gọi, vì <code>list[T]</code> là yêu cầu hẹp hơn mức hàm thực sự cần: nó chỉ đọc <code>items[0]</code>. Chữ ký rộng hơn (<code>Sequence[T]</code>) mới là mô tả đúng. (2) <code>def quote_summary(item: SupportsFare) -&gt; str: return f"{item.fare} VND"</code>. Cả <code>Trip</code> (property <code>fare</code>) lẫn <code>QuoteLine</code> (field <code>fare</code>) đều có thành viên <code>fare</code> kiểu <code>Decimal</code>, nên khớp theo cấu trúc mà không phải kế thừa gì; <code>Driver</code> không có <code>fare</code> nên bị từ chối.</p></details>

Bài trước: [bài 11](/articles/python-class-dataclass-property-encapsulation). Bài tiếp: [bài 13](/articles/python-frozen-dataclass-enum-immutability).

## Tài liệu gốc

- [typing: Support for type hints (Python 3.14)](https://docs.python.org/3.14/library/typing.html).
- [annotationlib: Functionality for introspecting annotations](https://docs.python.org/3.14/library/annotationlib.html).
- [PEP 695: Type Parameter Syntax](https://peps.python.org/pep-0695/) và [PEP 749: Implementing PEP 649](https://peps.python.org/pep-0749/).
- [PEP 544: Protocols](https://peps.python.org/pep-0544/) và [PEP 589: TypedDict](https://peps.python.org/pep-0589/).
- [mypy: Command line options (`--strict`)](https://mypy.readthedocs.io/en/stable/command_line.html).
- [Pyright documentation](https://microsoft.github.io/pyright/).
