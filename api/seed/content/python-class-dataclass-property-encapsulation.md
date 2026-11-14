---
slug: python-class-dataclass-property-encapsulation
title: "Class, dataclass, property và encapsulation: bảo vệ trạng thái chuyến"
summary: "Bài 11: dựng Trip bằng class thường rồi @dataclass, @property chỉ đọc, quy ước _private, __slots__ có đo thật và máy trạng thái 8 trạng thái với advance() từ chối chuyển trạng thái không hợp lệ. Có kết quả chạy thật và lab chạy được."
category: python-beginner
level: beginner
tags: [python, toi-noi, dataclass, encapsulation]
language: vi
---

Một service phân tích của [Tới Nơi](/articles/du-an-mau-toi-noi) nhận chuyến dưới dạng dict, đúng kiểu dữ liệu JSON ta đọc ở bài 09: `{"trip_id": "TN-0001", "status": "requested", ...}`. Một dòng `trip["status"] = "completed"` chạy thành công trên chuyến vừa được yêu cầu: chưa tìm tài xế, chưa đón khách, vẫn «hoàn thành». Dict không biết gì về nghiệp vụ nên không từ chối. Chuyến sai trạng thái đi tiếp vào báo cáo, vào bước tính tiền, và đến lúc ai đó thấy số liệu lạ thì không còn biết dòng nào gây ra lỗi. Bài này chuyển chuyến từ «túi dữ liệu» thành một **đối tượng tự bảo vệ trạng thái của mình**.

Đây là bài 11 của lộ trình **Python cơ bản qua Tới Nơi**, mở đầu phần 02. Bạn cần quen hàm (bài 03), dict và tuple (bài 05), exception (bài 08) và cách chạy kiểm chứng (bài 10).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ta viết <code>self._status</code> với dấu gạch dưới đầu tên. Dòng <code>trip._status = "completed"</code> ở bên ngoài lớp có bị Python chặn không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Dấu gạch dưới chỉ là quy ước đọc code («đừng đụng từ bên ngoài»), interpreter không kiểm tra. Bài sẽ chạy đúng dòng này và cho thấy nó có hiệu lực. Bảo vệ thật sự đến từ chỗ khác: property không có setter, và một phương thức duy nhất được phép đổi trạng thái.</p></details>
</div>

## Từ dict đến class: `__init__` và `self`

**Class** gom dữ liệu và hành vi vào một kiểu. Khi gọi `TripPlain("TN-0001", ...)`, Python tạo đối tượng rồi chạy `__init__` để gán thuộc tính. `self` là chính đối tượng đang được tạo (tên `self` là quy ước, không phải từ khoá); mỗi dòng `self.x = ...` tạo một thuộc tính riêng của đối tượng đó. `trip.advance(s)` thực chất là `Trip.advance(trip, s)`, và `trip` được truyền vào tham số `self`.

```python
class TripPlain:
    """Phiên bản class thường. Mọi thuộc tính công khai, ai cũng gán được."""

    def __init__(self, trip_id: str, customer_id: str, service: str, distance_km: Decimal):
        self.trip_id = trip_id
        self.customer_id = customer_id
        self.service = service
        self.distance_km = distance_km
        self.status = "requested"
        self.driver_id: str | None = None
```

Đó là code trong `main.py` của lab 11. So với dict, ta được tên thuộc tính cố định (gõ sai `trip.stauts` ra `AttributeError` ngay khi đọc) và một nơi để gắn hành vi. Nhưng `status` vẫn là thuộc tính công khai: `a.status = "completed"` vẫn chạy. Class thường còn có hai hành vi gây bất ngờ: `__eq__` mặc định so danh tính (`is`), nên hai chuyến cùng nội dung không bằng nhau, và `repr` mặc định chỉ in tên lớp với địa chỉ chứ không in nội dung.

Hai hạn chế đó dẫn đến công cụ tiếp theo.

## `@dataclass`: bỏ phần lặp, giữ ý nghĩa

Ba dòng `self.x = x` lặp theo số field, cộng `__repr__` và `__eq__` viết tay, là phần dễ sai nhất nhưng không chứa nghiệp vụ nào. Decorator `@dataclass` (trong module `dataclasses`, PEP 557) đọc các **annotation** của lớp và sinh `__init__`, `__repr__`, `__eq__` thay ta. Phiên bản đầy đủ của `Trip` có thêm vài điều ta cần để ý:

