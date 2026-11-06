import 'package:shelf/shelf.dart';
import 'package:test/test.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

final _now = DateTime.utc(2026, 11, 6, 3, 30);

void main() {
  late FakeApiStore store;
  late Handler handler;
  var seq = 0;

  setUp(() {
    seq = 0;
    store = FakeApiStore();
    handler = createFakeApiHandler(
      store: store,
      clock: () => _now,
      newId: () => 'trip-${++seq}',
    );
  });

  Future<Reply> create({
    String key = 'book-1',
    double km = 5,
    String kind = 'ride',
  }) => call(
    handler,
    'POST',
    '/trips',
    body: {'serviceKind': kind, 'distanceKm': km},
    headers: {'Idempotency-Key': key},
  );

  Future<Reply> accept(
    String id, {
    Object? expectedVersion = 0,
    String? driver = 'D-1',
  }) => call(
    handler,
    'POST',
    '/trips/$id/accept',
    body: {'expectedVersion': ?expectedVersion},
    headers: {'x-fake-driver-id': ?driver},
  );

  group('POST /trips', () {
    test('201, Location và chuyến requested với cước đúng công thức', () async {
      final reply = await create();
      expect(reply.status, 201);
      expect(reply.headers['location'], '/trips/trip-1');
      expect(asMap(reply), {
        'id': 'trip-1',
        'serviceKind': 'ride',
        'status': 'requested',
        'distanceMeters': 5000,
        'fare': 25500,
        'currency': 'VND',
        'driverId': null,
        'requestedAt': '2026-11-06T03:30:00.000Z',
        'version': 0,
      });
    });

    test('thiếu Idempotency-Key -> 422', () async {
      final reply = await call(
        handler,
        'POST',
        '/trips',
        body: {'serviceKind': 'ride', 'distanceKm': 5},
      );
      expect(reply.status, 422);
      expect(errorsOf(reply).keys, contains('Idempotency-Key'));
      expect(store.trips, isEmpty);
    });

    test('khoá chứa ký tự lạ -> 422', () async {
      final reply = await create(key: 'book 1!');
      expect(reply.status, 422);
    });

    test('khoá dài hơn 128 ký tự -> 422', () async {
      expect((await create(key: 'k' * 129)).status, 422);
      expect((await create(key: 'k' * 128)).status, 201);
    });

    test('ngoài miền km -> 422 và không tạo chuyến', () async {
      final reply = await create(km: 100.5);
      expect(reply.status, 422);
      expect(errorsOf(reply)['distanceKm'], ['Expected 0..100 km']);
      expect(store.trips, isEmpty);
    });

    test(
      'cùng khoá, cùng đầu vào: 201 và đúng chuyến cũ (không tạo thêm)',
      () async {
        final first = await create();
        final replay = await create();
        expect(replay.status, 201);
        expect(asMap(replay)['id'], asMap(first)['id']);
        expect(store.trips, hasLength(1));
      },
    );

    test('cùng khoá, đầu vào khác -> 409', () async {
      await create();
      final reply = await create(km: 6);
      expect(reply.status, 409);
      expect(store.trips, hasLength(1));
    });

    test('khoá khác -> chuyến khác', () async {
      await create(key: 'a');
      final second = await create(key: 'b');
      expect(asMap(second)['id'], 'trip-2');
      expect(store.trips, hasLength(2));
    });
  });

  group('GET /trips/{id}', () {
    test('đọc lại chuyến vừa tạo', () async {
      await create(km: 2.001, kind: 'parcel');
      final reply = await call(handler, 'GET', '/trips/trip-1');
      expect(reply.status, 200);
      expect(asMap(reply)['fare'], 12005);
      expect(asMap(reply)['serviceKind'], 'parcel');
      expect(asMap(reply)['distanceMeters'], 2001);
    });

    test('mã không tồn tại -> 404 problem+json có traceId', () async {
      final reply = await call(handler, 'GET', '/trips/khong-co');
      expect(reply.status, 404);
      expect(asMap(reply)['title'], 'Trip not found');
      expect(asMap(reply)['traceId'], matches(RegExp(r'^fake-\d{6}$')));
    });
  });

  group('POST /trips/{id}/accept', () {
    test('200, trạng thái accepted, có driverId, version tăng lên 1', () async {
      await create();
      final reply = await accept('trip-1');
      expect(reply.status, 200);
      expect(asMap(reply)['status'], 'accepted');
      expect(asMap(reply)['driverId'], 'D-1');
      expect(asMap(reply)['version'], 1);
      expect(store.trip('trip-1')!.trip.driverId, 'D-1');
    });

    test('accept trùng với phiên bản cũ -> 409 (bản đã cũ)', () async {
      await create();
      await accept('trip-1');
      final reply = await accept('trip-1', driver: 'D-2');
      expect(reply.status, 409);
      expect(store.trip('trip-1')!.trip.driverId, 'D-1');
    });

    test('accept trùng dù đúng phiên bản mới -> 409 (đã có tài xế)', () async {
      await create();
      await accept('trip-1');
      final reply = await accept('trip-1', expectedVersion: 1, driver: 'D-2');
      expect(reply.status, 409);
      expect(store.trip('trip-1')!.trip.driverId, 'D-1');
    });

    test('expectedVersion sai -> 409 và chuyến vẫn requested', () async {
      await create();
      final reply = await accept('trip-1', expectedVersion: 7);
      expect(reply.status, 409);
      expect(store.trip('trip-1')!.trip.status.name, 'requested');
    });

    test('thiếu header tài xế -> 403', () async {
      await create();
      expect((await accept('trip-1', driver: null)).status, 403);
      expect((await accept('trip-1', driver: '  ')).status, 403);
    });

    test('chuyến không tồn tại -> 404', () async {
      expect((await accept('khong-co')).status, 404);
    });

    test('thiếu hoặc sai kiểu expectedVersion -> 422', () async {
      await create();
      expect((await accept('trip-1', expectedVersion: null)).status, 422);
      expect((await accept('trip-1', expectedVersion: '0')).status, 422);
      expect((await accept('trip-1', expectedVersion: -1)).status, 422);
    });
  });

  group('route lạ', () {
    test('404 problem+json', () async {
      final reply = await call(handler, 'GET', '/khong-co-route');
      expect(reply.status, 404);
      expect(asMap(reply)['title'], 'Route not found');
    });

    test(
      'đúng route nhưng sai phương thức -> 404 (router không có route)',
      () async {
        final reply = await call(handler, 'DELETE', '/trips/trip-1');
        expect(reply.status, 404);
      },
    );
  });
}
