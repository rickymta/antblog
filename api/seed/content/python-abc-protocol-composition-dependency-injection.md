---
slug: python-abc-protocol-composition-dependency-injection
title: "ABC, Protocol, composition và dependency injection: thay chính sách cước"
summary: "Bài 14: từ duck typing đến abc.ABC và typing.Protocol, tiêm Clock và nguồn chính sách qua constructor để thay chính sách cước theo thành phố và test báo giá hết hạn mà không chờ, không cần DI container."
category: python-beginner
level: beginner
tags: [python, toi-noi, dependency-injection, protocol]
language: vi
---

Bộ phận sản phẩm của [Tới Nơi](/articles/du-an-mau-toi-noi) muốn thử giá 5.000 VND cho mỗi km vượt ở Đà Nẵng, các thành phố khác giữ 4.500 (đây là giả định thử nghiệm của bài, không phải bảng giá thật). Cùng lúc, báo giá phải có hạn: đủ 5 phút thì khách phải xin giá mới. Hai yêu cầu nghe khác nhau, nhưng vướng cùng một vật cản. Nếu hàm báo giá tự đọc đồng hồ hệ thống và tự chọn bảng giá bên trong, ta không thể thay bảng giá theo thành phố mà không sửa nó, và không thể kiểm tra «hết hạn sau 5 phút» mà không ngồi chờ đúng 5 phút. Thứ ta cần là một cách để **đưa phụ thuộc từ ngoài vào**.

Đây là bài 14 của lộ trình **Python cơ bản qua Tới Nơi**. Bạn cần quen Protocol và mypy (bài 12), pytest (bài 10) và cách đọc JSON (bài 09). Bài 10 đã khép lại bằng nhận định rằng thiết kế dễ test là thiết kế truyền phụ thuộc từ ngoài vào; bài này làm trọn ý đó.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Python không có từ khoá <code>interface</code>. Nếu <code>QuoteService</code> chỉ gọi <code>policies.policy_for(city)</code> trên bất kỳ đối tượng nào được đưa vào, và ta lỡ đưa một <code>object()</code> trơn, lỗi xuất hiện lúc dựng service hay lúc gọi <code>quote()</code>?</p>
<details><summary>Xem đáp án</summary><p><b>Lúc gọi.</b> Dựng service chỉ lưu tham chiếu, không hỏi đối tượng có làm được gì. Lỗi <code>AttributeError</code> chỉ xuất hiện khi dòng <code>policies.policy_for(...)</code> chạy, có thể là ở request thật đầu tiên. Bài sẽ chạy đúng trường hợp này, rồi dùng ABC và Protocol để lỗi lộ ra sớm hơn.</p></details>
</div>

## Duck typing: linh hoạt, nhưng lỗi đến muộn

**Duck typing** là cách Python mặc định: «nếu nó kêu như vịt thì coi là vịt». Hàm không hỏi kiểu của đối số, chỉ gọi phương thức cần dùng, và đối tượng nào có phương thức đó đều được chấp nhận. Sự linh hoạt này chính là cái ta muốn, vì đối tượng giả trong test và nguồn chính sách thật không cần chung gốc. Cái giá là lỗi đến muộn. Lab dựng `QuoteService` với một `object()` và gọi `quote`:

```text
AttributeError: 'object' object has no attribute 'policy_for'
```

Lệnh `QuoteService(...)` chạy xong bình thường; lỗi chỉ lộ ở lời gọi. Bài 12 đã cho ta công cụ: khai báo hợp đồng bằng kiểu rồi để mypy kiểm tra. Nhưng trước khi tới đó, ta xem đoạn code chưa có hợp đồng nào trông ra sao. Bản «trước» của lab (`service.py`) tự lấy cả đồng hồ lẫn bảng giá:

