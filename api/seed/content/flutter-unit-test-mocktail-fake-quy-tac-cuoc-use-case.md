---
slug: flutter-unit-test-mocktail-fake-quy-tac-cuoc-use-case
title: "Unit test với mocktail và fake: quy tắc cước và use case"
summary: "Bài 27: kiểm quy tắc cước bằng phân vùng tương đương và giá trị biên, lặp đủ 56 cặp (trạng thái, sự kiện) của chuyến, rồi test use case RequestTrip với dummy, stub, spy, mock và fake. Dùng mocktail (when, verify, any(named:), registerFallbackValue), chỉ ra khi nào fake bền hơn mock. 131 test."
category: flutter
level: intermediate
tags: [flutter, dart, mobile, toi-noi, testing, unit-test, mocktail, test-double]
language: vi
---

Trong một buổi dọn code, ai đó đổi `km <= 100` thành `km < 100` ở guard của công thức cước. Mọi test hiện có vẫn xanh vì chúng thử 1 km, 5 km và 50 km. Tuần sau, khách đặt chuyến liên tỉnh đúng 100 km bị từ chối, và không ai hiểu vì sao.

Phần 4 của lộ trình dạy cách chặn chuyện đó từ sớm. Bài mở đầu này đứng ở lớp thấp nhất và nhanh nhất: unit test thuần Dart, không widget, không mạng. Ta làm ba việc. Chọn ca cho quy tắc cước. Lặp đủ mọi cặp (trạng thái, sự kiện) của chuyến. Test use case `RequestTrip` với các loại test double, rồi so fake với mock.

Đây là bài 27 của lộ trình **Flutter qua Tới Nơi**, bài đầu của Phần 4. Ta nên đọc [bài 26 về code generation](/articles/flutter-code-generation-freezed-json-serializable-riverpod-generator) trước; use case lấy từ [bài 21 về Clean Architecture](/articles/flutter-clean-architecture-domain-data-presentation-mobile), còn quy tắc cước và máy trạng thái nằm trong `toi_noi_core` của [bài 06](/articles/dart-package-pubspec-lint-test-toi-noi-core). Thuật ngữ kiểm thử theo chuyên mục Kiểm thử, ví dụ [Equivalence Partitioning](/articles/equivalence-partitioning) và [Component Testing](/articles/component-testing-unit-testing-tong-quan). Bài dựa trên Flutter 3.47.4, Dart 3.13.3, `test` 1.33.0 và `mocktail` 1.0.5. Bảng giá là số minh hoạ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bộ test cước chỉ có sáu ca: -5, 1, 5, 50 và 150 km, cộng NaN (mỗi phân vùng quãng đường ít nhất một đại diện). Ta đổi guard thành <code>km &lt; 100</code>. Test nào đỏ?</p>
<details><summary>Xem đáp án</summary><p>Không test nào. 100 km là biên của phân vùng tính theo km vượt, còn các đại diện nằm giữa phân vùng nên không chạm tới nó. Mục 1 cài đúng lỗi này cùng bốn lỗi khác rồi đếm: ba trong năm lỗi lọt qua bộ đại diện, và một lỗi khác lọt qua bộ ca biên.</p></details>
</div>

## 1. Quy tắc cước: phân vùng tương đương rồi giá trị biên

`calculateFareVnd` là hàm thuần: cùng đầu vào thì cùng đầu ra, không đọc đồng hồ, không gọi mạng. Test nó không cần test double nào. Việc khó duy nhất là chọn ca, vì quãng đường là số thực và ta không thử hết được.

Cách chọn có hai bước. **Phân vùng tương đương** (EP) chia miền đầu vào thành các phân vùng mà quy tắc xử lý giống nhau, rồi chọn ít nhất một đại diện cho mỗi phân vùng. **Giá trị biên** (BVA) thêm các giá trị sát hai phía mỗi đường ranh, vì lỗi hay sống ở đó. Hình 1 vẽ bốn phân vùng, ba đường ranh và phân vùng NaN của quãng đường.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Trục quãng đường chia thành bốn phân vùng. Dưới 0 km bị từ chối. Từ 0 đến 2 km đồng giá 12.000 VND, đại diện 1 km. Trên 2 đến 100 km tính theo km vượt, đại diện 5 km và 50 km. Trên 100 km bị từ chối, đại diện 150 km. Ba đường ranh ở 0, 2 và 100 km, mỗi đường có hai ca biên sát hai phía: âm 0,001 và 0, 2 và 2,001, 100 và 100,001. NaN là một phân vùng riêng không nằm trên trục và không có biên.">
<text x="20" y="22" class="t-b">Phân vùng tương đương: đại diện ở giữa mỗi phân vùng</text>
<rect x="20" y="40" width="160" height="96" rx="8" class="f-warn-soft s-warn"/>
<text x="32" y="64" class="t-b">ngoài miền dưới</text><text x="32" y="84" class="t-small">bị từ chối</text><text x="32" y="116" class="t-mono">đại diện: -5</text>
<rect x="180" y="40" width="160" height="96" rx="8" class="f-pk-soft s-pk"/>
<text x="192" y="64" class="t-b">đồng giá</text><text x="192" y="84" class="t-small">12.000 VND</text><text x="192" y="116" class="t-mono">đại diện: 1</text>
<rect x="340" y="40" width="340" height="96" rx="8" class="f-phys-soft s-phys"/>
<text x="352" y="64" class="t-b">tính theo km vượt</text><text x="352" y="84" class="t-small">12.000 + 4.500 VND mỗi km quá 2 km</text><text x="352" y="116" class="t-mono">đại diện: 5 và 50</text>
<rect x="680" y="40" width="180" height="96" rx="8" class="f-warn-soft s-warn"/>
<text x="692" y="64" class="t-b">ngoài miền trên</text><text x="692" y="84" class="t-small">bị từ chối</text><text x="692" y="116" class="t-mono">đại diện: 150</text>
<path d="M180 32 L180 152" class="f-none s-muted" stroke-width="1.5" stroke-dasharray="6 4"/>
<path d="M340 32 L340 152" class="f-none s-muted" stroke-width="1.5" stroke-dasharray="6 4"/>
<path d="M680 32 L680 152" class="f-none s-muted" stroke-width="1.5" stroke-dasharray="6 4"/>
<text x="20" y="176" class="t-b">Ca biên: sát hai phía mỗi đường ranh</text>
<text x="180" y="200" text-anchor="middle" class="t-mono">-0,001 | 0</text>
<text x="340" y="200" text-anchor="middle" class="t-mono">2 | 2,001</text>
<text x="680" y="200" text-anchor="middle" class="t-mono">100 | 100,001</text>
<text x="180" y="218" text-anchor="middle" class="t-small">ranh 0 km</text>
<text x="340" y="218" text-anchor="middle" class="t-small">ranh 2 km</text>
<text x="680" y="218" text-anchor="middle" class="t-small">ranh 100 km</text>
<rect x="20" y="230" width="840" height="52" rx="8" class="f-surface s-muted"/>
<text x="36" y="252" class="t-b">NaN: một phân vùng riêng, không nằm trên trục</text>
<text x="36" y="270" class="t-small">Không có biên để đặt ca sát hai phía. Vô cực dương và âm là ca biên của hai phân vùng ngoài miền.</text>
</svg>
</div>
<figcaption>Hình 1. Bốn phân vùng trên trục quãng đường, ba đường ranh với hai ca biên mỗi đường, và NaN là phân vùng riêng không có biên.</figcaption>
</figure>

