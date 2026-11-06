---
slug: dart-package-pubspec-lint-test-toi-noi-core
title: "Package, pubspec, lint và test: thư viện toi_noi_core kiểm chứng được"
summary: "Bài 06: gom tiền, cước, trạng thái chuyến và sự kiện của bài 02–05 thành package Dart toi_noi_core; đọc pubspec.yaml, dependency và version constraint, bật lint strict, tổ chức test theo group và dùng package này từ một project khác bằng path dependency."
category: flutter
level: beginner
tags: [flutter, dart, mobile, toi-noi, package, testing]
language: vi
---

Đội Tới Nơi chuẩn bị làm app tài xế. Màn hình đầu tiên của app này cũng báo giá chuyến, và con số trên máy tài xế phải khớp từng đồng với con số trên máy khách. Nhìn lại các lab bài 02 đến 05, ta thấy `TripStatus` nằm trong hai thư mục (lab 03 và 04), `Trip` có hai phiên bản khác nhau (bản đầy đủ của lab 03 và bản rút gọn của lab 04), còn công thức cước thì có cả dạng `double` lẫn dạng mét nguyên. Chép thêm một lần nữa cho app tài xế là cách nhanh nhất để hai app tính giá lệch nhau.

Lối ra là **package**: một đơn vị Dart có `pubspec.yaml`, có API công khai rõ ràng, có test riêng, và nhiều project cùng dùng. Bài này gom code của bốn bài trước thành package `toi_noi_core`, đọc `pubspec.yaml`, bật lint chặt hơn, viết test có tổ chức, rồi dùng package từ một project khác. Đây là bài 06 của lộ trình **Flutter qua Tới Nơi**, nối tiếp [bài 05 về Future và Stream](/articles/dart-future-async-await-stream-bao-gia-va-vi-tri), và là bài cuối của Phần 1. Ví dụ lấy từ hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi), bảng giá là số minh hoạ. Bạn đã đọc bài C# về [record, struct và Money](/articles/csharp-record-struct-money-quote) sẽ thấy cùng ý tưởng value object, chỉ khác cú pháp.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một project khác viết <code>import 'package:toi_noi_core/src/money.dart';</code> để dùng thẳng class <code>Money</code>. Dart có cho chạy không? Và ta có nên làm vậy không?</p>
<details><summary>Xem đáp án</summary><p>Dart vẫn chạy được, vì <code>src</code> chỉ là một quy ước thư mục chứ không phải rào chắn của ngôn ngữ. Nhưng đó là import vào phần riêng của package: tác giả có quyền đổi tên file hay tách <code>Money</code> ra bất cứ lúc nào mà không coi là thay đổi phá vỡ. Lint <code>implementation_imports</code> trong bộ <code>recommended</code> báo đúng chỗ này. Hãy import file export công khai.</p></details>
</div>

## Vì sao cần package

Một lab là ví dụ để học, còn một package là thứ để người khác phụ thuộc vào: có tên và phiên bản, có API công khai tách khỏi phần cài đặt, có test đi kèm, và khai báo rõ nó cần gì từ bên ngoài.

Trong `toi_noi_core`, mỗi mảnh đến từ một bài trước và được rà lại cho thống nhất:

| Mảnh | Từ bài | Thay đổi khi vào package |
|---|---|---|
| `ServiceKind`, `TripStatus`, `TripEvent`, `Trip` | 03 | `Trip.fare` kiểu `Money` thay cho `int fareVnd`; `Trip` kiểm `driverId` khớp trạng thái; đổi `==` sang so mọi trường (lý do ở mục sau) |
| Công thức cước | 02 | Dạng chuẩn tính bằng mét nguyên, thêm `FarePolicy` đặt tên cho các hằng |
| `TripIterableX` | 04 | `totalFareVnd` đổi thành `totalFare` trả `Money`; bỏ `allSettled`, `priciest`, `sortedByFareDesc` |
| `Money` | mới | Bọc `int vnd` để cộng, nhân, so sánh và in ra `25.500 VND` |

Mỗi khái niệm giờ có một nơi định nghĩa: sửa bảng giá một chỗ, test của package báo ngay nếu con số đổi.

## Đọc pubspec.yaml

Đây là `pubspec.yaml` của package:

### pubspec.yaml · packages/toi_noi_core/pubspec.yaml

```yaml
name: toi_noi_core
description: "Thư viện domain dùng chung của Tới Nơi: Money, cước, trạng thái chuyến, sự kiện và Trip bất biến (Flutter qua Tới Nơi, bài 06)."
version: 0.1.0
publish_to: none

environment:
  sdk: ^3.13.3

dependencies:
  collection: ^1.19.1

dev_dependencies:
  lints: ^6.1.0
  test: ^1.33.0
```

Từng khoá có việc riêng:

- `name` là tên dùng trong `import 'package:toi_noi_core/...'`; chữ thường, gạch dưới.
- `version: 0.1.0` theo semantic versioning. Số `0.x` báo hiệu API chưa ổn định.
- `publish_to: none` chặn `dart pub publish` nhầm lên pub.dev. Đây là package nội bộ.
- `environment.sdk` là **ràng buộc phiên bản SDK**. `^3.13.3` nghĩa là `>=3.13.3 <4.0.0`. Dart từ chối chạy package nếu SDK nằm ngoài khoảng đó, nên ta không đặt cao hơn phiên bản đã kiểm.
- `dependencies` là thứ mã trong `lib/` cần lúc chạy. Ở đây là `collection`, vì `byStatus` dùng `groupBy`.
- `dev_dependencies` là thứ chỉ cần khi phát triển: `test` để chạy test, `lints` để phân tích. Người dùng package **không** nhận chúng.

