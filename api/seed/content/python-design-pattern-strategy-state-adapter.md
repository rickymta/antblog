---
slug: python-design-pattern-strategy-state-adapter
title: "Design pattern kiểu Python: Strategy, State, Adapter cho Tới Nơi"
summary: "Bài 19: Strategy bằng hàm và Protocol, State bằng bảng hay lớp, Adapter bọc SDK bản đồ trả mét. Khi nào pattern GoF rút gọn thành vài dòng Python, khi nào cần class. Có lab chạy được."
category: programming
level: beginner
tags: [python, toi-noi, design-pattern, strategy, adapter]
language: vi
---

Ba việc nhỏ cùng đến với nhóm backend của [Tới Nơi](/articles/du-an-mau-toi-noi) trong một tuần. Dịch vụ `food` muốn ghi lại mỗi lần tính cước để đối soát với quán, còn `parcel` hay tính lại cùng một quãng đường nên muốn nhớ kết quả; giá thì không đổi. Chuyến xe có tám trạng thái, và mỗi nơi trong code đang tự viết `if status == ...` theo cách riêng. Cuối cùng, nhà cung cấp bản đồ MapViet (tên giả) trả quãng đường bằng **mét** kiểu `float`, trong khi hàm tính cước của ta nhận **km** kiểu `Decimal`. Ba việc này đều có tên trong bộ từ vựng design pattern: Strategy, State và Adapter. Câu hỏi của bài là Python giúp ta viết chúng ngắn đến mức nào, và khi nào vẫn nên dùng class.

Đây là bài 19 của lộ trình **Python cơ bản qua Tới Nơi**. Bạn cần quen `Protocol` và việc truyền phụ thuộc qua constructor ([bài 14](/articles/python-abc-protocol-composition-dependency-injection)), cùng `match/case` và `None` ([bài 16](/articles/python-match-pattern-matching-none)). Lab dùng lại mô hình cước quen thuộc: 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, không phụ phí.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trong Java hay C#, Strategy thường cần một interface và một class cho mỗi cách tính. Trong Python, ta có bắt buộc làm vậy không?</p>
<details><summary>Xem đáp án</summary><p><b>Không bắt buộc.</b> Hàm là giá trị: ta truyền một hàm nhận km và trả cước đi khắp nơi như truyền một số. Class chỉ cần khi strategy có trạng thái riêng hoặc có nhiều method phải đổi cùng nhau. Phần dưới dùng cả hai dạng trên cùng một bài toán để so sánh.</p></details>
</div>

## Pattern là từ vựng, Python chỉ rút ngắn hình dạng

Loạt bài Design Pattern của blog giải thích pattern ở mức tổng quát, bằng TypeScript: [bản đồ 23 pattern GoF](/articles/design-pattern-la-gi), [nhóm Hành vi với Strategy và State](/articles/nhom-hanh-vi-dong-goi-hanh-vi), [nhóm Cấu trúc với Adapter](/articles/nhom-cau-truc-structural). Bài này không lặp lại lý thuyết đó, mà trả lời một câu hẹp hơn: với một ngôn ngữ có hàm first-class, `dict` và `Protocol`, mỗi pattern co lại còn gì.