```python
QUOTE_TTL = timedelta(minutes=5)


class HardcodedQuoteService:
    """TRƯỚC: tự gọi đồng hồ thật và dùng chính sách toàn cục; muốn test hạn phải chờ 5 phút."""

    def quote(self, distance_km: Decimal) -> Quote:
        fare = calculate_fare(distance_km, STANDARD)
        return Quote(distance_km, Money(fare), datetime.now(UTC) + QUOTE_TTL)
```

`Money` và `Quote` ở đây là bản rút gọn của bài 13, nơi chúng được thiết kế đầy đủ; bài 13 cũng hứa «bài 14 sẽ gói việc đọc giờ bằng `Clock`», và đó là việc ta làm bên dưới. Hai lời gọi liên tiếp cho cùng một giá nhưng hạn khác nhau, vì mỗi lần đọc, đồng hồ thật trả về một giá trị khác (trích output):

```text
hai báo giá 5 km cùng giá 25500 nhưng expires_at khác nhau: True
```

Kết quả không lặp lại được thì không viết được test chắc chắn. Ta cũng không thay được bảng giá: `STANDARD` được gắn cứng trong thân hàm, nên thử giá 5.000 ở Đà Nẵng nghĩa là sửa chính hàm báo giá.

## ABC: lớp cha trừu tượng để chia sẻ code

Hướng thứ nhất là cho các nguồn chính sách một lớp cha chung. Module `abc` cung cấp `ABC` và `@abstractmethod` để lớp cha khai báo «lớp con bắt buộc phải cài đặt phương thức này». Lab có hai nguồn: `DictPolicyProvider` (trong bộ nhớ) và `JsonFilePolicyProvider` (đọc `policies.json`). Cả hai chia sẻ một luật: thành phố không có chính sách riêng thì dùng mặc định. Luật đó nằm một lần ở lớp cha (`providers.py`):

```python
class PolicyProviderBase(ABC):
    """Template method: `policy_for` có sẵn luật «không có thì dùng mặc định»; lớp con chỉ làm `lookup`."""

    def __init__(self, default: FarePolicy = STANDARD) -> None:
        self._default = default

    @abstractmethod
    def lookup(self, city: str) -> FarePolicy | None:
        """Trả chính sách riêng của thành phố, hoặc None nếu không có."""

    def policy_for(self, city: str) -> FarePolicy:
        found = self.lookup(city)
        return self._default if found is None else found


class DictPolicyProvider(PolicyProviderBase):
    def __init__(self, by_city: dict[str, FarePolicy], default: FarePolicy = STANDARD) -> None:
        super().__init__(default)
        self._by_city = by_city

    def lookup(self, city: str) -> FarePolicy | None:
        return self._by_city.get(city)
```

Cách tổ chức này gọi là **template method**: lớp cha cố định khung thuật toán (`policy_for`), lớp con điền phần thay đổi (`lookup`). ABC cưỡng chế việc cài `lookup`, và Python kiểm tra điều đó **lúc khởi tạo**, không phải lúc gọi:

```text
TypeError: Can't instantiate abstract class PolicyProviderBase without an implementation for abstract method 'lookup'
TypeError: Can't instantiate abstract class IncompleteProvider without an implementation for abstract method 'lookup'
```

Dòng thứ nhất là khởi tạo trực tiếp lớp trừu tượng; dòng thứ hai là lớp con `IncompleteProvider` quên cài `lookup`. Lỗi đến sớm hơn so với duck typing (lúc dựng đối tượng, thường ở lúc khởi động chương trình), và mypy còn báo lỗi từ trước khi chạy (`[abstract]`, xem phần cuối). ABC cũng có giới hạn: nó chỉ ép **sự tồn tại** của phương thức, không ép chữ ký đúng, và buộc mọi cài đặt phải **kế thừa** từ nó, kể cả đối tượng giả trong test. Thư viện `collections.abc` mà bài 12 đã dùng (`Sequence`) cũng là một họ ABC của thư viện chuẩn.

## Protocol: nơi dùng khai báo điều nó cần

Hướng thứ hai đảo chiều: thay vì để cài đặt kế thừa, **nơi dùng** khai báo nó cần gì. Bài 12 đã giới thiệu `Protocol` (PEP 544). `QuoteService` cần hai thứ, và `ports.py` khai báo chúng:

