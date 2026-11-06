import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_26/quote_state.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

const _input = QuoteInput(serviceKind: ServiceKind.ride, distanceKm: 5);

/// Một `switch` vét cạn bốn nhánh. Xoá một nhánh là `flutter analyze` báo lỗi
/// `non_exhaustive_switch_expression`: đó là điều test này đứng sau.
String describe(QuoteScreenState state) => switch (state) {
  QuoteIdle() => 'idle',
  QuoteLoading(:final input) => 'loading ${input.serviceKind.name}',
  QuoteReady(:final fare) => 'ready $fare',
  QuoteFailed(:final failure) => 'failed ${failure.runtimeType}',
};

void main() {
  group('union sealed của freezed', () {
    test('bốn nhánh, mỗi nhánh mang đúng dữ liệu của mình', () {
      expect(describe(const QuoteScreenState.idle()), 'idle');
      expect(
        describe(const QuoteScreenState.loading(input: _input)),
        'loading ride',
      );
      expect(
        describe(
          const QuoteScreenState.ready(input: _input, fare: Money(25500)),
        ),
        'ready 25.500 VND',
      );
      expect(
        describe(
          const QuoteScreenState.failed(
            input: _input,
            failure: NetworkFailure(),
          ),
        ),
        'failed NetworkFailure',
      );
    });

    test('== so sánh theo giá trị trong cùng một nhánh', () {
      expect(
        const QuoteScreenState.loading(input: _input),
        const QuoteScreenState.loading(
          input: QuoteInput(serviceKind: ServiceKind.ride, distanceKm: 5),
        ),
      );
      expect(
        const QuoteScreenState.ready(input: _input, fare: Money(25500)),
        isNot(const QuoteScreenState.ready(input: _input, fare: Money(25501))),
      );
    });

    test('hai nhánh khác nhau không bao giờ bằng nhau', () {
      expect(
        const QuoteScreenState.loading(input: _input),
        isNot(const QuoteScreenState.ready(input: _input, fare: Money.zero)),
      );
      expect(const QuoteScreenState.idle(), const QuoteScreenState.idle());
    });

    test('nhánh failed so sánh ApiFailure theo tham chiếu (không có ==)', () {
      // Dựng lúc chạy (không const) để có hai đối tượng khác nhau.
      TimeoutFailure timeout() => TimeoutFailure(
        phase: TimeoutPhase.receive,
        limit: Duration(milliseconds: int.parse('100')),
      );
      final shared = timeout();
      expect(
        QuoteScreenState.failed(input: _input, failure: shared),
        QuoteScreenState.failed(input: _input, failure: shared),
      );
      expect(
        QuoteScreenState.failed(input: _input, failure: timeout()),
        isNot(QuoteScreenState.failed(input: _input, failure: timeout())),
      );
    });

    test('copyWith của một nhánh giữ nguyên nhánh', () {
      const ready = QuoteReady(input: _input, fare: Money(25500));
      final changed = ready.copyWith(fare: const Money(12005));
      expect(changed, isA<QuoteReady>());
      expect(changed.fare, const Money(12005));
      expect(changed.input, _input);
    });

    test('toString nêu tên nhánh và trường', () {
      expect(
        const QuoteScreenState.ready(
          input: _input,
          fare: Money(25500),
        ).toString(),
        allOf(startsWith('QuoteScreenState.ready('), contains('25.500 VND')),
      );
    });

    test('when và map vẫn có, ngoài pattern matching', () {
      const state = QuoteScreenState.ready(input: _input, fare: Money(25500));
      final text = state.when(
        idle: () => 'idle',
        loading: (_) => 'loading',
        ready: (_, fare) => 'ready $fare',
        failed: (_, _) => 'failed',
      );
      expect(text, 'ready 25.500 VND');
    });
  });
}