Điều không đổi là vấn đề và cái giá. Strategy giải quyết «nhiều cách làm một việc, chọn lúc chạy». State giải quyết «luật được làm gì phụ thuộc trạng thái hiện tại». Adapter giải quyết «thứ ta không sửa được có interface khác thứ ta cần». Mỗi pattern thêm một tầng gián tiếp, nên nó chỉ đáng giá khi có chỗ đau thật; nếu không, một hàm thẳng thắn dễ đọc hơn. Điều thay đổi là hình dạng: trong Python, nhiều chỗ ta thay lớp bằng hàm, interface bằng `Protocol` và `if` dài bằng bảng tra.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="quote_for chọn strategy theo dịch vụ: ride dùng hàm standard, food dùng audited bọc standard, parcel dùng lớp CachedFare. Cả ba cùng gọi calculate_fare và cho cùng giá 25.500 VND với 5 km."><defs><marker id="py19-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="80" width="220" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="115" class="t-b">quote_for</text><text x="36" y="140" class="t-small">chọn theo ServiceKind</text><text x="36" y="158" class="t-small">rồi gọi strategy</text><rect x="300" y="20" width="320" height="56" rx="8" class="f-pk-soft s-pk"/><text x="316" y="45" class="t-mono">RIDE   -&gt; standard(RIDE)</text><text x="316" y="64" class="t-small">hàm thuần</text><rect x="300" y="102" width="320" height="56" rx="8" class="f-pk-soft s-pk"/><text x="316" y="127" class="t-mono">FOOD   -&gt; audited(standard(FOOD), log)</text><text x="316" y="146" class="t-small">hàm bậc cao: ghi nhật ký mỗi lần tính</text><rect x="300" y="184" width="320" height="56" rx="8" class="f-pk-soft s-pk"/><text x="316" y="209" class="t-mono">PARCEL -&gt; CachedFare(standard(PARCEL))</text><text x="316" y="228" class="t-small">lớp, vì có bộ nhớ đệm riêng</text><rect x="680" y="80" width="180" height="100" rx="10" class="f-phys-soft s-phys"/><text x="696" y="115" class="t-b">calculate_fare</text><text x="696" y="140" class="t-small">cùng 25.500 VND</text><text x="696" y="158" class="t-small">cho 5 km</text><path d="M240 115 L300 48" class="f-none s-phys" stroke-width="2" marker-end="url(#py19-0-arrow)"/><path d="M240 130 L300 130" class="f-none s-phys" stroke-width="2" marker-end="url(#py19-0-arrow)"/><path d="M240 145 L300 212" class="f-none s-phys" stroke-width="2" marker-end="url(#py19-0-arrow)"/><path d="M620 48 L680 115" class="f-none s-phys" stroke-width="2" marker-end="url(#py19-0-arrow)"/><path d="M620 130 L680 130" class="f-none s-phys" stroke-width="2" marker-end="url(#py19-0-arrow)"/><path d="M620 212 L680 145" class="f-none s-phys" stroke-width="2" marker-end="url(#py19-0-arrow)"/></svg>
</div>
<figcaption>Hình 1. Ba dịch vụ dùng ba strategy khác nhau, cùng đi tới một quy tắc tính cước. Khác biệt nằm ở cách thực hiện (thuần, có nhật ký, có đệm), không nằm ở giá.</figcaption>
</figure>

## Strategy: hàm đã là strategy

Toàn bộ strategy trong lab có chung một hợp đồng: nhận khoảng cách `Decimal`, trả cước `Decimal`. Cú pháp `type` (PEP 695, đã có từ Python 3.12) cho ta đặt tên cho hợp đồng đó. Chính sách chuẩn là một hàm trả về hàm, tức closure (bài 17) giữ `service` và `policy` lại:

```python
# strategy.py (trích)
type FareStrategy = Callable[[Decimal], Decimal]


def standard(service: ServiceKind = ServiceKind.RIDE, policy: FarePolicy = STANDARD) -> FareStrategy:
    """Closure giữ `service` và `policy`; trả về hàm strategy.

    `service` được chuyển tiếp cho calculate_fare; hiện tại nó chưa làm đổi giá.
    """

    def strategy(distance_km: Decimal) -> Decimal:
        return calculate_fare(distance_km, policy, service=service)

    return strategy
```

Thêm hành vi mà không đổi giá cũng là một hàm, nhận strategy khác rồi bọc nó. Đây là hình dạng Decorator, nhưng với hàm thì ta gọi đơn giản là hàm bậc cao:

```python
def audited(inner: FareStrategy, log: list[str]) -> FareStrategy:
    """Hàm bậc cao: bọc strategy khác, ghi một dòng mỗi lần tính. Giá không đổi."""

    def strategy(distance_km: Decimal) -> Decimal:
        fare = inner(distance_km)
        log.append(f"{distance_km} km -> {fare} VND")
        return fare

    return strategy
```

Khi strategy cần **trạng thái riêng**, class mới có lý do tồn tại. `CachedFare` giữ một `dict` và một bộ đếm trúng cache; nhờ `__call__`, instance của nó dùng được ở mọi chỗ cần `FareStrategy`:

```python
class CachedFare:
    def __init__(self, inner: FareStrategy) -> None:
        self._inner = inner
        self._cache: dict[Decimal, Decimal] = {}
        self.hits = 0

    def __call__(self, distance_km: Decimal) -> Decimal:
        if distance_km in self._cache:
            self.hits += 1
        else:
            self._cache[distance_km] = self._inner(distance_km)
        return self._cache[distance_km]
```

Việc chọn strategy theo dịch vụ là một `dict` thay cho chuỗi `if/elif`; thêm dịch vụ mới là thêm một dòng vào bảng:

```python
def build_strategies(audit_log: list[str]) -> dict[ServiceKind, FareStrategy]:
    return {
        ServiceKind.RIDE: standard(ServiceKind.RIDE),
        ServiceKind.FOOD: audited(standard(ServiceKind.FOOD), audit_log),
        ServiceKind.PARCEL: CachedFare(standard(ServiceKind.PARCEL)),
    }
```

Còn khi một strategy gồm **nhiều thành viên đi cùng nhau** (một nhãn mô tả cùng một phương thức tính), hàm đơn lẻ không đủ và `Protocol` là công cụ phù hợp. Như bài 14 đã nói, nó khớp theo cấu trúc: `PolicyCalculator` không kế thừa gì, chỉ cần có `label` và `calculate`.

```python
@runtime_checkable
class FareCalculator(Protocol):
    label: str

    def calculate(self, distance_km: Decimal) -> Decimal: ...


def as_strategy(calculator: FareCalculator) -> FareStrategy:
    """Method gắn với instance cũng là một callable: hai dạng Strategy nối được với nhau."""
    return calculator.calculate
```

Hai dạng không loại trừ nhau, vì method gắn với một instance (`calculator.calculate`) tự nó là một callable. Quy tắc chọn: bắt đầu bằng hàm; sang class khi có trạng thái; sang `Protocol` khi có nhiều thành viên hoặc khi cần đặt tên rõ hợp đồng cho người khác hiện thực.

Điều đáng kiểm nhất là **hợp đồng**, không phải từng strategy. Lab chạy cả tám ca biên của bài 10 qua cả ba dịch vụ và đòi cùng bảng giá. Strategy đổi chế độ làm tròn sẽ đỏ ngay ở ca 2,001 km, vì cước thô 12.004,5 làm tròn half-up ra 12.005 nhưng half-even ra 12.004. Cho nên «strategy làm tròn» không còn là strategy cùng giá nữa; đó là quyết định nghiệp vụ, không phải chi tiết hiện thực.

## State: bảng chuyển hay mỗi trạng thái một lớp

Vòng đời chuyến có 8 trạng thái: sáu bước thuận từ `requested` đến `completed`, cộng `cancelled` đạt được từ bốn trạng thái đầu. Cách nhỏ nhất là một bảng: mỗi trạng thái ánh xạ tới tập trạng thái kế tiếp được phép (bài 11 đã giới thiệu cách này với chuỗi; bài 13 đổi sang `StrEnum`):

```python
# state.py (trích: lớp cha và một trong tám lớp con)
ALLOWED_TRANSITIONS: dict[TripStatus, frozenset[TripStatus]] = {
    TripStatus.REQUESTED: frozenset({TripStatus.SEARCHING, TripStatus.CANCELLED}),
    TripStatus.SEARCHING: frozenset({TripStatus.ACCEPTED, TripStatus.CANCELLED}),
    TripStatus.ACCEPTED: frozenset({TripStatus.ARRIVING, TripStatus.CANCELLED}),
    TripStatus.ARRIVING: frozenset({TripStatus.PICKED_UP, TripStatus.CANCELLED}),
    TripStatus.PICKED_UP: frozenset({TripStatus.IN_TRANSIT}),
    TripStatus.IN_TRANSIT: frozenset({TripStatus.COMPLETED}),
    TripStatus.COMPLETED: frozenset(),
    TripStatus.CANCELLED: frozenset(),
}
```

Bảng này đã là một máy trạng thái. Nhưng nó bắt đầu gượng khi một trạng thái có **luật riêng**. Chuyển `searching` sang `accepted` cần có tài xế, và luật đó chen vào `move_to` dưới dạng một `if` đứng ngoài bảng:

```python
    def move_to(self, new_status: TripStatus) -> None:
        if new_status not in ALLOWED_TRANSITIONS[self.status]:
            raise ValueError(f"{self.trip_id}: không chuyển {self.status} -> {new_status}")
        # Luật riêng của một trạng thái bắt đầu chen vào bảng dưới dạng if:
        if new_status is TripStatus.ACCEPTED and self.driver_id is None:
            raise ValueError(f"{self.trip_id}: chưa có tài xế nhận chuyến")
        self.status = new_status
```

Mỗi luật mới thêm một `if` ở đây, và các `if` ấy không nằm cạnh trạng thái mà chúng nói về. Pattern State đảo ngược việc đó: mỗi trạng thái là một lớp, mang theo luật của chính nó. Lớp cha cho mặc định của cả tám trạng thái, lớp con chỉ ghi đè điều khác biệt:

```python
class TripState:
    status: ClassVar[TripStatus]
    forward: ClassVar[TripStatus | None] = None  # bước thuận kế tiếp
    cancellable: ClassVar[bool] = False

    def advance(self, trip: Trip) -> TripStatus:
        if self.forward is None:
            raise ValueError(f"{trip.trip_id}: {self.status} là trạng thái cuối")
        return self.forward

    def cancel(self, trip: Trip) -> TripStatus:
        if not self.cancellable:
            raise ValueError(f"{trip.trip_id}: không hủy được khi {self.status}")
        return TripStatus.CANCELLED


class Searching(TripState):
    status = TripStatus.SEARCHING
    forward = TripStatus.ACCEPTED
    cancellable = True

    def advance(self, trip: Trip) -> TripStatus:
        # Luật riêng của trạng thái này nằm trong lớp của nó, không rải khắp nơi.
        if trip.driver_id is None:
            raise ValueError(f"{trip.trip_id}: chưa có tài xế nhận chuyến")
        return super().advance(trip)
```

Một chi tiết nhỏ của Python 3.14: chữ ký `advance(self, trip: Trip)` nhắc tới `Trip` dù lớp `Trip` khai báo ở cuối file, và ta không cần ngoặc kép. Annotation được tính khi có ai cần (PEP 649 và 749), không phải lúc `def` chạy; `annotationlib.get_annotations` trả về lớp `Trip` thật khi ta hỏi sau khi module đã nạp xong. `Trip` giữ một trạng thái hiện tại và **ủy quyền** cho nó, nên bên ngoài chỉ thấy `trip.advance()` và `trip.cancel()`:

```python
@dataclass
class Trip:
    trip_id: str
    driver_id: str | None = None
    _state: TripState = field(
        default_factory=lambda: STATES[TripStatus.REQUESTED], init=False, repr=False
    )

    def advance(self) -> None:
        self._state = STATES[self._state.advance(self)]
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Bảy trạng thái nối nhau từ requested tới completed bằng advance. Bốn trạng thái đầu có mũi tên nét đứt cancel tới cancelled. picked_up, in_transit và completed không có mũi tên hủy nên cancel bị từ chối."><defs><marker id="py19-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="30" y="40" width="100" height="56" rx="8" class="f-phys-soft s-phys"/><text x="42" y="73" class="t-mono">requested</text><rect x="150" y="40" width="100" height="56" rx="8" class="f-phys-soft s-phys"/><text x="162" y="73" class="t-mono">searching</text><rect x="270" y="40" width="100" height="56" rx="8" class="f-phys-soft s-phys"/><text x="282" y="73" class="t-mono">accepted</text><rect x="390" y="40" width="100" height="56" rx="8" class="f-phys-soft s-phys"/><text x="402" y="73" class="t-mono">arriving</text><rect x="510" y="40" width="100" height="56" rx="8" class="f-phys-soft s-phys"/><text x="522" y="73" class="t-mono">picked_up</text><rect x="630" y="40" width="100" height="56" rx="8" class="f-phys-soft s-phys"/><text x="642" y="73" class="t-mono">in_transit</text><rect x="750" y="40" width="100" height="56" rx="8" class="f-pk-soft s-pk"/><text x="762" y="73" class="t-mono">completed</text><path d="M130 68 L150 68" class="f-none s-phys" stroke-width="2" marker-end="url(#py19-1-arrow)"/><path d="M250 68 L270 68" class="f-none s-phys" stroke-width="2" marker-end="url(#py19-1-arrow)"/><path d="M370 68 L390 68" class="f-none s-phys" stroke-width="2" marker-end="url(#py19-1-arrow)"/><path d="M490 68 L510 68" class="f-none s-phys" stroke-width="2" marker-end="url(#py19-1-arrow)"/><path d="M610 68 L630 68" class="f-none s-phys" stroke-width="2" marker-end="url(#py19-1-arrow)"/><path d="M730 68 L750 68" class="f-none s-phys" stroke-width="2" marker-end="url(#py19-1-arrow)"/><text x="30" y="26" class="t-small">advance(): sáu bước thuận</text><rect x="370" y="204" width="140" height="56" rx="8" class="f-warn-soft s-warn"/><text x="396" y="237" class="t-mono">cancelled</text><path d="M80 96 L400 204" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py19-1-arrow)"/><path d="M200 96 L420 204" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py19-1-arrow)"/><path d="M320 96 L450 204" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py19-1-arrow)"/><path d="M440 96 L480 204" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py19-1-arrow)"/><text x="530" y="150" class="t-small">cancel() bị từ chối từ đây trở đi</text><text x="30" y="150" class="t-small">cancel(): bốn trạng thái đầu</text></svg>
</div>
<figcaption>Hình 2. Vòng đời chuyến: sáu bước thuận và bốn đường hủy, tổng cộng 10 cặp (từ, tới) được phép. Cả bảng chuyển lẫn bộ lớp trạng thái trong lab đều sinh ra đúng tập cặp này.</figcaption>
</figure>

Hai cách có cho **cùng một tập chuyển** hay không là câu hỏi kiểm được, không phải câu hỏi niềm tin. `--verify` hỏi từng trạng thái trong tám trạng thái xem lớp cho phép đi tới đâu (thử `advance` rồi `cancel`) và so với bảng: 8 × 8 = 64 cặp (từ, tới), 10 cặp được phép, kết quả khớp.

Chọn cái nào? Hãy trung thực với chính ví dụ này: phần luật của hầu hết lớp trạng thái trong lab chỉ là dữ liệu (`forward`, `cancellable`), tức một bảng nằm trong lớp; cả tám lớp đều ghi đè `message`, nhưng lời nhắn cho khách cũng chỉ là một chuỗi mà một `dict[TripStatus, str]` chứa được. Thứ mà bảng khó chứa là **luật có logic**: chỉ `Searching` có luật cần tài xế, và luật đó đọc dữ liệu của chuyến. Lớp xứng đáng khi số luật kiểu đó tăng dần. Bảng đủ dùng khi trạng thái chỉ khác nhau ở «đi tiếp được tới đâu». Nếu có một luật như «chỉ hủy được trong 3 phút sau khi tài xế nhận», lớp tiện hơn, nhưng nó cần đồng hồ để kiểm thời gian; ta sẽ phải truyền một `Clock` vào như bài 14.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">State không chống được hai tài xế cùng bấm nhận</span>
<p>Lớp <code>Searching</code> kiểm luật trên bản sao trong bộ nhớ của một tiến trình. Hai tài xế bấm nhận cùng một giây thì hai tiến trình đều thấy <code>searching</code> và đều cho đi tiếp. Chốt chặn cuối cùng phải ở database (cập nhật có điều kiện <code>WHERE status = 'searching'</code>) và sẽ được bàn ở <a href="/articles/python-transaction-optimistic-concurrency-idempotency">bài 25</a> (transaction và optimistic concurrency). State giữ luật nghiệp vụ gọn một chỗ; tính đúng khi chạy đồng thời là việc của giao dịch.</p>
</div>

## Adapter: SDK trả mét, ta cần km Decimal

MapViet là phía ta **không sửa được**. Lab giả lập SDK của họ bằng `MapVietClient`: nhận tọa độ theo thứ tự `(lng, lat)`, trả một `dict` kiểu JSON có quãng đường là `float` tính bằng mét, và báo lỗi bằng mã số riêng. Phía Tới Nơi cần một interface của chính ta, đặt tên theo nhu cầu của ta, không theo SDK:

```python
# adapter.py (trích)
class RoutingUnavailable(Exception):
    """Lỗi của Tới Nơi: không phụ thuộc nhà cung cấp nào."""


class RoutingService(Protocol):
    def distance_km(self, origin: Point, destination: Point) -> Decimal: ...


def meters_to_km(meters: float) -> Decimal:
    """Phần lõi thuần: float mét -> Decimal km. Dùng Decimal(str(...)), không Decimal(float)."""
    return Decimal(str(meters)) / Decimal(1000)
```

Adapter dịch **ba thứ** một lúc: thứ tự tọa độ, đơn vị cùng kiểu số, và cách báo lỗi.

```python
class MapVietAdapter:
    def __init__(self, client: MapVietClient) -> None:
        self._client = client

    def distance_km(self, origin: Point, destination: Point) -> Decimal:
        try:
            data = self._client.get_route(
                (origin[1], origin[0]), (destination[1], destination[0])
            )
        except MapVietError as exc:
            raise RoutingUnavailable(f"không có tuyến (mã nhà cung cấp {exc.code})") from exc
        return meters_to_km(data["routes"][0]["distance"])
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="quote_trip chỉ biết RoutingService. MapVietAdapter hiện thực RoutingService và gọi MapVietClient của nhà cung cấp, đồng thời dịch thứ tự tọa độ, đơn vị mét sang km Decimal và mã lỗi sang RoutingUnavailable."><defs><marker id="py19-2-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="200" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="75" class="t-b">quote_trip</text><text x="36" y="100" class="t-small">chỉ biết RoutingService</text><rect x="250" y="40" width="180" height="100" rx="10" class="f-bg s-line dash"/><text x="266" y="75" class="t-b">RoutingService</text><text x="266" y="100" class="t-small">Protocol của Tới Nơi</text><rect x="460" y="40" width="190" height="100" rx="10" class="f-pk-soft s-pk"/><text x="476" y="75" class="t-b">MapVietAdapter</text><text x="476" y="100" class="t-small">hiện thực Protocol</text><rect x="680" y="40" width="180" height="100" rx="10" class="f-warn-soft s-warn"/><text x="696" y="75" class="t-b">MapVietClient</text><text x="696" y="100" class="t-small">SDK bên thứ ba, trả mét</text><path d="M220 90 L250 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py19-2-arrow)"/><path d="M430 90 L460 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py19-2-arrow)"/><path d="M650 90 L680 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py19-2-arrow)"/><text x="460" y="176" class="t-small">(lat, lng) -&gt; (lng, lat)</text><text x="460" y="200" class="t-small">mét float -&gt; km Decimal</text><text x="460" y="224" class="t-small">mã lỗi 404 -&gt; RoutingUnavailable</text></svg>
</div>
<figcaption>Hình 3. Adapter nằm giữa use case và SDK. Phần còn lại của hệ thống chỉ thấy hợp đồng của Tới Nơi; ba phép dịch nằm gọn trong một lớp.</figcaption>
</figure>

