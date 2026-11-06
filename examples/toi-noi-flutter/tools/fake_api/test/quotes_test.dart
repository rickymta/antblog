import 'package:shelf/shelf.dart';
import 'package:test/test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

void main() {
  late Handler handler;

  setUp(() => handler = createFakeApiHandler());

  group('POST /quotes', () {
    // km, cước kỳ vọng (VND), mét. Số minh hoạ, không phải bảng giá thật.
    const cases = <(double, int, int)>[
      (0, 12000, 0),
      (2, 12000, 2000),
      (2.001, 12005, 2001),
      (5, 25500, 5000),
      (100, 453000, 100000),
    ];

    for (final (km, fare, meters) in cases) {
      test('$km km -> $fare VND', () async {
        final reply = await call(
          handler,
          'POST',
          '/quotes',
          body: {'serviceKind': 'ride', 'distanceKm': km},
        );
        expect(reply.status, 200);
        expect(asMap(reply), {
          'serviceKind': 'ride',
          'distanceMeters': meters,
          'fare': fare,
          'currency': 'VND',
        });
        // Chính công thức của toi_noi_core, không gõ tay một bản thứ hai.
        expect(fare, calculateFare(km).vnd);
      });
    }

    test('số nguyên JSON (5) cũng được nhận như số thực', () async {
      final reply = await call(
        handler,
        'POST',
        '/quotes',
        body: {'serviceKind': 'food', 'distanceKm': 5},
      );
      expect(reply.status, 200);
      expect(asMap(reply)['fare'], 25500);
    });

    test('trả content-type JSON có charset utf-8', () async {
      final reply = await call(
        handler,
        'POST',
        '/quotes',
        body: {'serviceKind': 'ride', 'distanceKm': 1},
      );
      expect(reply.headers['content-type'], 'application/json; charset=utf-8');
    });

    for (final km in [-1.0, -0.001, 100.001, 250.0]) {
      test('$km km ngoài miền -> 422 kèm lỗi theo trường', () async {
        final reply = await call(
          handler,
          'POST',
          '/quotes',
          body: {'serviceKind': 'ride', 'distanceKm': km},
        );
        expect(reply.status, 422);
        expect(
          reply.headers['content-type'],
          'application/problem+json; charset=utf-8',
        );
        expect(errorsOf(reply)['distanceKm'], ['Expected 0..100 km']);
      });
    }

    test('sai cả hai trường: báo đủ hai lỗi trong một lần', () async {
      final reply = await call(
        handler,
        'POST',
        '/quotes',
        body: {'serviceKind': 'helicopter', 'distanceKm': 'năm'},
      );
      expect(reply.status, 422);
      expect(errorsOf(reply).keys, containsAll(['serviceKind', 'distanceKm']));
    });

    test('thiếu trường -> 422', () async {
      final reply = await call(handler, 'POST', '/quotes', body: {});
      expect(reply.status, 422);
      expect(errorsOf(reply).keys, containsAll(['serviceKind', 'distanceKm']));
    });

    test('body không phải JSON -> 400', () async {
      final reply = await call(
        handler,
        'POST',
        '/quotes',
        rawBody: '{không phải json',
      );
      expect(reply.status, 400);
    });

    test('body là mảng thay vì object -> 400', () async {
      final reply = await call(handler, 'POST', '/quotes', body: [1, 2]);
      expect(reply.status, 400);
    });

    test('body rỗng -> 400', () async {
      final reply = await call(handler, 'POST', '/quotes');
      expect(reply.status, 400);
    });
  });
}