Mỗi ca là một dòng dữ liệu kèm loại ca và kết quả kỳ vọng. Kỳ vọng được tính tay từ bảng giá, không gọi `calculateFare`. Với 100 km: 12.000 + 98 × 4.500 = 453.000 VND. Với 2,001 km: phần vượt đúng 1 mét là 4,5 đồng, làm tròn nửa lên thành 5, tổng 12.005 VND.

### fare_cases.dart · 27-unit-test-mocktail/test/support/fare_cases.dart (trích)

```dart
const fareCases = <FareCase>[
  // Đại diện ở giữa từng phân vùng.
  FareCase(CaseKind.partition, 'ngoài miền dưới: -5 km', -5, null),
  FareCase(CaseKind.partition, 'đồng giá: 1 km', 1, 12000),
  FareCase(CaseKind.partition, 'km vượt: 5 km', 5, 25500),
  FareCase(CaseKind.partition, 'km vượt: 50 km', 50, 228000),
  FareCase(CaseKind.partition, 'ngoài miền trên: 150 km', 150, null),
  FareCase(CaseKind.partition, 'không phải số: NaN', double.nan, null),
  // Sát từng biên.
  FareCase(CaseKind.boundary, 'ngay dưới 0: -0,001 km', -0.001, null),
  FareCase(CaseKind.boundary, 'biên dưới: 0 km', 0, 12000),
  FareCase(CaseKind.boundary, 'hết đồng giá: 2 km', 2, 12000),
  FareCase(CaseKind.boundary, 'vượt 1 mét: 2,001 km', 2.001, 12005),
  FareCase(CaseKind.boundary, 'biên trên: 100 km', 100, 453000),
  FareCase(CaseKind.boundary, 'ngay trên 100: 100,001 km', 100.001, null),
  FareCase(CaseKind.boundary, 'vô cực dương', double.infinity, null),
  FareCase(CaseKind.boundary, 'vô cực âm', double.negativeInfinity, null),
];
```

Ca nào bắt được lỗi nào? Ta không đoán mà cài thử. `fare_mutants_test.dart` viết lại công thức với từng tham số là một chỗ có thể sai, tạo ra năm lỗi, rồi chạy riêng bộ đại diện và bộ biên qua chúng.

### fare_mutants_test.dart · 27-unit-test-mocktail/test/fare_mutants_test.dart (trích)

```dart
final mutants = <String, FareFn>{
  'biên trên loại trừ (< thay <=)': variant(
    accepts: (km) => km >= 0 && km < FarePolicy.maxKm,
  ),
  'trần nhầm 99 km': variant(accepts: (km) => km >= 0 && km <= 99),
  'bỏ cộng 500 khi làm tròn': variant(roundingBias: 0),
  'guard viết dạng phủ định (NaN lọt)': variant(
    accepts: (km) => !(km < 0 || km > FarePolicy.maxKm),
  ),
  'giá km vượt nhầm 4.000': variant(pricePerKm: 4000),
};
```

| Lỗi cài vào | Bộ đại diện bắt | Bộ biên bắt |
|---|---|---|
| `<` thay `<=` ở trần 100 km | không | có (ca 100 km) |
| trần viết nhầm 99 km | không | có (ca 100 km) |
| bỏ cộng 500 khi làm tròn | không | có (ca 2,001 km) |
| guard dạng phủ định, NaN lọt qua | có (ca NaN) | không |
| giá km vượt viết nhầm 4.000 | có | có |

Lỗi làm tròn lọt qua bộ đại diện vì 5 km và 50 km có phần vượt chẵn mét (3.000 và 48.000), nên 4,5 đồng mỗi mét luôn ra số nguyên; chỉ phần vượt lẻ mét như 2,001 km mới lộ. Lỗi NaN lọt qua bộ biên vì NaN không có biên. Công thức bị lỗi này vẫn ném, nhưng là `UnsupportedError` ("Infinity or NaN toInt") chứ không phải `ArgumentError`. Ca kiểm yêu cầu đúng `ArgumentError` nên vẫn đỏ; test chỉ kiểm "có ném gì đó" sẽ cho lỗi này qua. Hai kỹ thuật bổ sung nhau chứ không thay nhau.

Còn một cách chọn ca nữa: không chọn. Nếu chỉ xét quãng đường tròn mét, miền hợp lệ có 100.001 giá trị và ta chạy hết. Test so cước từng mét với `oracleFareVnd`, một cách tính độc lập dùng `(extra * 9 + 1) ~/ 2` thay vì nhân 4.500 rồi chia 1.000.

### fare_rule_test.dart · 27-unit-test-mocktail/test/fare_rule_test.dart (trích)

```dart
test('mọi mét từ 0 đến 100.000 khớp phép tính độc lập', () {
  final mismatches = <int>[];
  for (var m = 0; m <= 100000; m++) {
    if (calculateFareVnd(m / 1000) != oracleFareVnd(m)) {
      mismatches.add(m);
    }
  }
  expect(mismatches, isEmpty);
});
```

Test thứ hai cạnh nó kiểm đồng giá tới 2.000 m, sau đó mỗi mét thêm 4 hoặc 5 đồng. Oracle không thay được ca biên: nó chia sẻ cùng cách hiểu bảng giá, nên hiểu sai bảng thì cả hai cùng sai. Vì thế các ca biên có kỳ vọng tính tay.

## 2. Máy trạng thái: lặp đủ 56 cặp

