import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_38/location_buffer.dart';

import 'test_helpers.dart';

void main() {
  group('giới hạn số lượng', () {
    test('đầy thì bỏ mẫu cũ nhất, giữ các mẫu mới nhất', () {
      final buffer = LocationBuffer(capacity: 3);
      for (var i = 0; i < 5; i++) {
        buffer.add(sample(i));
      }
      expect(buffer.length, 3);
      expect(buffer.items, [sample(2), sample(3), sample(4)]);
      expect(buffer.droppedCount, 2);
    });

    test('chưa đầy thì không bỏ gì', () {
      final buffer = LocationBuffer(capacity: 3)
        ..add(sample(0))
        ..add(sample(1));
      expect(buffer.droppedCount, 0);
      expect(buffer.length, 2);
    });

    test('mặc định 120 mẫu và 10 phút', () {
      final buffer = LocationBuffer();
      expect(buffer.capacity, 120);
      expect(buffer.maxAge, const Duration(minutes: 10));
    });

    test('capacity không dương bị assert chặn', () {
      expect(() => LocationBuffer(capacity: 0), throwsA(isA<AssertionError>()));
    });
  });

  group('giới hạn tuổi', () {
    test('đúng bằng maxAge thì còn, quá 1 mili giây thì bỏ', () {
      final buffer = LocationBuffer(maxAge: const Duration(minutes: 10))
        ..add(sample(0)); // chụp lúc t0
      final exactly = t0.add(const Duration(minutes: 10));
      expect(buffer.purgeExpired(exactly), 0);
      expect(buffer.length, 1);
      expect(
        buffer.purgeExpired(exactly.add(const Duration(milliseconds: 1))),
        1,
      );
      expect(buffer.isEmpty, isTrue);
    });

    test('chỉ bỏ mẫu cũ, giữ mẫu mới', () {
      final buffer = LocationBuffer(maxAge: const Duration(minutes: 1))
        ..add(sample(0)) // t0
        ..add(sample(30)); // t0 + 150 giây
      final removed = buffer.purgeExpired(t0.add(const Duration(seconds: 200)));
      expect(removed, 1);
      expect(buffer.items, [sample(30)]);
    });
  });

  group('lấy ra và xoá', () {
    test('peek không xoá, trả mẫu cũ nhất trước', () {
      final buffer = LocationBuffer()
        ..add(sample(0))
        ..add(sample(1))
        ..add(sample(2));
      expect(buffer.peek(2), [sample(0), sample(1)]);
      expect(buffer.length, 3);
    });

    test('removeSent xoá đúng các mẫu đã gửi, theo danh tính đối tượng', () {
      final a = sample(0);
      final b = sample(1);
      final buffer = LocationBuffer()
        ..add(a)
        ..add(b);
      buffer.removeSent([a]);
      expect(buffer.items, [b]);
      // Một đối tượng khác nhưng bằng nhau theo giá trị thì không bị xoá.
      buffer.removeSent([sample(1)]);
      expect(buffer.items, [b]);
      buffer.removeSent([b]);
      expect(buffer.isEmpty, isTrue);
    });

    test('items là bản sao: sửa nó không đổi bộ đệm', () {
      final buffer = LocationBuffer()..add(sample(0));
      expect(() => buffer.items.add(sample(1)), throwsUnsupportedError);
      expect(buffer.length, 1);
    });

    test('replaceAll vẫn tuân theo capacity', () {
      final buffer = LocationBuffer(capacity: 2)
        ..replaceAll([sample(0), sample(1), sample(2)]);
      expect(buffer.items, [sample(1), sample(2)]);
    });
  });
}