Có một chi tiết dễ sai nằm trong hàm lõi `meters_to_km`: ta bọc `Decimal(str(meters))`, không viết `Decimal(meters)`. Số `float` 2000,1 không biểu diễn chính xác trong nhị phân; `Decimal(2000.1)` nhận đúng cái giá trị gần đúng đó, kèm một chuỗi chữ số rác (bài 02), còn `Decimal("2000.1")` nhận đúng văn bản. Ví dụ này không làm đổi giá cuối cùng, nhưng ta không muốn kết quả tiền phụ thuộc vào cách máy tính xấp xỉ số thực, và phép so sánh bằng trên `Decimal(float)` đã sai: `--verify` kiểm đúng điều đó (`Decimal(2000.1) != Decimal("2000.1")`). Với tiền, ta luôn đưa dữ liệu vào bằng chuỗi hoặc số nguyên.

Adapter cũng có giới hạn: nó dịch **cú pháp**, không dịch **ngữ nghĩa**. «Quãng đường» của MapViet là tuyến nhanh nhất hay ngắn nhất, có tính đường cấm hay không, adapter không biết. Nếu khác biệt đó ảnh hưởng tới tiền khách trả, nó phải hiện lên trong interface (một trường, một tham số), không được giấu sau lớp bọc.

## Khi nào không cần class, khi nào không cần pattern

