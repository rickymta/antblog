import 'dart:convert';

import 'package:dio/dio.dart' hide Response;
import 'package:flutter_test/flutter_test.dart';
import 'package:shelf/shelf.dart' show Handler, Request, Response;
import 'package:toi_noi_26/dto/quote_dto.dart' as gen;
import 'package:toi_noi_26/dto/trip_dto.dart' as gen;
import 'package:toi_noi_26/quote_source.dart';
import 'package:toi_noi_api/toi_noi_api.dart' as hand;
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/testing.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

/// Giải cùng một body JSON bằng hai DTO, trên dữ liệu thật của fake API chứ
/// không phải fixture tự viết.
Dio _dio(Handler handler) => hand.ToiNoiApiClient(
  baseUrl: Uri.parse('http://fake.test'),
  adapter: InProcessAdapter(handler),
).dio;

Handler _fixed(String body, {int status = 200}) =>
    (Request request) => Response(
      status,
      body: body,
      headers: {'content-type': 'application/json'},
    );

void main() {
  late Dio dio;

  setUp(() {
    var n = 0;
    dio = _dio(
      createFakeApiHandler(
        clock: () => DateTime.utc(2026, 11, 6, 3, 30),
        newId: () => 'T-${(++n).toString().padLeft(4, '0')}',
      ),
    );
  });

  Future<String> body(
    String method,
    String path, {
    Object? data,
    Map<String, String>? headers,
  }) async {
    final response = await dio.request<String>(
      path,
      data: data,
      options: Options(
        method: method,
        headers: headers,
        responseType: ResponseType.plain,
      ),
    );
    return response.data!;
  }

  test(
    'báo giá 5 km: hai DTO đọc cùng body ra cùng giá trị, 25.500 VND',
    () async {
      final text = await body(
        'POST',
        '/quotes',
        data: {'serviceKind': 'ride', 'distanceKm': 5},
      );
      final json = _decode(text);
      final generated = gen.QuoteDto.fromJson(json);
      final written = hand.QuoteDto.fromJson(json);
      expect(generated.toJson(), written.toJson());
      expect(generated.toMoney(), calculateFare(5));
      expect(generated.toMoney(), const Money(25500));
    },
  );

  test(
    'đặt chuyến, đọc lại, tài xế nhận: ba body, hai DTO cùng kết quả',
    () async {
      final created = await body(
        'POST',
        '/trips',
        data: {'serviceKind': 'parcel', 'distanceKm': 2.001},
        headers: {'idempotency-key': 'k-1'},
      );
      final fetched = await body('GET', '/trips/T-0001');
      final accepted = await body(
        'POST',
        '/trips/T-0001/accept',
        data: {'expectedVersion': 0},
        headers: {fakeDriverIdHeader: 'D-9'},
      );

      for (final text in [created, fetched, accepted]) {
        final json = _decode(text);
        expect(
          gen.TripDto.fromJson(json).toJson(),
          hand.TripDto.fromJson(json).toJson(),
        );
        expect(
          gen.TripDto.fromJson(json).toSnapshot(),
          hand.TripDto.fromJson(json).toSnapshot(),
        );
      }
      final snapshot = gen.TripDto.fromJson(_decode(accepted)).toSnapshot();
      expect(snapshot.trip.status, TripStatus.accepted);
      expect(snapshot.trip.driverId, 'D-9');
      expect(snapshot.trip.fare, calculateFare(2.001));
      expect(snapshot.trip.fare.vnd, 12005);
      expect(snapshot.version, 1);
    },
  );

  group('DioQuoteSource trả mọi lỗi dưới dạng ApiFailure', () {
    test('thành công đi qua fake API thật', () async {
      final quote = await DioQuoteSource(dio)
          .quote(serviceKind: ServiceKind.ride, distanceKm: 5);
      expect(quote.toMoney().vnd, 25500);
    });

    test('404 thành ClientFailure', () async {
      await expectLater(
        DioQuoteSource(dio).trip('khong-co'),
        throwsA(
          isA<hand.ClientFailure>().having(
            (f) => f.statusCode,
            'statusCode',
            404,
          ),
        ),
      );
    });

    test('200 nhưng cước lẻ thành ParseFailure nêu đúng trường fare', () async {
      final source = DioQuoteSource(
        _dio(
          _fixed(
            '{"serviceKind":"ride","distanceMeters":5000,"fare":25500.5,"currency":"VND"}',
          ),
        ),
      );
      await expectLater(
        source.quote(serviceKind: ServiceKind.ride, distanceKm: 5),
        throwsA(
          isA<hand.ParseFailure>()
              .having((f) => f.dto, 'dto', 'QuoteDto')
              .having((f) => f.field, 'field', 'fare'),
        ),
      );
    });

    test('200 nhưng body không phải JSON thành ParseFailure ở gốc', () async {
      final source = DioQuoteSource(_dio(_fixed('<html>proxy</html>')));
      await expectLater(
        source.trip('T-1'),
        throwsA(isA<hand.ParseFailure>().having((f) => f.field, 'field', r'$')),
      );
    });

    test('500 thành ServerFailure', () async {
      final source = DioQuoteSource(
        _dio(_fixed('{"title":"boom"}', status: 500)),
      );
      await expectLater(source.trip('T-1'), throwsA(isA<hand.ServerFailure>()));
    });
  });
}

Object? _decode(String text) => jsonDecode(text);
