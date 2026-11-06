import 'dart:async';

import 'package:dio/dio.dart' as dio show Response;
import 'package:dio/dio.dart' hide Response;
import 'package:shelf/shelf.dart' show Response;
import 'package:test/test.dart';
import 'package:toi_noi_api/fault_injection.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/testing.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'retry_support.dart';
import 'support.dart';

const _tripBody = {'serviceKind': 'ride', 'distanceKm': 5};

/// Adapter ném thẳng một lỗi `DioException` chọn trước.
final class _ThrowingAdapter implements HttpClientAdapter {
  _ThrowingAdapter(this.make);

  final DioException Function(RequestOptions options) make;
  int calls = 0;

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<dynamic>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    calls++;
    throw make(options);
  }

  @override
  void close({bool force = false}) {}
}

void main() {
  group('backoff', () {
    RetryInterceptor make({
      double random = 1.0,
      Duration base = const Duration(milliseconds: 500),
      Duration max = const Duration(seconds: 8),
    }) => RetryInterceptor(
      dio: Dio(),
      baseDelay: base,
      maxDelay: max,
      random: () => random,
    );

    test('trần nhân đôi mỗi lần và dừng ở maxDelay', () {
      final retry = make();
      expect(
        [for (var attempt = 2; attempt <= 8; attempt++) retry.backoff(attempt)],
        [
          const Duration(milliseconds: 500),
          const Duration(seconds: 1),
          const Duration(seconds: 2),
          const Duration(seconds: 4),
          const Duration(seconds: 8),
          const Duration(seconds: 8),
          const Duration(seconds: 8),
        ],
      );
    });

    test('jitter: khoảng chờ là random nhân với trần, nằm trong [0, trần]', () {
      expect(make(random: 0).backoff(4), Duration.zero);
      expect(make(random: 0.5).backoff(4), const Duration(seconds: 1));
      expect(
        make(random: 0.999).backoff(4),
        lessThan(const Duration(seconds: 2)),
      );
    });

    test('số lần nhân đôi rất lớn không tràn số', () {
      expect(make().backoff(500), const Duration(seconds: 8));
    });

    test('với Random thật, mọi giá trị nằm trong [0, trần]', () {
      final retry = RetryInterceptor(dio: Dio());
      for (var i = 0; i < 200; i++) {
        final wait = retry.backoff(3);
        expect(wait, greaterThanOrEqualTo(Duration.zero));
        expect(wait, lessThanOrEqualTo(const Duration(seconds: 1)));
      }
    });
  });

  group('isRepeatable', () {
    final retry = RetryInterceptor(dio: Dio());
    RequestOptions req(
      String method, {
      Map<String, Object?>? headers,
      String path = '/x',
    }) => RequestOptions(path: path, method: method, headers: headers);

    test('GET và HEAD có, POST/PUT/DELETE trần thì không', () {
      expect(retry.isRepeatable(req('GET')), isTrue);
      expect(retry.isRepeatable(req('get')), isTrue);
      expect(retry.isRepeatable(req('HEAD')), isTrue);
      expect(retry.isRepeatable(req('POST')), isFalse);
      expect(retry.isRepeatable(req('PUT')), isFalse);
      expect(retry.isRepeatable(req('DELETE')), isFalse);
    });

    test('POST có Idempotency-Key (không phân biệt hoa thường) thì có', () {
      expect(
        retry.isRepeatable(req('POST', headers: {'Idempotency-Key': 'book-1'})),
        isTrue,
      );
      expect(
        retry.isRepeatable(req('POST', headers: {'idempotency-key': 'book-1'})),
        isTrue,
      );
      // Khoá rỗng không chứng minh được gì.
      expect(
        retry.isRepeatable(req('POST', headers: {'idempotency-key': ''})),
        isFalse,
      );
    });

    test(
      'POST /quotes nằm trong readOnlyPostPaths nên có; đường khác thì không',
      () {
        expect(retry.isRepeatable(req('POST', path: '/quotes')), isTrue);
        expect(retry.isRepeatable(req('POST', path: '/trips')), isFalse);
        final none = RetryInterceptor(dio: Dio(), readOnlyPostPaths: const {});
        expect(none.isRepeatable(req('POST', path: '/quotes')), isFalse);
      },
    );

    test('POST đánh dấu safeToRetryExtra thì có', () {
      final marked = RequestOptions(
        path: '/search',
        method: 'POST',
        extra: RetryInterceptor.safeToRetryExtra,
      );
      expect(retry.isRepeatable(marked), isTrue);
    });
  });

  group('thử lại khi lỗi tạm thời', () {
    test('báo giá (POST chỉ đọc): 500 hai lần rồi thành công', () async {
      final lab = RetryLab(faults: const FaultConfig(failFirst: 2));
      final quote = await lab.client.quote(ride5km);
      expect(quote.toMoney(), const Money(25500));
      expect(lab.inProcess.requests, hasLength(3));
      expect(lab.waits, hasLength(2));
      // random = 1.0 nên khoảng chờ chạm trần: 500 ms rồi 1000 ms.
      expect(lab.waits, [
        const Duration(milliseconds: 500),
        const Duration(seconds: 1),
      ]);
    });

    test('GET: lỗi ở lần hai, thử lại một lần là xong', () async {
      final lab = RetryLab(faults: const FaultConfig(failEvery: 2));
      final trip = await lab.client.createTrip(
        ride5km,
        idempotencyKey: 'book-1',
      ); // request 1: bình thường
      final fetched = await lab.client.getTrip(trip.id); // 2: 500, 3: ok
      expect(fetched.id, trip.id);
      expect(lab.inProcess.requests.map((r) => r.method), [
        'POST',
        'GET',
        'GET',
      ]);
    });

    test(
      'hết maxAttempts: đúng 3 request, lỗi cuối là ApiException 500',
      () async {
        final lab = RetryLab(faults: const FaultConfig(failFirst: 99));
        await expectLater(
          lab.client.quote(ride5km),
          throwsA(
            isA<ApiException>().having((e) => e.statusCode, 'status', 500),
          ),
        );
        expect(lab.inProcess.requests, hasLength(3));
        expect(lab.waits, hasLength(2));
      },
    );

    test('maxAttempts: 1 là không thử lại', () async {
      final lab = RetryLab(
        faults: const FaultConfig(failFirst: 1),
        maxAttempts: 1,
      );
      await expectLater(
        lab.client.quote(ride5km),
        throwsA(isA<ApiException>()),
      );
      expect(lab.inProcess.requests, hasLength(1));
      expect(lab.waits, isEmpty);
    });

    test('mỗi lần thử có một dòng log riêng (log đứng trước retry)', () async {
      final lab = RetryLab(faults: const FaultConfig(failFirst: 2));
      await lab.client.quote(ride5km);
      expect(
        lab.logs.where((l) => l.startsWith('--> POST /quotes')),
        hasLength(3),
      );
      expect(
        lab.logs.where((l) => l.contains('ERROR badResponse 500')),
        hasLength(2),
      );
      expect(lab.logs.last, startsWith('<-- 200 POST /quotes'));
    });

    test('onRetry nhận số thứ tự lần thử, khoảng chờ và lỗi vừa rồi', () async {
      final events = <RetryEvent>[];
      final lab = RetryLab(
        faults: const FaultConfig(failFirst: 2),
        onRetry: events.add,
      );
      await lab.client.quote(ride5km);
      expect(events.map((e) => e.attempt), [2, 3]);
      expect(events.map((e) => e.wait), lab.waits);
      expect(events.first.error.response?.statusCode, 500);
    });
  });

  group('lỗi tầng mạng', () {
    test('kết nối bị từ chối rồi thành công', () async {
      final lab = RetryLab(script: [NetworkFault.refuse]);
      final quote = await lab.client.quote(ride5km);
      expect(quote.fareVnd, 25500);
      expect(lab.faulty.calls, 2);
      // Lần đầu không tới được server.
      expect(lab.inProcess.requests, hasLength(1));
    });

    test('connectTimeout hai lần rồi thành công', () async {
      final lab = RetryLab(
        script: [NetworkFault.connectTimeout, NetworkFault.connectTimeout],
        connectTimeout: const Duration(milliseconds: 1),
      );
      expect((await lab.client.quote(ride5km)).fareVnd, 25500);
      expect(lab.faulty.calls, 3);
    });

    test('lỗi mạng ở cả ba lần: DioException connectionError', () async {
      final lab = RetryLab(
        script: [NetworkFault.refuse, NetworkFault.refuse, NetworkFault.refuse],
      );
      await expectLater(
        lab.client.quote(ride5km),
        throwsA(
          isA<DioException>().having(
            (e) => e.type,
            'type',
            DioExceptionType.connectionError,
          ),
        ),
      );
      expect(lab.faulty.calls, 3);
    });

    test('chứng chỉ sai: không thử lại', () async {
      final adapter = _ThrowingAdapter(
        (o) => DioException.badCertificate(requestOptions: o),
      );
      final client = ToiNoiApiClient(
        baseUrl: Uri.parse('http://stub.test'),
        adapter: adapter,
      );
      client.dio.interceptors.add(
        RetryInterceptor(dio: client.dio, sleep: (_) async {}),
      );
      await expectLater(client.quote(ride5km), throwsA(isA<DioException>()));
      expect(adapter.calls, 1);
    });

    test('lỗi lạ (không phải mạng) không bị thử lại', () async {
      final adapter = _ThrowingAdapter(
        (o) => DioException(requestOptions: o, error: StateError('lập trình')),
      );
      final client = ToiNoiApiClient(
        baseUrl: Uri.parse('http://stub.test'),
        adapter: adapter,
      );
      client.dio.interceptors.add(
        RetryInterceptor(dio: client.dio, sleep: (_) async {}),
      );
      await expectLater(client.quote(ride5km), throwsA(isA<DioException>()));
      expect(adapter.calls, 1);
    });
  });

  group('không thử lại những thứ không được phép', () {
    test('4xx: 422 ngoài miền chỉ gửi một request', () async {
      final lab = RetryLab();
      await expectLater(
        lab.client.quote(
          const QuoteRequestDto(
            serviceKind: ServiceKind.ride,
            distanceKm: 100.5,
          ),
        ),
        throwsA(isA<ApiException>().having((e) => e.statusCode, 'status', 422)),
      );
      expect(lab.inProcess.requests, hasLength(1));
      expect(lab.waits, isEmpty);
    });

    test('4xx: 404 của GET (request lặp lại được) vẫn không thử lại', () async {
      final lab = RetryLab();
      await expectLater(
        lab.client.getTrip('khong-co'),
        throwsA(isA<ApiException>().having((e) => e.statusCode, 'status', 404)),
      );
      expect(lab.inProcess.requests, hasLength(1));
    });

    test(
      'POST /trips không Idempotency-Key: lỗi 500 không được thử lại',
      () async {
        final lab = RetryLab(faults: const FaultConfig(failFirst: 1));
        await expectLater(
          lab.client.dio.post<Object?>('/trips', data: _tripBody),
          throwsA(
            isA<DioException>().having(
              (e) => e.response?.statusCode,
              'status',
              500,
            ),
          ),
        );
        expect(lab.inProcess.requests, hasLength(1));
        expect(lab.waits, isEmpty);
      },
    );

    test('POST chỉ đọc nhưng không khai báo: không thử lại (khai báo mới quyết định)', () async {
      final lab = RetryLab(
        faults: const FaultConfig(failFirst: 1),
        readOnlyPostPaths: const {},
      );
      await expectLater(
        lab.client.quote(ride5km),
        throwsA(isA<ApiException>()),
      );
      expect(lab.inProcess.requests, hasLength(1));
    });

    test('POST không khoá, response mất: lỗi mạng, KHÔNG gửi lại, việc đã làm một lần', () async {
      // Một server chấp nhận POST không cần khoá (fake API thì từ chối bằng 422).
      var created = 0;
      final faulty = FaultyAdapter(
        InProcessAdapter((request) {
          created++;
          return okJson({'id': 'O-$created'}, status: 201);
        }),
        script: [NetworkFault.loseResponse],
      );
      final client = ToiNoiApiClient(
        baseUrl: Uri.parse('http://stub.test'),
        adapter: faulty,
      );
      client.dio.interceptors.add(
        RetryInterceptor(dio: client.dio, sleep: (_) async {}),
      );
      await expectLater(
        client.dio.post<Object?>('/orders', data: _tripBody),
        throwsA(isA<DioException>()),
      );
      expect(faulty.calls, 1);
      // Server đã làm xong mà app không biết: tự gửi lại sẽ làm lần hai.
      expect(created, 1);
      await client.dio.post<Object?>('/orders', data: _tripBody);
      expect(created, 2);
    });
  });

  group('nhận chuyến: POST có expectedVersion nhưng không có khoá', () {
    test('mất response: không thử lại; gửi lại thủ công nhận 409 vì chuyến đã có tài xế', () async {
      final lab = RetryLab();
      final trip = await lab.client.createTrip(
        ride5km,
        idempotencyKey: 'book-1',
      );
      // Đổi sang kịch bản mới: response của lần nhận chuyến đầu bị mất.
      final faulty = FaultyAdapter(
        lab.inProcess,
        script: [NetworkFault.loseResponse],
      );
      lab.client.dio.httpClientAdapter = faulty;
      Future<dio.Response<Object?>> accept() => lab.client.dio.post<Object?>(
        '/trips/${trip.id}/accept',
        data: {'expectedVersion': 0},
        options: Options(headers: {'x-fake-driver-id': 'D-1'}),
      );
      await expectLater(accept(), throwsA(isA<DioException>()));
      expect(faulty.calls, 1); // không thử lại
      await expectLater(
        accept(),
        throwsA(
          isA<DioException>().having(
            (e) => e.response?.statusCode,
            'status',
            409,
          ),
        ),
      );
    });
  });

  group('POST có Idempotency-Key', () {
    test('500 rồi thành công: gửi lại cùng khoá, cùng nội dung', () async {
      final lab = RetryLab(faults: const FaultConfig(failFirst: 1));
      final trip = await lab.client.createTrip(
        ride5km,
        idempotencyKey: 'book-42',
      );
      expect(trip.id, 'T-1');
      final sent = lab.inProcess.requests;
      expect(sent, hasLength(2));
      expect(sent.map((r) => r.headers['idempotency-key']), [
        'book-42',
        'book-42',
      ]);
      expect(sent[0].body, sent[1].body);
    });

    test(
      'mất response sau khi server tạo chuyến: thử lại vẫn chỉ một chuyến',
      () async {
        final lab = RetryLab(script: [NetworkFault.loseResponse]);
        final trip = await lab.client.createTrip(
          ride5km,
          idempotencyKey: 'book-7',
        );
        expect(trip.id, 'T-1');
        expect(lab.faulty.calls, 2);
        expect(lab.inProcess.requests, hasLength(2)); // server nhận hai lần
        expect(lab.store.trips, hasLength(1)); // nhưng chỉ có một chuyến
      },
    );

    test('hết thời gian chờ nhận: thử lại với cùng khoá, một chuyến', () async {
      final lab = RetryLab(
        script: [NetworkFault.hang],
        receiveTimeout: const Duration(milliseconds: 1),
      );
      final trip = await lab.client.createTrip(
        ride5km,
        idempotencyKey: 'book-8',
      );
      expect(trip.id, 'T-1');
      expect(lab.store.trips, hasLength(1));
    });
  });

  group('huỷ request', () {
    test('huỷ trong lúc chờ backoff thì dừng, không gửi thêm', () async {
      final token = CancelToken();
      final lab = RetryLab(faults: const FaultConfig(failFirst: 99));
      final dio = lab.client.dio;
      // Thay sleep: huỷ ngay khi bắt đầu chờ.
      final cancelling = RetryInterceptor(
        dio: dio,
        sleep: (_) async => token.cancel('người dùng thoát màn hình'),
      );
      dio.interceptors
        ..remove(lab.retry)
        ..add(cancelling);
      await expectLater(
        dio.get<Object?>('/trips/x', cancelToken: token),
        throwsA(
          isA<DioException>().having(
            (e) => e.type,
            'type',
            DioExceptionType.cancel,
          ),
        ),
      );
      expect(lab.inProcess.requests, hasLength(1));
    });
  });

  group('đồng hồ giả: timeout và backoff không chờ thật', () {
    test(
      'server treo: ba lần nhận quá hạn, tổng 31,5 giây trên đồng hồ giả',
      () {
        final lab = RetryLab(
          script: [NetworkFault.hang, NetworkFault.hang, NetworkFault.hang],
          receiveTimeout: const Duration(seconds: 10),
          realSleep: true,
        );
        final watch = Stopwatch()..start();
        final run = runFake(() => lab.client.quote(ride5km));
        expect(run.done, isTrue);
        expect(run.error, isA<DioException>());
        expect(
          (run.error! as DioException).type,
          DioExceptionType.receiveTimeout,
        );
        // 3 lần × 10 s chờ + backoff 0,5 s + 1 s (random = 1.0 nên chạm trần).
        expect(run.elapsed, const Duration(seconds: 31, milliseconds: 500));
        expect(lab.waits, [
          const Duration(milliseconds: 500),
          const Duration(seconds: 1),
        ]);
        expect(lab.faulty.calls, 3);
        // Thời gian thật gần như bằng 0.
        expect(watch.elapsed, lessThan(const Duration(seconds: 5)));
      },
    );

    test('lần đầu treo, lần hai thành công: xong sau 10,5 giây giả', () {
      final lab = RetryLab(
        script: [NetworkFault.hang],
        receiveTimeout: const Duration(seconds: 10),
        realSleep: true,
      );
      final run = runFake(() => lab.client.quote(ride5km));
      expect(run.error, isNull);
      expect(run.value!.fareVnd, 25500);
      expect(run.elapsed, const Duration(seconds: 10, milliseconds: 500));
    });

    test('POST không khoá bị treo: một lần duy nhất, đúng 10 giây giả', () {
      final lab = RetryLab(
        script: [NetworkFault.hang],
        receiveTimeout: const Duration(seconds: 10),
        realSleep: true,
      );
      final run = runFake(
        () => lab.client.dio.post<Object?>('/trips', data: _tripBody),
      );
      expect(run.error, isA<DioException>());
      expect(run.elapsed, const Duration(seconds: 10));
      expect(lab.faulty.calls, 1);
    });

    test('handler không bao giờ trả lời: InProcessAdapter tôn trọng receiveTimeout', () {
      final never = Completer<Response>();
      final stub = clientFor(
        (request) => never.future,
        receiveTimeout: const Duration(seconds: 3),
      );
      final run = runFake(() => stub.client.getTrip('T-1'));
      expect(run.error, isA<DioException>());
      expect(run.elapsed, const Duration(seconds: 3));
    });
  });
}
