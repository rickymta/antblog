---
slug: dart-ham-class-enum-sealed-pattern-matching-trang-thai-chuyen
title: "Hàm, class, enum và sealed class: mô hình trạng thái chuyến bằng Dart"
summary: "Bài 03: viết hàm có tham số đặt tên, class bất biến Trip, enhanced enum tám trạng thái chuyến, sealed class TripEvent và switch vét cạn; dùng record và pattern matching để chuyển trạng thái an toàn, có bảng test mọi cặp trạng thái–sự kiện."
category: flutter
level: beginner
tags: [flutter, dart, mobile, toi-noi, pattern-matching]
language: vi
---

Tài xế đang tới điểm đón thì khách bấm huỷ. Vài giây sau, màn hình tài xế chưa kịp cập nhật nên anh vẫn bấm "Đã đón khách". Nếu app nhận lệnh đó, chuyến đã huỷ quay lại thành "đã đón" và hệ thống tính tiền cho chuyến không tồn tại. Cách phòng thủ ít tốn kém nhất là để việc chuyển trạng thái sai **bị từ chối ngay tại chỗ**.

Dart có đủ công cụ: enum có hành vi, sealed class mà compiler biết hết biến thể, và `switch` mà compiler kiểm tra đã phủ mọi nhánh chưa. Ta dùng chúng để dựng một chuyến của [Tới Nơi](/articles/du-an-mau-toi-noi): tám trạng thái, bảy loại sự kiện, class `Trip` bất biến và hàm `apply`.

Đây là bài 03 của lộ trình **Flutter qua Tới Nơi**. Bạn cần biết kiểu dữ liệu và null safety của [bài 02](/articles/dart-bien-kieu-du-lieu-null-safety-tinh-cuoc). Vẫn là Dart thuần trong terminal, chưa có widget.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ta viết <code>switch</code> trên một giá trị thuộc <code>sealed class</code> và không có nhánh <code>_</code>. Sau đó có người thêm một lớp con mới mà chưa sửa switch. Chuyện gì xảy ra: chương trình chạy bình thường, ném lỗi lúc chạy, hay không biên dịch được?</p>
<details><summary>Xem đáp án</summary><p><b>Không biên dịch được.</b> Với sealed class, compiler biết đủ các lớp con nên báo lỗi <code>non_exhaustive_switch_expression</code> ngay khi phân tích (với câu lệnh <code>switch</code> là <code>non_exhaustive_switch_statement</code>). Nếu có nhánh <code>_</code>, lỗi biến mất và lớp con mới rơi vào nhánh đó mà không ai biết.</p></details>
</div>

## Hàm: tham số vị trí, đặt tên và hàm là giá trị

Hàm Dart khai báo kiểu trả về trước tên hàm. Tham số có ba loại:

- **Tham số vị trí**: truyền theo thứ tự. Đặt trong `[ ]` nếu tuỳ chọn, và khi đó nên có giá trị mặc định.
- **Tham số đặt tên**: khai báo trong `{ }`, gọi bằng `ten: giaTri`. Thứ tự lúc gọi không quan trọng.
- **`required`**: bắt buộc truyền tham số đặt tên. Quên truyền là lỗi biên dịch, không phải lỗi lúc chạy.

Tham số đặt tên nên là lựa chọn mặc định khi hàm có nhiều tham số cùng kiểu hoặc có cờ bool. Đọc `tripSummary('TN-1', kind: ServiceKind.ride, fareVnd: 25500)`, ta không cần mở khai báo để đoán số nào là cước.

Giá trị mặc định phải là hằng số lúc biên dịch. Tham số đặt tên kiểu nullable như `int? fareVnd` ngầm mặc định là `null`.

### functions_tour.dart — 03-ham-class-sealed-trang-thai/lib/functions_tour.dart

```dart
import 'service_kind.dart';
import 'trip.dart';

/// Tham số vị trí; `[unit]` là tham số vị trí tuỳ chọn có giá trị mặc định.
String formatVnd(int vnd, [String unit = 'VND']) {
  final digits = vnd.abs().toString();
  final buffer = StringBuffer(vnd < 0 ? '-' : '');
  for (var i = 0; i < digits.length; i++) {
    if (i > 0 && (digits.length - i) % 3 == 0) buffer.write('.');
    buffer.write(digits[i]);
  }
  return '$buffer $unit';
}

/// Tham số đặt tên: `kind` bắt buộc, `fareVnd` có thể thiếu, `unit` có mặc định.
String tripSummary(
  String id, {
  required ServiceKind kind,
  int? fareVnd,
  String unit = 'VND',
}) {
  final fare = fareVnd == null ? 'chưa có giá' : formatVnd(fareVnd, unit);
  return '$id | ${kind.label} | $fare';
}

/// Hàm mũi tên: thân chỉ là một biểu thức.
bool isLongTrip(double distanceKm) => distanceKm > 10;

/// Hàm có tên cùng chữ ký `bool Function(Trip)`, truyền được như tear-off.
bool isCancellable(Trip trip) => trip.status.canCancel;

/// Kiểu hàm: nhận một chuyến, trả bool.
typedef TripFilter = bool Function(Trip trip);

/// Hàm là giá trị: nhận một hàm khác làm tham số.
int countWhere(Iterable<Trip> trips, TripFilter test) {
  var count = 0;
  for (final trip in trips) {
    if (test(trip)) count++;
  }
  return count;
}
```

Ba chi tiết đáng nhìn trong file này:

1. `formatVnd(int vnd, [String unit = 'VND'])` gọi được là `formatVnd(453000)` hoặc `formatVnd(453000, 'đ')`.
2. `isLongTrip(double distanceKm) => distanceKm > 10;` là **hàm mũi tên**: `=>` thay cho `{ return ...; }` khi thân hàm chỉ là một biểu thức.
3. Hàm là **giá trị**. `TripFilter` là tên của kiểu `bool Function(Trip)`; `countWhere` nhận một hàm làm tham số. Ta truyền được lambda `(t) => t.status.isTerminal` hoặc một hàm có tên cùng chữ ký như `isCancellable` (*tear-off*: viết tên hàm, không kèm ngoặc). `isLongTrip` nhận `double` nên không truyền thẳng được, phải bọc `(t) => isLongTrip(t.distanceKm)`. Cơ chế này làm nền cho `where`, `map` ở bài 04 và `onPressed` của widget sau này.

## Class và constructor: vì sao Trip không có setter

Một chuyến có `id`, dịch vụ, trạng thái, khoảng cách, cước, tài xế (có thể chưa có) và thời điểm đặt. Mọi trường đều `final`: gán đúng một lần ở constructor. Dart không có từ khoá property; trường `final` chỉ có getter ngầm, nên thiếu setter là điều tự nhiên.