Dấu `^` (caret) là cách viết gọn của "bản này trở lên, nhưng chưa tới bản major kế tiếp":

| Viết | Nghĩa | Ghi chú |
|---|---|---|
| `^1.19.1` | `>=1.19.1 <2.0.0` | nhận bản vá và bản minor mới |
| `^0.3.2` | `>=0.3.2 <0.4.0` | với `0.x`, bản minor được coi là phá vỡ |
| `'>=1.2.0 <1.5.0'` | khoảng tự viết | dùng khi cần chặn chặt hơn |
| `any` | mọi phiên bản | tránh dùng |

`dart pub get` chọn phiên bản cụ thể thoả mọi ràng buộc trong cây phụ thuộc, tải về và ghi **chính xác** từng phiên bản vào `pubspec.lock`. **App** commit file này để mọi máy build ra cùng một bản; **package và lab** thì không, vì người dùng package tự giải phiên bản theo ràng buộc của họ. Vì vậy `pubspec.lock` nằm trong `.gitignore` của lab và chỉ được giữ từ `toi-noi-app`.

Các lệnh hay dùng: `dart pub outdated` cho biết package nào có bản mới hơn; `dart pub upgrade` nâng lên bản mới nhất mà ràng buộc cho phép; `dart pub upgrade --major-versions` còn sửa cả ràng buộc để vượt major; `dart pub add <tên>` thêm một dòng vào `dependencies`.

## Bố cục package: lib công khai, lib/src riêng

Quy ước của Dart chia `lib/` thành hai vùng (Hình 1). Mọi thứ trực tiếp trong `lib/` là **API công khai**. Mọi thứ trong `lib/src/` là **chi tiết riêng**, và người dùng không nên import trực tiếp. Cầu nối là file `lib/toi_noi_core.dart`, chỉ chứa các dòng `export`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Project khác import toi_noi_core.dart trong lib. File này export các mảnh từ lib/src. Import thẳng vào lib/src chạy được nhưng lint implementation_imports báo."><defs><marker id="fl06-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl06-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="40" width="200" height="110" rx="10" class="f-phys-soft s-phys"/><text x="36" y="76" class="t-b">Project khác</text><text x="36" y="104" class="t-small">06-dung-toi-noi-core</text><text x="36" y="128" class="t-small">hoặc app tài xế</text>
<rect x="320" y="40" width="240" height="110" rx="10" class="f-pk-soft s-pk"/><text x="336" y="76" class="t-b">lib/ (công khai)</text><text x="336" y="104" class="t-mono">toi_noi_core.dart</text><text x="336" y="128" class="t-small">chỉ có dòng export</text>
<rect x="640" y="20" width="220" height="300" rx="10" class="f-phys-soft s-phys"/><text x="656" y="56" class="t-b">lib/src/ (riêng)</text><text x="656" y="94" class="t-mono">money.dart</text><text x="656" y="122" class="t-mono">fare.dart</text><text x="656" y="150" class="t-mono">service_kind.dart</text><text x="656" y="178" class="t-mono">trip_status.dart</text><text x="656" y="206" class="t-mono">trip_event.dart</text><text x="656" y="234" class="t-mono">trip.dart</text><text x="656" y="262" class="t-mono">trip_queries.dart</text><text x="656" y="298" class="t-small">đổi tên, tách file: tự do</text>
<path d="M222 95 L318 95" class="f-none s-pk" stroke-width="2" marker-end="url(#fl06-1-a)"/><text x="228" y="82" class="t-small">import</text>
<path d="M562 95 L638 95" class="f-none s-pk" stroke-width="2" marker-end="url(#fl06-1-a)"/><text x="568" y="82" class="t-small">export</text>
<path d="M120 152 L120 250 L636 250" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl06-1-w)"/><text x="140" y="238" class="t-small t-warn">import thẳng vào src: chạy được, nhưng lint báo</text>
<text x="140" y="276" class="t-mono t-warn">implementation_imports</text>
</svg>
</div>
<figcaption>Hình 1. Ranh giới công khai và riêng của package. Chỉ file trong lib được export là hợp đồng với người dùng; lib/src có thể đổi mà không báo trước.</figcaption>
</figure>

### toi_noi_core.dart · packages/toi_noi_core/lib/toi_noi_core.dart

```dart
/// Thư viện domain dùng chung của Tới Nơi: tiền, cước, trạng thái và sự kiện
/// của chuyến.
///
/// Chỉ những gì được `export` ở đây mới là API công khai. Mọi thứ trong
/// `lib/src/` là chi tiết nội bộ, đổi lúc nào cũng được.
library;

export 'src/fare.dart' show FarePolicy, calculateFare, calculateFareVnd;
export 'src/money.dart' show Money;
export 'src/service_kind.dart' show ServiceKind;
export 'src/trip.dart' show Trip;
export 'src/trip_event.dart'
    show
        Cancelled,
        CancelledBy,
        Completed,
        DriverAccepted,
        DriverArrived,
        PickedUp,
        SearchStarted,
        TripEvent,
        TripStarted;
export 'src/trip_queries.dart' show TripIterableX;
export 'src/trip_status.dart' show TripStatus;
```

