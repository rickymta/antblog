import 'package:test/test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/fare_cases.dart';

/// Quy tắc cước là hàm thuần: không cần test double nào, chỉ cần chọn ca cho
/// khéo. Mỗi ca bảng dưới đây là một test có tên.
void main() {
  group('phân vùng tương đương: một đại diện cho mỗi lớp', () {
    for (final c in fareCases.where((c) => c.kind == CaseKind.partition)) {
      _caseTest(c);
    }
  });

  group('giá trị biên: sát từng đường ranh giữa hai lớp', () {
    for (final c in fareCases.where((c) => c.kind == CaseKind.boundary)) {
      _caseTest(c);
    }
  });

  group('lặp thay vì viết tay', () {
    test('mọi mét từ 0 đến 100.000 khớp phép tính độc lập', () {
      final mismatches = <int>[];
      for (var m = 0; m <= 100000; m++) {
        if (calculateFareVnd(m / 1000) != oracleFareVnd(m)) {
          mismatches.add(m);
        }
      }
      expect(mismatches, isEmpty);
    });

    test('đồng giá tới 2.000 m; sau đó mỗi mét thêm 4 hoặc 5 đồng', () {
      var previous = calculateFareVnd(0);
      for (var m = 1; m <= 100000; m++) {
        final fare = calculateFareVnd(m / 1000);
        final step = fare - previous;
        final allowed = m <= FarePolicy.includedMeters ? {0} : {4, 5};
        expect(allowed, contains(step), reason: 'tại $m m bước giá là $step');
        previous = fare;
      }
    });

    test('calculateFare trả Money khớp calculateFareVnd', () {
      for (final km in [0.0, 1.0, 2.0, 2.001, 5.0, 100.0]) {
        expect(calculateFare(km), Money(calculateFareVnd(km)));
      }
    });
  });

  group('bảng ca cũng được kiểm', () {
    test('có cả hai loại ca và không trùng tên', () {
      final names = fareCases.map((c) => c.name).toSet();
      expect(names, hasLength(fareCases.length));
      expect(fareCases.map((c) => c.kind).toSet(), CaseKind.values.toSet());
    });
  });
}

void _caseTest(FareCase c) {
  test(c.name, () {
    final expected = c.expected;
    if (expected == null) {
      expect(() => calculateFareVnd(c.km), throwsArgumentError);
    } else {
      expect(calculateFareVnd(c.km), expected);
    }
  });
}