Chuyến có 8 trạng thái và 7 sự kiện, tức 56 cặp. Chỉ 9 cặp hợp lệ: sáu bước của đường đi bình thường và huỷ ở `searching`, `accepted`, `arriving`. 47 cặp còn lại phải ném `StateError`. Viết tay từng ca thì thế nào cũng quên vài cặp ném lỗi, mà đó lại là chỗ lỗi hay sống, chẳng hạn cho huỷ khi khách đã lên xe. Ta để vòng lặp sinh đủ.

Bảng 9 cặp được viết tay một lần, từ hồ sơ Tới Nơi, và là nguồn duy nhất:

### transition_table.dart · 27-unit-test-mocktail/test/support/transition_table.dart (trích)

```dart
const validTransitions = <(TripStatus, String), TripStatus>{
  (TripStatus.requested, 'SearchStarted'): TripStatus.searching,
  (TripStatus.searching, 'DriverAccepted'): TripStatus.accepted,
  (TripStatus.accepted, 'DriverArrived'): TripStatus.arriving,
  (TripStatus.arriving, 'PickedUp'): TripStatus.pickedUp,
  (TripStatus.pickedUp, 'TripStarted'): TripStatus.inTransit,
  (TripStatus.inTransit, 'Completed'): TripStatus.completed,
  (TripStatus.searching, 'Cancelled'): TripStatus.cancelled,
  (TripStatus.accepted, 'Cancelled'): TripStatus.cancelled,
  (TripStatus.arriving, 'Cancelled'): TripStatus.cancelled,
};
```

Hai vòng `for` lồng nhau sinh 56 test có tên dạng `(searching, PickedUp) ném StateError`:

### trip_state_machine_test.dart · 27-unit-test-mocktail/test/trip_state_machine_test.dart (trích)