Ý tưởng giống [bài C# về encapsulation](/articles/csharp-class-property-encapsulation): không ghi thẳng vào trạng thái mà đi qua một hành vi giữ quy tắc. C# dùng `private set`; Dart bỏ hẳn setter và trả về đối tượng mới.

Phần khai báo của `lib/trip.dart`:

```dart
class Trip {
  Trip({
    required this.id,
    required this.serviceKind,
    required this.status,
    required this.distanceKm,
    required this.fareVnd,
    this.driverId,
    required this.requestedAt,
  }) : assert(requestedAt.isUtc, 'requestedAt phải là giờ UTC');

  /// Tạo chuyến mới ở trạng thái `requested`, tính cước bằng [quoteFor].
  Trip.request({
    required String id,
    required ServiceKind serviceKind,
    required double distanceKm,
    required DateTime requestedAt,
  }) : this(
         id: id,
         serviceKind: serviceKind,
         status: TripStatus.requested,
         distanceKm: distanceKm,
         fareVnd: quoteFor(distanceKm).fareVnd,
         requestedAt: requestedAt,
       );

  final String id;
  final ServiceKind serviceKind;
  final TripStatus status;
  final double distanceKm;
  final int fareVnd;
  final String? driverId;
  final DateTime requestedAt;
```

Mấy điểm cần đọc kỹ:

- **Initializing formal** `required this.id` vừa khai báo tham số vừa gán thẳng vào trường `id`. Không cần thân constructor.
- **Initializer list** sau dấu `:` chạy trước thân constructor. Ta dùng nó cho `assert(requestedAt.isUtc, ...)` vì thời điểm lưu và truyền đều ở UTC. `assert` chỉ chạy khi bật assert: `dart test`, `dart run --enable-asserts`, Flutter debug. `dart run` mặc định và bản release bỏ qua, nên đừng dùng nó để kiểm dữ liệu người dùng.
- **Named constructor** `Trip.request` cung cấp một cách tạo riêng: chuyến mới luôn ở `requested` và cước được tính bằng `quoteFor` (hàm ở mục 5). Constructor này **chuyển tiếp** sang constructor chính bằng `: this(...)` nên quy tắc `assert` vẫn được áp dụng.
- **`const` constructor** tạo đối tượng hằng: hai lần viết `const PickedUp()` cho ra *cùng một* đối tượng. Mọi trường phải `final` và biết lúc biên dịch. `Trip` không khai báo `const` constructor vì `assert(requestedAt.isUtc)` gọi getter, không phải biểu thức hằng; và `DateTime` không có `const` constructor nên đằng nào cũng không tạo được `Trip` hằng. Các sự kiện ở mục 4 thì có.

`final` chỉ ngăn gán lại biến; một `final List<String>` vẫn `add` được. Ở `Trip`, mọi trường đều là kiểu bất biến (`String`, `int`, `double`, enum, `DateTime`), nên cả đối tượng là bất biến thật sự. Muốn đổi trạng thái, ta gọi `apply` và nhận về **bản sao mới**; bản cũ giữ nguyên.

Phần còn lại của class là `==` và `hashCode`:

```dart
  /// Hai bản chụp của cùng một chuyến bằng nhau khi cùng id và cùng trạng thái.
  @override
  bool operator ==(Object other) =>
      other is Trip && other.id == id && other.status == status;

  @override
  int get hashCode => Object.hash(id, status);
```

Ta chọn so sánh theo **id và status**: hai bản chụp của cùng một chuyến ở cùng trạng thái được coi là bằng nhau, dù `driverId` khác nhau. Chọn "chỉ id" cũng hợp lệ nếu coi `Trip` là entity thuần; thêm `status` giúp phân biệt "cùng chuyến, trạng thái mới". Điều quan trọng là `==` và `hashCode` dựa trên **cùng** các trường.

## Enhanced enum: tám trạng thái có hành vi

Hồ sơ Tới Nơi có đúng tám trạng thái, và Hình 1 vẽ chúng cùng các sự kiện nối chúng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Máy trạng thái của chuyến: requested, searching, accepted, arriving, pickedUp, inTransit, completed nối bằng sáu sự kiện; Cancelled đi từ searching, accepted hoặc arriving tới cancelled."><defs><marker id="fl03-a1" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker><marker id="fl03-w1" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker></defs><rect x="11" y="60" width="102" height="44" rx="8" class="f-pk-soft s-pk"/><text x="62" y="79" text-anchor="middle" class="t-mono t-b">requested</text><text x="62" y="95" text-anchor="middle" class="t-small">đã đặt</text><rect x="137" y="60" width="102" height="44" rx="8" class="f-pk-soft s-pk"/><text x="188" y="79" text-anchor="middle" class="t-mono t-b">searching</text><text x="188" y="95" text-anchor="middle" class="t-small">tìm tài xế</text><rect x="263" y="60" width="102" height="44" rx="8" class="f-pk-soft s-pk"/><text x="314" y="79" text-anchor="middle" class="t-mono t-b">accepted</text><text x="314" y="95" text-anchor="middle" class="t-small">tài xế nhận</text><rect x="389" y="60" width="102" height="44" rx="8" class="f-pk-soft s-pk"/><text x="440" y="79" text-anchor="middle" class="t-mono t-b">arriving</text><text x="440" y="95" text-anchor="middle" class="t-small">đang tới đón</text><rect x="515" y="60" width="102" height="44" rx="8" class="f-pk-soft s-pk"/><text x="566" y="79" text-anchor="middle" class="t-mono t-b">pickedUp</text><text x="566" y="95" text-anchor="middle" class="t-small">đã đón / lấy</text><rect x="641" y="60" width="102" height="44" rx="8" class="f-pk-soft s-pk"/><text x="692" y="79" text-anchor="middle" class="t-mono t-b">inTransit</text><text x="692" y="95" text-anchor="middle" class="t-small">đang chở / giao</text><rect x="767" y="60" width="102" height="44" rx="8" class="f-phys-soft s-phys"/><text x="818" y="79" text-anchor="middle" class="t-mono t-b t-phys">completed</text><text x="818" y="95" text-anchor="middle" class="t-small">hoàn thành</text><line x1="113" y1="82" x2="135" y2="82" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a1)"/><line x1="239" y1="82" x2="261" y2="82" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a1)"/><line x1="365" y1="82" x2="387" y2="82" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a1)"/><line x1="491" y1="82" x2="513" y2="82" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a1)"/><line x1="617" y1="82" x2="639" y2="82" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a1)"/><line x1="743" y1="82" x2="765" y2="82" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a1)"/><text x="125" y="50" text-anchor="middle" class="t-small">SearchStarted</text><text x="251" y="50" text-anchor="middle" class="t-small">DriverAccepted</text><text x="377" y="50" text-anchor="middle" class="t-small">DriverArrived</text><text x="503" y="50" text-anchor="middle" class="t-small">PickedUp</text><text x="629" y="50" text-anchor="middle" class="t-small">TripStarted</text><text x="755" y="50" text-anchor="middle" class="t-small">Completed</text><rect x="330" y="150" width="220" height="50" rx="8" class="f-warn-soft s-warn"/><text x="440" y="171" text-anchor="middle" class="t-mono t-b t-warn">cancelled</text><text x="440" y="188" text-anchor="middle" class="t-small">đã huỷ (khách hoặc tài xế)</text><line x1="188" y1="104" x2="372" y2="148" class="s-warn dash" stroke-width="1.4" marker-end="url(#fl03-w1)"/><line x1="314" y1="104" x2="420" y2="148" class="s-warn dash" stroke-width="1.4" marker-end="url(#fl03-w1)"/><line x1="440" y1="104" x2="470" y2="148" class="s-warn dash" stroke-width="1.4" marker-end="url(#fl03-w1)"/><text x="572" y="146" class="t-small">Cancelled(by: customer | driver)</text><text x="572" y="162" class="t-small">chỉ từ searching, accepted, arriving</text><text x="11" y="150" class="t-small">Mũi tên đặc: sự kiện đi tiếp.</text><text x="11" y="166" class="t-small">Mũi tên đứt: huỷ chuyến.</text><text x="11" y="182" class="t-small">completed và cancelled là trạng thái cuối.</text></svg>
</div>
<figcaption>Hình 1. Tám trạng thái và bảy loại sự kiện. Sáu mũi tên đặc là đường đi bình thường; ba mũi tên đứt xuất phát từ ba trạng thái có <code>canCancel == true</code>. Mỗi mũi tên đặc là một sự kiện; <code>SearchStarted</code> và <code>TripStarted</code> là hai mũi tên mà hồ sơ không đặt tên.</figcaption>
</figure>

Với `String status = 'picked_up'`, một lỗi gõ `'pickedup'` vẫn biên dịch và chỉ lộ ra khi khách phàn nàn. **Enhanced enum** của Dart chặn điều đó và cho phép gắn dữ liệu cùng hành vi vào từng giá trị:

### trip_status.dart — 03-ham-class-sealed-trang-thai/lib/trip_status.dart

```dart
/// Tám trạng thái của một chuyến, theo hồ sơ Tới Nơi.
enum TripStatus {
  requested('Đã đặt'),
  searching('Đang tìm tài xế'),
  accepted('Tài xế đã nhận'),
  arriving('Tài xế đang tới'),
  pickedUp('Đã đón / lấy hàng'),
  inTransit('Đang chở / giao'),
  completed('Hoàn thành'),
  cancelled('Đã huỷ');

  const TripStatus(this.label);

  /// Nhãn tiếng Việt để hiển thị.
  final String label;

  /// Khách hoặc tài xế chỉ được huỷ ở ba trạng thái này.
  ///
  /// Liệt kê đủ tám trạng thái, không dùng `_`: thêm trạng thái mới mà chưa
  /// quyết định huỷ được hay không sẽ khiến `dart analyze` báo lỗi.
  bool get canCancel => switch (this) {
    searching || accepted || arriving => true,
    requested || pickedUp || inTransit || completed || cancelled => false,
  };

  /// Trạng thái cuối: không còn sự kiện nào hợp lệ.
  bool get isTerminal => switch (this) {
    completed || cancelled => true,
    requested ||
    searching ||
    accepted ||
    arriving ||
    pickedUp ||
    inTransit => false,
  };
}
```

- Mỗi giá trị gọi một `const` constructor, ở đây truyền nhãn tiếng Việt. `label` là trường `final`; `canCancel` và `isTerminal` là getter, nên hành vi nằm cạnh dữ liệu thay vì rải rác trong các `if` ở UI.
- `TripStatus.values` trả về danh sách mọi giá trị theo thứ tự khai báo; `status.name` trả tên chuỗi (`'pickedUp'`). Dùng `name` khi lưu hoặc gửi qua mạng; server và database Tới Nơi dùng snake_case (`'picked_up'`, `'in_transit'`), nên cần ánh xạ ở bài dữ liệu sau.
- `switch (this)` trên enum cũng vét cạn. Hai getter ở trên cố ý liệt kê đủ tám giá trị thay vì dùng `_`: thêm trạng thái thứ chín, compiler buộc ta quyết định nó có huỷ được hay không. Một nhánh `_ => false` sẽ tự trả lời thay ta mà không báo gì.

`ServiceKind` ở `lib/service_kind.dart` làm tương tự cho ba dịch vụ `ride`, `food`, `parcel`, mỗi dịch vụ có `label` riêng.

## Sealed class và switch vét cạn

Mỗi sự kiện mang dữ liệu khác nhau: `DriverAccepted` cần `driverId`, `Cancelled` cần biết ai huỷ, `PickedUp` không mang gì. Enum không làm được vì mọi giá trị cùng một hình dạng. Ta cần một họ kiểu có dữ liệu riêng cho từng biến thể, và compiler phải biết **toàn bộ** họ: đó là `sealed class`.

### trip_event.dart — 03-ham-class-sealed-trang-thai/lib/trip_event.dart

```dart
/// Ai là người huỷ chuyến.
enum CancelledBy { customer, driver }

/// Sự kiện làm chuyến đổi trạng thái. `sealed` để compiler biết đủ các biến thể.
sealed class TripEvent {
  const TripEvent();
}

/// Hệ thống bắt đầu tìm tài xế.
final class SearchStarted extends TripEvent {
  const SearchStarted();
}

/// Một tài xế nhận chuyến.
final class DriverAccepted extends TripEvent {
  const DriverAccepted(this.driverId);

  final String driverId;
}

/// Tài xế báo đang tới điểm đón.
final class DriverArrived extends TripEvent {
  const DriverArrived();
}

/// Khách lên xe hoặc tài xế đã lấy hàng.
final class PickedUp extends TripEvent {
  const PickedUp();
}

/// Xe bắt đầu chở khách hoặc giao hàng tới điểm đến.
final class TripStarted extends TripEvent {
  const TripStarted();
}

/// Chuyến kết thúc thành công.
final class Completed extends TripEvent {
  const Completed();
}

/// Chuyến bị huỷ bởi khách hoặc tài xế.
final class Cancelled extends TripEvent {
  const Cancelled({required this.by});

  final CancelledBy by;
}

/// Mô tả một sự kiện bằng tiếng Việt.
///
/// Không có nhánh `_`: thêm một lớp con mới vào [TripEvent] mà quên sửa hàm
/// này sẽ khiến `dart analyze` báo lỗi.
String describeEvent(TripEvent event) => switch (event) {
  SearchStarted() => 'Bắt đầu tìm tài xế',
  DriverAccepted(:final driverId) => 'Tài xế $driverId nhận chuyến',
  DriverArrived() => 'Tài xế đang tới điểm đón',
  PickedUp() => 'Đã đón khách hoặc lấy hàng',
  TripStarted() => 'Bắt đầu chở hoặc giao',
  Completed() => 'Hoàn thành chuyến',
  Cancelled(:final by) => switch (by) {
    CancelledBy.customer => 'Khách huỷ chuyến',
    CancelledBy.driver => 'Tài xế huỷ chuyến',
  },
};
```

`sealed` nghĩa là lớp con trực tiếp phải cùng library với lớp cha (thường là cùng file, trừ khi dùng `part`), nên compiler đếm được đủ bảy lớp con. Lớp `sealed` ngầm là `abstract`; `final` ở mỗi biến thể cấm `extends` và `implements` từ ngoài library.

Hàm `describeEvent` ở cuối file là mẫu cho **switch biểu thức vét cạn**. Hàm này không có `_`. Nếu thử thêm `final class Rated extends TripEvent` mà chưa sửa hàm, `dart analyze` sẽ báo:

```text
error - trip_event.dart:52:42 - The type 'TripEvent' isn't exhaustively matched by the switch cases
since it doesn't match the pattern 'Rated()'. - non_exhaustive_switch_expression
```

Mẫu `DriverAccepted(:final driverId)` là **object pattern**: kiểm kiểu rồi lấy `driverId` ra biến cục bộ cùng tên. `Cancelled(:final by)` lại `switch` tiếp trên enum `CancelledBy`, cũng không có `_`.

Giờ tới `Trip.apply`. Điều kiện hợp lệ phụ thuộc vào **cặp** (trạng thái, sự kiện), nên ta switch trên một **record** gồm cả hai:

```dart
  /// Áp dụng [event] và trả về chuyến mới; chuyến hiện tại không bị sửa.
  ///
  /// Ném [StateError] khi cặp (trạng thái, sự kiện) không hợp lệ.
  Trip apply(TripEvent event) => switch ((status, event)) {
    (TripStatus.requested, SearchStarted()) => _next(TripStatus.searching),
    (TripStatus.searching, DriverAccepted(:final driverId))
        when driverId.trim().isEmpty =>
      throw ArgumentError.value(driverId, 'driverId', 'không được để trống'),
    (TripStatus.searching, DriverAccepted(:final driverId)) => _next(
      TripStatus.accepted,
      driverId: driverId,
    ),
    (TripStatus.accepted, DriverArrived()) => _next(TripStatus.arriving),
    (TripStatus.arriving, PickedUp()) => _next(TripStatus.pickedUp),
    (TripStatus.pickedUp, TripStarted()) => _next(TripStatus.inTransit),
    (TripStatus.inTransit, Completed()) => _next(TripStatus.completed),
    (
      TripStatus.searching || TripStatus.accepted || TripStatus.arriving,
      Cancelled(),
    ) =>
      _next(TripStatus.cancelled),
    _ => throw StateError(
      'Không thể áp dụng ${event.runtimeType} khi chuyến $id ở '
      'trạng thái ${status.name}',
    ),
  };
```

Cách đọc:

- `switch ((status, event))`: ngoặc ngoài là cú pháp `switch`, ngoặc trong tạo record. Mỗi nhánh là một **record pattern** như `(TripStatus.requested, SearchStarted())`: phần đầu khớp một hằng enum, phần sau khớp một lớp con.
- Nhánh `Cancelled()` dùng **or-pattern**: ba trạng thái chung một nhánh huỷ, và ta không cần `by` vì ai huỷ cũng sang `cancelled`.
- `when driverId.trim().isEmpty` là **guard**: nhánh chỉ khớp khi điều kiện đúng. Nhánh có guard đứng **trước** nhánh `DriverAccepted` thường vì Dart chọn nhánh khớp đầu tiên từ trên xuống. Đảo thứ tự thì nhánh guard không bao giờ được chạy.

Nhánh cuối `_ => throw StateError(...)` bắt 47 trên 56 cặp (trạng thái, loại sự kiện) không hợp lệ. Đây là chỗ ta **cố ý** dùng `_`, khác với lỗi nhánh nuốt biến thể mới: mặc định ở đây là **từ chối**. Liệt kê hết 47 cặp thì code dài gấp ba mà không an toàn hơn. Cái giá: có `_` thì compiler không còn kiểm được ta đã nghĩ tới mọi cặp. Ta bù bằng bảng test duyệt **mọi** cặp (mục "Chạy ví dụ và kiểm chứng").

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. switch trên record (status, event): chín cặp hợp lệ khớp từng nhánh và tạo bản sao Trip mới; mọi cặp còn lại rơi vào nhánh cuối và ném StateError."><defs><marker id="fl03-a2" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker><marker id="fl03-w2" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker></defs><rect x="11" y="95" width="170" height="130" rx="10" class="f-phys-soft s-phys"/><text x="96" y="130" text-anchor="middle" class="t-b">(status, event)</text><text x="96" y="152" text-anchor="middle" class="t-small">record gồm hai phần</text><text x="96" y="172" text-anchor="middle" class="t-small">8 trạng thái</text><text x="96" y="188" text-anchor="middle" class="t-small">x 7 loại sự kiện</text><text x="96" y="208" text-anchor="middle" class="t-small">= 56 cặp</text><line x1="181" y1="160" x2="214" y2="160" class="s-muted" stroke-width="1.4"/><line x1="214" y1="34" x2="214" y2="286" class="s-muted" stroke-width="1.4"/><line x1="214" y1="34" x2="230" y2="34" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a2)"/><line x1="214" y1="70" x2="230" y2="70" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a2)"/><line x1="214" y1="106" x2="230" y2="106" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a2)"/><line x1="214" y1="142" x2="230" y2="142" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a2)"/><line x1="214" y1="178" x2="230" y2="178" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a2)"/><line x1="214" y1="214" x2="230" y2="214" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a2)"/><line x1="214" y1="250" x2="230" y2="250" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a2)"/><line x1="214" y1="286" x2="230" y2="286" class="s-warn" stroke-width="1.6" marker-end="url(#fl03-w2)"/><rect x="232" y="20" width="368" height="28" rx="6" class="f-pk-soft s-pk"/><text x="244" y="38" class="t-mono">(requested, SearchStarted())</text><rect x="232" y="56" width="368" height="28" rx="6" class="f-pk-soft s-pk"/><text x="244" y="74" class="t-mono">(searching, DriverAccepted(:final driverId))</text><rect x="232" y="92" width="368" height="28" rx="6" class="f-pk-soft s-pk"/><text x="244" y="110" class="t-mono">(accepted, DriverArrived())</text><rect x="232" y="128" width="368" height="28" rx="6" class="f-pk-soft s-pk"/><text x="244" y="146" class="t-mono">(arriving, PickedUp())</text><rect x="232" y="164" width="368" height="28" rx="6" class="f-pk-soft s-pk"/><text x="244" y="182" class="t-mono">(pickedUp, TripStarted())</text><rect x="232" y="200" width="368" height="28" rx="6" class="f-pk-soft s-pk"/><text x="244" y="218" class="t-mono">(inTransit, Completed())</text><rect x="232" y="236" width="368" height="28" rx="6" class="f-pk-soft s-pk"/><text x="244" y="254" class="t-mono">(searching || accepted || arriving, Cancelled())</text><rect x="232" y="272" width="368" height="28" rx="6" class="f-warn-soft s-warn"/><rect x="232" y="272" width="8" height="28" rx="3" class="f-warn"/><text x="252" y="290" class="t-mono t-warn">_  (mọi cặp còn lại)</text><line x1="600" y1="34" x2="636" y2="34" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a2)"/><line x1="600" y1="70" x2="636" y2="70" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a2)"/><line x1="600" y1="106" x2="636" y2="106" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a2)"/><line x1="600" y1="142" x2="636" y2="142" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a2)"/><line x1="600" y1="178" x2="636" y2="178" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a2)"/><line x1="600" y1="214" x2="636" y2="214" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a2)"/><line x1="600" y1="250" x2="636" y2="250" class="s-pk" stroke-width="1.6" marker-end="url(#fl03-a2)"/><line x1="600" y1="286" x2="636" y2="286" class="s-warn" stroke-width="1.6" marker-end="url(#fl03-w2)"/><rect x="638" y="20" width="231" height="28" rx="6" class="f-phys-soft s-phys"/><text x="650" y="38" class="t-mono">Trip(status: searching)</text><rect x="638" y="56" width="231" height="28" rx="6" class="f-phys-soft s-phys"/><text x="650" y="74" class="t-mono">Trip(accepted, driverId)</text><rect x="638" y="92" width="231" height="28" rx="6" class="f-phys-soft s-phys"/><text x="650" y="110" class="t-mono">Trip(status: arriving)</text><rect x="638" y="128" width="231" height="28" rx="6" class="f-phys-soft s-phys"/><text x="650" y="146" class="t-mono">Trip(status: pickedUp)</text><rect x="638" y="164" width="231" height="28" rx="6" class="f-phys-soft s-phys"/><text x="650" y="182" class="t-mono">Trip(status: inTransit)</text><rect x="638" y="200" width="231" height="28" rx="6" class="f-phys-soft s-phys"/><text x="650" y="218" class="t-mono">Trip(status: completed)</text><rect x="638" y="236" width="231" height="28" rx="6" class="f-phys-soft s-phys"/><text x="650" y="254" class="t-mono">Trip(status: cancelled)</text><rect x="638" y="272" width="231" height="28" rx="6" class="f-warn-soft s-warn"/><rect x="638" y="272" width="8" height="28" rx="3" class="f-warn"/><text x="658" y="290" class="t-mono t-warn">throw StateError</text></svg>
</div>
<figcaption>Hình 2. Các nhánh của <code>apply</code> được thử từ trên xuống, nhánh khớp đầu tiên thắng. Hình lược bỏ guard <code>when</code>. Bảy nhánh đầu cho ra một bản sao <code>Trip</code> mới; nhánh <code>_</code> (màu cảnh báo) ném <code>StateError</code> cho 47 cặp còn lại.</figcaption>
</figure>