Mỗi `export ... show ...` chỉ phơi ra đúng những tên liệt kê. Nếu sau này `fare.dart` thêm một hàm phụ không có dấu `_`, nó vẫn **không** lộ ra ngoài cho tới khi ta thêm vào danh sách; đọc một chỗ là biết API công khai gồm gì. Dart còn có `part` và `part of` để chia một thư viện ra nhiều file; ta không dùng, mỗi file nên là một thư viện nhỏ rõ ràng.

`Money` là value object bất biến:

### money.dart · packages/toi_noi_core/lib/src/money.dart

```dart
/// Số tiền VND bất biến, lưu bằng số nguyên đồng (không có phần lẻ).
///
/// Không âm: tiền của Tới Nơi chỉ là giá và tổng giá. `==` so sánh theo giá
/// trị nên hai `Money(25500)` bằng nhau dù là hai đối tượng khác nhau.
final class Money implements Comparable<Money> {
  /// Tạo từ số đồng cho hằng và giá trị đã biết chắc.
  ///
  /// Giá trị âm chỉ bị `assert` chặn khi bật assertions (`dart test`,
  /// `--enable-asserts`, Flutter debug). Dữ liệu từ ngoài vào phải dùng
  /// [Money.fromVnd].
  const Money(this.vnd) : assert(vnd >= 0, 'Số tiền không được âm');

  /// Tạo từ số đồng chưa được tin cậy (JSON, form, database).
  ///
  /// Luôn kiểm tra, kể cả khi assertions tắt; ném [ArgumentError] nếu [vnd] âm.
  factory Money.fromVnd(int vnd) {
    if (vnd < 0) {
      throw ArgumentError.value(vnd, 'vnd', 'không được âm');
    }
    return Money(vnd);
  }

  /// 0 đồng, dùng làm giá trị khởi đầu khi cộng dồn.
  static const Money zero = Money(0);

  /// Số đồng.
  final int vnd;

  /// Cộng hai số tiền.
  Money operator +(Money other) => Money(vnd + other.vnd);

  /// Trừ hai số tiền; ném [ArgumentError] nếu kết quả âm.
  Money operator -(Money other) {
    final result = vnd - other.vnd;
    if (result < 0) {
      throw ArgumentError.value(other, 'other', 'lớn hơn số tiền hiện có');
    }
    return Money(result);
  }

  /// Nhân với số lượng nguyên không âm; ném [ArgumentError] nếu [factor] âm.
  Money operator *(int factor) {
    if (factor < 0) {
      throw ArgumentError.value(factor, 'factor', 'không được âm');
    }
    return Money(vnd * factor);
  }

  bool operator <(Money other) => vnd < other.vnd;

  bool operator <=(Money other) => vnd <= other.vnd;

  bool operator >(Money other) => vnd > other.vnd;

  bool operator >=(Money other) => vnd >= other.vnd;

  @override
  int compareTo(Money other) => vnd.compareTo(other.vnd);

  @override
  bool operator ==(Object other) => other is Money && other.vnd == vnd;

  @override
  int get hashCode => vnd.hashCode;

  /// Dạng hiển thị có dấu chấm nghìn: `Money(25500)` thành `"25.500 VND"`.
  ///
  /// Chèn tay để chưa phải dùng package `intl`.
  @override
  String toString() {
    final digits = vnd.abs().toString();
    final buffer = StringBuffer();
    for (var i = 0; i < digits.length; i++) {
      final remaining = digits.length - i;
      if (i > 0 && remaining % 3 == 0) {
        buffer.write('.');
      }
      buffer.write(digits[i]);
    }
    final sign = vnd < 0 ? '-' : '';
    return '$sign$buffer VND';
  }
}
```

Hai lối vào kiểm tra khác nhau. Constructor `const` chỉ có thể dùng `assert`, mà `assert` chỉ chạy khi **bật assertions**: `dart test` bật sẵn, `dart run` phải thêm `--enable-asserts`, Flutter bật ở debug và tắt ở profile và release, còn bản AOT (`dart compile exe`) luôn bỏ qua. Vì vậy `const Money` dành cho hằng và giá trị đã biết chắc. Dữ liệu từ ngoài (JSON của server, form, database) đi qua `Money.fromVnd`, luôn ném `ArgumentError` nếu âm; các toán tử `-` và `*` cũng kiểm tường minh nên không bao giờ tạo ra `Money` âm. `implements Comparable<Money>` cho `list.sort()` chạy trên danh sách `Money`; `==` và `hashCode` đi cùng nhau để `Money` làm được phần tử của `Set` hay khoá của `Map`.

Công thức cước nằm trong `fare.dart`:

### fare.dart · packages/toi_noi_core/lib/src/fare.dart