```dart
group('bảng chuyển trạng thái', () {
  for (final status in TripStatus.values) {
    for (final event in events) {
      final name = '(${status.name}, ${eventName(event)})';
      final expected = validTransitions[(status, eventName(event))];
      if (expected != null) {
        test('$name sang ${expected.name}', () {
          final before = tripIn(status);
          final after = before.apply(event);
          expect(after.status, expected);
          expect(after.id, before.id);
          expect(after.fare, before.fare);
          expect(before.status, status, reason: 'chuyến cũ không bị sửa');
        });
      } else {
        test('$name ném StateError', () {
          expect(() => tripIn(status).apply(event), throwsStateError);
        });
      }
    }
  }
});
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 372" role="img" aria-label="Hình 2. Lưới 8 trạng thái nhân 7 sự kiện. Chín ô hợp lệ: requested với SearchStarted sang searching, searching với DriverAccepted sang accepted, accepted với DriverArrived sang arriving, arriving với PickedUp sang pickedUp, pickedUp với TripStarted sang inTransit, inTransit với Completed sang completed, và Cancelled hợp lệ ở searching, accepted, arriving, đều sang cancelled. 47 ô còn lại ném StateError.">
<text x="219" y="22" text-anchor="middle" class="t-b">Search</text>
<text x="219" y="40" text-anchor="middle" class="t-b">Started</text>
<text x="317" y="22" text-anchor="middle" class="t-b">Driver</text>
<text x="317" y="40" text-anchor="middle" class="t-b">Accepted</text>
<text x="415" y="22" text-anchor="middle" class="t-b">Driver</text>
<text x="415" y="40" text-anchor="middle" class="t-b">Arrived</text>
<text x="513" y="22" text-anchor="middle" class="t-b">Picked</text>
<text x="513" y="40" text-anchor="middle" class="t-b">Up</text>
<text x="611" y="22" text-anchor="middle" class="t-b">Trip</text>
<text x="611" y="40" text-anchor="middle" class="t-b">Started</text>
<text x="709" y="22" text-anchor="middle" class="t-b">Completed</text>
<text x="807" y="22" text-anchor="middle" class="t-b">Cancelled</text>
<text x="20" y="82" class="t-mono">requested</text>
<rect x="170" y="62" width="94" height="26" rx="4" class="f-pk-soft s-pk"/>
<text x="217" y="81" text-anchor="middle" class="t-b">searching</text>
<rect x="268" y="62" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="315" y="81" text-anchor="middle" class="t-small">×</text>
<rect x="366" y="62" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="413" y="81" text-anchor="middle" class="t-small">×</text>
<rect x="464" y="62" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="511" y="81" text-anchor="middle" class="t-small">×</text>
<rect x="562" y="62" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="609" y="81" text-anchor="middle" class="t-small">×</text>
<rect x="660" y="62" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="707" y="81" text-anchor="middle" class="t-small">×</text>
<rect x="758" y="62" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="805" y="81" text-anchor="middle" class="t-small">×</text>
<text x="20" y="112" class="t-mono">searching</text>
<rect x="170" y="92" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="217" y="111" text-anchor="middle" class="t-small">×</text>
<rect x="268" y="92" width="94" height="26" rx="4" class="f-pk-soft s-pk"/>
<text x="315" y="111" text-anchor="middle" class="t-b">accepted</text>
<rect x="366" y="92" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="413" y="111" text-anchor="middle" class="t-small">×</text>
<rect x="464" y="92" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="511" y="111" text-anchor="middle" class="t-small">×</text>
<rect x="562" y="92" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="609" y="111" text-anchor="middle" class="t-small">×</text>
<rect x="660" y="92" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="707" y="111" text-anchor="middle" class="t-small">×</text>
<rect x="758" y="92" width="94" height="26" rx="4" class="f-warn-soft s-warn"/>
<text x="805" y="111" text-anchor="middle" class="t-b">cancelled</text>
<text x="20" y="142" class="t-mono">accepted</text>
<rect x="170" y="122" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="217" y="141" text-anchor="middle" class="t-small">×</text>
<rect x="268" y="122" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="315" y="141" text-anchor="middle" class="t-small">×</text>
<rect x="366" y="122" width="94" height="26" rx="4" class="f-pk-soft s-pk"/>
<text x="413" y="141" text-anchor="middle" class="t-b">arriving</text>
<rect x="464" y="122" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="511" y="141" text-anchor="middle" class="t-small">×</text>
<rect x="562" y="122" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="609" y="141" text-anchor="middle" class="t-small">×</text>
<rect x="660" y="122" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="707" y="141" text-anchor="middle" class="t-small">×</text>
<rect x="758" y="122" width="94" height="26" rx="4" class="f-warn-soft s-warn"/>
<text x="805" y="141" text-anchor="middle" class="t-b">cancelled</text>
<text x="20" y="172" class="t-mono">arriving</text>
<rect x="170" y="152" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="217" y="171" text-anchor="middle" class="t-small">×</text>
<rect x="268" y="152" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="315" y="171" text-anchor="middle" class="t-small">×</text>
<rect x="366" y="152" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="413" y="171" text-anchor="middle" class="t-small">×</text>
<rect x="464" y="152" width="94" height="26" rx="4" class="f-pk-soft s-pk"/>
<text x="511" y="171" text-anchor="middle" class="t-b">pickedUp</text>
<rect x="562" y="152" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="609" y="171" text-anchor="middle" class="t-small">×</text>
<rect x="660" y="152" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="707" y="171" text-anchor="middle" class="t-small">×</text>
<rect x="758" y="152" width="94" height="26" rx="4" class="f-warn-soft s-warn"/>
<text x="805" y="171" text-anchor="middle" class="t-b">cancelled</text>
<text x="20" y="202" class="t-mono">pickedUp</text>
<rect x="170" y="182" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="217" y="201" text-anchor="middle" class="t-small">×</text>
<rect x="268" y="182" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="315" y="201" text-anchor="middle" class="t-small">×</text>
<rect x="366" y="182" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="413" y="201" text-anchor="middle" class="t-small">×</text>
<rect x="464" y="182" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="511" y="201" text-anchor="middle" class="t-small">×</text>
<rect x="562" y="182" width="94" height="26" rx="4" class="f-pk-soft s-pk"/>
<text x="609" y="201" text-anchor="middle" class="t-b">inTransit</text>
<rect x="660" y="182" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="707" y="201" text-anchor="middle" class="t-small">×</text>
<rect x="758" y="182" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="805" y="201" text-anchor="middle" class="t-small">×</text>
<text x="20" y="232" class="t-mono">inTransit</text>
<rect x="170" y="212" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="217" y="231" text-anchor="middle" class="t-small">×</text>
<rect x="268" y="212" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="315" y="231" text-anchor="middle" class="t-small">×</text>
<rect x="366" y="212" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="413" y="231" text-anchor="middle" class="t-small">×</text>
<rect x="464" y="212" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="511" y="231" text-anchor="middle" class="t-small">×</text>
<rect x="562" y="212" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="609" y="231" text-anchor="middle" class="t-small">×</text>
<rect x="660" y="212" width="94" height="26" rx="4" class="f-pk-soft s-pk"/>
<text x="707" y="231" text-anchor="middle" class="t-b">completed</text>
<rect x="758" y="212" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="805" y="231" text-anchor="middle" class="t-small">×</text>
<text x="20" y="262" class="t-mono">completed</text>
<rect x="170" y="242" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="217" y="261" text-anchor="middle" class="t-small">×</text>
<rect x="268" y="242" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="315" y="261" text-anchor="middle" class="t-small">×</text>
<rect x="366" y="242" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="413" y="261" text-anchor="middle" class="t-small">×</text>
<rect x="464" y="242" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="511" y="261" text-anchor="middle" class="t-small">×</text>
<rect x="562" y="242" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="609" y="261" text-anchor="middle" class="t-small">×</text>
<rect x="660" y="242" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="707" y="261" text-anchor="middle" class="t-small">×</text>
<rect x="758" y="242" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="805" y="261" text-anchor="middle" class="t-small">×</text>
<text x="20" y="292" class="t-mono">cancelled</text>
<rect x="170" y="272" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="217" y="291" text-anchor="middle" class="t-small">×</text>
<rect x="268" y="272" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="315" y="291" text-anchor="middle" class="t-small">×</text>
<rect x="366" y="272" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="413" y="291" text-anchor="middle" class="t-small">×</text>
<rect x="464" y="272" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="511" y="291" text-anchor="middle" class="t-small">×</text>
<rect x="562" y="272" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="609" y="291" text-anchor="middle" class="t-small">×</text>
<rect x="660" y="272" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="707" y="291" text-anchor="middle" class="t-small">×</text>
<rect x="758" y="272" width="94" height="26" rx="4" class="f-surface s-line"/>
<text x="805" y="291" text-anchor="middle" class="t-small">×</text>
<rect x="20" y="316" width="26" height="18" rx="4" class="f-pk-soft s-pk"/><text x="54" y="330" class="t-small">đường đi bình thường (6 ô)</text>
<rect x="300" y="316" width="26" height="18" rx="4" class="f-warn-soft s-warn"/><text x="334" y="330" class="t-small">huỷ được (3 ô)</text>
<rect x="500" y="316" width="26" height="18" rx="4" class="f-surface s-line"/><text x="534" y="330" class="t-small">×: ném StateError (47 ô)</text>
<text x="20" y="362" class="t-small">Hàng: trạng thái hiện tại. Cột: sự kiện áp dụng. Chữ trong ô: trạng thái kế tiếp.</text>
</svg>
</div>
<figcaption>Hình 2. Lưới 8 x 7 của máy trạng thái: 9 ô hợp lệ, 47 ô ném StateError. Vòng lặp sinh một test cho mỗi ô.</figcaption>
</figure>

Ba chi tiết giữ cho vòng lặp trung thực. Một, `eventName` dùng `switch` vét cạn trên `sealed class TripEvent`, nên thêm sự kiện thứ tám mà quên cập nhật bảng thì file không biên dịch. Hai, mỗi chuyến thử được dựng thẳng bằng constructor (`tripIn`) chứ không đi qua `apply`, để test không dựa vào chính thứ nó kiểm. Ba, bảng tự được kiểm: đúng 56 cặp, 9 hợp lệ, 47 bị từ chối, và tập trạng thái huỷ được khớp `TripStatus.canCancel`.

Bảng đủ nhạy không? Ta cài lỗi. Một hàm `apply` rộng tay cho huỷ ở `pickedUp`: `transitionFailures` chạy cả lưới và báo đúng một cặp, `(pickedUp, Cancelled)`. Lỗi ngược lại, chặn nhầm huỷ ở `accepted`, báo đúng `(accepted, Cancelled)`. Tên cặp nằm sẵn trong báo cáo, không phải đọc stack trace. Giới hạn: `Cancelled` chỉ có một đại diện (`by: customer`); biến thể `by` khác và `DriverAccepted` với mã rỗng được kiểm riêng, ngoài bảng.

## 3. Năm vai của test double

`RequestTrip` có hai phụ thuộc: `TripRepository`, nói chuyện với server, và `BookingLog`, nhật ký chỉ ghi. Để test nó ta thay hai thứ đó bằng đồ giả. Từ "mock" hay bị dùng cho tất cả, nhưng có năm vai khác nhau (cách gọi có thể khác giữa các đội; [bài C# về unit test và test double](/articles/csharp-unit-test-test-double) chia tương tự).

| Vai | Làm gì | Trong lab | Test kiểm gì |
|---|---|---|---|
| dummy | điền chỗ trống, không được dùng | `DummyTripRepository`, `DummyBookingLog` | không gì cả: bị gọi là nổ |
| stub | trả câu trả lời dựng sẵn | `MockTripRepository` chỉ dùng `when` | kết quả của mã dưới test |
| spy | ghi lại lời gọi, test hỏi sau | `SpyBookingLog` | danh sách ghi được, sau khi chạy |
| mock | framework ghi lời gọi, test đối chiếu đối số và số lần | `MockBookingLog` với `verify` | lời gọi đúng đối số, đúng số lần |
| fake | cài đặt thật nhưng gọn | `InMemoryTripRepository` | trạng thái "server" sau khi chạy |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 370" role="img" aria-label="Hình 3. RequestTrip gọi hai interface, TripRepository và BookingLog, và chỉ biết interface. Ba vai thay thế được TripRepository: InMemoryTripRepository là fake, MockTripRepository là stub khi dùng when và mock khi dùng verify, DummyTripRepository là dummy nổ nếu bị gọi. Ba vai thay thế được BookingLog: SpyBookingLog là spy chỉ ghi lại, MockBookingLog là mock kiểm bằng verify, DummyBookingLog là dummy nổ nếu bị gọi. Mũi tên liền chỉ lời gọi từ RequestTrip tới interface. Mũi tên đứt chỉ việc thực thi interface.">
<defs><marker id="fl27-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl27-3-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="20" y="164" width="190" height="56" rx="10" class="f-pk-soft s-pk"/><text x="34" y="188" class="t-b">RequestTrip</text><text x="34" y="206" class="t-small">use case dưới test</text>
<rect x="290" y="72" width="210" height="52" rx="10" class="f-phys-soft s-phys"/><text x="304" y="94" class="t-b">TripRepository</text><text x="304" y="112" class="t-small">interface của domain</text>
<rect x="290" y="260" width="210" height="52" rx="10" class="f-phys-soft s-phys"/><text x="304" y="282" class="t-b">BookingLog</text><text x="304" y="300" class="t-small">interface của domain</text>
<rect x="590" y="14" width="270" height="52" rx="8" class="f-surface s-muted"/><text x="604" y="36" class="t-mono">InMemoryTripRepository</text><text x="604" y="54" class="t-small">fake: chạy thật trong bộ nhớ</text>
<rect x="590" y="74" width="270" height="52" rx="8" class="f-surface s-muted"/><text x="604" y="96" class="t-mono">MockTripRepository</text><text x="604" y="114" class="t-small">stub (when), mock (verify)</text>
<rect x="590" y="134" width="270" height="52" rx="8" class="f-surface s-muted"/><text x="604" y="156" class="t-mono">DummyTripRepository</text><text x="604" y="174" class="t-small">dummy: nổ nếu bị gọi</text>
<rect x="590" y="202" width="270" height="52" rx="8" class="f-surface s-muted"/><text x="604" y="224" class="t-mono">SpyBookingLog</text><text x="604" y="242" class="t-small">spy: chỉ ghi lại để hỏi sau</text>
<rect x="590" y="262" width="270" height="52" rx="8" class="f-surface s-muted"/><text x="604" y="284" class="t-mono">MockBookingLog</text><text x="604" y="302" class="t-small">mock: kiểm bằng verify</text>
<rect x="590" y="322" width="270" height="44" rx="8" class="f-surface s-muted"/><text x="604" y="342" class="t-mono">DummyBookingLog</text><text x="604" y="358" class="t-small">dummy: nổ nếu bị gọi</text>
<path d="M212 180 L288 110" class="f-none s-pk" stroke-width="2" marker-end="url(#fl27-3-a)"/>
<path d="M212 204 L288 272" class="f-none s-pk" stroke-width="2" marker-end="url(#fl27-3-a)"/>
<text x="236" y="134" class="t-small">gọi</text><text x="238" y="268" class="t-small">gọi</text>
<path d="M588 40 L502 90" class="f-none s-muted" stroke-width="1.5" stroke-dasharray="6 4" marker-end="url(#fl27-3-m)"/>
<path d="M588 100 L502 98" class="f-none s-muted" stroke-width="1.5" stroke-dasharray="6 4" marker-end="url(#fl27-3-m)"/>
<path d="M588 160 L502 106" class="f-none s-muted" stroke-width="1.5" stroke-dasharray="6 4" marker-end="url(#fl27-3-m)"/>
<path d="M588 228 L502 270" class="f-none s-muted" stroke-width="1.5" stroke-dasharray="6 4" marker-end="url(#fl27-3-m)"/>
<path d="M588 288 L502 288" class="f-none s-muted" stroke-width="1.5" stroke-dasharray="6 4" marker-end="url(#fl27-3-m)"/>
<path d="M588 344 L502 302" class="f-none s-muted" stroke-width="1.5" stroke-dasharray="6 4" marker-end="url(#fl27-3-m)"/>
<text x="520" y="52" text-anchor="middle" class="t-small">thực thi</text>
</svg>
</div>
<figcaption>Hình 3. RequestTrip chỉ biết hai interface; mỗi vai test double thay thế một interface mà mã dưới test không hay biết.</figcaption>
</figure>

Mocktail chỉ có một lớp `Mock`, nên vai của nó do cách dùng: chỉ `when` thì là stub, có `verify` thì là mock. Định nghĩa gốc của mock là dựng kỳ vọng trước khi chạy, còn `verify` của mocktail chạy sau khi chạy xong, nên về bản chất nó gần spy do framework ghi hộ; bài gọi "mock" theo cách gọi của thư viện. Dummy và spy ta viết tay vì chúng ngắn:

### doubles.dart · 27-unit-test-mocktail/test/support/doubles.dart (trích)

```dart
final class DummyBookingLog implements BookingLog {
  @override
  dynamic noSuchMethod(Invocation invocation) => throw StateError(
    'Dummy bị gọi: ${invocation.memberName}. Test này không cho phép.',
  );
}
// ...
final class SpyBookingLog implements BookingLog {
  final List<Trip> bookedTrips = [];
  final List<({QuoteRequest request, TripFailure failure})> failedAttempts = [];