Hàm `_next` ở cuối class chỉ làm một việc là dựng bản sao: giữ nguyên mọi trường, đổi `status` và (nếu có) `driverId`. Mọi thay đổi trạng thái đi qua `apply`, nên quy tắc không thể bị vòng qua. `Cancelled` mang `by` nhưng `Trip` không có trường ghi ai đã huỷ; thông tin đó thuộc nhật ký sự kiện, sẽ quay lại ở bài máy trạng thái trên client.

## Record: trả nhiều giá trị và so sánh theo cấu trúc

Ta vừa dùng record làm "phiếu" cho `switch`. Công dụng đầu tiên của record là để hàm **trả nhiều giá trị** mà không phải tạo class chỉ để chứa chúng:

Toàn bộ `lib/quote.dart`:

```dart
import 'dart:math' as math;

const int _baseFareVnd = 12000;
const int _includedMeters = 2000;
const int _pricePerExtraKmVnd = 4500;
const double _maxDistanceKm = 100;

/// Báo giá minh hoạ: trả một record đặt tên thay vì tạo class riêng.
///
/// 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt (số minh hoạ, không phải
/// bảng giá thật). Ném [ArgumentError] khi [distanceKm] ngoài miền 0 đến 100.
({int fareVnd, double distanceKm}) quoteFor(double distanceKm) {
  if (distanceKm < 0 || distanceKm > _maxDistanceKm) {
    throw ArgumentError.value(
      distanceKm,
      'distanceKm',
      'phải nằm trong khoảng 0 đến 100 km',
    );
  }
  // Tính bằng mét nguyên để tránh sai số số thực; cộng 500 rồi chia nguyên
  // cho 1000 là làm tròn nửa đồng lên.
  final meters = (distanceKm * 1000).round();
  final extraMeters = math.max(0, meters - _includedMeters);
  final fare = _baseFareVnd + (extraMeters * _pricePerExtraKmVnd + 500) ~/ 1000;
  return (fareVnd: fare, distanceKm: distanceKm);
}
```

