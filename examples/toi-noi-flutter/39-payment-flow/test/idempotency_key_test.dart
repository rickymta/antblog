import 'dart:math';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_39/idempotency_key.dart';

void main() {
  final serverPattern = RegExp(r'^[A-Za-z0-9_-]{1,128}$');

  test('khoá hợp lệ với mẫu của server và có 128 bit ngẫu nhiên', () {
    final next = idempotencyKeyGenerator();
    final key = next();
    expect(key, matches(serverPattern));
    expect(key, matches(RegExp(r'^pay-[0-9a-f]{32}$')));
  });

  test('1.000 khoá liên tiếp không trùng nhau', () {
    final next = idempotencyKeyGenerator();
    final keys = {for (var i = 0; i < 1000; i++) next()};
    expect(keys, hasLength(1000));
  });

  test('cùng hạt giống cho cùng dãy khoá (để test lặp lại được)', () {
    final a = idempotencyKeyGenerator(Random(7));
    final b = idempotencyKeyGenerator(Random(7));
    expect([a(), a()], [b(), b()]);
  });
}