  @override
  void booked(Trip trip) => bookedTrips.add(trip);

  @override
  void failed(QuoteRequest request, TripFailure failure) =>
      failedAttempts.add((request: request, failure: failure));
}
```

Dummy nổ bằng `StateError` khi bị gọi, nên ca "ngoài miền không chạm repository" xanh thật: use case lỡ gọi thì ta thấy `StateError` chứ không phải `Rejected`. Một `Mock` trống dùng làm dummy sẽ im lặng cho qua với phương thức `void` như `booked`, nên không canh được điều đó.

Fake là phần đáng đầu tư nhất. `InMemoryTripRepository` cư xử như server ở đúng điều use case dựa vào: cùng `intentId` và cùng nội dung thì trả lại đúng chuyến đã tạo, khác nội dung thì từ chối `conflict`. Hàm `failNext(afterStore: true)` dựng tình huống khó nhất của bài 21, server đã tạo chuyến rồi mới mất response:

### doubles.dart · 27-unit-test-mocktail/test/support/doubles.dart (trích)

```dart
Future<Trip> requestTrip(
  QuoteRequest request, {
  required String intentId,
}) async {
  sentIntentIds.add(intentId);
  final scripted = _script.isEmpty ? null : _script.removeAt(0);
  if (scripted != null && !scripted.afterStore) {
    throw scripted.failure;
  }
  final known = _byIntent[intentId];
  if (known != null && known.request != request) {
    throw const Rejected(RejectionKind.conflict);
  }
  final trip =
      known?.trip ?? tripFor(request, id: 'T-${_byIntent.length + 1}');
  _byIntent[intentId] = (request: request, trip: trip);
  if (scripted != null) {
    throw scripted.failure;
  }
  return trip;
}
```

Test dùng nó chỉ nhìn kết quả: dù mất response, thử lại vẫn chỉ một chuyến, dùng lại đúng `intentId`.

### request_trip_fake_test.dart · 27-unit-test-mocktail/test/request_trip_fake_test.dart (trích)

```dart
test(
  'mất response sau khi server đã tạo chuyến: thử lại vẫn chỉ một chuyến',
  () async {
    server.failNext(const Unreachable(), afterStore: true);
    await expectLater(requestTrip(ride5km), throwsA(isA<Unreachable>()));
    final trip = await requestTrip(ride5km);
    expect(server.trips, [trip]);
    expect(server.sentIntentIds, ['intent-1', 'intent-1']);
    expect(log.failedAttempts.map((f) => f.failure), [isA<Unreachable>()]);
    expect(log.bookedTrips, [trip]);
  },
);
```

Fake cũng là code, nên cũng phải được kiểm: `in_memory_repository_test.dart` có 7 test cho riêng nó. Nếu fake cư xử khác server thật, mọi test dựa vào nó đều nói dối.

## 4. mocktail: when, verify và ba cái bẫy

Mock hợp khi chính lời gọi là điều cần kiểm. `BookingLog` không trả gì, nên cách duy nhất để biết use case có ghi hay không là hỏi mock:

### request_trip_mock_test.dart · 27-unit-test-mocktail/test/request_trip_mock_test.dart (trích)

```dart
test('TripFailure: ghi failed với đúng yêu cầu rồi ném lại', () async {
  when(
    () => repository.requestTrip(any(), intentId: any(named: 'intentId')),
  ).thenAnswer((_) => Future.error(const Unreachable()));

  await expectLater(requestTrip(ride5km), throwsA(isA<Unreachable>()));

  verify(() => log.failed(ride5km, any(that: isA<Unreachable>())))
      .called(1);
  verifyNever(() => log.booked(any()));
});
```

`when(() => ...)` dựng stub, và `thenAnswer` là cách trả một `Future`. `any()` thay cho tham số vị trí; tham số có tên cần `any(named: 'intentId')`, thiếu `named:` thì mocktail ném `ArgumentError` nêu đúng cách sửa. `any(that: isA<Unreachable>())` thêm điều kiện cho đối số. Về kiểm: `verify(...).called(1)` đếm số lần, `verifyNever` khẳng định không bao giờ, `verifyZeroInteractions(mock)` khẳng định mock không bị chạm, `verifyInOrder` kiểm thứ tự (kể cả giữa hai mock), `captureAny` lấy đối số thật đã truyền.

Ba cái bẫy đã gặp khi viết lab, mỗi bẫy có một test trong `mocktail_traps_test.dart`:

1. **`thenThrow` trên phương thức trả `Future` ném đồng bộ**, ngay lúc gọi, trước khi có `Future` nào. Repository thật là hàm `async` nên lỗi luôn nằm trong `Future`. Code dùng `.catchError` bắt được bản `thenAnswer((_) => Future.error(...))` nhưng không bắt được bản `thenThrow`. Use case của ta dùng `await` trong `try` nên cả hai bản đều qua; bẫy nằm ở code khác.
2. **Phương thức chưa stub trả `null`.** Với `Future<Trip>` đó là `TypeError` ("'Null' is not a subtype of type 'Future<Trip>'") ngay chỗ gọi, chứ không phải thông báo "chưa stub". Với phương thức `void` thì im lặng.
3. **Mỗi lời gọi chỉ verify được một lần.** Sau `verify`, lời gọi bị đánh dấu VERIFIED, và `verify` lần hai cho cùng lời gọi ném `TestFailure`.

## 5. registerFallbackValue: viết ca thiếu rồi xem lỗi thật

`any()` phải trả về một giá trị đúng kiểu để dòng `repository.requestTrip(any(), ...)` biên dịch với tham số không nullable. Mocktail không tự dựng được `QuoteRequest`, nên ta đưa nó một bản dự phòng bằng `registerFallbackValue`. Theo source `mocktail` 1.0.5, kiểu có sẵn (bool, int, double, String, List, Map, Set, DateTime, hàm) đã được đăng ký, và kiểu nullable không cần. Kiểu của ta thì cần.

Ta viết đúng ca thiếu. File `missing_fallback_test.dart` cố ý không gọi `registerFallbacks()`:

### missing_fallback_test.dart · 27-unit-test-mocktail/test/missing_fallback_test.dart (trích)

```dart
// Bước 1: chưa đăng ký. Lỗi nổ ngay khi `any()` được tính, tức là trước
// khi mã sản phẩm chạy.
expect(
  stub,
  throwsA(
    isA<StateError>().having(
      (e) => e.message,
      'message',
      allOf(
        contains('on a parameter of type `QuoteRequest`'),
        contains('registerFallbackValue was not previously called'),
      ),
    ),
  ),
);
```

Lỗi thật là `StateError`: "A test tried to use `any` or `captureAny` on a parameter of type `QuoteRequest`, but registerFallbackValue was not previously called". Hai điều đáng để ý. Lỗi nổ ngay khi `any()` được tính, trong `when`, trước khi mã sản phẩm chạy; dễ nhầm là lỗi của use case. Và theo source, `when` bị ngắt giữa chừng để lại cờ "đang stub", nên test dọn bằng `resetMocktailState()`.

Cách sửa là một hàm dùng chung, gọi trong `setUpAll` của mọi file có dùng `any()` cho kiểu của ta:

### doubles.dart · 27-unit-test-mocktail/test/support/doubles.dart (trích)

```dart
void registerFallbacks() {
  registerFallbackValue(ride5km);
  registerFallbackValue(const Unreachable());
  registerFallbackValue(tripFor(ride5km));
}
```

Ba điều cần biết. Một, `Trip` là `final class`, nên không viết được `class FakeTrip extends Fake implements Trip` (analyzer báo `invalid_use_of_type_outside_library`); ta đưa một `Trip` thật, nó chỉ được truyền đi, không bị chạm. (`Fake` ở đây là lớp của mocktail để dựng giá trị dự phòng, không phải vai fake ở mục 3.) Hai, fallback là trạng thái toàn cục của isolate: đăng ký xong không gỡ được, kể cả bằng `resetMocktailState()`. Test này chứng minh ở bước cuối. Ba, hệ quả là bẫy thứ tự: một test quên đăng ký vẫn xanh nếu test khác cùng file đã đăng ký trước. Ta đã thử, nó xanh khi chạy cả file và đỏ khi chạy riêng bằng `dart test --plain-name`. Với `dart test`, mỗi file chạy trong isolate riêng (ở bước 1, file này vẫn gặp `StateError` dù `request_trip_mock_test.dart` đã đăng ký); với `flutter test`, mỗi file là một tiến trình riêng. Nên mỗi file cần `setUpAll(registerFallbacks)` của chính nó.

## 6. Fake hay mock: một lần refactor trả lời

Giả sử đội muốn use case trả bản chuyến chuẩn của server, nên sau khi đặt xong thì đọc lại bằng `getTrip`:

### refactor_brittle_test.dart · 27-unit-test-mocktail/test/refactor_brittle_test.dart (trích)

```dart
final class ReadBackRequestTrip {
  ReadBackRequestTrip(this._inner, this._repository);