Kiểu trả về `({int fareVnd, double distanceKm})` là record có hai trường đặt tên; `return (fareVnd: fare, distanceKm: distanceKm);` dựng nó. Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) là **số minh hoạ**, không phải bảng giá thật của Tới Nơi. Cước tính bằng số nguyên mét rồi chia nguyên để tránh sai số số thực; đây là dạng sẽ dùng trong `toi_noi_core` ở bài 06. Người gọi lấy từng trường bằng `quoteFor(5).fareVnd`, hoặc **destructuring**:

```dart
final (:fareVnd, :distanceKm) = quoteFor(5);
print(fareVnd); // 25500
```

Cú pháp `:fareVnd` nghĩa là "lấy trường `fareVnd` ra biến cục bộ cùng tên". Record không tên trường (như `(TripStatus.searching, 'driverAccepted')`) destructure theo vị trí: `final (status, name) = pair;`.

Record còn có **equality theo cấu trúc**: hai record bằng nhau khi cùng hình dạng và mọi trường bằng nhau, thứ tự khai báo tên trường không ảnh hưởng. Vì vậy `quoteFor(5) == (fareVnd: 25500, distanceKm: 5.0)` là `true`, và record dùng được làm khoá của `Map` mà không cần tự viết `==`. Bên C#, [bài record, struct và Money](/articles/csharp-record-struct-money-quote) bàn cùng ý tưởng value equality.

