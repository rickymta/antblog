import 'dart:convert';

import 'package:flutter/foundation.dart';

/// Các claim đọc từ payload của một JWT **mà không kiểm chữ ký**.
///
/// Client không có khoá để kiểm chữ ký (và dù có, không được tin kết quả hơn
/// server). Payload chỉ là JSON mã hoá base64url: ai cũng sửa được. Vì vậy
/// lớp này chỉ để **hiển thị** (tên, vai trò trên màn hình tài khoản) hoặc
/// **đoán** (access token hết hạn lúc nào để biết có nên refresh trước).
/// Mọi quyết định cho phép hay cấm một hành động là của server: nó nhận cùng
/// token và tự kiểm. Đừng viết `if (claims.role == 'driver')` để mở tính năng.
@immutable
final class UnverifiedClaims {
  const UnverifiedClaims({required this.subject, this.role, this.expiresAt});

  /// Claim `sub`: mã người dùng.
  final String subject;

  /// Claim `role`, chỉ để hiển thị.
  final String? role;

  /// Claim `exp` (UTC), chỉ để đoán.
  final DateTime? expiresAt;

  /// Đọc payload của [jwt]; `null` nếu không phải JWT ba phần, base64url hay
  /// JSON hỏng, hoặc thiếu `sub`. Access token của server thật có thể là chuỗi
  /// đục (không phải JWT): khi đó ta chỉ không có gì để hiển thị.
  static UnverifiedClaims? tryParse(String jwt) {
    final parts = jwt.split('.');
    if (parts.length != 3) return null;
    try {
      final json = jsonDecode(
        utf8.decode(base64Url.decode(base64Url.normalize(parts[1]))),
      );
      if (json is! Map<String, dynamic>) return null;
      final sub = json['sub'];
      if (sub is! String || sub.isEmpty) return null;
      final role = json['role'];
      final exp = json['exp'];
      return UnverifiedClaims(
        subject: sub,
        role: role is String ? role : null,
        expiresAt: exp is int
            ? DateTime.fromMillisecondsSinceEpoch(exp * 1000, isUtc: true)
            : null,
      );
    } on FormatException {
      return null;
    }
  }

  @override
  bool operator ==(Object other) =>
      other is UnverifiedClaims &&
      other.subject == subject &&
      other.role == role &&
      other.expiresAt == expiresAt;

  @override
  int get hashCode => Object.hash(subject, role, expiresAt);

  @override
  String toString() => 'UnverifiedClaims($subject, $role, $expiresAt)';
}
