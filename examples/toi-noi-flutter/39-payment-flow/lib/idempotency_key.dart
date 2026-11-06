import 'dart:math';

/// Sinh khoá idempotency: `pay-` cộng 32 ký tự hex (128 bit ngẫu nhiên).
///
/// Chỉ gồm chữ, số và gạch ngang nên hợp lệ với mẫu `[A-Za-z0-9_-]{1,128}`
/// của server. Truyền `Random(seed)` để có dãy lặp lại trong test.
String Function() idempotencyKeyGenerator([Random? random]) {
  final rng = random ?? Random.secure();
  return () {
    final hex = List.generate(
      16,
      (_) => rng.nextInt(256).toRadixString(16).padLeft(2, '0'),
    ).join();
    return 'pay-$hex';
  };
}