```python
@runtime_checkable
class Clock(Protocol):
    """Ai có `now() -> datetime` (aware, UTC) đều là Clock; không cần kế thừa gì."""

    def now(self) -> datetime: ...


class FarePolicyProvider(Protocol):
    """Cho biết chính sách cước áp dụng ở một thành phố."""

    def policy_for(self, city: str) -> FarePolicy: ...
```

Cài đặt thật và cài đặt giả chỉ cần có phương thức đúng chữ ký, không kế thừa gì:

```python
class SystemClock:
    def now(self) -> datetime:
        return datetime.now(UTC)


class FixedClock:
    """Đồng hồ giả cho test: đứng yên cho tới khi ta tua."""

    def __init__(self, instant: datetime) -> None:
        if instant.tzinfo is None:
            raise ValueError("FixedClock cần datetime có múi giờ (aware)")
        self._instant = instant

    def now(self) -> datetime:
        return self._instant

    def advance(self, delta: timedelta) -> None:
        self._instant += delta
```

Lúc chạy, Protocol không buộc gì: nó là thông tin cho mypy. Nếu cần kiểm tra cả lúc chạy, `@runtime_checkable` cho phép dùng `isinstance`, nhưng kiểm tra đó **chỉ xem đối tượng có thuộc tính đúng tên hay không, không so chữ ký**. Output của lab cho thấy điều đó:

```text
isinstance(FixedClock, Clock) = True
isinstance(object(), Clock) = False
isinstance(WrongSignatureClock(), Clock) = True  (mypy mới bắt được)
```

`WrongSignatureClock.now(self, tz: str) -> str` hoàn toàn sai hợp đồng nhưng `isinstance` vẫn trả về True. Kết luận thực hành: dựa vào mypy để kiểm tra chữ ký; cùng lắm chỉ dùng `isinstance` với Protocol để phân nhánh lúc chạy, đừng coi nó là cổng an toàn. Hình dưới đặt hai cách cạnh nhau cho cùng vai trò «nguồn chính sách».

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Bên trái, hai lớp DictPolicyProvider và JsonFilePolicyProvider kế thừa lớp trừu tượng PolicyProviderBase để dùng chung policy_for. Bên phải, giao thức FarePolicyProvider chỉ yêu cầu có policy_for, nên hai lớp trên và đối tượng giả trong test đều thoả mà không kế thừa."><defs><marker id="py14-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py14-0-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="20" y="20" width="400" height="70" rx="10" class="f-phys-soft s-phys"/><text x="36" y="48" class="t-b">PolicyProviderBase (ABC)</text><text x="36" y="72" class="t-small">policy_for có sẵn; lookup bắt buộc cài</text><rect x="20" y="150" width="190" height="60" rx="10" class="f-phys-soft s-phys"/><text x="36" y="175" class="t-small">DictPolicyProvider</text><text x="36" y="195" class="t-small">kế thừa</text><rect x="230" y="150" width="190" height="60" rx="10" class="f-phys-soft s-phys"/><text x="246" y="175" class="t-small">JsonFilePolicyProvider</text><text x="246" y="195" class="t-small">kế thừa</text><path d="M115 150 L115 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py14-0-arrow)"/><path d="M325 150 L325 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py14-0-arrow)"/><rect x="460" y="20" width="400" height="70" rx="10" class="f-pk-soft s-pk"/><text x="476" y="48" class="t-b">FarePolicyProvider (Protocol)</text><text x="476" y="72" class="t-small">chỉ cần có policy_for(city) -> FarePolicy</text><rect x="460" y="150" width="190" height="60" rx="10" class="f-pk-soft s-pk"/><text x="476" y="175" class="t-small">Dict và Json provider</text><text x="476" y="195" class="t-small">thoả vì có policy_for</text><rect x="670" y="150" width="190" height="60" rx="10" class="f-pk-soft s-pk"/><text x="686" y="175" class="t-small">RecordingPolicyProvider</text><text x="686" y="195" class="t-small">đối tượng giả, không kế thừa</text><path d="M555 150 L555 90" class="f-none s-pk dash" stroke-width="2" marker-end="url(#py14-0-pk)"/><path d="M765 150 L765 90" class="f-none s-pk dash" stroke-width="2" marker-end="url(#py14-0-pk)"/><text x="20" y="245" class="t-small">ABC: ta sở hữu họ cài đặt và muốn chia sẻ code</text><text x="460" y="245" class="t-small">Protocol: đặt ở ranh giới, để đối tượng giả lắp được</text></svg>
</div>
<figcaption>Hình 1. Cùng vai trò nguồn chính sách, hai cách ràng buộc. Nét liền là kế thừa, nét đứt là khớp theo cấu trúc; chúng bổ sung cho nhau, không loại trừ nhau.</figcaption>
</figure>

