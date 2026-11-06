import 'package:test/test.dart';
import 'package:toi_noi_27/booking/booking.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/doubles.dart';

/// Một fake dùng để làm chỗ dựa cho test khác thì cũng phải được kiểm: nếu nó
/// cư xử khác dịch vụ thật, mọi test dựa vào nó đều nói dối.
void main() {
  late InMemoryTripRepository repository;

  setUp(() => repository = InMemoryTripRepository());

  test('cùng intentId và cùng nội dung trả lại đúng chuyến đã tạo', () async {
    final first = await repository.requestTrip(ride5km, intentId: 'i-1');
    final again = await repository.requestTrip(ride5km, intentId: 'i-1');
    expect(again, first);
    expect(repository.trips, [first]);
  });

  test('cùng intentId nhưng khác nội dung bị từ chối conflict', () async {
    await repository.requestTrip(ride5km, intentId: 'i-1');
    await expectLater(
      repository.requestTrip(food3km, intentId: 'i-1'),
      throwsA(
        isA<Rejected>().having((r) => r.kind, 'kind', RejectionKind.conflict),
      ),
    );
    expect(repository.trips, hasLength(1));
  });

  test('intentId khác nhau tạo chuyến khác nhau, id tăng dần', () async {
    final a = await repository.requestTrip(ride5km, intentId: 'i-1');
    final b = await repository.requestTrip(ride5km, intentId: 'i-2');
    expect([a.id, b.id], ['T-1', 'T-2']);
  });

  test('failNext trước khi lưu: lỗi và chưa có chuyến nào', () async {
    repository.failNext(const Unreachable());
    await expectLater(
      repository.requestTrip(ride5km, intentId: 'i-1'),
      throwsA(isA<Unreachable>()),
    );
    expect(repository.trips, isEmpty);
  });

  test('failNext afterStore: lỗi nhưng server đã tạo chuyến', () async {
    repository.failNext(const Unreachable(), afterStore: true);
    await expectLater(
      repository.requestTrip(ride5km, intentId: 'i-1'),
      throwsA(isA<Unreachable>()),
    );
    expect(repository.trips, hasLength(1));
  });

  test('lỗi chỉ áp dụng cho một lần gọi', () async {
    repository.failNext(const ServiceFault());
    await expectLater(
      repository.requestTrip(ride5km, intentId: 'i-1'),
      throwsA(isA<ServiceFault>()),
    );
    final trip = await repository.requestTrip(ride5km, intentId: 'i-1');
    expect(trip.status, TripStatus.requested);
  });

  test('getTrip đọc đúng chuyến, không có thì notFound', () async {
    final trip = await repository.requestTrip(ride5km, intentId: 'i-1');
    expect(await repository.getTrip(trip.id), trip);
    await expectLater(
      repository.getTrip('T-99'),
      throwsA(
        isA<Rejected>().having((r) => r.kind, 'kind', RejectionKind.notFound),
      ),
    );
  });
}
