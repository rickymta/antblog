import 'package:mocktail/mocktail.dart';
import 'package:toi_noi_27/booking/booking.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Mốc thời gian cố định để test không phụ thuộc đồng hồ thật.
final fixedNow = DateTime.utc(2026, 11, 6, 3, 30);

const ride5km = QuoteRequest(serviceKind: ServiceKind.ride, distanceKm: 5);
const food3km = QuoteRequest(serviceKind: ServiceKind.food, distanceKm: 3);

/// Chuyến mà test dùng khi không quan tâm nội dung; cước do `calculateFare`
/// tính, không gõ tay.
Trip tripFor(QuoteRequest request, {String id = 'T-1'}) => Trip.request(
  id: id,
  serviceKind: request.serviceKind,
  distanceKm: request.distanceKm,
  requestedAt: fixedNow,
);

/// DUMMY: có mặt cho đủ tham số, không được chạm tới. Bị gọi là test hỏng
/// ngay tại chỗ gọi, kèm tên phương thức.
final class DummyTripRepository implements TripRepository {
  @override
  dynamic noSuchMethod(Invocation invocation) => throw StateError(
    'Dummy bị gọi: ${invocation.memberName}. Test này không cho phép.',
  );
}

final class DummyBookingLog implements BookingLog {
  @override
  dynamic noSuchMethod(Invocation invocation) => throw StateError(
    'Dummy bị gọi: ${invocation.memberName}. Test này không cho phép.',
  );
}

/// SPY: không có kỳ vọng dựng sẵn. Chỉ ghi lại, test hỏi sau.
final class SpyBookingLog implements BookingLog {
  final List<Trip> bookedTrips = [];
  final List<({QuoteRequest request, TripFailure failure})> failedAttempts = [];

  @override
  void booked(Trip trip) => bookedTrips.add(trip);

  @override
  void failed(QuoteRequest request, TripFailure failure) =>
      failedAttempts.add((request: request, failure: failure));
}

/// FAKE: một `TripRepository` chạy thật, chỉ khác là sống trong bộ nhớ. Nó cư
/// xử như dịch vụ thật ở điều use case dựa vào: cùng `intentId` và cùng nội
/// dung thì trả lại đúng chuyến đã tạo, khác nội dung thì từ chối.
final class InMemoryTripRepository implements TripRepository {
  final Map<String, ({QuoteRequest request, Trip trip})> _byIntent = {};
  final List<({TripFailure failure, bool afterStore})> _script = [];

  /// Mọi `intentId` đã gửi tới, theo thứ tự, kể cả lần bị lỗi.
  final List<String> sentIntentIds = [];

  /// Các chuyến "server" đã tạo.
  List<Trip> get trips => [for (final e in _byIntent.values) e.trip];

  /// Lần gọi kế tiếp thất bại với [failure]. Với [afterStore] là `true`, server
  /// đã tạo chuyến rồi mới mất response: tình huống nguy hiểm nhất.
  void failNext(TripFailure failure, {bool afterStore = false}) =>
      _script.add((failure: failure, afterStore: afterStore));

  @override
  Future<Trip> requestTrip(
    QuoteRequest request, {
    required String intentId,
  }) async {
    sentIntentIds.add(intentId);
    final scripted = _script.isEmpty ? null : _script.removeAt(0);
    if (scripted != null && !scripted.afterStore) {
      throw scripted.failure;
    }
    final known = _byIntent[intentId];
    if (known != null && known.request != request) {
      throw const Rejected(RejectionKind.conflict);
    }
    final trip =
        known?.trip ?? tripFor(request, id: 'T-${_byIntent.length + 1}');
    _byIntent[intentId] = (request: request, trip: trip);
    if (scripted != null) {
      throw scripted.failure;
    }
    return trip;
  }

  @override
  Future<Trip> getTrip(String id) async {
    for (final entry in _byIntent.values) {
      if (entry.trip.id == id) {
        return entry.trip;
      }
    }
    throw const Rejected(RejectionKind.notFound);
  }
}

/// MOCK (của mocktail): đối tượng ghi lại mọi lời gọi để test kiểm bằng
/// `verify`. Chỉ dùng `when` thì nó là stub: vai do cách ta dùng.
class MockTripRepository extends Mock implements TripRepository {}

class MockBookingLog extends Mock implements BookingLog {}

/// Đăng ký giá trị dự phòng cho các kiểu của ta, để `any()` dùng được cho
/// tham số không nullable. Gọi trong `setUpAll`. Đăng ký xong không gỡ được.
/// `Trip` là `final class` nên không `implements Fake` được: dùng bản thật.
void registerFallbacks() {
  registerFallbackValue(ride5km);
  registerFallbackValue(const Unreachable());
  registerFallbackValue(tripFor(ride5km));
}