```python
@dataclass
class Trip:
    """Chuyến xe. Bên ngoài chỉ đọc status/driver_id/history; đổi trạng thái qua advance()."""

    # Thứ tự trường theo từ điển miền: trip_id, customer_id, driver_id, service, distance_km,
    # status, created_at. Trường init=False không nhận từ constructor: chuyến luôn bắt đầu
    # ở trạng thái đầu tiên, chưa có tài xế. Dấu _ đầu tên là quy ước «riêng»; Python không
    # chặn truy cập.
    trip_id: str
    customer_id: str
    _driver_id: str | None = field(default=None, init=False)
    service: str
    distance_km: Decimal
    _status: str = field(default="requested", init=False)
    created_at: datetime  # aware, UTC
    _history: list[str] = field(default_factory=lambda: ["requested"], init=False, repr=False)

    def __post_init__(self) -> None:
        if not self.trip_id.startswith("TN-"):
            raise ValueError(f"mã chuyến phải có dạng TN-0001: {self.trip_id!r}")
        if self.created_at.tzinfo is None:
            raise ValueError("created_at phải là datetime aware (UTC)")
        calculate_fare(self.distance_km, service=self.service)  # từ chối khoảng cách/dịch vụ sai
```

Đây là bản đã **đóng gói**: từ điển miền của Tới Nơi mô tả chuyến với bảy trường (`trip_id`, `customer_id`, `driver_id`, `service`, `distance_km`, `status`, `created_at`), và lớp này giữ đủ bảy, đúng thứ tự đó. Khác biệt là `driver_id` và `status` được truy cập qua property nên constructor không nhận hai trường này: gọi `Trip(trip_id, customer_id, service, distance_km, created_at)` với **năm** tham số, chuyến luôn ra đời ở `requested` và chưa có tài xế. Các bài sau dùng lại cùng tập trường (bài 16 so khớp mẫu trên chúng), dù lab của mỗi bài có thể khai báo `Trip` gọn hơn.

Năm điều đáng nhớ:

- **`field(default_factory=...)`** cho giá trị mặc định mutable. `_history: list[str] = []` sẽ bị Python từ chối ngay lúc khai báo lớp bằng `ValueError: mutable default <class 'list'> ... use default_factory`; đây là chính cái bẫy «list mặc định dùng chung» của bài 03, nhưng dataclass chặn sớm thay vì để bạn tự phát hiện.
- **`init=False`** loại field khỏi tham số của constructor: người gọi không thể tạo chuyến đã ở trạng thái `completed`. **`repr=False`** bỏ field khỏi `repr`; ta ẩn lịch sử cho gọn.
- **`__post_init__`** chạy ngay sau `__init__` do dataclass sinh. Đó là chỗ từ chối dữ liệu sai lúc tạo: mã chuyến, thời điểm phải aware (bài 04), khoảng cách và dịch vụ hợp lệ (ta gọi `calculate_fare` một lần để dùng lại đúng quy tắc miền, không viết hai lần).
- **Thứ tự field.** Trong số các field nhận qua constructor, field không có mặc định phải đứng trước field có mặc định, nếu không Python báo `TypeError: non-default argument ... follows default argument`.
- **Annotation là cách dataclass nhận ra field, không phải ép kiểu.** `str` hay `Decimal` ở đây không bị kiểm tra lúc chạy; bài 12 nói về công cụ kiểm tra kiểu.

Hành vi `__eq__` cũng thay đổi: hai `Trip` bằng nhau khi **mọi field** bằng nhau, kể cả `_status`. Hệ quả đi kèm là dataclass có `eq=True` mà không `frozen` sẽ đặt `__hash__` thành `None`, nghĩa là `hash(trip)` ném `TypeError: unhashable type: 'Trip'`. Điều này hợp lý: đối tượng đổi được thì hash cũng sẽ đổi, mà phần tử của `set` và khoá của `dict` không được phép đổi hash. Vì vậy ta dùng `trip.trip_id` làm khoá trong `dict[str, Trip]`, giống dict tài xế ở bài 05, và bài 13 giải thích cách tạo đối tượng hash được bằng `frozen=True`.

## `@property` và «riêng theo quy ước»

