import 'package:clock/clock.dart';
import 'package:fake_async/fake_async.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_30/quote_providers.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

ValidQuote _quoteExpiringAt(DateTime expiresAt) => ValidQuote(
  dto: const QuoteDto(
    serviceKind: ServiceKind.ride,
    distanceMeters: 5000,
    fareVnd: 25500,
  ),
  expiresAt: expiresAt,
);

void main() {
  group('withClock đóng băng thời gian', () {
    test('báo giá hết hạn đúng 2 phút sau lúc nhận', () async {
      final api = ApiHarness();
      final container = ProviderContainer.test(
        overrides: [api.override],
        retry: (_, _) => null,
      );

      final quote = await withClock(
        Clock.fixed(fixedNow),
        () => container.read(quoteProvider(ride5km).future),
      );

      expect(quote.expiresAt, DateTime.utc(2026, 11, 6, 3, 2));
      expect(quote.expiresAt, fixedNow.add(quoteValidity));
    });

    test('đồng hồ giữ nguyên qua các await trong cùng zone', () async {
      final seen = await withClock(Clock.fixed(fixedNow), () async {
        await Future<void>.delayed(Duration.zero);
        return clock.now();
      });

      expect(seen, fixedNow);
    });

    test('ra khỏi withClock là về đồng hồ thật', () {
      withClock(Clock.fixed(fixedNow), () {
        expect(clock.now(), fixedNow);
      });

      expect(clock.now(), isNot(fixedNow));
      expect(clock.now().year, greaterThanOrEqualTo(2026));
    });
  });

  group('ValidQuote.isExpiredAt: ca biên', () {
    final quote = _quoteExpiringAt(fixedNow);

    test('1 micro giây trước hạn: còn hiệu lực', () {
      expect(
        quote.isExpiredAt(fixedNow.subtract(const Duration(microseconds: 1))),
        isFalse,
      );
    });

    test('đúng hạn: đã hết hiệu lực', () {
      expect(quote.isExpiredAt(fixedNow), isTrue);
    });

    test('sau hạn: đã hết hiệu lực', () {
      expect(
        quote.isExpiredAt(fixedNow.add(const Duration(seconds: 1))),
        isTrue,
      );
    });

    test('isExpired đọc đồng hồ hiện hành của zone', () {
      expect(
        withClock(
          Clock.fixed(fixedNow.subtract(const Duration(seconds: 1))),
          () => quote.isExpired,
        ),
        isFalse,
      );
      expect(withClock(Clock.fixed(fixedNow), () => quote.isExpired), isTrue);
    });
  });

  group('fakeAsync tự đặt clock', () {
    test('clock bắt đầu ở initialTime và đi theo elapse', () {
      fakeAsync((async) {
        expect(clock.now(), fixedNow);

        async.elapse(const Duration(minutes: 90));

        expect(clock.now(), DateTime.utc(2026, 11, 6, 4, 30));
      }, initialTime: fixedNow);
    });

    test('báo giá lấy qua provider hết hạn sau đúng 2 phút giả', () {
      fakeAsync((async) {
        final api = ApiHarness();
        final container = ProviderContainer.test(
          overrides: [api.override],
          retry: (_, _) => null,
        );
        container.listen(quoteProvider(ride5km), (_, _) {});
        async.elapse(Duration.zero);

        final quote = container.read(quoteProvider(ride5km)).requireValue;
        expect(quote.expiresAt, fixedNow.add(quoteValidity));
        expect(quote.isExpired, isFalse);

        async.elapse(quoteValidity - const Duration(seconds: 1));
        expect(quote.isExpired, isFalse);

        async.elapse(const Duration(seconds: 1));
        expect(quote.isExpired, isTrue);
      }, initialTime: fixedNow);
    });

    test('DateTime.now() không đi theo thời gian giả', () {
      final realStart = DateTime.now();

      fakeAsync((async) {
        async.elapse(const Duration(days: 1));

        expect(clock.now(), fixedNow.add(const Duration(days: 1)));
        expect(
          DateTime.now().difference(realStart),
          lessThan(const Duration(hours: 1)),
        );
      }, initialTime: fixedNow);
    });

    test(
      'elapseBlocking cũng làm đồng hồ đi tới, dù không có Timer nào chạy',
      () {
        fakeAsync((async) {
          final quote = _quoteExpiringAt(
            fixedNow.add(const Duration(minutes: 2)),
          );

          async.elapseBlocking(const Duration(minutes: 3));

          expect(quote.isExpired, isTrue);
        }, initialTime: fixedNow);
      },
    );
  });
}