| | Duck typing thuần | `abc.ABC` | `typing.Protocol` |
| --- | --- | --- | --- |
| Lỗi thiếu phương thức lộ ra | lúc gọi (`AttributeError`) | lúc khởi tạo lớp con (`TypeError`) và ở mypy | ở mypy; lúc chạy chỉ qua `isinstance` nếu `runtime_checkable`, và chỉ kiểm tên |
| Cài đặt phải kế thừa | không | có | không |
| Chia sẻ code giữa các cài đặt | không | có (phương thức cụ thể ở lớp cha) | không |
| Hợp với | script nhỏ, mã thử | họ cài đặt do ta sở hữu | ranh giới giữa module, đối tượng giả, mã bên thứ ba |

Với người đã đọc loạt C#, ta có thể đối chiếu như sau: `interface` trong C# là **nominal** (khớp theo tên khai báo), lớp phải khai báo `: IFarePolicy`, còn Protocol của Python là **structural** (khớp theo cấu trúc). Bài [Interface, composition và dependency injection trong C#](/articles/csharp-interface-composition-di) giải cùng bài toán thay chính sách cước; hai bài có chung cách nghĩ, khác ở chỗ Python không đòi khai báo quan hệ.

## Composition và tiêm qua constructor

**Composition** là ghép đối tượng bằng cách để đối tượng này **giữ** đối tượng kia (quan hệ «có một»), thay vì kế thừa («là một»). `QuoteService` có một `FarePolicyProvider` và có một `Clock`. **Dependency injection** là đưa những đối tượng đó vào từ ngoài, ở đây qua constructor (`service.py`):

```python
class QuoteService:
    """SAU: phụ thuộc đến từ ngoài qua constructor, nên thay được từng cái."""

    def __init__(self, policies: FarePolicyProvider, clock: Clock, ttl: timedelta = QUOTE_TTL) -> None:
        self._policies = policies
        self._clock = clock
        self._ttl = ttl

    def quote(self, city: str, distance_km: Decimal, *, service: ServiceKind = ServiceKind.RIDE) -> Quote:
        policy = self._policies.policy_for(city)
        fare = calculate_fare(distance_km, policy, service=service)
        return Quote(distance_km, Money(fare), self._clock.now() + self._ttl)

    def is_expired(self, quote: Quote) -> bool:
        return not quote.is_valid(self._clock.now())
```

Hai chi tiết thiết kế. Thứ nhất, **không phải thứ gì cũng cần cổng (port)**: `calculate_fare` là hàm thuần (không đọc giờ, không đọc file, không biết thành phố), nên ta gọi thẳng nó, không bọc vào interface. Trừu tượng hoá chỉ đáng khi có thứ thật sự đổi hoặc cần thay trong test. Thứ hai, **nơi lắp ráp phải tập trung**. Hàm `build_service` trong `main.py` là *composition root*, chỗ duy nhất biết cài đặt cụ thể:

```python
def build_service(clock: Clock) -> QuoteService:
    """Composition root: nơi DUY NHẤT chọn cài đặt cụ thể và nối chúng lại với nhau."""
    return QuoteService(JsonFilePolicyProvider(HERE / "policies.json"), clock)
```

Với `FixedClock` đặt ở 08:00 UTC ngày 1/1/2026, ba thành phố báo giá 5 km như sau. Đà Nẵng ra 27.000 (12.000 + 3 × 5.000) nhờ `policies.json`; Cần Thơ không có dòng riêng nên rơi về mặc định (trích output):

```text
Hà Nội: 5 km -> 25500 VND
Đà Nẵng: 5 km -> 27000 VND
Cần Thơ: 5 km -> 25500 VND
expires_at = 2026-01-01T08:05:00+00:00
sau khi tua thêm 0:04:59: hết hạn = False
sau khi tua thêm 0:00:01: hết hạn = True
```

Việc so giờ nằm ở `Quote.is_valid(now)` (bài 13 đã cho hàm này nhận `now` làm tham số; giờ `QuoteService` truyền `Clock.now()` vào). Bản trong lab (trích):

```python
@dataclass(frozen=True)
class Quote:
    trip_distance_km: Decimal
    fare: Money
    expires_at: datetime  # aware

    def is_valid(self, now: datetime) -> bool:
        """Còn hiệu lực khi now < expires_at: đúng mốc 5 phút là hết (cùng biên với bài 13)."""
        if now.tzinfo is None:
            raise ValueError("now cần datetime có múi giờ (aware)")
        return now < self.expires_at
```

Quy ước biên giống bài 13: báo giá còn hiệu lực khi `now < expires_at`, nên ở mốc 5 phút tròn báo giá đã hết hạn, và `is_expired` chỉ là phủ định của `is_valid`. Đây là lựa chọn nghiệp vụ, và test bên dưới ghi lại lựa chọn đó như một phần đặc tả. Tham số `service` giờ là `ServiceKind` (`StrEnum` của bài 13): ba dịch vụ vẫn cùng một giá, vì mô hình học tập chưa có phụ phí. Giá 5.000 của Đà Nẵng là dữ liệu của thử nghiệm học tập, không phải bảng giá thật.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Composition root lắp SystemClock và JsonFilePolicyProvider khi chạy thật, hoặc FixedClock và RecordingPolicyProvider trong test, vào cùng một QuoteService; QuoteService gọi hàm thuần calculate_fare."><defs><marker id="py14-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="250" height="80" rx="10" class="f-phys-soft s-phys"/><text x="36" y="58" class="t-b">Lúc chạy thật</text><text x="36" y="80" class="t-small">SystemClock</text><text x="36" y="98" class="t-small">JsonFilePolicyProvider</text><rect x="20" y="140" width="250" height="80" rx="10" class="f-pk-soft s-pk"/><text x="36" y="168" class="t-b">Trong test</text><text x="36" y="190" class="t-small">FixedClock</text><text x="36" y="208" class="t-small">RecordingPolicyProvider</text><path d="M270 70 L330 105" class="f-none s-phys" stroke-width="2" marker-end="url(#py14-1-arrow)"/><path d="M270 180 L330 145" class="f-none s-phys" stroke-width="2" marker-end="url(#py14-1-arrow)"/><rect x="330" y="65" width="270" height="120" rx="10" class="f-phys-soft s-phys"/><text x="346" y="98" class="t-b">QuoteService</text><text x="346" y="124" class="t-small">cần: Clock, FarePolicyProvider</text><text x="346" y="144" class="t-small">(Protocol khai báo ở ports.py)</text><text x="346" y="164" class="t-small">không biết cài đặt cụ thể</text><path d="M600 125 L650 125" class="f-none s-phys" stroke-width="2" marker-end="url(#py14-1-arrow)"/><rect x="650" y="75" width="210" height="100" rx="10" class="f-pk-soft s-pk"/><text x="666" y="108" class="t-b">calculate_fare</text><text x="666" y="134" class="t-small">hàm thuần, gọi thẳng</text><text x="666" y="154" class="t-small">không cần cổng</text></svg>
</div>
<figcaption>Hình 2. Cùng một QuoteService, hai cách lắp. Chỉ nơi lắp ráp biết cài đặt cụ thể; QuoteService chỉ biết hai Protocol và gọi thẳng hàm thuần.</figcaption>
</figure>

