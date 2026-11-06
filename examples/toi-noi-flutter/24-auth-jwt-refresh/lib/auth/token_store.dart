// Sao từ lab 22 (22-persistence-drift/lib/auth/token_store.dart), không đổi hành vi.
// Lab mỗi bài tự chạy độc lập nên file nằm ở đây thay vì import chéo.
import 'dart:convert';

import 'package:flutter/services.dart' show PlatformException;
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

/// Cặp token của một phiên đăng nhập.
///
/// `toString` không in giá trị, để token không vô tình vào log hay thông báo lỗi.
final class AuthTokens {
  const AuthTokens({required this.accessToken, required this.refreshToken});

  final String accessToken;
  final String refreshToken;

  @override
  bool operator ==(Object other) =>
      other is AuthTokens &&
      other.accessToken == accessToken &&
      other.refreshToken == refreshToken;

  @override
  int get hashCode => Object.hash(accessToken, refreshToken);

  @override
  String toString() => 'AuthTokens(***)';
}

/// Nơi giữ token giữa các lần mở app. Phần còn lại của app chỉ biết interface
/// này, không biết Keychain hay Keystore.
abstract interface class TokenStore {
  /// Token đang lưu, hoặc `null` nếu chưa đăng nhập hoặc không đọc được.
  Future<AuthTokens?> read();

  Future<void> save(AuthTokens tokens);

  /// Xoá khi đăng xuất. Gọi nhiều lần vẫn an toàn.
  Future<void> clear();
}

/// Bản dùng `flutter_secure_storage`: Keychain trên iOS/macOS, khoá bọc trong
/// Android Keystore trên Android.
///
/// Cả hai token nằm trong **một** khoá dạng JSON, nên một lần ghi hoặc có đủ
/// cả hai hoặc không có gì; không bao giờ có access mới đi với refresh cũ.
class SecureTokenStore implements TokenStore {
  const SecureTokenStore(this._storage);

  final FlutterSecureStorage _storage;

  static const storageKey = 'auth.tokens';

  @override
  Future<AuthTokens?> read() async {
    final String? raw;
    try {
      raw = await _storage.read(key: storageKey);
    } on PlatformException {
      // Ví dụ Android không giải mã được giá trị: coi như chưa đăng nhập để
      // người dùng đăng nhập lại, đừng làm app sập lúc khởi động.
      return null;
    }
    if (raw == null) return null;
    final tokens = _decode(raw);
    if (tokens == null) {
      // Dữ liệu hỏng thì xoá để lần sau khỏi đọc lại.
      await clear();
    }
    return tokens;
  }

  @override
  Future<void> save(AuthTokens tokens) => _storage.write(
    key: storageKey,
    value: jsonEncode({
      'access': tokens.accessToken,
      'refresh': tokens.refreshToken,
    }),
  );

  @override
  Future<void> clear() async {
    try {
      await _storage.delete(key: storageKey);
    } on PlatformException {
      // Đăng xuất không được thất bại vì kho lỗi; lần đọc sau đã trả null.
    }
  }

  AuthTokens? _decode(String raw) {
    try {
      final json = jsonDecode(raw);
      if (json is Map<String, dynamic> &&
          json['access'] is String &&
          json['refresh'] is String) {
        return AuthTokens(
          accessToken: json['access'] as String,
          refreshToken: json['refresh'] as String,
        );
      }
    } on FormatException {
      // rơi xuống return null
    }
    return null;
  }
}

/// Bản trong bộ nhớ cho test và cho màn hình không cần kho thật.
class InMemoryTokenStore implements TokenStore {
  InMemoryTokenStore([this._tokens]);

  AuthTokens? _tokens;

  @override
  Future<AuthTokens?> read() async => _tokens;

  @override
  Future<void> save(AuthTokens tokens) async => _tokens = tokens;

  @override
  Future<void> clear() async => _tokens = null;
}