Phần lõi của adapter là hàm `meters_to_km`, thuần và dễ kiểm. Nếu Tới Nơi chỉ có một chỗ gọi một SDK, một hàm bọc vài dòng đã đủ, chưa cần đặt tên «Adapter» hay dựng `Protocol`. Class bắt đầu có giá trị khi ta cần giữ client (kết nối, API key), muốn thay nhà cung cấp lúc chạy, hoặc muốn đưa một đối tượng giả vào lúc test.

Đối tượng giả cho test không cần framework mock. Lab có `FixedRouting` chỉ với một method; vì `Protocol` khớp theo cấu trúc nên `quote_trip` nhận nó mà không cần kế thừa hay đăng ký:

```python
class FixedRouting:
    def __init__(self, km: str) -> None:
        self._km = Decimal(km)

    def distance_km(self, origin: Point, destination: Point) -> Decimal:
        return self._km
```

| Dấu hiệu | Chọn |
| --- | --- |
| Nhiều cách làm một việc, mỗi cách không có trạng thái | Hàm (Strategy) |
| Cách làm có trạng thái riêng (đệm, bộ đếm) | Class có `__call__` |
| Strategy gồm nhiều thành viên phải đổi cùng nhau | `Protocol` |
| Trạng thái chỉ khác nhau ở «đi tiếp tới đâu» | `dict` (bảng chuyển) |
| Trạng thái khác nhau về hành vi và luật | Mỗi trạng thái một lớp (State) |
| Thứ có sẵn khác interface ta cần, có nhiều nơi gọi | Adapter |
| Chỉ một nơi gọi, không cần thay | Hàm bọc vài dòng |