`@property` biến một phương thức thành thuộc tính đọc được, gọi bằng `trip.fare` chứ không phải `trip.fare()`. Decorator này phục vụ hai việc trong `Trip`:

```python
    @property
    def status(self) -> str:
        return self._status

    @property
    def fare(self) -> Decimal:
        # Giá trị suy ra: tính theo yêu cầu nên không bao giờ lệch khỏi distance_km/service.
        return calculate_fare(self.distance_km, service=self.service)
```

`fare` là **giá trị suy ra**: không lưu, mỗi lần đọc tính lại từ `distance_km` và `service`, nên không thể lệch khỏi chúng. `status` là **lối truy cập chỉ đọc** tới `_status`. Property không khai báo setter thì phép gán bị từ chối. `driver_id` làm tương tự, còn `history` trả `tuple(self._history)`, một bản sao bất biến, để bên ngoài không cầm được list nội bộ mà sửa.

Còn dấu gạch dưới? Tên bắt đầu bằng `_` là quy ước PEP 8 cho «phần bên trong, đừng đụng từ ngoài». Python không có `private` như C# hay Java: nếu bạn viết `trip._status = "completed"` thì nó chạy. Tên bắt đầu bằng hai gạch dưới (`self.__x`) cũng không khoá gì; Python chỉ đổi tên thuộc tính thành `_TênLớp__x` để tránh trùng tên khi kế thừa, và ai biết tên đã đổi vẫn truy cập được. Tức là encapsulation trong Python là **thoả thuận giữa những người viết code**, và được cấu trúc code hỗ trợ: không có setter, chỉ có một cửa vào để đổi trạng thái. Nếu bạn quen `private set` của C# ở [bài C# về class và property](/articles/csharp-class-property-encapsulation), đây là phần khác biệt đáng chú ý nhất.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 310" role="img" aria-label="Đọc status đi qua property, đổi trạng thái đi qua advance có kiểm tra rồi mới ghi vào _status; gán thẳng status bị từ chối vì không có setter; gán thẳng _status không bị gì chặn."><defs><marker id="py11-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py11-0-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="30" width="250" height="50" rx="8" class="f-pk-soft s-pk"/><text x="34" y="60" class="t-mono">đọc trip.status</text><rect x="20" y="100" width="250" height="50" rx="8" class="f-pk-soft s-pk"/><text x="34" y="130" class="t-mono">trip.advance("accepted", ...)</text><rect x="20" y="190" width="250" height="50" rx="8" class="f-pk-soft s-pk"/><text x="34" y="220" class="t-mono">trip.status = "completed"</text><rect x="20" y="255" width="250" height="40" rx="8" class="f-pk-soft s-pk"/><text x="34" y="280" class="t-mono">trip._status = "completed"</text><path d="M270 55 L330 55" class="f-none s-phys" stroke-width="2" marker-end="url(#py11-0-arrow)"/><path d="M270 125 L330 125" class="f-none s-phys" stroke-width="2" marker-end="url(#py11-0-arrow)"/><rect x="330" y="30" width="250" height="50" rx="8" class="f-phys-soft s-phys"/><text x="344" y="60" class="t-small">@property status: chỉ đọc</text><rect x="330" y="100" width="250" height="50" rx="8" class="f-phys-soft s-phys"/><text x="344" y="130" class="t-small">kiểm tra hết, rồi mới sửa</text><path d="M580 55 L650 70" class="f-none s-phys" stroke-width="2" marker-end="url(#py11-0-arrow)"/><path d="M580 125 L650 110" class="f-none s-phys" stroke-width="2" marker-end="url(#py11-0-arrow)"/><rect x="650" y="30" width="210" height="120" rx="8" class="f-phys-soft s-phys"/><text x="664" y="62" class="t-b">Trạng thái trong</text><text x="664" y="86" class="t-mono">_status</text><text x="664" y="106" class="t-mono">_history</text><text x="664" y="126" class="t-mono">_driver_id</text><path d="M270 215 L330 215" class="f-none s-warn" stroke-width="2" marker-end="url(#py11-0-warn)"/><rect x="330" y="190" width="250" height="50" rx="8" class="f-warn-soft s-warn"/><text x="344" y="220" class="t-small">AttributeError: không có setter</text><path d="M270 275 L755 275 L755 150" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py11-0-warn)"/><text x="345" y="268" class="t-small">không có gì chặn: chỉ là quy ước</text></svg>
</div>
<figcaption>Hình 1. Ba lối vào trạng thái của chuyến. Hai lối đầu có kiểm tra; lối gán thẳng vào thuộc tính có gạch dưới vẫn đi qua được nếu ai đó cố tình.</figcaption>
</figure>