Record hợp với dữ liệu ngắn hạn, không có quy tắc đi kèm: kết quả trả về, khoá ghép, cặp để `switch`. Khi dữ liệu có invariant hay hành vi, như `Trip`, hãy dùng class.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Đọc kỹ trước khi sửa</span>
<p>Cả bốn lỗi dưới đây đều biên dịch và chạy được; chúng chỉ lộ ra khi có người thêm trạng thái mới hoặc ghi thẳng vào chuyến.</p>
</div>

1. **Nhánh `_` nuốt biến thể mới.** Dấu hiệu: thêm sự kiện hay trạng thái mới, `dart analyze` im lặng nhưng UI hiện "Không rõ". Cách sửa: bỏ `_` và liệt kê đủ; khi buộc phải dùng như ở `apply`, cho nó **từ chối** (ném lỗi) và có bảng test phủ mọi cặp.
2. **Trường có thể gán lại.** Dấu hiệu: một widget viết `trip.status = TripStatus.pickedUp` và bỏ qua `apply`, nên chuyến đã huỷ vẫn chuyển được. Cách sửa: khai báo `final`, chỉ đổi trạng thái qua `apply`. Nhớ `final List` vẫn sửa được nội dung; dùng `List.unmodifiable` khi cần.
3. **So sánh enum bằng `index` hoặc chuỗi.** Dấu hiệu: điều kiện kiểu `status.index >= TripStatus.pickedUp.index` đúng cho cả `completed` lẫn `cancelled`, và vỡ khi ai đó chèn một trạng thái vào giữa. Lưu `index` xuống database cũng hỏng theo. Cách sửa: dùng getter nghiệp vụ như `canCancel`, `isTerminal`; lưu và gửi bằng `name`.
4. **Ghi đè `==` mà quên `hashCode`.** Dấu hiệu: hai `Trip` bằng nhau nhưng cùng nằm trong một `Set`, hoặc `Map` không tìm lại được khoá. Cách sửa: ghi đè cả hai, cùng dựa trên đúng các trường đó. Lint `hash_and_equals` bắt lỗi này.