Dấu hiệu lạm dụng cũng quen: một `Protocol` chỉ có một lớp hiện thực và không ai thay; năm lớp trạng thái mà luật của chúng chỉ là dữ liệu và bảng đã nói đủ; một «Strategy» thật ra chỉ là một `if`. Pattern phải trả được tiền cho tầng gián tiếp nó thêm vào.

## Chạy và kiểm chứng

Từ thư mục gốc repository (không cần mạng khi chạy; lần đầu uv tải mypy):

```bash
uv run --project examples/toi-noi-python/19-design-pattern-strategy-state-adapter examples/toi-noi-python/19-design-pattern-strategy-state-adapter/main.py
uv run --project examples/toi-noi-python/19-design-pattern-strategy-state-adapter examples/toi-noi-python/19-design-pattern-strategy-state-adapter/main.py --verify
uv run --project examples/toi-noi-python/19-design-pattern-strategy-state-adapter mypy --config-file examples/toi-noi-python/19-design-pattern-strategy-state-adapter/pyproject.toml examples/toi-noi-python/19-design-pattern-strategy-state-adapter/*.py
```

Lệnh đầu in ba phần Strategy, State, Adapter (trích):

```text
ride    5 km -> 25500 VND
food    5 km -> 25500 VND
parcel  5 km -> 25500 VND
nhật ký của food: ['5 km -> 25500 VND', '2.5 km -> 14250 VND']
CachedFare sau 3 lần gọi (2 quãng đường khác nhau): hits=1
PolicyCalculator khớp Protocol: True (cuoc-chuan)
```