Hình 1 tóm lại: nếu mọi code trong dự án tuân thủ quy ước, mọi thay đổi trạng thái đều đi qua `advance()`. Điều ta có thể làm là biến việc vi phạm thành chuyện **lộ liễu**: người review thấy `_status =` ngoài thân lớp `Trip` là biết có vấn đề, và một lệnh `grep "\._status ="` đủ để tìm (bỏ các dòng nằm trong chính lớp `Trip`, như dòng ghi trong `advance()`). Test ở bài 10 là lớp bảo vệ còn lại.

## Máy trạng thái 8 trạng thái trong `advance()`

Chuyến Tới Nơi đi qua bảy trạng thái theo thứ tự (`requested`, `searching`, `accepted`, `arriving`, `picked_up`, `in_transit`, `completed`) và có thể bị huỷ (`cancelled`) từ bốn trạng thái đầu trước khi đón khách. Ta biểu diễn bằng một dict: khoá là trạng thái hiện tại, giá trị là các trạng thái được phép đi tới.

```python
ALLOWED_TRANSITIONS: dict[str, tuple[str, ...]] = {
    "requested": ("searching", "cancelled"),
    "searching": ("accepted", "cancelled"),
    "accepted": ("arriving", "cancelled"),
    "arriving": ("picked_up", "cancelled"),
    "picked_up": ("in_transit",),
    "in_transit": ("completed",),
    "completed": (),
    "cancelled": (),
}
```

Trạng thái ở đây là chuỗi thường. Bài 13 sẽ thay bằng `Enum`, và lý do hiện ra ngay trong code: gõ nhầm `"acepted"` chỉ bị bắt lúc chạy, bằng dòng kiểm tra tên trạng thái trong `advance()`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Bảy trạng thái nối tiếp từ requested đến completed; bốn trạng thái đầu đều có mũi tên đứt nét xuống cancelled; completed và cancelled là trạng thái cuối."><defs><marker id="py11-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py11-1-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="18" y="40" width="100" height="70" rx="8" class="f-phys-soft s-phys"/><text x="28" y="80" class="t-mono">requested</text><rect x="142" y="40" width="100" height="70" rx="8" class="f-phys-soft s-phys"/><text x="152" y="80" class="t-mono">searching</text><rect x="266" y="40" width="100" height="70" rx="8" class="f-phys-soft s-phys"/><text x="276" y="80" class="t-mono">accepted</text><rect x="390" y="40" width="100" height="70" rx="8" class="f-phys-soft s-phys"/><text x="400" y="80" class="t-mono">arriving</text><rect x="514" y="40" width="100" height="70" rx="8" class="f-phys-soft s-phys"/><text x="524" y="80" class="t-mono">picked_up</text><rect x="638" y="40" width="100" height="70" rx="8" class="f-phys-soft s-phys"/><text x="648" y="80" class="t-mono">in_transit</text><rect x="762" y="40" width="100" height="70" rx="8" class="f-phys-soft s-phys"/><text x="772" y="80" class="t-mono">completed</text><path d="M118 75 L142 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py11-1-arrow)"/><path d="M242 75 L266 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py11-1-arrow)"/><path d="M366 75 L390 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py11-1-arrow)"/><path d="M490 75 L514 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py11-1-arrow)"/><path d="M614 75 L638 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py11-1-arrow)"/><path d="M738 75 L762 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py11-1-arrow)"/><path d="M68 110 L68 170" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py11-1-warn)"/><path d="M192 110 L192 170" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py11-1-warn)"/><path d="M316 110 L316 170" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py11-1-warn)"/><path d="M440 110 L440 170" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py11-1-warn)"/><rect x="18" y="170" width="496" height="56" rx="8" class="f-warn-soft s-warn"/><text x="34" y="203" class="t-small">cancelled (từ requested, searching, accepted, arriving)</text><rect x="560" y="170" width="302" height="56" rx="8" class="f-bg s-muted dash"/><text x="574" y="194" class="t-small">completed, cancelled: trạng thái cuối</text><text x="574" y="214" class="t-mono">ALLOWED_TRANSITIONS[...] == ()</text></svg>
</div>
<figcaption>Hình 2. Tám trạng thái và các chuyển hợp lệ. Từ khi đón khách (picked_up) trở đi không còn đường huỷ.</figcaption>
</figure>

