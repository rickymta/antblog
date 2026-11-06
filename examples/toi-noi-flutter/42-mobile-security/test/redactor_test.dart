import 'dart:convert';

import 'package:test/test.dart';
import 'package:toi_noi_42/toi_noi_42.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

void main() {
  final redactor = Redactor.standard();

  // Số điện thoại giả, không thuộc ai: dãy 000 ở cuối.
  const phones = [
    '0900000001',
    '0900 000 001',
    '0900.000.001',
    '090-000-0001',
    '+84900000001',
    '+84 900 000 001',
    '84900000001',
  ];

  /// JWT giả ghép lúc chạy từ hai đoạn JSON, không chứa token thật.
  String fakeJwt() {
    String part(Map<String, Object?> json) =>
        base64Url.encode(utf8.encode(jsonEncode(json))).replaceAll('=', '');
    final signature = base64Url
        .encode(List.generate(32, (i) => i))
        .replaceAll('=', '');
    return '${part({'alg': 'none'})}.${part({'sub': 'khach-1'})}.$signature';
  }

  group('số điện thoại', () {
    for (final phone in phones) {
      test('che "$phone"', () {
        final out = redactor.redact('gọi tài xế $phone để hỏi đường');
        expect(out, 'gọi tài xế [phone] để hỏi đường');
      });
    }

    test('không che số có ít hoặc nhiều chữ số hơn', () {
      expect(redactor.redact('mã 090000000'), 'mã 090000000');
      expect(redactor.redact('mã 09000000012'), 'mã 09000000012');
    });

    test('không che tiền, mã chuyến, UUID', () {
      final fare = calculateFare(5);
      final line =
          'trip T-0001 cước $fare uuid 123e4567-e89b-12d3-a456-426614174000';
      expect(redactor.redact(line), line);
    });
  });

  group('token', () {
    test('JWT thành [jwt]', () {
      final jwt = fakeJwt();
      final out = redactor.redact('refresh xong, token mới $jwt (hạn 15 phút)');
      expect(out, isNot(contains(jwt.substring(0, 10))));
      expect(out, contains('[jwt]'));
    });

    test('Bearer giữ chữ Bearer, bỏ giá trị', () {
      expect(
        redactor.redact('Authorization: Bearer abc.def-123_xyz'),
        'Authorization: Bearer [redacted]',
      );
      expect(redactor.redact('bearer   thuong-hoa'), 'Bearer [redacted]');
    });

    test('chuỗi chữ-số dài 32 ký tự trở lên thành [token]', () {
      final hex = List.generate(32, (i) => 'a0'[i % 2]).join();
      expect(redactor.redact('khoá $hex.'), 'khoá [token].');
      final short = hex.substring(0, 31);
      expect(redactor.redact('khoá $short.'), 'khoá $short.');
    });

    test('số điện thoại nằm trong token dài không để lộ nửa token', () {
      final token = 'ab0900000001cd${'x' * 30}';
      expect(redactor.redact('k=$token'), isNot(contains('ab09')));
      expect(redactor.redact('k=$token'), isNot(contains('cdx')));
    });
  });

  group('trường nhạy cảm theo tên', () {
    const cases = {
      'otp=123456': 'otp=[redacted]',
      'OTP: 123456': 'OTP: [redacted]',
      'password=hunter2 và tiếp': 'password=[redacted] và tiếp',
      '"refresh_token":"abc"': '"refresh_token":[redacted]',
      "{'accessToken': 'abc'}": "{'accessToken': [redacted]}",
      'client_secret=xyz&grant=otp': 'client_secret=[redacted]&grant=otp',
      'phone: +84 900 000 001': 'phone: [redacted]',
    };
    cases.forEach((input, expected) {
      test('"$input"', () {
        expect(redactor.redact(input), expected);
      });
    });

    test('từ chứa tên khoá nhưng không phải khoá thì giữ nguyên', () {
      const line = 'phoneme status=ok tokenizer=fast';
      expect(redactor.redact(line), contains('status=ok'));
    });
  });

  group('toạ độ', () {
    test('cặp toạ độ chính xác giữ 2 chữ số thập phân', () {
      expect(
        redactor.redact('đón tại 10.762622, 106.660172 lúc 8h'),
        'đón tại 10.76,106.66 lúc 8h',
      );
    });

    test('số âm và làm tròn về 0', () {
      expect(
        redactor.redact('vị trí -33.868820,151.209296'),
        'vị trí -33.87,151.21',
      );
      expect(redactor.redact('0.00012,-0.00034'), '0.00,0.00');
    });

    test('trường lat/lng có tên', () {
      expect(
        redactor.redact('lat=10.762622 lng=106.660172'),
        'lat=10.76 lng=106.66',
      );
      expect(
        redactor.redact('{"latitude": 10.762622, "longitude": 106.660172}'),
        '{"latitude": 10.76, "longitude": 106.66}',
      );
    });

    test('cặp số tiền có dấu chấm nghìn không bị nuốt', () {
      const line = 'cước 25.500, phụ phí 12.000';
      expect(redactor.redact(line), line);
    });

    test(
      'giới hạn đã biết: toạ độ 3 chữ số thập phân dạng cặp không bị che',
      () {
        // Ghi lại điều bộ che KHÔNG làm, để không ai tin nhầm. 3 chữ số thập
        // phân vẫn cho vị trí cỡ trăm mét.
        expect(redactor.redact('10.762, 106.660'), '10.762, 106.660');
      },
    );
  });

  group('redactJson', () {
    test('khoá nhạy cảm bị thay hoàn toàn, chuỗi còn lại đi qua redact', () {
      final out = redactor.redactJson({
        'tripId': 'T-0001',
        'phone': '0900000001',
        'refresh-token': 'abc',
        'note': 'gọi 0900000001',
        'stops': [
          {'lat': 10.762622, 'label': 'đón 10.762622, 106.660172'},
        ],
        'fare': 25500,
      });

      expect(out, {
        'tripId': 'T-0001',
        'phone': '[redacted]',
        'refresh-token': '[redacted]',
        'note': 'gọi [phone]',
        'stops': [
          {'lat': 10.76, 'label': 'đón 10.76,106.66'},
        ],
        'fare': 25500,
      });
    });

    test('số lat/lng dạng double trong Map cũng được làm tròn', () {
      expect(
        redactor.redactJson({
          'lat': 10.762622,
          'lng': -106.660172,
          'n': 1.23456,
        }),
        {'lat': 10.76, 'lng': -106.66, 'n': 1.23456},
      );
    });

    test('không sửa cấu trúc đầu vào', () {
      final input = {'phone': '0900000001'};
      redactor.redactJson(input);
      expect(input['phone'], '0900000001');
    });
  });

  group('bất biến', () {
    test('idempotent: che hai lần bằng che một lần', () {
      final line =
          'gọi ${phones.first} otp=123456 Bearer abc ${fakeJwt()} '
          '10.762622, 106.660172';
      final once = redactor.redact(line);
      expect(redactor.redact(once), once);
    });

    test('chuỗi rỗng và chuỗi không nhạy cảm đi qua nguyên vẹn', () {
      expect(redactor.redact(''), '');
      expect(
        redactor.redact('Tới Nơi: chuyến T-0001 hoàn thành'),
        'Tới Nơi: chuyến T-0001 hoàn thành',
      );
    });
  });
}
