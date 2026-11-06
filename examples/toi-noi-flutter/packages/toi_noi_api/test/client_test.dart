import 'dart:async';
import 'dart:convert';

import 'package:dio/dio.dart' hide Response;
import 'package:shelf/shelf.dart';
import 'package:test/test.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

void main() {
  group('quote', () {
    test('5 km -> 25.500 VND, khớp calculateFare của toi_noi_core', () async {
      final lab = Lab();
      final quote = await lab.client.quote(
        const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
      );
      expect(quote.toMoney(), const Money(25500));
      expect(quote.toMoney(), calculateFare(5));
      expect(quote.distanceMeters, 5000);
    });

    test('2,001 km -> 12.005 và 100 km -> 453.000', () async {
      final lab = Lab();
      for (final (km, vnd) in [(2.001, 12005), (100.0, 453000)]) {
        final quote = await lab.client.quote(
          QuoteRequestDto(serviceKind: ServiceKind.food, distanceKm: km),
        );
        expect(quote.fareVnd, vnd);
        expect(quote.toMoney(), calculateFare(km));
      }
    });

    test('request gửi đúng method, đường dẫn, header và JSON', () async {
      final lab = Lab();
      await lab.client.quote(
        const QuoteRequestDto(serviceKind: ServiceKind.parcel, distanceKm: 2.5),
      );
      final sent = lab.adapter.requests.single;
      expect(sent.method, 'POST');
      expect(sent.uri.toString(), 'http://fake.test/quotes');
      expect(sent.headers['accept'], 'application/json');
      expect(sent.headers['content-type'], startsWith('application/json'));
      expect(jsonDecode(sent.body), {
        'serviceKind': 'parcel',
        'distanceKm': 2.5,
      });
    });

    test('ngoài miền: ApiException 422 kèm lỗi theo trường', () async {
      final lab = Lab();
      await expectLater(
        lab.client.quote(
          const QuoteRequestDto(
            serviceKind: ServiceKind.ride,
            distanceKm: 100.5,
          ),
        ),
        throwsA(
          isA<ApiException>()
              .having((e) => e.statusCode, 'statusCode', 422)
              .having((e) => e.fieldErrors['distanceKm'], 'distanceKm', [
                'Expected 0..100 km',
              ])
              .having((e) => e.traceId, 'traceId', isNotNull),
        ),
      );
    });

    test('NaN bị chặn ở client, không gửi request nào', () async {
      final lab = Lab();
      await expectLater(
        lab.client.quote(
          const QuoteRequestDto(
            serviceKind: ServiceKind.ride,
            distanceKm: double.nan,
          ),
        ),
        throwsArgumentError,
      );
      expect(lab.adapter.requests, isEmpty);
    });
  });

  group('createTrip', () {
    const request = QuoteRequestDto(
      serviceKind: ServiceKind.ride,
      distanceKm: 5,
    );

    test('201, gửi Idempotency-Key và map sang Trip', () async {
      final lab = Lab();
      final dto = await lab.client.createTrip(request, idempotencyKey: 'k-1');
      expect(lab.adapter.requests.single.headers['idempotency-key'], 'k-1');
      expect(dto.status, TripStatus.requested);
      expect(
        dto.toDomain(),
        Trip.request(
          id: 'T-1',
          serviceKind: ServiceKind.ride,
          distanceKm: 5,
          requestedAt: fixedNow,
        ),
      );
    });

    test('gửi lại cùng khoá: cùng chuyến, server chỉ có một', () async {
      final lab = Lab();
      final first = await lab.client.createTrip(request, idempotencyKey: 'k-1');
      final again = await lab.client.createTrip(request, idempotencyKey: 'k-1');
      expect(again.id, first.id);
      expect(lab.store.trips, hasLength(1));
    });

    test('khoá khác: chuyến khác', () async {
      final lab = Lab();
      await lab.client.createTrip(request, idempotencyKey: 'k-1');
      await lab.client.createTrip(request, idempotencyKey: 'k-2');
      expect(lab.store.trips, hasLength(2));
    });

    test('cùng khoá nhưng đổi nội dung: ApiException 409', () async {
      final lab = Lab();
      await lab.client.createTrip(request, idempotencyKey: 'k-1');
      await expectLater(
        lab.client.createTrip(
          const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 6),
          idempotencyKey: 'k-1',
        ),
        throwsA(
          isA<ApiException>().having((e) => e.statusCode, 'statusCode', 409),
        ),
      );
    });

    test('khoá sai định dạng bị chặn trước khi gửi', () async {
      final lab = Lab();
      for (final key in ['', 'có dấu', 'a b', 'k' * 129]) {
        await expectLater(
          lab.client.createTrip(request, idempotencyKey: key),
          throwsArgumentError,
          reason: key,
        );
      }
      expect(lab.adapter.requests, isEmpty);
    });
  });

  group('getTrip', () {
    test('đọc lại chuyến đã tạo', () async {
      final lab = Lab();
      await lab.client.createTrip(
        const QuoteRequestDto(serviceKind: ServiceKind.food, distanceKm: 2.001),
        idempotencyKey: 'k-1',
      );
      final dto = await lab.client.getTrip('T-1');
      expect(dto.fareVnd, 12005);
      expect(dto.serviceKind, ServiceKind.food);
    });

    test('không tồn tại: ApiException 404 "Trip not found"', () async {
      final lab = Lab();
      await expectLater(
        lab.client.getTrip('khong-co'),
        throwsA(
          isA<ApiException>()
              .having((e) => e.statusCode, 'statusCode', 404)
              .having((e) => e.title, 'title', 'Trip not found'),
        ),
      );
    });

    test('?, # và khoảng trắng trong mã chuyến được mã hoá', () async {
      final seen = <Uri>[];
      final stub = clientFor((request) {
        seen.add(request.requestedUri);
        return Response.notFound('');
      });
      await expectLater(
        stub.client.getTrip('a?x=1#z b'),
        throwsA(isA<ApiException>()),
      );
      expect(seen.single.path, '/trips/a%3Fx%3D1%23z%20b');
      expect(seen.single.query, isEmpty);
    });

    test('mã rỗng, ".", ".." hoặc chứa "/" bị từ chối trước khi gửi', () async {
      // Không từ chối thì: '..' gửi tới /api/v1/ (dio chuẩn hoá đường dẫn),
      // '' gửi tới /trips/, và acceptTrip('..') gửi POST /api/v1/accept.
      final seen = <Uri>[];
      final stub = clientFor((request) {
        seen.add(request.requestedUri);
        return Response.notFound('');
      }, baseUrl: 'http://stub.test/api/v1');
      for (final id in ['', '.', '..', 'a/b', '../admin', '/']) {
        await expectLater(
          stub.client.getTrip(id),
          throwsArgumentError,
          reason: 'getTrip("$id")',
        );
        await expectLater(
          stub.client.acceptTrip(id, expectedVersion: 0),
          throwsArgumentError,
          reason: 'acceptTrip("$id")',
        );
      }
      expect(seen, isEmpty);
      expect(stub.adapter.requests, isEmpty);
    });

    test('bằng chứng: dio chuẩn hoá ".." ra khỏi /trips nếu không chặn', () {
      final options = RequestOptions(
        baseUrl: 'http://stub.test/api/v1',
        path: '/trips/..',
      );
      expect(options.uri.path, '/api/v1/');
    });
  });

  group('acceptTrip', () {
    Future<Lab> labWithTrip() async {
      final lab = Lab();
      await lab.client.createTrip(
        const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
        idempotencyKey: 'k-1',
      );
      return lab;
    }

    test('tài xế nhận: accepted, có driverId, version tăng', () async {
      final lab = await labWithTrip();
      final dto = await lab.driver('D-1').acceptTrip('T-1', expectedVersion: 0);
      expect(dto.status, TripStatus.accepted);
      expect(dto.version, 1);
      final trip = dto.toDomain();
      expect(trip.driverId, 'D-1');
      expect(trip.status, TripStatus.accepted);
    });

    test('khách đọc lại thấy trạng thái mới', () async {
      final lab = await labWithTrip();
      await lab.driver('D-1').acceptTrip('T-1', expectedVersion: 0);
      final seen = await lab.client.getTrip('T-1');
      expect(seen.status, TripStatus.accepted);
      expect(seen.driverId, 'D-1');
    });

    test('nhận trùng: ApiException 409', () async {
      final lab = await labWithTrip();
      await lab.driver('D-1').acceptTrip('T-1', expectedVersion: 0);
      await expectLater(
        lab.driver('D-2').acceptTrip('T-1', expectedVersion: 1),
        throwsA(
          isA<ApiException>().having((e) => e.statusCode, 'statusCode', 409),
        ),
      );
    });

    test('chuyến không có: 404', () async {
      final lab = Lab();
      await expectLater(
        lab.driver('D-1').acceptTrip('T-9', expectedVersion: 0),
        throwsA(
          isA<ApiException>().having((e) => e.statusCode, 'statusCode', 404),
        ),
      );
    });
  });

  group('lỗi server và payload sai', () {
    test('500 do fake tiêm: ApiException 500 có traceId', () async {
      final lab = Lab(faults: const FaultConfig(failFirst: 1));
      await expectLater(
        lab.client.quote(
          const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
        ),
        throwsA(
          isA<ApiException>()
              .having((e) => e.statusCode, 'statusCode', 500)
              .having((e) => e.title, 'title', 'Unexpected server error')
              .having((e) => e.traceId, 'traceId', startsWith('fake-')),
        ),
      );
    });

    test(
      'lỗi không phải problem+json (HTML của proxy): vẫn là ApiException',
      () async {
        final stub = clientFor(
          (request) => Response(
            502,
            body: '<html>Bad Gateway</html>',
            headers: const {'content-type': 'text/html'},
          ),
        );
        await expectLater(
          stub.client.getTrip('T-1'),
          throwsA(
            isA<ApiException>()
                .having((e) => e.statusCode, 'statusCode', 502)
                .having((e) => e.title, 'title', 'HTTP 502')
                .having((e) => e.fieldErrors, 'fieldErrors', isEmpty)
                .having((e) => e.traceId, 'traceId', isNull),
          ),
        );
      },
    );

    test('200 nhưng JSON sai hợp đồng: DtoFormatException, không phải ApiException', () async {
      final stub = clientFor(
        (request) => okJson({
          'serviceKind': 'ride',
          'distanceMeters': 5000,
          'fare': '25500',
          'currency': 'VND',
        }),
      );
      await expectLater(
        stub.client.quote(
          const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
        ),
        throwsA(
          isA<DtoFormatException>()
              .having((e) => e.dto, 'dto', 'QuoteDto')
              .having((e) => e.field, 'field', 'fare'),
        ),
      );
    });

    test('200 nhưng body là HTML: DtoFormatException ở gốc payload', () async {
      final stub = clientFor(
        (request) => Response.ok(
          '<html>captive portal</html>',
          headers: const {'content-type': 'text/html'},
        ),
      );
      await expectLater(
        stub.client.getTrip('T-1'),
        throwsA(
          isA<DtoFormatException>().having((e) => e.field, 'field', r'$'),
        ),
      );
    });

    test(
      '200 nhưng JSON bị cắt dở: DtoFormatException, không phải lỗi của dio',
      () async {
        final stub = clientFor(
          (request) => Response.ok(
            '{"id": "T-1", "serviceKind": "ri',
            headers: const {'content-type': 'application/json'},
          ),
        );
        await expectLater(
          stub.client.getTrip('T-1'),
          throwsA(
            isA<DtoFormatException>()
                .having((e) => e.dto, 'dto', 'TripDto')
                .having((e) => e.field, 'field', r'$'),
          ),
        );
      },
    );

    test('200 body rỗng: DtoFormatException', () async {
      final stub = clientFor((request) => Response.ok(''));
      await expectLater(
        stub.client.quote(
          const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
        ),
        throwsA(isA<DtoFormatException>()),
      );
    });

    test('200 mà JSON không phải object (mảng): DtoFormatException', () async {
      final stub = clientFor((request) => okJson([1, 2, 3]));
      await expectLater(
        stub.client.getTrip('T-1'),
        throwsA(isA<DtoFormatException>()),
      );
    });

    test(
      '500 với body không phải JSON vẫn giữ mã 500 trong ApiException',
      () async {
        final stub = clientFor(
          (request) => Response(
            500,
            body: 'upstream connect error',
            headers: const {'content-type': 'text/plain'},
          ),
        );
        await expectLater(
          stub.client.getTrip('T-1'),
          throwsA(
            isA<ApiException>()
                .having((e) => e.statusCode, 'statusCode', 500)
                .having((e) => e.title, 'title', 'HTTP 500'),
          ),
        );
      },
    );

    test('500 với JSON hỏng dở vẫn giữ mã 500', () async {
      final stub = clientFor(
        (request) => Response(
          500,
          body: '{"title": "Unexp',
          headers: const {'content-type': 'application/problem+json'},
        ),
      );
      await expectLater(
        stub.client.getTrip('T-1'),
        throwsA(
          isA<ApiException>().having((e) => e.statusCode, 'statusCode', 500),
        ),
      );
    });

    test('server chậm hơn receiveTimeout: DioException receiveTimeout, không phải ApiException', () async {
      final never = Completer<Response>();
      final stub = clientFor(
        (request) => never.future,
        receiveTimeout: const Duration(milliseconds: 20),
      );
      await expectLater(
        stub.client.getTrip('T-1'),
        throwsA(
          isA<DioException>().having(
            (e) => e.type,
            'type',
            DioExceptionType.receiveTimeout,
          ),
        ),
      );
    });
  });

  group('baseUrl', () {
    Future<String> pathFor(String baseUrl) async {
      final seen = <Uri>[];
      final stub = clientFor((request) {
        seen.add(request.requestedUri);
        return okJson({
          'serviceKind': 'ride',
          'distanceMeters': 5000,
          'fare': 25500,
          'currency': 'VND',
        });
      }, baseUrl: baseUrl);
      await stub.client.quote(
        const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
      );
      return seen.single.toString();
    }

    test('có hay không có dấu / cuối, có hay không tiền tố gateway', () async {
      expect(await pathFor('http://x.test'), 'http://x.test/quotes');
      expect(await pathFor('http://x.test/'), 'http://x.test/quotes');
      expect(
        await pathFor('http://x.test/api/v1'),
        'http://x.test/api/v1/quotes',
      );
      expect(
        await pathFor('https://x.test:8443/api/v1/'),
        'https://x.test:8443/api/v1/quotes',
      );
    });

    test('địa chỉ không tuyệt đối hoặc không phải http(s) bị từ chối', () {
      for (final bad in [
        '/api',
        'localhost:8787',
        'ftp://x.test',
        'http://',
        'http://x.test/api?y=1',
        'http://x.test/api#frag',
      ]) {
        expect(
          () => ToiNoiApiClient(baseUrl: Uri.parse(bad)),
          throwsArgumentError,
          reason: bad,
        );
      }
    });

    test('baseUrl và timeout mặc định có thể đọc lại', () {
      final client = ToiNoiApiClient(baseUrl: Uri.parse('http://x.test/'));
      expect(client.baseUrl, 'http://x.test');
      expect(client.dio.options.connectTimeout, const Duration(seconds: 10));
      expect(client.dio.options.receiveTimeout, const Duration(seconds: 15));
    });
  });

  test(
    'toàn luồng: báo giá, đặt, tài xế nhận, khách thấy trạng thái mới',
    () async {
      final lab = Lab();
      const request = QuoteRequestDto(
        serviceKind: ServiceKind.ride,
        distanceKm: 5,
      );
      final quote = await lab.client.quote(request);
      final booked = await lab.client.createTrip(
        request,
        idempotencyKey: 'k-1',
      );
      expect(booked.fareVnd, quote.fareVnd);
      final accepted = await lab
          .driver('D-1')
          .acceptTrip(booked.id, expectedVersion: booked.version);
      final seen = await lab.client.getTrip(booked.id);
      expect(seen.status, TripStatus.accepted);
      expect(seen.toDomain(), accepted.toDomain());
      expect(lab.adapter.requests.map((r) => '${r.method} ${r.uri.path}'), [
        'POST /quotes',
        'POST /trips',
        'POST /trips/T-1/accept',
        'GET /trips/T-1',
      ]);
    },
  );
}