```dart
import 'dart:math' show max;

import 'money.dart';

/// Bảng giá minh hoạ của Tới Nơi (không phải bảng giá thực tế).
///
/// Dùng `abstract final class` làm "không gian tên": không tạo được đối tượng,
/// không bị kế thừa, và `FarePolicy.maxKm` không chen vào tên toàn cục.
abstract final class FarePolicy {
  /// Giá mở cửa, đã gồm [includedMeters] đầu.
  static const int baseFareVnd = 12000;

  /// Số mét đã nằm trong giá mở cửa.
  static const int includedMeters = 2000;

  /// Giá mỗi km vượt.
  static const int pricePerKmVnd = 4500;

  /// Quãng đường tối đa hợp lệ.
  static const int maxKm = 100;
}

/// Tính cước bằng số đồng cho quãng đường [distanceKm], miền hợp lệ 0..100 km.
///
/// Quy đổi sang mét nguyên một lần, rồi chỉ tính bằng số nguyên để kết quả
/// không phụ thuộc sai số `double`. Phần vượt làm tròn nửa đồng lên.
/// Ném [ArgumentError] khi ngoài miền, kể cả NaN và vô cực.
int calculateFareVnd(double distanceKm) {
  // NaN so sánh với số nào cũng cho false, nên guard viết dưới dạng
  // "điều kiện hợp lệ" rồi phủ định.
  final isValid = distanceKm >= 0 && distanceKm <= FarePolicy.maxKm;
  if (!isValid) {
    throw ArgumentError.value(
      distanceKm,
      'distanceKm',
      'phải nằm trong khoảng 0..${FarePolicy.maxKm} km',
    );
  }

  final meters = (distanceKm * 1000).round();
  final extraMeters = max(0, meters - FarePolicy.includedMeters);
  return FarePolicy.baseFareVnd +
      (extraMeters * FarePolicy.pricePerKmVnd + 500) ~/ 1000;
}

/// Giống [calculateFareVnd] nhưng trả [Money] thay vì `int` trần.
Money calculateFare(double distanceKm) => Money(calculateFareVnd(distanceKm));
```

Hàm quy đổi sang mét nguyên một lần, rồi chỉ tính bằng số nguyên. `(extraMeters * 4500 + 500) ~/ 1000` là cách làm tròn nửa đồng lên mà không cần `double`: 1 mét vượt là 4,5 đồng, cộng 500 rồi chia nguyên cho 1000 ra 5. Kết quả khớp bảng biên của bài 02 mà không phụ thuộc sai số dấu phẩy động. `Trip` bất biến như ở bài 03, chỉ đổi kiểu cước và nguồn tính cước; phần `apply` giữ nguyên:

### trip.dart · packages/toi_noi_core/lib/src/trip.dart (trích, trong class Trip)

```dart
  /// Áp dụng [event] và trả về chuyến mới; chuyến hiện tại không bị sửa.
  ///
  /// Ném [StateError] khi cặp (trạng thái, sự kiện) không hợp lệ, và
  /// [ArgumentError] khi tài xế nhận chuyến có `driverId` rỗng.
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

Hai chi tiết khác bản lab 03. Một, constructor kiểm `driverId` bằng `ArgumentError`: `requested` và `searching` phải chưa có tài xế, từ `accepted` tới `completed` phải có, `cancelled` thì tuỳ (huỷ lúc đang tìm thì chưa có ai). Hai, `==` và `hashCode` so **mọi** trường thay vì chỉ `id` và `status`: bộ lọc cập nhật của Riverpod ở các bài sau so bằng `==`, nên hai chuyến khác tài xế mà bị coi là bằng nhau thì giao diện không vẽ lại; `freezed` sau này cũng sinh equality theo mọi trường; và `Set<Trip>` sẽ gộp nhầm. Câu hỏi "có phải cùng một chuyến" đã có hàm riêng, `isSameTrip`, so theo `id`. `Money` và `Trip` đều là `final class`, và `Trip` không có `copyWith`: mọi chuyển trạng thái đi qua `apply`.

Tài liệu API nằm ngay trong code dưới dạng comment `///`: câu đầu là tóm tắt, `[tên]` là liên kết tới tên khác. `dart doc` đọc các comment này và sinh trang HTML (lần chạy cuối báo `Found 0 warnings and 0 errors`), nên ở chỗ người dùng package sẽ đọc ta viết `///` chứ không phải `//`.

## Lint và analyzer

`dart analyze` phát hiện lỗi mà không cần chạy code. Nó đọc `analysis_options.yaml`, nơi ta chọn bộ **lint** (quy tắc phong cách và an toàn) và tuỳ chỉnh chính **analyzer**.

### analysis_options.yaml · packages/toi_noi_core/analysis_options.yaml

```yaml
# Bộ lint nền của Dart team, cộng thêm vài rule chặt hơn cho package dùng chung.
include: package:lints/recommended.yaml

analyzer:
  language:
    # Ép kiểu ngầm từ dynamic thành lỗi, không để lọt qua âm thầm.
    strict-casts: true
    # Không tự suy ra dynamic khi thiếu thông tin kiểu.
    strict-inference: true
    # List, Map, Future... trống tham số kiểu bị báo, tránh List<dynamic> vô tình.
    strict-raw-types: true

linter:
  rules:
    # Biến cục bộ không đổi thì phải final.
    - prefer_final_locals
    # Future bị bỏ quên (không await, không unawaited) là lỗi thật khi có async.
    - unawaited_futures
    # Gọi member trên dynamic là chỗ compiler hết bảo vệ.
    - avoid_dynamic_calls
    # Giá trị bất biến dùng const để tái sử dụng một instance.
    - prefer_const_constructors
    - prefer_const_declarations
    # import và export theo thứ tự cố định cho diff dễ đọc.
    - directives_ordering
```

Dòng `include` kéo vào bộ `recommended` của package `lints`, bộ quy tắc Dart team khuyến nghị cho mọi project (bộ nhỏ hơn là `core`, chính là bộ pub.dev dùng để chấm điểm). Phần bên dưới là các lựa chọn chặt hơn của riêng package này:

- `strict-casts`, `strict-inference`, `strict-raw-types` là ba cờ của analyzer. Chúng chặn ba kẽ hở khiến kiểu `dynamic` lọt vào mà không ai hay: ép kiểu ngầm, suy luận ra `dynamic`, và `List` trơn trụi thay vì `List<Money>`.
- `prefer_final_locals` và `prefer_const_*` làm giá trị bất biến trông bất biến ngay ở chỗ khai báo.
- `avoid_dynamic_calls` báo mọi lần gọi member trên `dynamic`, nơi compiler hết bảo vệ ta.
- `unawaited_futures` chưa có việc ở package này vì chưa có `async`; ta bật sớm vì một `Future` bị quên `await` (bài 05) là lỗi âm thầm.
- `directives_ordering` cố định thứ tự `import` và `export`.

Thử đặt vào `lib/src` một file nghịch ngợm có `var a = 1;`, `List raw = [];` và một lời gọi `d.length` trên biến `dynamic`: `dart analyze` báo `prefer_final_locals`, `strict_raw_type` (mức warning) và `avoid_dynamic_calls`. Nếu file còn `import 'package:test/test.dart'` trong `lib/`, ta nhận thêm `depend_on_referenced_packages`, vì `test` chỉ là `dev_dependency`.

Ba lệnh đi cùng:

```bash
dart analyze                  # báo lỗi, warning và info
dart format .                 # chuẩn hoá cách trình bày
dart fix --dry-run            # xem những chỗ analyzer sửa tự động được
```

`dart format` quyết định kiểu trình bày theo **language version** suy ra từ `sdk` constraint của pubspec, nên hai package khác SDK constraint có thể format khác nhau. `dart fix --apply` áp dụng các sửa tự động, hữu ích khi nâng SDK và API cũ bị đánh dấu deprecated. Trên CI, chạy `dart analyze --fatal-infos` để lint mức info cũng làm bước phân tích thất bại.

## Test có tổ chức

Test của `toi_noi_core` nằm trong `test/`, mỗi file ứng với một phần của `lib/src`, tên kết thúc bằng `_test.dart`; đuôi này là điều kiện để `dart test` tự tìm thấy file.

Khối nền tảng là ba hàm: `test` cho một ca, `group` gom các ca cùng chủ đề (có thể lồng nhau), `setUp` chạy lại trước **mỗi** test để mỗi ca có dữ liệu mới. `expect(giá_trị_thật, matcher)` so sánh với một **matcher**: `equals`, `isTrue`, `hasLength`, `contains`, `throwsArgumentError`, `throwsStateError`, `isA<T>().having(...)`. Test của công thức cước dùng một bảng ca biên thay vì viết từng `test` tay:

### fare_test.dart · packages/toi_noi_core/test/fare_test.dart (trích)

```dart
    group('ca biên và ca thường', () {
      // km, cước kỳ vọng (VND). Số minh hoạ, không phải bảng giá thật.
      const cases = <(double, int)>[
        (0, 12000),
        (1.999, 12000),
        (2, 12000),
        (2.001, 12005),
        (2.499, 14246),
        (2.501, 14255),
        (5, 25500),
        (10, 48000),
        (100, 453000),
      ];

      for (final (km, expected) in cases) {
        test('$km km -> $expected VND', () {
          expect(calculateFareVnd(km), expected);
        });
      }
    });

    // ...

    group('ngoài miền ném ArgumentError', () {
      const invalid = <double>[
        -1,
        -0.001,
        100.001,
        100.5,
        double.nan,
        double.infinity,
        double.negativeInfinity,
      ];

      for (final km in invalid) {
        test('$km km', () {
          expect(() => calculateFareVnd(km), throwsArgumentError);
        });
      }

      test('thông báo ghi miền 0..100 km', () {
        expect(
          () => calculateFareVnd(101),
          throwsA(
            isA<ArgumentError>().having(
              (e) => e.message,
              'message',
              contains('0..100 km'),
            ),
          ),
        );
      });

      test('thông báo nêu tên tham số', () {
        expect(
          () => calculateFareVnd(-1),
          throwsA(
            isA<ArgumentError>().having((e) => e.name, 'name', 'distanceKm'),
          ),
        );
      });
    });
```

Vòng `for` tạo một `test` cho mỗi dòng, nên khi hỏng, tên test cho biết đúng km nào sai. Bảng có đủ các điểm hay gãy: 0, đúng 2 km, 2,001 km (mét đầu tiên vượt), 100 km (cận trên). Phần "ngoài miền" kiểm bằng `throwsArgumentError`, kể cả vô cực; riêng `NaN` là cái bẫy mà `km < 0 || km > 100` bỏ lọt.

Với `Trip`, ta không viết tay 64 ca. Bảng đặc tả liệt kê **chỉ 12 cặp hợp lệ** (trạng thái, sự kiện) và mọi cặp còn lại phải ném `StateError`:

### trip_test.dart · packages/toi_noi_core/test/trip_test.dart (trích)

```dart
    for (final status in TripStatus.values) {
      for (final entry in _events.entries) {
        final expected = _valid[(status, entry.key)];
        if (expected != null) {
          test('${status.name} + ${entry.key} -> ${expected.name}', () {
            expect(_tripAt(status).apply(entry.value).status, expected);
          });
        } else {
          test('${status.name} + ${entry.key} ném StateError', () {
            expect(() => _tripAt(status).apply(entry.value), throwsStateError);
          });
        }
      }
    }
```

