import 'dart:io';

import 'package:dio/dio.dart';
import 'package:test/test.dart';
import 'package:toi_noi_42/toi_noi_42.dart';

import 'support/test_certs.dart';
import 'support/tls_server.dart';

void main() {
  late TestPki pki;
  late TestCert current; // khoá đang dùng của server
  late TestCert next; // khoá dự phòng, sẽ dùng sau rotation
  late TestCert stranger; // khoá của kẻ chen giữa
  late CertPin currentPin;
  late CertPin nextPin;

  final servers = <TlsTestServer>[];

  setUpAll(() async {
    pki = await TestPki.create();
    current = await pki.issue('current');
    next = await pki.issue('next');
    stranger = await pki.issue('stranger');
    currentPin = CertPin.of(current.der);
    nextPin = CertPin.of(next.der);
  });

  tearDownAll(() => pki.dispose());

  tearDown(() async {
    for (final server in servers) {
      await server.close();
    }
    servers.clear();
  });

  Future<TlsTestServer> serve(TestCert cert) async {
    final server = await TlsTestServer.start(cert);
    servers.add(server);
    return server;
  }

  Dio clientFor(TlsTestServer server, HttpClientAdapter adapter) =>
      Dio(BaseOptions(baseUrl: 'https://localhost:${server.port}'))
        ..httpClientAdapter = adapter;

  PinPolicy policy(Set<CertPin> pins, {int minPins = 2}) =>
      PinPolicy({'localhost': pins}, minPinsPerHost: minPins);

  Future<DioException> failureOf(Future<Object?> call) async {
    try {
      await call;
    } on DioException catch (e) {
      return e;
    }
    fail('Mong request thất bại nhưng nó thành công');
  }

  group('ghim ở bước bắt tay (createPinnedAdapter)', () {
    test('khoá khớp ghim: request thành công và tới server', () async {
      final server = await serve(current);
      final dio = clientFor(
        server,
        createPinnedAdapter(policy({currentPin, nextPin})),
      );

      final response = await dio.get<String>('/trips/T1');

      expect(response.statusCode, 200);
      expect(server.requestCount, 1);
    });

    test(
      'khoá lạ: bắt tay thất bại và server không nhận request nào',
      () async {
        final server = await serve(stranger);
        final dio = clientFor(
          server,
          createPinnedAdapter(policy({currentPin, nextPin})),
        );

        final error = await failureOf(
          dio.get<String>(
            '/trips/T1',
            options: Options(headers: {'Authorization': 'Bearer test-only'}),
          ),
        );

        // Dio chỉ gắn badCertificate cho nhánh validateCertificate; lỗi bắt tay
        // đi ra dưới dạng unknown kèm HandshakeException.
        expect(error.type, DioExceptionType.unknown);
        expect(error.error, isA<HandshakeException>());
        expect(server.requestCount, 0);
      },
    );

    test('host ngoài chính sách bị từ chối dù khoá trùng ghim', () async {
      final server = await serve(current);
      final dio = Dio(
        BaseOptions(baseUrl: 'https://127.0.0.1:${server.port}'),
      )..httpClientAdapter = createPinnedAdapter(policy({currentPin, nextPin}));

      final error = await failureOf(dio.get<String>('/trips/T1'));

      expect(error.error, isA<HandshakeException>());
      expect(server.requestCount, 0);
    });

    test(
      'chứng chỉ hết hạn theo đồng hồ của chính sách thì bị từ chối',
      () async {
        final server = await serve(current);
        final farFuture = DateTime.now().add(const Duration(days: 3650));
        final expiring = PinPolicy({
          'localhost': {currentPin, nextPin},
        }, now: () => farFuture);
        final dio = clientFor(server, createPinnedAdapter(expiring));

        final error = await failureOf(dio.get<String>('/trips/T1'));

        expect(error.error, isA<HandshakeException>());
        expect(server.requestCount, 0);
      },
    );
  });

  group('ghim thay thế kiểm CA, không cộng thêm', () {
    test(
      'chứng chỉ có SAN của host khác vẫn được nhận nếu khoá khớp ghim',
      () async {
        // SAN chỉ có other.example, không phủ localhost, và CA không có trong
        // kho tin cậy của client: không còn kiểm CA, SAN hay thu hồi.
        final odd = await pki.issue('odd', san: 'DNS:other.example');
        final server = await serve(odd);
        final dio = clientFor(
          server,
          createPinnedAdapter(policy({CertPin.of(odd.der), nextPin})),
        );

        final response = await dio.get<String>('/trips/T1');

        expect(response.statusCode, 200);
      },
    );

    test('chuỗi có chứng chỉ trung gian: callback chỉ nhận lá, không ghim được trung gian', () async {
      final leaf = await pki.issue('leaf', viaIntermediate: true);
      final server = await serve(leaf);

      final pinLeaf = clientFor(
        server,
        createPinnedAdapter(policy({CertPin.of(leaf.der), nextPin})),
      );
      expect((await pinLeaf.get<String>('/x')).statusCode, 200);

      final pinIntermediate = clientFor(
        server,
        createPinnedAdapter(
          policy({CertPin.of(leaf.intermediateDer!), nextPin}),
        ),
      );
      final error = await failureOf(pinIntermediate.get<String>('/x'));
      expect(error.error, isA<HandshakeException>());
    });
  });

  group('rotation với hai ghim', () {
    test('app có cả hai ghim chạy trước và sau khi server đổi khoá', () async {
      final both = policy({currentPin, nextPin});

      final before = await serve(current);
      expect(
        (await clientFor(before, createPinnedAdapter(both)).get<String>('/x'))
            .statusCode,
        200,
      );

      final after = await serve(next);
      expect(
        (await clientFor(after, createPinnedAdapter(both)).get<String>('/x'))
            .statusCode,
        200,
      );
    });

    test(
      'khoá chết: app chỉ ghim khoá cũ không nói chuyện được với server đã đổi',
      () async {
        final server = await serve(next);
        // minPins: 1 chỉ để dựng lại sự cố; app thật bị PinPolicy chặn.
        final oldOnly = policy({currentPin}, minPins: 1);
        final dio = clientFor(server, createPinnedAdapter(oldOnly));

        final error = await failureOf(dio.get<String>('/trips/T1'));

        expect(error.error, isA<HandshakeException>());
        expect(server.requestCount, 0);
      },
    );

    test('ghim hash chứng chỉ hỏng khi gia hạn, ghim SPKI thì không', () async {
      final renewed = await pki.reissue('current', current.keyFile);
      final server = await serve(renewed);
      final certPins = policy({
        CertPin.of(current.der, kind: PinKind.certificate),
        CertPin.of(next.der, kind: PinKind.certificate),
      });
      final spkiPins = policy({currentPin, nextPin});

      final broken = await failureOf(
        clientFor(server, createPinnedAdapter(certPins)).get<String>('/x'),
      );
      expect(broken.error, isA<HandshakeException>());

      final response = await clientFor(
        server,
        createPinnedAdapter(spkiPins),
      ).get<String>('/x');
      expect(response.statusCode, 200);
    });
  });

  group('validateCertificate: kiểm sau khi đã gửi', () {
    test(
      'ghim sai làm request ném lỗi nhưng Authorization đã tới server',
      () async {
        final server = await serve(stranger);
        // Client tin chứng chỉ của kẻ chen giữa, như khi nó có chứng chỉ do một
        // CA hợp lệ cấp.
        final adapter = createCheckAfterSendAdapter(
          policy({currentPin, nextPin}),
          context: stranger.trustingClientContext(),
        );
        final dio = clientFor(server, adapter);

        final error = await failureOf(
          dio.get<String>(
            '/trips/T1',
            options: Options(headers: {'Authorization': 'Bearer test-only'}),
          ),
        );

        expect(error.type, DioExceptionType.badCertificate);
        expect(server.requestCount, 1);
        expect(server.receivedAuthorization.single, 'Bearer test-only');
      },
    );

    test('ghim đúng thì thành công', () async {
      final server = await serve(current);
      final adapter = createCheckAfterSendAdapter(
        policy({currentPin, nextPin}),
        context: current.trustingClientContext(),
      );

      final response = await clientFor(server, adapter).get<String>('/x');

      expect(response.statusCode, 200);
    });
  });

  group('đối chứng', () {
    test('HttpClient thường cộng callback: chứng chỉ do CA tin cấp thì callback không chạy', () async {
      final server = await serve(current);
      var callbackCalls = 0;
      final client = HttpClient(context: current.trustingClientContext())
        ..badCertificateCallback = (certificate, host, port) {
          callbackCalls++;
          return false;
        };
      addTearDown(client.close);

      final response = await (await client.getUrl(server.uri('/x'))).close();
      await response.drain<void>();

      // Kết nối thành công và callback không được gọi dù nó trả false: nếu
      // chỉ ghim ở đây thì với chứng chỉ CA tin, ta không ghim gì cả.
      expect(response.statusCode, 200);
      expect(callbackCalls, 0);
      expect(server.requestCount, 1);
    });

    test(
      'Dio mặc định từ chối chứng chỉ do CA không có trong kho tin cậy',
      () async {
        final server = await serve(current);
        final dio = Dio(
          BaseOptions(baseUrl: 'https://localhost:${server.port}'),
        );

        final error = await failureOf(dio.get<String>('/x'));

        expect(error.error, isA<HandshakeException>());
        expect(server.requestCount, 0);
      },
    );
  });
}
