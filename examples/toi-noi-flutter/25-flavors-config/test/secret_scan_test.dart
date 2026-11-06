import 'package:flutter_test/flutter_test.dart';

import 'support/secret_scan.dart';

void main() {
  // Mẫu giả được ghép lúc chạy để mã nguồn không chứa chuỗi trông như thật
  // (và để các công cụ quét bí mật của repo không báo nhầm).
  final samples = <String, String>{
    'khoá PEM': '-----BEGIN ${'PRIVATE'} KEY-----',
    'access key AWS': 'AKIA${'A' * 16}',
    'API key Google': 'AIza${'a' * 35}',
    'JWT': 'eyJ${'a' * 10}.${'b' * 10}.${'c' * 10}',
    'token GitHub': 'ghp_${'a' * 30}',
    'khoá Stripe': 'sk_${'live'}_${'a' * 24}',
    'user:password trong URL': 'https://app:pw@api.toinoi.example',
    'chuỗi ngẫu nhiên dài': 'Zm9vYmFy${'Q' * 30}',
  };

  for (final entry in samples.entries) {
    test('bắt ${entry.key}', () {
      final findings = findSecretLike({'THAM_SO': entry.value});
      // Một mẫu có thể khớp nhiều luật (chuỗi dài cũng giống chuỗi ngẫu nhiên);
      // chỉ cần có đúng luật đang thử.
      expect(
        findings.any((f) => f.contains(entry.key)),
        isTrue,
        reason: '$findings',
      );
    });
  }

  test('bắt theo tên khoá dù giá trị trông vô hại', () {
    for (final key in [
      'API_KEY',
      'client_secret',
      'AUTH_HEADER',
      'DB_PASSWORD',
      'privateKey',
    ]) {
      expect(findSecretLike({key: 'abc'}), isNotEmpty, reason: key);
    }
  });

  test('kết quả không bao giờ chứa giá trị', () {
    final value = samples['access key AWS']!;
    expect(findSecretLike({'THAM_SO': value}).join(), isNot(contains(value)));
  });

  test('giá trị lồng nhau bị từ chối', () {
    expect(
      findSecretLike({
        'X': {'a': 1},
      }),
      hasLength(1),
    );
    expect(
      findSecretLike({
        'X': [1, 2],
      }),
      hasLength(1),
    );
  });

  test('cấu hình bình thường không bị báo nhầm', () {
    expect(
      findSecretLike({
        'FLAVOR': 'staging',
        'API_BASE_URL': 'https://api.staging.toinoi.example/v1',
        'LOG_HTTP': true,
        'CONNECT_TIMEOUT_MS': 10000,
      }),
      isEmpty,
    );
  });
}