Nếu ai đó thêm lậu một cặp chuyển trạng thái vào `apply`, một trong 64 ca sẽ đỏ vì bảng không có cặp đó. Cùng file còn kiểm điều quan trọng không kém: `apply` không sửa bản gốc, dù thành công hay bị từ chối.

Test truy vấn dùng `setUp` để dựng lại năm chuyến mỗi lần:

### trip_queries_test.dart · packages/toi_noi_core/test/trip_queries_test.dart (trích)

```dart
    late List<Trip> trips;

    setUp(() {
      // Hai completed, một inTransit, một searching, một cancelled.
      trips = [
        _trip('T1', TripStatus.completed, 25500),
        _trip('T2', TripStatus.inTransit, 17400),
        _trip('T3', TripStatus.searching, 36750),
        _trip('T4', TripStatus.cancelled, 12000),
        _trip('T5', TripStatus.completed, 12000),
      ];
    });

    test('totalFare cộng cước mọi chuyến và trả Money', () {
      expect(trips.totalFare, const Money(103650));
      expect(trips.totalFare.toString(), '103.650 VND');
    });

    test('totalFare của danh sách rỗng là Money.zero', () {
      expect(<Trip>[].totalFare, Money.zero);
    });
```

`dart test --reporter expanded` in mỗi test một dòng (tên đầy đủ gồm cả tên group), `dart test --name "ca biên"` chỉ chạy các test có tên chứa chuỗi đó, và `dart test test/fare_test.dart` chạy một file. Toàn bộ package có **138 test**: `money` 17, `fare` 23, `trip` 89, `trip_event` 4, `trip_queries` 5.

Bạn đã quen test bằng xUnit sẽ thấy `group` và `test` tương ứng `class` và `[Fact]`; so sánh sâu hơn có trong bài C# về [unit test và test double](/articles/csharp-unit-test-test-double).

## Dùng package từ project khác

Project `06-dung-toi-noi-core` (package `toi_noi_06`) đóng vai app tài xế tương lai. Nó khai báo `toi_noi_core` bằng **path dependency**:

### pubspec.yaml · 06-dung-toi-noi-core/pubspec.yaml

```yaml
name: toi_noi_06
description: "Lab bài 06 (Flutter qua Tới Nơi): dùng package toi_noi_core từ một project khác bằng path dependency."
version: 1.0.0
publish_to: none

environment:
  sdk: ^3.13.3

dependencies:
  toi_noi_core:
    path: ../packages/toi_noi_core

dev_dependencies:
  lints: ^6.1.0
  test: ^1.33.0
```

`path:` là đường dẫn tương đối từ thư mục chứa `pubspec.yaml` này tới thư mục gốc của package. Pub không sao chép gì mà trỏ thẳng vào nguồn, nên sửa `toi_noi_core` rồi chạy lại là project thấy ngay. Hình 2 vẽ cây phụ thuộc: `collection` đi theo `toi_noi_core` sang `toi_noi_06` mà project không phải khai báo, còn `lints` và `test` của package lõi thì không.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Cây phụ thuộc. toi_noi_06 phụ thuộc toi_noi_core qua path. toi_noi_core phụ thuộc collection. lints và test chỉ là dev dependency của toi_noi_core, không đi theo người dùng."><defs><marker id="fl06-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="fl06-2-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="20" y="30" width="230" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="66" class="t-b">toi_noi_06</text><text x="36" y="92" class="t-small">06-dung-toi-noi-core</text><text x="36" y="114" class="t-small">dùng package</text>
<rect x="340" y="30" width="230" height="100" rx="10" class="f-pk-soft s-pk"/><text x="356" y="66" class="t-b">toi_noi_core</text><text x="356" y="92" class="t-small">version 0.1.0</text><text x="356" y="114" class="t-small">package dùng chung</text>
<rect x="650" y="30" width="210" height="100" rx="10" class="f-phys-soft s-phys"/><text x="666" y="66" class="t-b">collection</text><text x="666" y="92" class="t-small">^1.19.1</text><text x="666" y="114" class="t-small">từ pub.dev</text>
<path d="M252 80 L338 80" class="f-none s-phys" stroke-width="2" marker-end="url(#fl06-2-a)"/><text x="262" y="68" class="t-mono">path</text>
<path d="M572 80 L648 80" class="f-none s-phys" stroke-width="2" marker-end="url(#fl06-2-a)"/><text x="580" y="68" class="t-mono">pub</text>
<rect x="300" y="190" width="130" height="70" rx="10" class="f-surface s-muted"/><text x="316" y="222" class="t-b">lints</text><text x="316" y="246" class="t-small">^6.1.0</text>
<rect x="480" y="190" width="130" height="70" rx="10" class="f-surface s-muted"/><text x="496" y="222" class="t-b">test</text><text x="496" y="246" class="t-small">^1.33.0</text>
<path d="M430 132 L390 188" class="f-none s-muted" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl06-2-m)"/>
<path d="M490 132 L520 188" class="f-none s-muted" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl06-2-m)"/>
<text x="640" y="214" class="t-b">dev_dependencies</text><text x="640" y="238" class="t-small">chỉ khi phát triển package,</text><text x="640" y="260" class="t-small">không đi theo người dùng</text>
</svg>
</div>
<figcaption>Hình 2. Phụ thuộc của toi_noi_06. Đường liền là dependencies, đường đứt là dev_dependencies của toi_noi_core: người dùng package nhận collection nhưng không nhận lints hay test.</figcaption>
</figure>