Phương thức `advance` là **cửa duy nhất** được sửa `_status`. Nguyên tắc chính là **kiểm tra hết trước, sửa sau**: nếu bất kỳ kiểm tra nào thất bại, ta ném `ValueError` khi đối tượng chưa bị đụng tới, nên không bao giờ có trạng thái dở dang.

```python
    def advance(self, new_status: str, *, driver_id: str | None = None) -> None:
        """Chuyển trạng thái. Kiểm tra HẾT trước khi sửa: lỗi thì đối tượng không đổi."""
        if new_status not in ALLOWED_TRANSITIONS:
            raise ValueError(f"trạng thái không tồn tại: {new_status!r}")
        if new_status not in ALLOWED_TRANSITIONS[self._status]:
            raise ValueError(f"chuyến {self.trip_id}: không thể chuyển {self._status} -> {new_status}")
        if new_status == "accepted":
            if not driver_id or not driver_id.strip():
                raise ValueError(f"chuyến {self.trip_id}: nhận chuyến phải có driver_id")
        elif driver_id is not None:
            raise ValueError(f"driver_id chỉ đi kèm bước accepted, không đi kèm {new_status}")
        # Qua hết kiểm tra mới sửa.
        if new_status == "accepted":
            self._driver_id = driver_id
        self._status = new_status
        self._history.append(new_status)
```

Ba lớp kiểm tra tương ứng ba loại lỗi: tên trạng thái không tồn tại, chuyển không có trong dict, và dữ liệu đi kèm không khớp (nhận chuyến mà không nói ai nhận). Điều kiện cuối giữ một **bất biến** của mô hình: từ `accepted` trở đi chuyến luôn có `driver_id`. Bất biến ấy chỉ giữ được vì `driver_id` không có setter; nếu ai gán `trip.driver_id = None` được, mọi kiểm tra trên mất ý nghĩa.

Việc chuyển sang `cancelled` được ghi nhận ở đây như một bước thường, giống các bước khác; hồ sơ thật còn cần lý do huỷ và ai huỷ (khách, tài xế hay hệ thống), mà bài này cố ý chưa mô hình hoá.

## `__slots__`: đo trước khi tin

Mặc định, mỗi đối tượng có chỗ lưu thuộc tính riêng dạng động, nên bạn thêm được `trip.note = "..."` bất cứ lúc nào, kể cả gõ nhầm tên thuộc tính mà không bị bắt. Khai báo `__slots__` (hoặc `@dataclass(slots=True)`) cố định danh sách thuộc tính, bỏ chỗ lưu động đó. Ta so hai dataclass cùng năm field, một có `slots=True`:

```python
@dataclass(slots=True)
class TripRowSlots:
    trip_id: str
    customer_id: str
    service: str
    distance_km: Decimal
    status: str
```

`TripRow` giống hệt nhưng chỉ có `@dataclass`. Hàm `bytes_per_instance` tạo 50.000 đối tượng, đo phần bộ nhớ cấp thêm bằng `tracemalloc` (trừ phần con trỏ của list chứa); các giá trị field là cùng một object, nên con số chỉ phản ánh khung đối tượng. Kết quả (lần chạy ở phần sau): **112 byte** cho `TripRow`, **72 byte** cho `TripRowSlots`, ít hơn khoảng 36%. Trên 2 triệu chuyến mỗi ngày, 40 byte chênh lệch là chừng 80 MB, nhưng chỉ nếu bạn nạp cả ngày vào bộ nhớ cùng lúc; bài 06 đã cho thấy ta thường tránh điều đó bằng generator.