## Test bằng đối tượng giả, không cần mock framework

Vì `QuoteService` nhận phụ thuộc từ ngoài, test chỉ cần đưa vào đối tượng giả. Python có `unittest.mock`, nhưng ở đây ta không cần: một lớp nhỏ tự viết đủ dùng, và vì Protocol khớp theo cấu trúc nên nó không kế thừa gì. Trích `tests/test_quote_service.py`:

```python
class RecordingPolicyProvider:
    """Đối tượng giả: KHÔNG kế thừa gì, chỉ có `policy_for` đúng chữ ký, vậy là thỏa Protocol."""

    def __init__(self, policy: FarePolicy) -> None:
        self.policy = policy
        self.cities: list[str] = []

    def policy_for(self, city: str) -> FarePolicy:
        self.cities.append(city)
        return self.policy


def test_quote_uses_policy_from_provider_and_city() -> None:
    provider = RecordingPolicyProvider(POLICY_5000)
    service = QuoteService(provider, FixedClock(START))

    quote = service.quote("Đà Nẵng", D("5"))

    assert provider.cities == ["Đà Nẵng"]
    assert quote.fare.amount == D("27000")  # 12.000 + 3 km x 5.000
```

Test kiểm tra hạn báo giá không phải chờ: `FixedClock` tua thẳng tới ba mốc (4 phút 59 giây, đúng 5 phút, 6 phút) bằng `@pytest.mark.parametrize` như ở bài 10.

```python
@pytest.mark.parametrize(
    ("elapsed", "expired"),
    [
        (timedelta(minutes=4, seconds=59), False),
        (timedelta(minutes=5), True),
        (timedelta(minutes=6), True),
    ],
    ids=["4m59s", "5m00s", "6m"],
)
def test_quote_expiry_without_waiting(elapsed: timedelta, expired: bool) -> None:
    clock = FixedClock(START)
    service = QuoteService(DictPolicyProvider({}), clock)
    quote = service.quote("Hà Nội", D("5"))

    clock.advance(elapsed)

    assert service.is_expired(quote) is expired
```

Lab còn có test riêng cho `Quote.is_valid` ở đúng biên 5 phút và cho `ServiceKind`. Các test chạy tức thì và cho kết quả như nhau ở mọi lần chạy: đồng hồ và nguồn chính sách giờ là thứ ta điều khiển.

## Không cần DI container

Trong nhiều framework, «dependency injection» đi kèm một **container** tự dựng đối tượng theo cấu hình. Ở đây không có container nào và vẫn là DI đầy đủ: đối tượng nhận phụ thuộc qua constructor, và một hàm duy nhất (`build_service`) lắp chúng. Với quy mô vài chục lớp, lắp bằng tay rõ ràng hơn và dễ lần theo hơn một container. Khi dự án lớn dần, các framework sẽ có cơ chế riêng; ví dụ FastAPI có `Depends`, ta gặp ở [bài 33](/articles/python-fastapi-dependency-middleware-settings-logging). Cơ chế đó giải quyết việc *lắp ráp*, còn việc *thiết kế điểm tách* (phụ thuộc nào đáng thành cổng) vẫn là của ta, và là nội dung của bài này. [Bài 19](/articles/python-design-pattern-strategy-state-adapter) quay lại pattern Strategy trên nền này.

## Chạy và kiểm chứng

Từ gốc repository (lần đầu uv tải pytest và mypy nên cần mạng):

```bash
uv run --project examples/toi-noi-python/14-abc-protocol-di examples/toi-noi-python/14-abc-protocol-di/main.py
```