## Chạy ví dụ và kiểm chứng

Lab nằm ở `examples/toi-noi-flutter/03-ham-class-sealed-trang-thai/`. Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/03-ham-class-sealed-trang-thai
dart pub get
dart run bin/main.dart
dart analyze
dart test
```

`bin/main.dart` chạy một chuyến tới `completed`, một chuyến bị khách huỷ ở `arriving`, rồi thử gửi `PickedUp` vào chuyến đã huỷ:

```text
Đã đặt: Trip(TN-1001, ride, requested, 5.0 km, 25500 VND, driver: -)
Bắt đầu tìm tài xế -> Đang tìm tài xế (searching)
Tài xế D-17 nhận chuyến -> Tài xế đã nhận (accepted)
...
Hoàn thành chuyến -> Hoàn thành (completed)
...
Khách huỷ chuyến -> Đã huỷ
Từ chối: Không thể áp dụng PickedUp khi chuyến TN-1002 ở trạng thái cancelled
```

Test nằm ở `test/trip_test.dart`. Phần quan trọng nhất là **bảng đặc tả** chỉ liệt kê các cặp hợp lệ, rồi duyệt toàn bộ 8 trạng thái x 8 sự kiện mẫu (`Cancelled` thử với cả `customer` và `driver`):

```dart
/// Mỗi sự kiện mẫu có một tên ngắn để làm khoá của bảng kiểm.
const Map<String, TripEvent> _events = {
  'searchStarted': SearchStarted(),
  'driverAccepted': DriverAccepted('D-17'),
  'driverArrived': DriverArrived(),
  'pickedUp': PickedUp(),
  'tripStarted': TripStarted(),
  'completed': Completed(),
  'cancelledByCustomer': Cancelled(by: CancelledBy.customer),
  'cancelledByDriver': Cancelled(by: CancelledBy.driver),
};

