import 'dart:convert';
import 'dart:typed_data';

import 'package:test/test.dart';
import 'package:toi_noi_42/toi_noi_42.dart';

import 'support/test_certs.dart';

void main() {
  late TestPki pki;
  late TestCert certA;

  setUpAll(() async {
    pki = await TestPki.create();
    certA = await pki.issue('a');
  });

  tearDownAll(() => pki.dispose());

  group('SPKI', () {
    test('khớp kết quả của openssl', () async {
      final expected = await pki.opensslSpkiHash(certA);
      expect(CertPin.of(certA.der).hash, expected);
    });

    test(
      'cấp lại chứng chỉ trên cùng khoá: SPKI giữ, hash chứng chỉ đổi',
      () async {
        final renewed = await pki.reissue('a', certA.keyFile);

        expect(renewed.der, isNot(equals(certA.der)));
        expect(CertPin.of(renewed.der), CertPin.of(certA.der));
        expect(
          CertPin.of(renewed.der, kind: PinKind.certificate),
          isNot(CertPin.of(certA.der, kind: PinKind.certificate)),
        );
      },
    );

    test('khoá khác thì SPKI khác', () async {
      final other = await pki.issue('other');
      expect(CertPin.of(other.der), isNot(CertPin.of(certA.der)));
    });

    test('DER hỏng ném FormatException thay vì trả hash rác', () {
      expect(
        () => extractSubjectPublicKeyInfo(Uint8List(0)),
        throwsFormatException,
      );
      expect(
        () => extractSubjectPublicKeyInfo(
          Uint8List.sublistView(certA.der, 0, certA.der.length ~/ 2),
        ),
        throwsFormatException,
      );
      expect(
        () =>
            extractSubjectPublicKeyInfo(Uint8List.fromList([0x04, 0x01, 0x00])),
        throwsFormatException,
      );
    });
  });

  group('CertPin.parse', () {
    test('đọc lại đúng chuỗi đã in', () {
      final pin = CertPin.of(certA.der);
      expect(CertPin.parse(pin.toString()), pin);
    });

    test('loại sai, thiếu dấu gạch chéo, độ dài hash sai đều bị từ chối', () {
      final hash = base64.encode(List.filled(32, 7));
      expect(() => CertPin.parse('md5/$hash'), throwsFormatException);
      expect(() => CertPin.parse(hash), throwsFormatException);
      expect(
        () => CertPin.parse('spki-sha256/${base64.encode([1, 2, 3])}'),
        throwsFormatException,
      );
    });
  });

  group('PinPolicy', () {
    late CertPin pinA;
    late CertPin backup;
    final valid = (start: DateTime.utc(2000), end: DateTime.utc(2100));

    setUpAll(() async {
      pinA = CertPin.of(certA.der);
      backup = CertPin.of((await pki.issue('backup')).der);
    });

    test('một ghim duy nhất bị từ chối: không có đường rotation', () {
      expect(
        () => PinPolicy({
          'api.toinoi.example': {pinA},
        }),
        throwsA(
          isA<ArgumentError>().having(
            (e) => e.message,
            'message',
            contains('rotation'),
          ),
        ),
      );
    });

    test('khớp một trong hai ghim, không phân biệt hoa thường ở host', () {
      final policy = PinPolicy({
        'API.toinoi.example': {pinA, backup},
      });
      expect(
        policy.accepts(
          'api.TOINOI.example',
          certA.der,
          startValidity: valid.start,
          endValidity: valid.end,
        ),
        isTrue,
      );
    });

    test('host ngoài danh sách luôn bị từ chối, kể cả khi khoá khớp', () {
      final policy = PinPolicy({
        'api.toinoi.example': {pinA, backup},
      });
      expect(
        policy.accepts(
          'other.example',
          certA.der,
          startValidity: valid.start,
          endValidity: valid.end,
        ),
        isFalse,
      );
    });

    test(
      'khoá khớp nhưng chứng chỉ hết hạn hoặc chưa hiệu lực thì từ chối',
      () {
        var now = DateTime.utc(2030);
        final policy = PinPolicy({
          'h': {pinA, backup},
        }, now: () => now);
        bool accepts() => policy.accepts(
          'h',
          certA.der,
          startValidity: DateTime.utc(2020),
          endValidity: DateTime.utc(2040),
        );

        expect(accepts(), isTrue);
        now = DateTime.utc(2041);
        expect(accepts(), isFalse);
        now = DateTime.utc(2019);
        expect(accepts(), isFalse);
      },
    );

    test('ghim sai loại không khớp: ghim hash chứng chỉ không nhận SPKI', () {
      final certPin = CertPin.of(certA.der, kind: PinKind.certificate);
      expect(certPin.matches(certA.der), isTrue);
      expect(
        CertPin.parse('spki-sha256/${certPin.hash}').matches(certA.der),
        isFalse,
      );
    });
  });
}