  final RequestTrip _inner;
  final TripRepository _repository;

  Future<Trip> call(QuoteRequest request) async {
    final trip = await _inner(request);
    return _repository.getTrip(trip.id);
  }
}
// ...
Future<void> _mockScenario(
  RequestTripLike book,
  MockTripRepository repo,
) async {
  when(() => repo.requestTrip(any(), intentId: any(named: 'intentId')))
      .thenAnswer((_) async => tripFor(ride5km));
  final trip = await book(ride5km);
  expect(trip.fare, const Money(25500));
  verify(() => repo.requestTrip(ride5km, intentId: 'intent-1')).called(1);
  verifyNoMoreInteractions(repo);
}
```

Điều khách thấy không đổi: vẫn một chuyến, đúng id, đúng cước. Chỉ thêm một lời gọi. Ta chạy cùng một kịch bản viết hai kiểu. Kiểu mock dựng stub cho `requestTrip` và khoá chính xác tương tác. Kiểu fake chỉ nhìn kết quả và số chuyến trên "server".

| Kịch bản | Với `RequestTrip` | Với bản đọc lại |
|---|---|---|
| viết kiểu mock | xanh | gãy: `TypeError`, mock chưa biết `getTrip`; thêm stub `getTrip` vẫn gãy vì `verifyNoMoreInteractions` thấy lời gọi lạ (`TestFailure`) |
| viết kiểu fake | xanh | xanh |

Test mock không sai. Nó đang làm đúng việc của mock là khoá cách làm. Nhưng cách làm là chi tiết nội bộ: khi nó đổi mà hành vi quan sát được không đổi, test gãy chỉ sinh việc sửa test, tức là giòn. Fake khoá điều khách và server nhìn thấy nên qua cả hai bản.

Điều ngược lại cũng đúng, fake không luôn thắng:

| Tình huống | Chọn | Vì sao |
|---|---|---|
| Phụ thuộc có luật riêng mà mã dưới test dựa vào (idempotency) | fake, kèm test riêng cho fake | kiểm kết quả, không kiểm lời gọi |
| Lệnh không trả gì: ghi log, gửi thông báo | spy hoặc mock | lời gọi chính là hành vi; `verifyNever` cho "không bao giờ" |
| Chỉ cần một câu trả lời cố định | stub (`when`) | nhanh, không phải viết fake |
| Cần một lỗi hiếm, như mất response sau khi lưu | fake có `failNext`, hoặc `thenAnswer` với `Future.error` | dựng lỗi theo ý |

Cả hai kiểu đều bắt lỗi thật. Ta cài bốn lỗi vào `RequestTrip` rồi đếm test đỏ trong từng file:

| Lỗi cài vào `RequestTrip` | `request_trip_fake_test` (13 test) | `request_trip_mock_test` (8 test) |
|---|---|---|
| luôn sinh `intentId` mới | 3 đỏ | 1 đỏ |
| bỏ kiểm miền 0 đến 100 km | 4 đỏ | 1 đỏ |
| không ghi `failed` | 2 đỏ | 1 đỏ |
| không ghi `booked` | 2 đỏ | 2 đỏ |

Cả hai bộ bắt đủ bốn lỗi. Bộ fake báo bằng kết quả ("server có hai chuyến"), bộ mock báo bằng tương tác ("`requestTrip` được gọi với `intent-2`").

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi viết unit test</span>
<ul>
<li><strong>Kỳ vọng sao chép từ công thức đang test.</strong> Dấu hiệu: đổi công thức mà test vẫn xanh. Nguyên nhân: kỳ vọng được tính bằng chính hàm cần kiểm. Cách sửa: tính tay từ bảng giá cho ca biên, và dùng oracle độc lập cho vòng lặp.</li>
<li><strong>Chỉ có ca giữa phân vùng.</strong> Dấu hiệu: đổi <code>&lt;=</code> thành <code>&lt;</code> không test nào đỏ. Cách sửa: thêm giá trị sát hai phía mỗi đường ranh và một ca NaN, vì NaN không có biên.</li>
<li><strong><code>thenThrow</code> cho <code>Future</code>, hoặc quên stub.</strong> Dấu hiệu: lỗi ném ngay chỗ gọi, hoặc <code>TypeError</code> về <code>Null</code>. Cách sửa: <code>thenAnswer((_) =&gt; Future.error(...))</code>, và stub mọi phương thức trả <code>Future</code> mà mã dưới test gọi.</li>
<li><strong>Quên <code>registerFallbackValue</code>, hoặc nhờ test khác đăng ký hộ.</strong> Dấu hiệu: <code>StateError</code> nêu tên kiểu, hoặc test xanh cả bộ nhưng đỏ khi chạy riêng. Cách sửa: <code>setUpAll(registerFallbacks)</code> trong mỗi file.</li>
<li><strong>Khoá cách làm thay vì kết quả.</strong> Dấu hiệu: refactor không đổi hành vi mà nhiều test gãy; <code>verifyNoMoreInteractions</code> rải khắp nơi. Cách sửa: dùng fake cho phụ thuộc có trạng thái, chỉ dùng <code>verify</code> khi lời gọi chính là hành vi.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/27-unit-test-mocktail
dart pub get
dart analyze
dart test
```