/// Bảng đặc tả: chỉ liệt kê các cặp hợp lệ. Mọi cặp còn lại phải ném StateError.
/// Khoá là record (trạng thái, tên sự kiện), so sánh theo cấu trúc.
const Map<(TripStatus, String), TripStatus> _valid = {
  (TripStatus.requested, 'searchStarted'): TripStatus.searching,
  (TripStatus.searching, 'driverAccepted'): TripStatus.accepted,
  (TripStatus.accepted, 'driverArrived'): TripStatus.arriving,
  (TripStatus.arriving, 'pickedUp'): TripStatus.pickedUp,
  (TripStatus.pickedUp, 'tripStarted'): TripStatus.inTransit,
  (TripStatus.inTransit, 'completed'): TripStatus.completed,
  (TripStatus.searching, 'cancelledByCustomer'): TripStatus.cancelled,
  (TripStatus.accepted, 'cancelledByCustomer'): TripStatus.cancelled,
  (TripStatus.arriving, 'cancelledByCustomer'): TripStatus.cancelled,
  (TripStatus.searching, 'cancelledByDriver'): TripStatus.cancelled,
  (TripStatus.accepted, 'cancelledByDriver'): TripStatus.cancelled,
  (TripStatus.arriving, 'cancelledByDriver'): TripStatus.cancelled,
};

void main() {
  group('bảng trạng thái x sự kiện', () {
    test('bảng phủ đủ 8 trạng thái x 8 sự kiện mẫu, 12 cặp hợp lệ', () {
      expect(TripStatus.values, hasLength(8));
      expect(_events, hasLength(8));
      expect(_valid, hasLength(12));
    });

    for (final status in TripStatus.values) {
      for (final entry in _events.entries) {
        final expected = _valid[(status, entry.key)];
        if (expected != null) {
          test('${status.name} + ${entry.key} -> ${expected.name}', () {
            final next = _tripAt(status).apply(entry.value);
            expect(next.status, expected);
          });
        } else {
          test('${status.name} + ${entry.key} ném StateError', () {
            expect(
              () => _tripAt(status).apply(entry.value),
              throwsA(isA<StateError>()),
            );
          });
        }
```

Vòng lặp sinh 64 test: 12 cặp hợp lệ phải ra đúng trạng thái đích, 52 cặp còn lại phải ném `StateError`. 32 test còn lại (tổng 96) kiểm: thông báo lỗi; `driverId` rỗng là `ArgumentError`; bản gốc không đổi sau `apply`, kể cả khi bị từ chối; `requestedAt` không UTC bị `assert` chặn; `==` theo id và status; `canCancel`, `isTerminal`; record; các giá trị biên 0, 1,999, 2, 2,001, 100 km và ngoài miền; các hàm của `functions_tour.dart`.

Mức kiểm chứng: **Kiểm chứng tự động**. `dart analyze` báo 0 issue, `dart test` chạy xanh 96 test trên Flutter 3.47.4 / Dart 3.13.3. Lab chưa làm: lưu chuyến vào database, đồng bộ với server và UI; các phần đó thuộc những bài sau.

## Bài tập

1. Thêm `final class Rated extends TripEvent` (khách chấm sao sau khi hoàn thành). `dart analyze` báo lỗi ở file nào và vì sao `Trip.apply` không báo lỗi?
2. Cho phép huỷ ngay ở `requested`. Cần sửa những chỗ nào trong code, và test nào sẽ đỏ trước khi ta sửa bảng đặc tả?
3. `(fareVnd: 25500, distanceKm: 5.0) == (distanceKm: 5.0, fareVnd: 25500)` cho kết quả gì? Vì sao?

<details><summary>Xem lời giải</summary><p>1. Lỗi <code>non_exhaustive_switch_expression</code> xuất hiện ở <code>describeEvent</code> trong <code>trip_event.dart</code> vì hàm đó vét cạn. <code>Trip.apply</code> không báo vì nhánh <code>_</code> nhận mọi cặp, nên cặp có <code>Rated</code> sẽ ném <code>StateError</code> lúc chạy. 2. Sửa <code>canCancel</code> (chuyển <code>requested</code> sang nhóm <code>true</code>), thêm <code>TripStatus.requested</code> vào or-pattern của nhánh <code>Cancelled</code> trong <code>apply</code>, và thêm hai cặp <code>(requested, cancelledByCustomer/Driver)</code> vào bảng <code>_valid</code>. Trước khi sửa bảng, hai test <code>requested + cancelledBy...</code> đỏ, cùng với "canCancel chỉ đúng với searching, accepted, arriving" và "canCancel khớp với bảng" vì hai nguồn không còn khớp nhau. 3. <code>true</code>: record so sánh theo cấu trúc, tên trường và giá trị quyết định, thứ tự viết trường thì không.</p></details>

Bài trước: [Dart cơ bản: biến, kiểu dữ liệu và null safety](/articles/dart-bien-kieu-du-lieu-null-safety-tinh-cuoc). Bài tiếp: [Collections, generics và extension: quản lý danh sách chuyến và tài xế](/articles/dart-collections-generics-extension-danh-sach-chuyen).

## Tài liệu gốc

- [Dart: Functions](https://dart.dev/language/functions).
- [Dart: Classes](https://dart.dev/language/classes) và [Constructors](https://dart.dev/language/constructors).
- [Dart: Enumerated types](https://dart.dev/language/enums).
- [Dart: Class modifiers (sealed, final)](https://dart.dev/language/class-modifiers).
- [Dart: Patterns](https://dart.dev/language/patterns) và [Pattern types](https://dart.dev/language/pattern-types).
- [Dart: Records](https://dart.dev/language/records).
- [Dart: Branches (switch)](https://dart.dev/language/branches) và [Error handling](https://dart.dev/language/error-handling).
- [pub.dev: package lints](https://pub.dev/packages/lints) và [package test](https://pub.dev/packages/test).
