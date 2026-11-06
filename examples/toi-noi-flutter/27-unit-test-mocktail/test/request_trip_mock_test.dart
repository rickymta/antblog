import 'package:mocktail/mocktail.dart';
import 'package:test/test.dart';
import 'package:toi_noi_27/booking/booking.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/doubles.dart';

/// Test use case bằng mock của mocktail. Mock hợp ở đây khi chính tương tác
/// là điều cần kiểm: nhật ký được gọi đúng một lần, hay không được gọi.
void main() {
  setUpAll(registerFallbacks);

  late MockTripRepository repository;
  late MockBookingLog log;
  late RequestTrip requestTrip;
  late int issued;
  final trip = tripFor(ride5km);

  /// Stub dùng chung: mọi lần đặt đều trả [trip].
  void stubRequestTrip() {
    when(() => repository.requestTrip(any(), intentId: any(named: 'intentId')))
        .thenAnswer((_) async => trip);
  }

  setUp(() {
    repository = MockTripRepository();
    log = MockBookingLog();
    issued = 0;
    requestTrip = RequestTrip(
      repository,
      log,
      newIntentId: () => 'intent-${++issued}',
    );
  });

  group('stub trả lời, verify kiểm tương tác', () {
    test(
      'thành công: gọi repository một lần với intentId, ghi nhật ký booked',
      () async {
        stubRequestTrip();

        final result = await requestTrip(ride5km);

        expect(result, trip);
        verify(() => repository.requestTrip(ride5km, intentId: 'intent-1'))
            .called(1);
        verify(() => log.booked(trip)).called(1);
        verifyNever(() => log.failed(any(), any()));
      },
    );

    test('captureAny lấy ra đối số thật đã truyền', () async {
      stubRequestTrip();

      await requestTrip(food3km);

      final captured = verify(
        () => repository.requestTrip(
          captureAny(),
          intentId: captureAny(named: 'intentId'),
        ),
      ).captured;
      expect(captured, [food3km, 'intent-1']);
    });

    test('thenAnswer đổi câu trả lời theo từng lần gọi', () async {
      var calls = 0;
      when(
        () => repository.requestTrip(any(), intentId: any(named: 'intentId')),
      ).thenAnswer((invocation) async {
        calls++;
        if (calls == 1) {
          throw const Unreachable();
        }
        return tripFor(
          invocation.positionalArguments.first as QuoteRequest,
          id: 'T-${invocation.namedArguments[#intentId]}',
        );
      });

      await expectLater(requestTrip(ride5km), throwsA(isA<Unreachable>()));
      final second = await requestTrip(ride5km);

      expect(second.id, 'T-intent-1');
      verify(() => repository.requestTrip(ride5km, intentId: 'intent-1'))
          .called(2);
    });

    test('thứ tự: repository trước, nhật ký sau', () async {
      stubRequestTrip();

      await requestTrip(ride5km);

      verifyInOrder([
        () => repository.requestTrip(ride5km, intentId: 'intent-1'),
        () => log.booked(trip),
      ]);
    });
  });

  group('verifyNever và verifyZeroInteractions', () {
    test('ngoài miền: repository và nhật ký không bị chạm', () async {
      const outside = QuoteRequest(
        serviceKind: ServiceKind.ride,
        distanceKm: 100.001,
      );

      await expectLater(requestTrip(outside), throwsA(isA<Rejected>()));

      verifyZeroInteractions(repository);
      verifyZeroInteractions(log);
      expect(issued, 0);
    });

    test('verifyNever bắt được lời gọi lẽ ra không có', () async {
      stubRequestTrip();
      await requestTrip(ride5km);

      expect(
        () => verifyNever(
          () => repository.requestTrip(any(), intentId: any(named: 'intentId')),
        ),
        throwsA(isA<TestFailure>()),
      );
    });
  });

  group('thenThrow và nhật ký thất bại', () {
    test('TripFailure: ghi failed với đúng yêu cầu rồi ném lại', () async {
      when(
        () => repository.requestTrip(any(), intentId: any(named: 'intentId')),
      ).thenAnswer((_) => Future.error(const Unreachable()));

      await expectLater(requestTrip(ride5km), throwsA(isA<Unreachable>()));

      verify(() => log.failed(ride5km, any(that: isA<Unreachable>())))
          .called(1);
      verifyNever(() => log.booked(any()));
    });

    test(
      'lỗi lập trình (không phải TripFailure) không bị ghi như lỗi nghiệp vụ',
      () async {
        when(
          () => repository.requestTrip(any(), intentId: any(named: 'intentId')),
        ).thenAnswer((_) => Future.error(StateError('bug')));

        await expectLater(requestTrip(ride5km), throwsStateError);

        verifyZeroInteractions(log);
      },
    );
  });
}
