import 'package:mocktail/mocktail.dart';
import 'package:test/test.dart';
import 'package:toi_noi_27/booking/booking.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/doubles.dart';

typedef RequestTripLike = Future<Trip> Function(QuoteRequest request);

/// "Cải tiến" giữ nguyên điều khách thấy: sau khi đặt, đọc lại chuyến từ
/// server để lấy bản chuẩn của server (cùng id, cùng nội dung). Hành vi quan
/// sát được không đổi; chỉ có thêm một lời gọi `getTrip`.
final class ReadBackRequestTrip {
  ReadBackRequestTrip(this._inner, this._repository);

  final RequestTrip _inner;
  final TripRepository _repository;

  Future<Trip> call(QuoteRequest request) async {
    final trip = await _inner(request);
    return _repository.getTrip(trip.id);
  }
}

/// Kịch bản viết kiểu mock: dựng stub, chạy, rồi khoá chính xác các tương tác.
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

/// Cùng kịch bản viết kiểu fake: chỉ nhìn kết quả và trạng thái "server".
Future<void> _fakeScenario(
  RequestTripLike book,
  InMemoryTripRepository server,
) async {
  final trip = await book(ride5km);
  expect(trip.status, TripStatus.requested);
  expect(trip.fare, const Money(25500));
  expect(server.trips, [trip]);
}

RequestTrip _plain(TripRepository repo) =>
    RequestTrip(repo, SpyBookingLog(), newIntentId: () => 'intent-1');

ReadBackRequestTrip _readBack(TripRepository repo) =>
    ReadBackRequestTrip(_plain(repo), repo);

void main() {
  setUpAll(registerFallbacks);

  group('kịch bản viết kiểu mock', () {
    test('xanh với RequestTrip', () async {
      final repo = MockTripRepository();
      await _mockScenario(_plain(repo).call, repo);
    });

    test('gãy với bản đọc lại: mock chưa biết getTrip, TypeError', () async {
      final repo = MockTripRepository();
      await expectLater(
        _mockScenario(_readBack(repo).call, repo),
        throwsA(isA<TypeError>()),
      );
    });

    test('bổ sung stub getTrip vẫn gãy: verifyNoMoreInteractions', () async {
      final repo = MockTripRepository();
      when(() => repo.getTrip(any())).thenAnswer((_) async => tripFor(ride5km));
      await expectLater(
        _mockScenario(_readBack(repo).call, repo),
        throwsA(isA<TestFailure>()),
      );
    });
  });

  group('kịch bản viết kiểu fake', () {
    test('xanh với RequestTrip', () async {
      final server = InMemoryTripRepository();
      await _fakeScenario(_plain(server).call, server);
    });

    test('vẫn xanh với bản đọc lại', () async {
      final server = InMemoryTripRepository();
      await _fakeScenario(_readBack(server).call, server);
    });
  });

  test(
    'mock đúng chỗ: nhật ký không có gì để quan sát ngoài lời gọi',
    () async {
      final log = MockBookingLog();
      final useCase = RequestTrip(
        InMemoryTripRepository(),
        log,
        newIntentId: () => 'intent-1',
      );

      final trip = await useCase(ride5km);

      verify(() => log.booked(trip)).called(1);
    },
  );
}