Project chỉ cần một dòng import. Nhờ `sealed class` của bài 03, `switch` dưới đây vẫn bị compiler bắt buộc đủ nhánh dù `TripEvent` nằm ở package khác:

### report.dart · 06-dung-toi-noi-core/lib/report.dart

```dart
import 'package:toi_noi_core/toi_noi_core.dart';

/// Một dòng báo giá: `"5.0 km: 25.500 VND"`.
String quoteLine(double distanceKm) =>
    '$distanceKm km: ${calculateFare(distanceKm)}';

/// Mô tả sự kiện cho nhật ký.
///
/// `switch` trên `sealed class` của package khác vẫn bị bắt buộc vét cạn:
/// nếu `toi_noi_core` thêm biến thể mới, `dart analyze` ở đây sẽ báo lỗi.
String describeEvent(TripEvent event) => switch (event) {
  SearchStarted() => 'bắt đầu tìm tài xế',
  DriverAccepted(:final driverId) => 'tài xế $driverId nhận chuyến',
  DriverArrived() => 'tài xế đang tới điểm đón',
  PickedUp() => 'đã đón khách',
  TripStarted() => 'bắt đầu chở',
  Completed() => 'hoàn thành',
  Cancelled(:final by) => switch (by) {
    CancelledBy.customer => 'khách huỷ',
    CancelledBy.driver => 'tài xế huỷ',
  },
};

/// Chạy [events] lần lượt trên [trip] và trả về từng dòng nhật ký.
///
/// Dòng đầu là trạng thái ban đầu; mỗi sự kiện thêm một dòng
/// `"sự kiện -> trạng thái"`. Sự kiện không hợp lệ ném `StateError` như
/// `Trip.apply`.
List<String> runTrip(Trip trip, List<TripEvent> events) {
  final lines = <String>['${trip.id}: ${trip.status.label}'];
  var current = trip;
  for (final event in events) {
    current = current.apply(event);
    lines.add('${describeEvent(event)} -> ${current.status.label}');
  }
  return lines;
}
```

Thử xoá một nhánh, ví dụ `PickedUp() => ...`, `dart analyze` báo `non_exhaustive_switch_expression`. Khi `toi_noi_core` thêm một biến thể `TripEvent`, mọi app dùng nó biết ngay chỗ cần sửa, trước khi chạy.

Từ nay, các bài sau dùng `Money` thay cho `int` trần để biểu diễn tiền: lấy `fare.vnd` khi cần số đồng (gửi lên API, ghi database), còn trong logic và giao diện giữ `Money`, để không ai cộng nhầm số đồng với số mét. Khi code dồn vào workspace `toi-noi-app`, ta sẽ chuyển sang **pub workspace**: một thư mục gốc quản lý nhiều package, mỗi package khai `resolution: workspace`, và `dart pub get` giải phiên bản một lần cho cả cây. Ở bài này ta dùng `path:` trần để thấy cơ chế gốc trước.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với package</span>
<ul>
<li><strong>Import vào <code>lib/src</code> của package khác.</strong> Dấu hiệu: lint <code>implementation_imports</code>. Cách sửa: import <code>package:toi_noi_core/toi_noi_core.dart</code>; nếu thiếu tên cần dùng, thêm nó vào danh sách <code>show</code> trong file export.</li>
<li><strong>Thêm file mới vào <code>lib/src</code> mà quên <code>export</code>.</strong> Dấu hiệu: project dùng báo <code>Undefined class</code> dù <code>dart analyze</code> trong package vẫn sạch. Cách sửa: thêm dòng <code>export</code> và nhớ test dùng qua file công khai như người dùng thật.</li>
<li><strong>Đặt sai chỗ giữa <code>dependencies</code> và <code>dev_dependencies</code>.</strong> Dấu hiệu: <code>depend_on_referenced_packages</code> khi <code>lib/</code> import một package chỉ ở <code>dev_dependencies</code>. Cách sửa: chuyển nó sang <code>dependencies</code> nếu mã trong <code>lib/</code> thật sự cần.</li>
<li><strong>Sai đường dẫn <code>path:</code>.</strong> Dấu hiệu: <code>dart pub get</code> báo "depends on toi_noi_core from path which doesn't exist". Cách sửa: tính đường dẫn từ thư mục chứa <code>pubspec.yaml</code> của project dùng, không từ thư mục đang đứng.</li>
<li><strong>Test không chạy vì tên file.</strong> Dấu hiệu: file test có mặt nhưng <code>dart test</code> không đếm. Cách sửa: đổi tên thành <code>*_test.dart</code>. Lỗi liên quan: tin vào <code>assert</code> để bảo vệ dữ liệu; <code>dart run</code> không bật assertions nếu thiếu <code>--enable-asserts</code>, nên dữ liệu từ ngoài phải đi qua <code>Money.fromVnd</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/packages/toi_noi_core
dart pub get
dart analyze
dart test --reporter expanded

