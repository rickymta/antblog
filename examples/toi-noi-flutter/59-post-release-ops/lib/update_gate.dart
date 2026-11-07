import 'dart:async';

import 'app_version.dart';
import 'min_version_policy.dart';
import 'reminder_throttle.dart';
import 'store_platform.dart';

/// Nguồn chính sách, thường là một endpoint của BFF
/// (ví dụ `GET /app-config/min-version`). Ném bất cứ lỗi nào khi thất bại.
abstract interface class PolicySource {
  Future<Object?> fetchRaw();
}

/// Quyết định ở lúc mở app. Nguyên tắc: **fail open**. Lỗi mạng, hết thời
/// gian chờ, JSON hỏng hay cấu hình tự mâu thuẫn đều cho người dùng vào app;
/// chỉ một chính sách đọc được và hợp lệ mới có quyền chặn.
final class UpdateGate {
  UpdateGate({
    required this._source,
    required this._throttle,
    this.timeout = const Duration(seconds: 3),
  });

  final PolicySource _source;
  final ReminderThrottle _throttle;
  final Duration timeout;

  Future<UpdateDecision> check({
    required AppVersion current,
    required StorePlatform platform,
  }) async {
    final VersionPolicyConfig config;
    try {
      final raw = await _source.fetchRaw().timeout(timeout);
      config = VersionPolicyConfig.fromJson(raw);
    } on Object {
      return const UpdateOk(OkReason.serverUnavailable);
    }

    final decision = MinVersionPolicy.evaluate(
      current: current,
      platform: platform,
      config: config,
    );
    if (decision is! UpdateRecommend) return decision;

    if (!await _throttle.shouldRemind(decision.recommended)) {
      return const UpdateOk(OkReason.throttled);
    }
    // Ghi nhận ngay khi trả về: từ lúc này ta coi như người dùng đã được nhắc.
    await _throttle.markShown(decision.recommended);
    return decision;
  }
}
