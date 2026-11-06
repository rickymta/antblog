import 'package:shelf/shelf.dart';
import 'package:shelf_router/shelf_router.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'input.dart';
import 'module.dart';
import 'problem.dart';
import 'store.dart';

/// Header giả lập danh tính tài xế khi nhận chuyến.
///
/// Service .NET lấy mã tài xế từ claim `sub` của JWT. Fake API chưa có xác
/// thực (bài 24), nên dùng header này thay thế; không dùng ở production.
const fakeDriverIdHeader = 'x-fake-driver-id';

/// Header chống đặt trùng, cùng tên với service .NET.
const idempotencyKeyHeader = 'idempotency-key';

final _keyPattern = RegExp(r'^[A-Za-z0-9_-]{1,128}$');

/// `POST /trips`, `GET /trips/{id}`, `POST /trips/{id}/accept`.
final class TripsModule implements FakeApiModule {
  const TripsModule();

  @override
  void mount(Router router, FakeApiContext context) {
    router.post('/trips', (Request request) => _create(request, context));
    router.get('/trips/<id>', (Request request) {
      final stored = context.store.trip(request.params['id']!);
      if (stored == null) {
        throw const ProblemException(404, 'Trip not found');
      }
      return jsonResponse(200, tripToJson(stored));
    });
    router.post('/trips/<id>/accept', (Request request) {
      return _accept(request, context);
    });
  }

  Future<Response> _create(Request request, FakeApiContext context) async {
    final key = request.headers[idempotencyKeyHeader] ?? '';
    if (!_keyPattern.hasMatch(key)) {
      throw const ProblemException(
        422,
        'Unprocessable request',
        errors: {
          'Idempotency-Key': [
            'Use 1..128 ASCII letters, digits, hyphen or underscore.',
          ],
        },
      );
    }
    final input = readTripInput(await readJsonObject(request));
    final meters = toMeters(input.distanceKm);

    final previous = context.store.booking(key);
    if (previous != null) {
      // Cùng khoá và cùng đầu vào: trả lại đúng chuyến đã tạo.
      if (previous.serviceKind != input.serviceKind ||
          previous.distanceMeters != meters) {
        throw const ProblemException(
          409,
          'Request conflicts with current state',
        );
      }
      return _created(context.store.trip(previous.tripId)!);
    }

    final trip = Trip.request(
      id: context.newId(),
      serviceKind: input.serviceKind,
      distanceKm: input.distanceKm,
      requestedAt: context.clock(),
    );
    final stored = StoredTrip(trip, 0);
    context.store.putTrip(stored);
    context.store.putBooking(
      key,
      Booking(
        serviceKind: input.serviceKind,
        distanceMeters: meters,
        tripId: trip.id,
      ),
    );
    return _created(stored);
  }

  Response _created(StoredTrip stored) => jsonResponse(
    201,
    tripToJson(stored),
    headers: {'location': '/trips/${stored.trip.id}'},
  );

  Future<Response> _accept(Request request, FakeApiContext context) async {
    final driverId = (request.headers[fakeDriverIdHeader] ?? '').trim();
    if (driverId.isEmpty) {
      throw const ProblemException(403, 'Forbidden');
    }
    final stored = context.store.trip(request.params['id']!);
    if (stored == null) {
      throw const ProblemException(404, 'Trip not found');
    }

    final body = await readJsonObject(request);
    final expected = body['expectedVersion'];
    if (expected is! int || expected < 0) {
      throw const ProblemException(
        422,
        'Unprocessable request',
        errors: {
          'expectedVersion': ['Nonnegative version required.'],
        },
      );
    }
    if (expected != stored.version) {
      throw const ProblemException(409, 'Request conflicts with current state');
    }
    if (stored.trip.status != TripStatus.requested) {
      // Chuyến đã có người nhận: đây là ca "accept trùng".
      throw const ProblemException(409, 'Request conflicts with current state');
    }

    // Fake không có bộ ghép cặp: đi qua `searching` rồi tài xế nhận ngay.
    final accepted = stored.trip
        .apply(const SearchStarted())
        .apply(DriverAccepted(driverId));
    final next = StoredTrip(accepted, stored.version + 1);
    context.store.putTrip(next);
    return jsonResponse(200, tripToJson(next));
  }
}

/// Hình dạng JSON của một chuyến (camelCase, tiền là `int` đồng, khoảng cách
/// là `int` mét, thời điểm UTC ISO-8601, trạng thái là tên enum).
Map<String, Object?> tripToJson(StoredTrip stored) {
  final trip = stored.trip;
  return <String, Object?>{
    'id': trip.id,
    'serviceKind': trip.serviceKind.name,
    'status': trip.status.name,
    'distanceMeters': toMeters(trip.distanceKm),
    'fare': trip.fare.vnd,
    'currency': 'VND',
    'driverId': trip.driverId,
    'requestedAt': trip.requestedAt.toUtc().toIso8601String(),
    'version': stored.version,
  };
}
