import 'package:mocktail/mocktail.dart';
import 'package:test/test.dart';
import 'package:toi_noi_27/booking/booking.dart';

import 'support/doubles.dart';

/// Ba cái bẫy của mocktail đã gặp khi viết lab này, mỗi bẫy một test chứng
/// minh. Mọi bẫy đều đỏ ở chỗ khác với chỗ ta nghĩ.
void main() {
  setUpAll(registerFallbacks);
  tearDown(resetMocktailState);

  late MockTripRepository repository;

  setUp(() => repository = MockTripRepository());

  group('thenThrow với phương thức trả Future', () {
    test('ném đồng bộ: chưa có Future nào được trả về', () {
      when(
        () => repository.requestTrip(any(), intentId: any(named: 'intentId')),
      ).thenThrow(const Unreachable());

      expect(
        () => repository.requestTrip(ride5km, intentId: 'i-1'),
        throwsA(isA<Unreachable>()),
      );
    });

    test(
      'thenAnswer với Future.error: trả về Future đã hỏng, như repository thật',
      () async {
        when(
          () => repository.requestTrip(any(), intentId: any(named: 'intentId')),
        ).thenAnswer((_) => Future.error(const Unreachable()));

        final future = repository.requestTrip(ride5km, intentId: 'i-1');

        expect(future, isA<Future<Object?>>());
        await expectLater(future, throwsA(isA<Unreachable>()));
      },
    );

    test('code dùng .catchError chỉ bắt được bản Future', () async {
      Future<String> describe(TripRepository r) => r
          .requestTrip(ride5km, intentId: 'i-1')
          .then((trip) => trip.id)
          .catchError((Object _) => 'lỗi');

      when(
        () => repository.requestTrip(any(), intentId: any(named: 'intentId')),
      ).thenAnswer((_) => Future.error(const Unreachable()));
      expect(await describe(repository), 'lỗi');

      when(
        () => repository.requestTrip(any(), intentId: any(named: 'intentId')),
      ).thenThrow(const Unreachable());
      expect(() => describe(repository), throwsA(isA<Unreachable>()));
    });
  });

  group('phương thức chưa được stub', () {
    test('trả Future: TypeError ngay chỗ gọi, không phải lỗi rõ ràng', () {
      expect(
        () => repository.requestTrip(ride5km, intentId: 'i-1'),
        throwsA(
          isA<TypeError>().having(
            (e) => e.toString(),
            'message',
            contains("'Null' is not a subtype of type 'Future<Trip>'"),
          ),
        ),
      );
    });

    test('trả void: chạy lặng lẽ, nên mock nhật ký không cần stub', () {
      final log = MockBookingLog();
      expect(() => log.booked(tripFor(ride5km)), returnsNormally);
    });
  });

  group('any() cho tham số có tên', () {
    test('thiếu named: bị từ chối với thông báo nêu đúng cách sửa', () {
      expect(
        () => when(() => repository.requestTrip(any(), intentId: any())),
        throwsA(
          isA<ArgumentError>().having(
            (e) => e.toString(),
            'message',
            contains('without the Mocktail "named" API'),
          ),
        ),
      );
    });
  });

  group('một lời gọi chỉ được verify một lần', () {
    test('lời gọi đã VERIFIED không khớp lần verify thứ hai', () async {
      when(
        () => repository.requestTrip(any(), intentId: any(named: 'intentId')),
      ).thenAnswer((_) async => tripFor(ride5km));
      await repository.requestTrip(ride5km, intentId: 'i-1');

      verify(() => repository.requestTrip(ride5km, intentId: 'i-1')).called(1);

      expect(
        () => verify(() => repository.requestTrip(ride5km, intentId: 'i-1')),
        throwsA(isA<TestFailure>()),
      );
    });
  });
}
