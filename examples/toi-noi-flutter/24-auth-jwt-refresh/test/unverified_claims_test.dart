import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_24/auth/unverified_claims.dart';

String _b64(Object json) =>
    base64Url.encode(utf8.encode(jsonEncode(json))).replaceAll('=', '');

String _jwt(Map<String, Object?> claims, {String signature = ''}) =>
    '${_b64({'alg': 'none', 'typ': 'JWT'})}.${_b64(claims)}.$signature';

void main() {
  group('UnverifiedClaims.tryParse', () {
    test('đọc sub, role và exp (UTC)', () {
      final claims = UnverifiedClaims.tryParse(
        _jwt({'sub': 'U-1001', 'role': 'customer', 'exp': 1793935800}),
      );
      expect(claims!.subject, 'U-1001');
      expect(claims.role, 'customer');
      expect(claims.expiresAt, DateTime.utc(2026, 11, 6, 3, 30));
    });

    test('role và exp là tuỳ chọn, sai kiểu thì bỏ qua', () {
      final claims = UnverifiedClaims.tryParse(
        _jwt({'sub': 'U-1', 'role': 7, 'exp': '1793935800'}),
      );
      expect(claims!.role, isNull);
      expect(claims.expiresAt, isNull);
    });

    test('chữ ký là gì cũng không ảnh hưởng: client không kiểm chữ ký', () {
      final claims = {'sub': 'U-1001', 'role': 'driver'};
      final unsigned = UnverifiedClaims.tryParse(_jwt(claims));
      final garbage = UnverifiedClaims.tryParse(_jwt(claims, signature: 'x'));
      expect(unsigned, garbage);
    });

    test('payload bị sửa (role thành driver) vẫn đọc ra bình thường', () {
      final original = _jwt({'sub': 'U-1001', 'role': 'customer'});
      final parts = original.split('.');
      final forged =
          '${parts[0]}.${_b64({'sub': 'U-1001', 'role': 'driver'})}.${parts[2]}';
      expect(UnverifiedClaims.tryParse(original)!.role, 'customer');
      // Đây là lý do client không được dùng role để phân quyền.
      expect(UnverifiedClaims.tryParse(forged)!.role, 'driver');
    });

    test('đọc được payload mọi độ dài (base64url thiếu padding)', () {
      for (final sub in ['a', 'ab', 'abc', 'abcd', 'U-1001', 'tiếng Việt']) {
        expect(UnverifiedClaims.tryParse(_jwt({'sub': sub}))!.subject, sub);
      }
    });

    test('không phải JWT đọc được thì trả null, không ném', () {
      final cases = <String>[
        '',
        'opaque-token-khong-co-dau-cham',
        'a.b',
        'a.b.c.d',
        'a.!!!.c',
        'a.${base64Url.encode(utf8.encode('not json'))}.c',
        'a.${_b64([1, 2])}.c',
        _jwt({'role': 'customer'}),
        _jwt({'sub': ''}),
        _jwt({'sub': 5}),
      ];
      for (final token in cases) {
        expect(UnverifiedClaims.tryParse(token), isNull, reason: token);
      }
    });
  });
}
