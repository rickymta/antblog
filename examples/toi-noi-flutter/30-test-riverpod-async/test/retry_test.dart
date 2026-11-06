import 'package:fake_async/fake_async.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_30/quote_providers.dart';
import 'package:toi_noi_30/retry_policy.dart';
import 'package:toi_noi_api/fault_injection.dart';
import 'package:toi_noi_api/toi_noi_api.dart';

import 'support.dart';

/// Cho `dio` chạy hết một vòng: nó dùng cả microtask lẫn một hàng đợi sự kiện,
/// nên `flushMicrotasks` một mình chưa đủ để request tới adapter.
void settle(FakeAsync async) => async.elapse(Duration.zero);

void main() {
  group('retryTransient', () {
    const network = NetworkFailure();

    test('lỗi mạng: chờ 500 ms, 1 s, 2 s rồi dừng', () {
      expect(
        [for (var n = 0; n < 4; n++) retryTransient(n, network)],
        [
          const Duration(milliseconds: 500),
          const Duration(seconds: 1),
          const Duration(seconds: 2),
          null,
        ],
      );
    });

    test('lỗi không retryable thì không thử lại', () {
      const invalid = ClientFailure(statusCode: 422, title: 'Unprocessable');
      const bad = ParseFailure(dto: 'QuoteDto', field: 'fare', message: 'sai');

      expect(retryTransient(0, invalid), isNull);
      expect(retryTransient(0, bad), isNull);
    });

    test('lỗi không phải ApiFailure (lỗi lập trình) không thử lại', () {
      expect(retryTransient(0, StateError('bug')), isNull);
      expect(retryTransient(0, Exception('lạ')), isNull);
    });
  });

  group('tắt retry: lỗi tới ngay, không cần đồng hồ giả', () {
    test('lỗi mạng thành AsyncError sau đúng một lần gọi', () async {
      final api = ApiHarness(faults: [NetworkFault.refuse]);
      final container = ProviderContainer.test(
        overrides: [api.override],
        retry: (_, _) => null,
      );

      await expectLater(
        container.read(quoteProvider(ride5km).future),
        throwsA(isA<NetworkFailure>()),
      );
      expect(api.attempts, 1);
    });
  });

  group('retry chạy trên đồng hồ giả', () {
    test('hai lần mất kết nối rồi thành công: 3 lần gọi, 1,5 giây giả', () {
      fakeAsync((async) {
        final api = ApiHarness(
          faults: [NetworkFault.refuse, NetworkFault.refuse],
        );
        final container = ProviderContainer.test(
          overrides: [api.override],
          retry: retryTransient,
        );
        final states = record(container, quoteProvider(ride5km));

        settle(async);
        expect(api.attempts, 1);
        expect(states.last.isLoading, isTrue);

        async.elapse(const Duration(milliseconds: 499));
        expect(api.attempts, 1); // chưa tới 500 ms
        async.elapse(const Duration(milliseconds: 1));
        expect(api.attempts, 2);

        async.elapse(const Duration(milliseconds: 999));
        expect(api.attempts, 2); // chưa tới 500 + 1000 ms
        async.elapse(const Duration(milliseconds: 1));
        expect(api.attempts, 3);

        expect(states.last, isA<AsyncData<ValidQuote>>());
        expect(states.first, isA<AsyncLoading<ValidQuote>>());
        expect(states.first.hasError, isFalse);
        // Giữa chừng là các AsyncLoading mang lỗi cũ: đang thử lại.
        final retrying = states.sublist(1, states.length - 1);
        expect(retrying, isNotEmpty);
        expect(
          retrying.every(
            (s) => s.isLoading && s.retrying && s.error is NetworkFailure,
          ),
          isTrue,
        );
      });
    });

    test('hết lượt: 4 lần gọi trong 3,5 giây giả rồi AsyncError', () {
      fakeAsync((async) {
        final api = ApiHarness(faults: List.filled(10, NetworkFault.refuse));
        final container = ProviderContainer.test(
          overrides: [api.override],
          retry: retryTransient,
        );
        final states = record(container, quoteProvider(ride5km));

        settle(async);
        async.elapse(const Duration(milliseconds: 3499));
        expect(api.attempts, 3);
        expect(states.last.retrying, isTrue);

        async.elapse(const Duration(milliseconds: 1));
        async.elapse(const Duration(minutes: 5));

        expect(api.attempts, 4);
        expect(states.last, isA<AsyncError<ValidQuote>>());
        expect(states.last.retrying, isFalse);
        expect(states.last.error, isA<NetworkFailure>());
      });
    });

    test('lỗi 422 không bị thử lại: AsyncError ngay, một request', () {
      fakeAsync((async) {
        final api = ApiHarness();
        final container = ProviderContainer.test(
          overrides: [api.override],
          retry: retryTransient,
        );
        final states = record(container, quoteProvider(ride150km));

        settle(async);
        async.elapse(const Duration(minutes: 5));

        expect(states.last, isA<AsyncError<ValidQuote>>());
        expect(api.serverRequests, 1);
      });
    });
  });

  group('chính sách mặc định của Riverpod', () {
    test('422 vẫn bị gọi 11 lần vì ApiFailure là Exception', () {
      fakeAsync((async) {
        final api = ApiHarness();
        // Không truyền retry: dùng ProviderContainer.defaultRetry.
        final container = ProviderContainer.test(overrides: [api.override]);
        final states = record(container, quoteProvider(ride150km));

        settle(async);
        async.elapse(const Duration(minutes: 1));

        expect(api.serverRequests, 11); // 1 lần đầu và 10 lần thử lại
        expect(states.last, isA<AsyncError<ValidQuote>>());
      });
    });
  });
}