```text
== 1. Duck typing: lỗi đến muộn ==
AttributeError: 'object' object has no attribute 'policy_for'
== 2. Trước: phụ thuộc gắn cứng ==
hai báo giá 5 km cùng giá 25500 nhưng expires_at khác nhau: True
== 3. Sau: tiêm đồng hồ và nguồn chính sách ==
Hà Nội: 5 km -> 25500 VND
Đà Nẵng: 5 km -> 27000 VND
Cần Thơ: 5 km -> 25500 VND
expires_at = 2026-01-01T08:05:00+00:00
sau khi tua thêm 0:04:59: hết hạn = False
sau khi tua thêm 0:00:01: hết hạn = True
== 4. ABC chặn lớp thiếu phương thức ==
TypeError: Can't instantiate abstract class PolicyProviderBase without an implementation for abstract method 'lookup'
TypeError: Can't instantiate abstract class IncompleteProvider without an implementation for abstract method 'lookup'
== 5. Protocol: isinstance chỉ kiểm tên ==
isinstance(FixedClock, Clock) = True
isinstance(object(), Clock) = False
isinstance(WrongSignatureClock(), Clock) = True  (mypy mới bắt được)
```

Chạy test, mypy cho mã và test, rồi mypy cho các lỗi cố ý trong `broken.py`:

```bash
uv run --project examples/toi-noi-python/14-abc-protocol-di pytest examples/toi-noi-python/14-abc-protocol-di -q
uv run --project examples/toi-noi-python/14-abc-protocol-di mypy examples/toi-noi-python/14-abc-protocol-di/main.py examples/toi-noi-python/14-abc-protocol-di/tests --strict
uv run --project examples/toi-noi-python/14-abc-protocol-di mypy examples/toi-noi-python/14-abc-protocol-di/broken.py --strict
```

```text
..........                                                               [100%]
12 passed in 0.14s
Success: no issues found in 2 source files
```

Lệnh thứ ba kết thúc với mã thoát 1 và báo năm lỗi (trích; đã rút tiền tố đường dẫn `examples/toi-noi-python/14-abc-protocol-di/`, bỏ các dòng `note` chỉ rõ chữ ký lệch):

```text
broken.py:29: error: Argument 1 to "QuoteService" has incompatible type "object"; expected "FarePolicyProvider"  [arg-type]
broken.py:30: error: Argument 2 to "QuoteService" has incompatible type "BadClock"; expected "Clock"  [arg-type]
broken.py:31: error: Argument 1 to "QuoteService" has incompatible type "WrongReturnProvider"; expected "FarePolicyProvider"  [arg-type]
broken.py:32: error: Cannot instantiate abstract class "PolicyProviderBase" with abstract attribute "lookup"  [abstract]
broken.py:33: error: Cannot instantiate abstract class "IncompleteProvider" with abstract attribute "lookup"  [abstract]
Found 5 errors in 1 file (checked 1 source file)
```

Dòng 29 chính là trường hợp của câu hỏi đầu bài, nay bị bắt trước khi chạy. Dòng 30 và 31 là hai kiểu «có tên đúng nhưng hợp đồng sai» (sai tham số và kiểu trả về); mypy kèm ghi chú so từng chữ ký. Chế độ `--verify` kiểm tra các kết quả trên, và còn chạy thật pytest cùng hai lần mypy:

```bash
uv run --project examples/toi-noi-python/14-abc-protocol-di examples/toi-noi-python/14-abc-protocol-di/main.py --verify
```

```text
OK 21/21
```