Hai lưu ý khi đo. Thứ nhất, `sys.getsizeof` không phải thước đo phù hợp ở đây: lab in `48` cho bản không slots và `72` cho bản slots, tức là kết quả **ngược** với thực tế, vì với bản không slots nó không tính dữ liệu thuộc tính (cách CPython lưu chúng thay đổi theo phiên bản). `tracemalloc` đo cái chương trình thật sự cấp phát. Thứ hai, con số này đo trên một cấu hình cụ thể (Python 3.14.7, macOS arm64); đổi phiên bản hay nền tảng thì chênh lệch đổi theo, nên lab chỉ kiểm «có slots thì nhỏ hơn».

Cái giá của slots: không thêm được thuộc tính ngoài danh sách (`AttributeError: ... no __dict__ for setting new attributes`), kế thừa phức tạp hơn, và một số tính năng như `weakref` phải khai báo riêng. Slots **không** thay thế encapsulation: nó ngăn thuộc tính lạ, không ngăn gán vào thuộc tính đã khai báo. Với `Trip` chỉ vài nghìn đối tượng trong bộ nhớ, không cần slots; hãy nghĩ đến nó khi có tới hàng triệu đối tượng nhỏ.

## Chạy và kiểm chứng

Từ thư mục gốc repository (không cần mạng, không thư viện ngoài):

```bash
uv run --project examples/toi-noi-python/11-class-dataclass-property examples/toi-noi-python/11-class-dataclass-property/main.py
```

Output (một lần chạy thật, Python 3.14.7):

```text
== 1. class thường ==
a == b: False (== mặc định so danh tính `is`, không so nội dung)
a.status sau khi gán thẳng: completed | a.driver_id: None

== 2. dataclass ==
Trip(trip_id='TN-0001', customer_id='CU-001', _driver_id=None, service='ride', distance_km=Decimal('2.5'), _status='requested', created_at=datetime.datetime(2026, 11, 6, 1, 30, tzinfo=datetime.timezone.utc))
t1 == t2: True
hash(t1): TypeError: unhashable type: 'Trip'
list mặc định: ValueError: mutable default <class 'list'> for field tags is not allowed: use default_factory

== 3. property ==
fare: 14250 | status: requested | driver_id: None
t1.fare = 1: AttributeError: property 'fare' of 'Trip' object has no setter
t1.status = 'completed': AttributeError: property 'status' of 'Trip' object has no setter
sau t1._status = 'completed': status = completed (quy ước, không phải khóa)

== 4. máy trạng thái ==
history: requested -> searching -> accepted -> arriving -> picked_up -> in_transit -> completed
requested -> completed: ValueError: chuyến TN-0003: không thể chuyển requested -> completed
trạng thái sau lỗi: requested
accepted không driver: ValueError: chuyến TN-0003: nhận chuyến phải có driver_id
trạng thái sau lỗi: searching | driver_id: None
trạng thái lạ: ValueError: trạng thái không tồn tại: 'flying'

== 5. __slots__ ==
TripRow       : 112 byte/đối tượng
TripRowSlots  : 72 byte/đối tượng
tiết kiệm     : 36%
sys.getsizeof: 48 (không slots) so với 72 (slots)
thêm thuộc tính lạ: AttributeError: 'TripRowSlots' object has no attribute 'note' and no __dict__ for setting new attributes
```

Cách đọc. Phần 1 cho thấy lỗ hổng của class thường: chuyến nhảy thẳng sang `completed` mà không ai hỏi. Phần 2: `==` của dataclass so nội dung, `hash` bị từ chối, list mặc định bị chặn từ lúc khai báo. Phần 3: gán vào property thì lỗi, nhưng `t1._status = 'completed'` chạy được, đúng như đáp án câu hỏi đầu bài. Phần 4: `advance` từ chối bước chuyển sai và việc nhận chuyến thiếu tài xế, còn đối tượng giữ nguyên. Dòng `trạng thái sau lỗi: searching` không mâu thuẫn: bước `searching` trước đó hợp lệ, chỉ bước `accepted` thiếu `driver_id` bị từ chối.

Chế độ `--verify` so các kết quả này (cùng cước 0, 2, 2,5, 5 km là 12.000, 12.000, 14.250, 25.500; mọi chuyển hợp lệ; bốn trạng thái huỷ được; hai trạng thái cuối; các test case từ chối ở `__post_init__`) với giá trị kỳ vọng:

```bash
uv run --project examples/toi-noi-python/11-class-dataclass-property examples/toi-noi-python/11-class-dataclass-property/main.py --verify
```