```text
searching -> accepted khi chưa có tài xế: TN-0001: chưa có tài xế nhận chuyến
accepted: Tài xế DR-001 đã nhận chuyến
hủy khi in_transit: TN-0001: không hủy được khi in_transit
số cặp (từ, tới) được phép trong bảng chuyển: 10
```

```text
2.001 km -> 12005 VND
5.23 km -> 26535 VND
RoutingUnavailable: không có tuyến (mã nhà cung cấp 404)
Decimal(2000.1)      = 2000.09999999999990905052982270717620849609375
Decimal('2000.1')    = 2000.1
meters_to_km(2000.1) = 2.0001
```

Cước 5,23 km là 12.000 + 3,23 × 4.500 = 26.535, tính tay khớp với output. Lệnh thứ hai chạy 26 kiểm tra (bảng giá cả ba dịch vụ, nhật ký, cache, Protocol, vòng đời, 64 cặp trạng thái, luật cần tài xế, adapter, lỗi, đối tượng giả), lệnh thứ ba kiểm kiểu:

```text
OK 26/26
Success: no issues found in 5 source files
```

Có ca sai thì `--verify` in `SAI: <tên ca>`, dòng `FAIL n/26` (n là số ca đạt) và thoát với mã 1. Lab dùng Python 3.14.7 và mypy 2.4.1 (ghi trong `uv.lock`).

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Hai dịch vụ cho giá khác nhau dù không có phụ phí | Có strategy nào đổi cách làm tròn hoặc thứ tự phép tính; chạy lại bảng ca biên |
| Cache trả giá cũ sau khi đổi chính sách | `CachedFare` bọc một `policy` cố định; tạo instance mới khi đổi chính sách |
| `Decimal` có hàng chục chữ số lẻ sau khi đổi mét sang km | Có dùng `Decimal(float)` thay vì `Decimal(str(...))` không |
| Lớp trạng thái cho phép bước mà bảng cấm | So hai tập chuyển bằng đúng phép so 64 cặp của `--verify` |
| `NameError` ở annotation khi chạy trên bản Python cũ | Annotation lười chỉ có từ Python 3.14; bản cũ cần `from __future__ import annotations` hoặc ngoặc kép |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>SDK bản đồ ở đây là đối tượng giả trong bộ nhớ: không có mạng, timeout, retry hay giới hạn tần suất, những thứ mà một adapter thật phải xử lý. Trạng thái chuyến sống trong bộ nhớ của một tiến trình, chưa được lưu và chưa chống tranh chấp (xem callout ở mục State). Mô hình cước vẫn là bản học tập: ba dịch vụ cùng giá, không phụ phí, thuế hay giảm giá; các con số 12.000 và 4.500 không phải bảng giá thật.</p>
</div>

