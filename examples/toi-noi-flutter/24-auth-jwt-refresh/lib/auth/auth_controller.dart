import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers.dart';
import 'auth_api.dart';
import 'unverified_claims.dart';

/// Ai đang đăng nhập, **để hiển thị**. Không dùng để cho phép hay cấm hành động.
@immutable
final class AuthSession {
  const AuthSession({this.claims});

  /// Claim đọc từ access token mà không kiểm chữ ký; `null` nếu token không
  /// phải JWT đọc được.
  final UnverifiedClaims? claims;

  @override
  bool operator ==(Object other) =>
      other is AuthSession && other.claims == claims;

  @override
  int get hashCode => claims.hashCode;
}

AuthSession _sessionOf(String accessToken) =>
    AuthSession(claims: UnverifiedClaims.tryParse(accessToken));

/// Trạng thái đăng nhập: `AsyncLoading` lúc đọc kho khi khởi động, rồi
/// `AsyncData(null)` (chưa đăng nhập) hoặc `AsyncData(session)`.
///
/// Phiên kết thúc ở bất cứ đâu (người dùng bấm đăng xuất, hoặc server từ chối
/// refresh token giữa chừng một request) đều thành `AsyncData(null)` qua
/// luồng `signedOut` của `SessionManager`; màn hình không cần biết lý do.
class AuthController extends AsyncNotifier<AuthSession?> {
  @override
  Future<AuthSession?> build() async {
    final manager = ref.watch(sessionManagerProvider);
    final subscription = manager.signedOut.listen((_) {
      if (ref.mounted) state = const AsyncData(null);
    });
    ref.onDispose(subscription.cancel);
    final tokens = await manager.read();
    return tokens == null ? null : _sessionOf(tokens.accessToken);
  }

  /// Gửi OTP tới [phone]; trả mã yêu cầu để đưa vào [verifyOtp].
  Future<OtpChallenge> requestOtp(String phone) =>
      ref.read(authApiProvider).requestOtp(phone: phone, app: 'customer');

  /// Xác minh mã; thành công thì lưu token và chuyển sang đã đăng nhập.
  Future<void> verifyOtp(OtpChallenge challenge, String code) async {
    final tokens = await ref
        .read(authApiProvider)
        .verifyOtp(requestId: challenge.requestId, code: code);
    await ref.read(sessionManagerProvider).signIn(tokens);
    if (!ref.mounted) return;
    state = AsyncData(_sessionOf(tokens.accessToken));
  }

  Future<void> signOut() => ref.read(sessionManagerProvider).signOut();
}

final authControllerProvider =
    AsyncNotifierProvider<AuthController, AuthSession?>(AuthController.new);