```text
OK 36/36
```

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `ValueError: mutable default <class 'list'> ... use default_factory` | List, dict, set làm giá trị mặc định của field: dùng `field(default_factory=list)` |
| `TypeError: unhashable type: 'Trip'` khi bỏ vào `set` hoặc làm khoá dict | Dataclass có `eq` mà không `frozen` thì không hash được; dùng `trip.trip_id` làm khoá |
| `AttributeError: property '...' has no setter` | Đúng thiết kế; đổi trạng thái qua `advance()`, hoặc thêm setter có kiểm tra nếu thật sự cần |
| `TypeError: non-default argument 'b' follows default argument 'a'` | Trong các field nhận qua constructor, field không mặc định phải đứng trước field có mặc định (hoặc dùng `kw_only=True`) |
| Trạng thái chuyến «nhảy» dù đã có `advance()` | Tìm `_status =` ngoài thân lớp `Trip`; ai đó gán thẳng thuộc tính gạch dưới |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Mọi bảo vệ ở đây nằm trong bộ nhớ của một process. Hai request ở hai server có thể nạp cùng một chuyến <code>requested</code>, mỗi bên gọi <code>advance("searching")</code> thành công trên bản sao riêng của nó; encapsulation không phải khoá phân tán, và <code>advance()</code> cũng không an toàn khi nhiều thread cùng sửa một object. Trạng thái thật phải đi qua database với kiểm tra lúc ghi (bài 25 nói về optimistic concurrency). Mô hình 8 trạng thái là bản học tập: chưa có lý do huỷ, chưa có thời điểm của từng bước, và <code>created_at</code> được truyền vào thay vì đọc đồng hồ để lab chạy lặp lại được. Việc dùng đồng hồ được inject là chủ đề của bài 14.</p>
</div>

**Bài tập 1:** `trip.history.append("completed")` và `trip._history.append("completed")` khác nhau thế nào về hậu quả?

**Bài tập 2:** viết thuộc tính hoặc phương thức `can_advance(new_status) -> bool` trả `True` nếu chuyển được mà không ném lỗi. Có nên dùng nó rồi gọi `advance` hai lần (kiểm tra, rồi chuyển) không?

<details><summary>Xem lời giải</summary><p>(1) Câu đầu ném <code>AttributeError: 'tuple' object has no attribute 'append'</code>, vì <code>history</code> trả tuple (một bản sao), nên không có gì bị đổi. Câu sau chạy được và để lịch sử ghi <code>completed</code> trong khi <code>status</code> vẫn là trạng thái cũ: hai thông tin lệch nhau mà không báo lỗi nào. Đó là lý do chỉ có một nơi (<code>advance</code>) được ghi cả hai, và đó là việc quy ước gạch dưới chỉ nhắc chứ không cưỡng chế. (2) Một cách viết: <code>return new_status in ALLOWED_TRANSITIONS.get(self._status, ())</code>. Hàm này chỉ trả lời về <i>bảng chuyển</i>, chưa tính điều kiện <code>driver_id</code> đi kèm, nên «<code>can_advance</code> đúng» vẫn có thể gặp <code>ValueError</code> khi gọi <code>advance</code>. Dùng nó để hiển thị nút trên giao diện thì được; còn để «kiểm tra rồi chuyển» thì hai bước tách rời dễ lệch nhau khi code khác chen vào giữa, nên đường chính vẫn là gọi <code>advance</code> và bắt <code>ValueError</code>.</p></details>

Bài trước: [bài 10](/articles/python-pytest-kiem-thu-chinh-sach-cuoc). Bài tiếp: [bài 12](/articles/python-type-hints-mypy-protocol-generics).

## Tài liệu gốc

- [Thư viện chuẩn: dataclasses](https://docs.python.org/3.14/library/dataclasses.html).
- [PEP 557: Data Classes](https://peps.python.org/pep-0557/).
- [Hàm dựng sẵn: property](https://docs.python.org/3.14/library/functions.html#property).
- [Tutorial: Classes (gồm quy ước tên gạch dưới và name mangling)](https://docs.python.org/3.14/tutorial/classes.html).
- [Data model: `__slots__`](https://docs.python.org/3.14/reference/datamodel.html#slots).
- [Thư viện chuẩn: tracemalloc](https://docs.python.org/3.14/library/tracemalloc.html).