cd ../../06-dung-toi-noi-core
dart pub get
dart analyze
dart test
dart run bin/main.dart
```

Hoặc chạy cả bài 01 đến 06 bằng `node examples/toi-noi-flutter/verify-01-06.mjs`. Đã kiểm với Dart 3.13.3 (Flutter 3.47.4): `dart analyze` 0 issue ở cả hai project, package có 138 test xanh, project dùng có 7 test xanh, `dart doc` 0 warning. Mức: **Kiểm chứng tự động**, không cần thiết bị.

| Test | Chứng minh điều gì |
|---|---|
| `money_test.dart` | `25.500 VND`, `0 VND`, `1.234.567 VND`; `-` và `*` ra số âm ném `ArgumentError`; `==`, `hashCode`, `Set`/`Map`; `Money.fromVnd` ném `ArgumentError` khi âm; `assert` chặn `const Money` âm khi bật assertions |
| `fare_test.dart` | 0, 2, 2,001, 5, 100 km cho đúng 12.000, 12.000, 12.005, 25.500, 453.000; không bao giờ giảm khi đi xa hơn; -1, 100,5, NaN, vô cực ném `ArgumentError` ghi rõ "0..100 km" |
| `trip_test.dart` | Bảng 8 trạng thái x 8 sự kiện (12 cặp hợp lệ, 52 cặp `StateError`); `apply` không sửa bản gốc; `==` so mọi trường (khác `driverId` thì khác nhau), `isSameTrip` so theo `id`; `driverId` phải khớp trạng thái; `requestedAt` không UTC bị chặn |
| `trip_event_test.dart` | `DriverAccepted` và `Cancelled` bằng nhau theo giá trị, vào `Set` không nhân đôi |
| `trip_queries_test.dart` | `totalFare` trả `Money` (rỗng cho `Money.zero`); `byStatus` giữ thứ tự; `active` lười |
| `06.../report_test.dart` | Project khác dùng package qua path; lỗi của package đi xuyên qua; mô tả đủ 7 biến thể (8 kể cả hai `CancelledBy`) |

**Chưa làm**: publish lên pub.dev hoặc git; pub workspace (từ lúc có `toi-noi-app`); `toi_noi_api` và `toi_noi_ui`; serialize JSON cho `Trip` và `Money`; định dạng tiền theo locale bằng `intl`; kiểm tra phiên bản package tối thiểu bằng `dart pub downgrade`.

## Bài tập

1. `const Money(25500) - const Money(30000)` và `Money(int.parse('-1'))` đều lỗi khi chạy `dart test`. Hai lỗi giống hay khác nhau, và khi assertions tắt thì chuyện gì xảy ra?
2. Ta thêm `lib/src/tip.dart` với hàm `Money addTip(Money fare)` rồi chạy `dart test` trong package: xanh. Nhưng `06-dung-toi-noi-core` báo không thấy `addTip`. Vì sao và sửa thế nào?
3. `collection: ^1.19.1` và một ngày collection ra bản `2.0.0`. `dart pub upgrade` có kéo bản đó về không? Muốn dùng nó thì làm gì?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Khác. Phép trừ ném <code>ArgumentError</code> do code kiểm tra tường minh, và lỗi này luôn có hiệu lực. <code>Money(-1)</code> chỉ bị <code>assert</code> chặn (<code>AssertionError</code>) khi bật assertions; khi assertions tắt (<code>dart run</code> mặc định, bản release) một <code>Money</code> âm lọt qua. Dữ liệu từ ngoài vào phải đi qua <code>Money.fromVnd</code>.</p>
<p><strong>2.</strong> Test của package có thể import thẳng file trong <code>lib/src</code> (hoặc chưa từng gọi <code>addTip</code>), còn project dùng chỉ thấy những gì <code>toi_noi_core.dart</code> export. Thêm <code>export 'src/tip.dart' show addTip;</code> vào file công khai, rồi viết test của <code>addTip</code> import qua <code>package:toi_noi_core/toi_noi_core.dart</code>.</p>
<p><strong>3.</strong> Không: <code>^1.19.1</code> nghĩa là <code>&lt;2.0.0</code>, nên <code>dart pub upgrade</code> dừng ở bản 1.x mới nhất. Muốn lên 2.0.0 phải đổi ràng buộc, thủ công hoặc bằng <code>dart pub upgrade --major-versions</code>, rồi chạy lại <code>dart analyze</code> và <code>dart test</code> vì major mới có thể phá vỡ API.</p>
</details>

Bài trước: [Future, async/await và Stream](/articles/dart-future-async-await-stream-bao-gia-va-vi-tri). Bài tiếp: [Widget, cây widget và hot reload: màn hình báo giá đầu tiên](/articles/flutter-widget-cay-widget-hot-reload-man-hinh-bao-gia).

## Tài liệu gốc

- [Create packages](https://dart.dev/tools/pub/create-packages) và [Package layout conventions](https://dart.dev/tools/pub/package-layout) (dart.dev).
- [The pubspec file](https://dart.dev/tools/pub/pubspec) và [Pub dependencies](https://dart.dev/tools/pub/dependencies) (dart.dev).
- [Pub workspaces](https://dart.dev/tools/pub/workspaces) (dart.dev).
- [Customizing static analysis](https://dart.dev/tools/analysis) và [Linter rules](https://dart.dev/tools/linter-rules) (dart.dev).
- [Testing](https://dart.dev/tools/testing) (dart.dev) và [package:test](https://pub.dev/packages/test) (pub.dev).
- [package:lints](https://pub.dev/packages/lints) và [package:collection](https://pub.dev/packages/collection) (pub.dev).
- [Effective Dart: Documentation](https://dart.dev/effective-dart/documentation) (dart.dev).
