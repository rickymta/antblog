import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_30/quote_providers.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

void main() {
  late ApiHarness api;
  late ProviderContainer container;

  setUp(() {
    api = ApiHarness();
    // Tắt retry: test này kiểm trạng thái, không kiểm chính sách thử lại.
    container = ProviderContainer.test(
      overrides: [api.override],
      retry: (_, _) => null,
    );
  });

  group('đọc kết quả bằng .future', () {
    test('5 km ra 25.500 VND', () async {
      final quote = await container.read(quoteProvider(ride5km).future);

      expect(quote.fare, const Money(25500));
      expect(quote.dto.distanceMeters, 5000);
    });

    test('2,001 km ra 12.005 VND: mét đầu tiên vượt 2 km', () async {
      final quote = await container.read(
        quoteProvider((serviceKind: ServiceKind.ride, distanceKm: 2.001))
            .future,
      );

      expect(quote.fare, const Money(12005));
    });

    test('lỗi 422 làm future ném ClientFailure', () async {
      await expectLater(
        container.read(quoteProvider(ride150km).future),
        throwsA(
          isA<ClientFailure>().having((e) => e.statusCode, 'statusCode', 422),
        ),
      );
      expect(api.serverRequests, 1);
    });

    test('hai khoá khác nhau là hai provider độc lập', () async {
      final five = await container.read(quoteProvider(ride5km).future);
      final ten = await container.read(
        quoteProvider((serviceKind: ServiceKind.ride, distanceKm: 10.0)).future,
      );

      expect(five.fare, const Money(25500));
      expect(ten.fare, const Money(48000));
    });
  });

  group('listen ghi lại chuỗi trạng thái', () {
    test('ngay sau khi đọc, state là AsyncLoading', () {
      final states = record(container, quoteProvider(ride5km));

      expect(states, hasLength(1));
      expect(states.single, isA<AsyncLoading<ValidQuote>>());
    });

    test('thành công: AsyncLoading rồi AsyncData', () async {
      final states = record(container, quoteProvider(ride5km));

      await container.read(quoteProvider(ride5km).future);

      expect(states, [
        isA<AsyncLoading<ValidQuote>>(),
        isA<AsyncData<ValidQuote>>().having(
          (s) => s.value.fare,
          'fare',
          const Money(25500),
        ),
      ]);
    });

    test('thất bại: AsyncLoading rồi AsyncError mang ApiFailure', () async {
      final states = record(container, quoteProvider(ride150km));

      await expectLater(
        container.read(quoteProvider(ride150km).future),
        throwsA(isA<ClientFailure>()),
      );

      expect(states, [
        isA<AsyncLoading<ValidQuote>>(),
        isA<AsyncError<ValidQuote>>().having(
          (s) => s.error,
          'error',
          isA<ClientFailure>(),
        ),
      ]);
    });
  });

  group('autoDispose cần người nghe', () {
    test('không ai nghe: mỗi lần đọc là một request mới', () async {
      await container.read(quoteProvider(ride5km).future);
      await container.read(quoteProvider(ride5km).future);

      expect(api.serverRequests, 2);
    });

    test('có listener: hai lần đọc dùng chung một request', () async {
      final subscription = container.listen(quoteProvider(ride5km), (_, _) {});

      await container.read(quoteProvider(ride5km).future);
      await container.read(quoteProvider(ride5km).future);

      expect(api.serverRequests, 1);
      subscription.close();
    });

    test(
      'đóng listener cuối: provider bị bỏ, đọc lại gọi request mới',
      () async {
        final subscription = container.listen(
          quoteProvider(ride5km),
          (_, _) {},
        );
        await container.read(quoteProvider(ride5km).future);

        subscription.close();
        await container.pump();
        await container.read(quoteProvider(ride5km).future);

        expect(api.serverRequests, 2);
      },
    );
  });
}