Đã kiểm với Flutter 3.47.4, Dart 3.13.3: `dart analyze` báo `No issues found!`, `dart test` chạy **131 test xanh**, kể cả khi đảo thứ tự ngẫu nhiên (`--test-randomize-ordering-seed=random`, thử 6 lần). Mức: **Kiểm chứng tự động**, không cần thiết bị hay mạng. Phiên bản: `test` 1.33.0, `mocktail` 1.0.5, `lints` 6.1.0, `toi_noi_core` 0.1.0.

Mỗi nhóm test chứng minh một điều. `fare_rule_test` (18) kiểm cước ở mọi ca đại diện, ca biên và từng mét. `fare_mutants_test` (6) chứng minh bộ ca bắt được lỗi cài vào. `trip_state_machine_test` (64) kiểm 56 cặp và bảng của chính nó. Hai file `request_trip_fake_test` (13) và `request_trip_mock_test` (8) kiểm cùng use case bằng hai kiểu double, còn `in_memory_repository_test` (7) kiểm chính fake. `mocktail_traps_test` (7), `missing_fallback_test` (2) và `refactor_brittle_test` (6) chứng minh các khẳng định ở mục 4, 5 và 6.

**Chưa làm**: `RequestTrip` ở lab là bản rút gọn của lab 21 để thực hành test double, chưa nối vào app; chưa có contract test chạy chung trên fake và repository thật (lab 21 kiểm repository thật bằng fake API trong tiến trình); chưa đo coverage (bài 33); widget test và Riverpod nằm ở bài 28 và 30.