**Bài tập 1:** Trong lab, `CachedFare(audited(standard(), log))` gọi hai lần cùng 5 km. `log` có mấy dòng và `hits` bằng bao nhiêu? Đổi thứ tự thành `audited(CachedFare(standard()), log)` thì sao?

**Bài tập 2:** Tới Nơi muốn thêm luật «hủy được khi `picked_up` nếu chưa đi quá 500 m». Bảng chuyển hay lớp trạng thái hợp hơn, và cần thêm gì để test được?

<details><summary>Xem lời giải</summary><p><b>Bài 1.</b> Lần chạy thật cho <code>['5 km -> 25500 VND']</code> và <code>hits == 1</code>: lần thứ hai trúng cache nên không đi xuống <code>audited</code>, log chỉ có một dòng. Với thứ tự ngược <code>audited(CachedFare(standard()), log)</code>, <code>audited</code> bọc ngoài nên ghi cả hai lần gọi (hai dòng), còn <code>CachedFare</code> bên trong vẫn có <code>hits == 1</code>. Thứ tự bọc là một phần hành vi, như Decorator trong <a href="/articles/nhom-cau-truc-structural">nhóm Cấu trúc</a> đã nhấn mạnh. <b>Bài 2.</b> Lớp trạng thái hợp hơn, vì luật chỉ có ở <code>PickedUp</code> và cần dữ liệu của chuyến (quãng đường đã đi). Muốn test được, <code>Trip</code> phải nhận quãng đường đã đi như một thuộc tính (hoặc nhận một nguồn cung cấp nó qua <code>Protocol</code>), để test đặt giá trị 499 và 501 m rồi kiểm hai kết quả; không được đọc từ SDK thật trong lớp trạng thái. Và như callout ở mục State, kiểm trong bộ nhớ chưa thay được điều kiện ở database.</p></details>

Bài trước: [bài 18](/articles/python-functools-itertools-pipeline). Bài tiếp: [bài 20](/articles/python-dong-goi-ruff-pre-commit-cau-truc-du-an).

## Tài liệu gốc

- [Python 3.14: typing, Protocol](https://docs.python.org/3.14/library/typing.html).
- [PEP 544: Protocols, structural subtyping](https://peps.python.org/pep-0544/).
- [PEP 649: Deferred evaluation of annotations](https://peps.python.org/pep-0649/) và [PEP 749](https://peps.python.org/pep-0749/).
- [Python 3.14: decimal](https://docs.python.org/3.14/library/decimal.html).
- [Python 3.14: Data model, emulating callable objects](https://docs.python.org/3.14/reference/datamodel.html).