Lab chạy bằng CPython 3.14.7, mypy 2.4.1 và pytest 9.2.0 (phiên bản ghi trong `uv.lock`).

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `TypeError: Can't instantiate abstract class ...` | Lớp con chưa cài hết các `@abstractmethod`; thông báo nêu tên phương thức còn thiếu |
| mypy: `Argument ... has incompatible type ... expected "Clock"` kèm ghi chú `Following member(s) ... have conflicts` | Tên phương thức đúng nhưng chữ ký hoặc kiểu trả về lệch Protocol; so hai dòng `Expected` và `Got` |
| `isinstance(x, Protocol)` ném `TypeError` | Protocol thiếu `@runtime_checkable`; và nhớ rằng nó chỉ kiểm tên thuộc tính |
| Test báo giá lúc đạt lúc không đạt, tuỳ giờ chạy | Có chỗ gọi `datetime.now()` trực tiếp trong mã nghiệp vụ; đưa `Clock` vào qua constructor |
| Thêm cổng cho mọi hàm khiến mã khó đọc hẳn đi | Hàm thuần không phụ thuộc gì bên ngoài thì gọi thẳng, không cần Protocol |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Mọi đối tượng ở đây nằm trong một process. <code>FixedClock</code> chứng minh logic hết hạn đúng, nhưng chưa nói gì về nhiều server có đồng hồ lệch nhau: hệ thống thật cần nơi lưu và kiểm tra báo giá phía server, không chỉ so giờ trên một đối tượng. <code>JsonFilePolicyProvider</code> đọc file một lần lúc khởi tạo (đổi file phải khởi động lại) và chưa kiểm tra nội dung file; bài 15 giới thiệu công cụ cho việc đó. <code>Money</code> và <code>Quote</code> là bản rút gọn của bài 13, và chính sách giá riêng cho Đà Nẵng chỉ là thử nghiệm học tập: không có phụ phí, giảm giá hay thuế. Cuối cùng, chữ ký trong Protocol chỉ được kiểm tra khi ta chạy mypy; nếu không có bước đó trong quy trình, Protocol chỉ còn là tài liệu.</p>
</div>

**Bài tập 1:** trong `tests/`, thêm test cho thành phố «Hải Phòng» dùng `DictPolicyProvider({"Hải Phòng": FarePolicy(price_per_extra_km=Decimal("4800"))})`, báo giá 5 km. Kỳ vọng là bao nhiêu, và ta có phải sửa dòng nào trong `QuoteService` không?

**Bài tập 2:** `QuoteService.__init__` có nên thêm `assert isinstance(clock, Clock)` để chắc chắn chữ ký đúng? Giải thích bằng output `isinstance(WrongSignatureClock(), Clock) = True`.

<details><summary>Xem lời giải</summary><p>(1) Phần vượt là 5 − 2 = 3 km, cước = 12.000 + 3 × 4.800 = <b>26.400</b>: <code>assert service.quote("Hải Phòng", D("5")).fare.amount == D("26400")</code>. Không phải sửa dòng nào trong <code>QuoteService</code>: chính sách mới đi vào qua nguồn chính sách, đúng mục tiêu của DI. (2) Không nên coi nó là cổng an toàn. <code>isinstance</code> với Protocol chỉ kiểm sự tồn tại của thuộc tính <code>now</code>, nên <code>WrongSignatureClock</code> (nhận thêm tham số <code>tz</code>, trả <code>str</code>) vẫn qua, rồi lỗi mới xuất hiện ở lần gọi <code>self._clock.now()</code>. Chữ ký phải được mypy kiểm trong quy trình phát triển; <code>assert</code> còn bị bỏ qua khi chạy <code>python -O</code> (bài 08).</p></details>

Bài trước: [bài 13](/articles/python-frozen-dataclass-enum-immutability). Bài tiếp: [bài 15](/articles/python-pydantic-validation-settings).

## Tài liệu gốc

- [abc: Abstract Base Classes](https://docs.python.org/3.14/library/abc.html).
- [typing: Protocol (Python 3.14)](https://docs.python.org/3.14/library/typing.html#typing.Protocol).
- [PEP 544: Protocols, structural subtyping](https://peps.python.org/pep-0544/) và [PEP 3119: Introducing Abstract Base Classes](https://peps.python.org/pep-3119/).
- [mypy: Protocols and structural subtyping](https://mypy.readthedocs.io/en/stable/protocols.html).
- [pytest: How to use fixtures](https://docs.pytest.org/en/stable/how-to/fixtures.html).