## Bài tập

1. Thêm `(TripStatus.inTransit, 'Cancelled'): TripStatus.cancelled` vào `validTransitions`. Test nào đỏ, và vì sao?
2. Trong `request_trip_mock_test.dart`, đổi `thenAnswer((_) => Future.error(const Unreachable()))` thành `thenThrow(const Unreachable())`. Test có còn xanh không? Cho một đoạn code mà hai bản cho kết quả khác nhau.
3. Vì sao `registerFallbackValue(ride5km)` không đủ cho `log.booked(any())`, và vì sao không viết `class FakeTrip extends Fake implements Trip`?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Ta đã thử: 6 test đỏ. Test <code>(inTransit, Cancelled) sang cancelled</code> đỏ vì <code>apply</code> ném <code>StateError</code> (bảng nói hợp lệ, code nói không). Test đếm cặp đỏ vì có 10 cặp hợp lệ thay vì 9, test khớp <code>canCancel</code> đỏ, và ba test <code>transitionFailures</code> đỏ vì lưới giờ lệch một cặp. Bảng sai thì bộ test báo ngay; sửa bảng hoặc sửa <code>Trip.apply</code> là quyết định về nghiệp vụ, không phải về test.</p>
<p><strong>2.</strong> Vẫn xanh (đã thử), vì <code>RequestTrip.call</code> là hàm <code>async</code> và lời gọi nằm trong <code>try</code>: lỗi đồng bộ trong thân hàm <code>async</code> thành lỗi của <code>Future</code>. Hai bản khác nhau ở code không <code>async</code>, như <code>describe</code> trong <code>mocktail_traps_test.dart</code>: <code>r.requestTrip(...).then(...).catchError(...)</code> bắt được lỗi của <code>Future.error</code> nhưng để lỗi của <code>thenThrow</code> thoát ra ngoài.</p>
<p><strong>3.</strong> Fallback được chọn theo kiểu (<code>element is T</code>): <code>QuoteRequest</code> và <code>Trip</code> là hai kiểu nên cần hai lần đăng ký. Còn <code>Trip</code> là <code>final class</code>, không thể <code>implements</code> hay <code>extends</code> ngoài library của nó (<code>invalid_use_of_type_outside_library</code>), nên ta đưa một <code>Trip</code> thật, như <code>registerFallbacks</code> làm.</p>
</details>

Bài trước: [Code generation: freezed, json_serializable, riverpod_generator và build_runner](/articles/flutter-code-generation-freezed-json-serializable-riverpod-generator). Bài tiếp: [Widget test: pumpWidget, finder, matcher và màn hình báo giá](/articles/flutter-widget-test-pumpwidget-finder-matcher-man-hinh-bao-gia).

## Tài liệu gốc

- [mocktail](https://pub.dev/packages/mocktail) và [registerFallbackValue](https://pub.dev/documentation/mocktail/latest/mocktail/registerFallbackValue.html), [verifyZeroInteractions](https://pub.dev/documentation/mocktail/latest/mocktail/verifyZeroInteractions.html), [resetMocktailState](https://pub.dev/documentation/mocktail/latest/mocktail/resetMocktailState.html) (pub.dev).
- [package:test](https://pub.dev/packages/test) và [dart test](https://dart.dev/tools/dart-test) (pub.dev, dart.dev).
- [Testing Flutter apps](https://docs.flutter.dev/testing/overview) và [An introduction to unit testing](https://docs.flutter.dev/cookbook/testing/unit/introduction) (docs.flutter.dev).
- [Class modifiers](https://dart.dev/language/class-modifiers) và [Records](https://dart.dev/language/records) (dart.dev).
