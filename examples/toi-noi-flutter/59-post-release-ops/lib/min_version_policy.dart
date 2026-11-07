import 'app_version.dart';
import 'store_link.dart';
import 'store_platform.dart';

/// Kết quả so phiên bản đang chạy với chính sách của server.
sealed class UpdateDecision {
  const UpdateDecision();
}

/// Không cần làm gì. [reason] chỉ để quan sát và test.
final class UpdateOk extends UpdateDecision {
  const UpdateOk(this.reason);
  final OkReason reason;
}

enum OkReason { upToDate, noRuleForPlatform, serverUnavailable, throttled }

/// Nên cập nhật nhưng vẫn dùng được.
final class UpdateRecommend extends UpdateDecision {
  const UpdateRecommend({
    required this.recommended,
    required this.message,
    required this.storeUri,
  });
  final AppVersion recommended;
  final String message;
  final Uri storeUri;
}

/// Phải cập nhật mới được dùng tiếp.
final class UpdateForce extends UpdateDecision {
  const UpdateForce({
    required this.minSupported,
    required this.message,
    required this.storeUri,
  });
  final AppVersion minSupported;
  final String message;
  final Uri storeUri;
}

/// Quy tắc của một nền tảng, đã kiểm tính nhất quán.
final class PlatformRule {
  PlatformRule({
    required this.minSupported,
    required this.latestPublished,
    required this.storeUri,
    this.recommended,
    this.message,
  }) {
    // Hai ràng buộc chống cấu hình sai: không đòi phiên bản chưa có trên
    // cửa hàng, và thứ tự min <= recommended <= latest.
    final rec = recommended;
    if (minSupported > latestPublished) {
      throw FormatException(
        'minSupported $minSupported lớn hơn latestPublished $latestPublished',
      );
    }
    if (rec != null && (rec < minSupported || rec > latestPublished)) {
      throw FormatException(
        'recommended $rec phải nằm trong [$minSupported, $latestPublished]',
      );
    }
  }

  factory PlatformRule.fromJson(
    Map<String, Object?> json,
    StorePlatform platform,
  ) {
    AppVersion version(String key) {
      final value = json[key];
      if (value is! String) {
        throw FormatException('Thiếu hoặc sai kiểu khoá "$key"');
      }
      return AppVersion.parse(value);
    }

    final rec = json['recommended'];
    final message = json['message'];
    final url = json['storeUrl'];
    return PlatformRule(
      minSupported: version('minSupported'),
      latestPublished: version('latestPublished'),
      recommended: rec is String ? AppVersion.parse(rec) : null,
      message: message is String && message.trim().isNotEmpty
          ? message.trim()
          : null,
      storeUri: StoreLink.resolve(url is String ? url : null, platform),
    );
  }

  final AppVersion minSupported;
  final AppVersion? recommended;
  final AppVersion latestPublished;
  final String? message;
  final Uri storeUri;
}

/// Chính sách phiên bản tối thiểu do server trả về.
final class VersionPolicyConfig {
  const VersionPolicyConfig(this.rules);

  /// Đọc JSON dạng `{"android": {...}, "ios": {...}}`. Nền tảng vắng mặt
  /// nghĩa là không có quy tắc. Mọi sai sót ném [FormatException].
  factory VersionPolicyConfig.fromJson(Object? json) {
    if (json is! Map<String, Object?>) {
      throw const FormatException('Chính sách phải là một object JSON');
    }
    final rules = <StorePlatform, PlatformRule>{};
    for (final platform in StorePlatform.values) {
      final entry = json[platform.name];
      if (entry == null) continue;
      if (entry is! Map<String, Object?>) {
        throw FormatException('Khoá "${platform.name}" phải là object');
      }
      rules[platform] = PlatformRule.fromJson(entry, platform);
    }
    return VersionPolicyConfig(rules);
  }

  final Map<StorePlatform, PlatformRule> rules;
}

/// Hàm thuần: so [current] với [config], không IO, không đồng hồ.
abstract final class MinVersionPolicy {
  static const String defaultForceMessage =
      'Phiên bản này không còn được hỗ trợ. Hãy cập nhật Tới Nơi để tiếp tục.';
  static const String defaultRecommendMessage =
      'Đã có bản Tới Nơi mới với cải tiến và sửa lỗi.';

  static UpdateDecision evaluate({
    required AppVersion current,
    required StorePlatform platform,
    required VersionPolicyConfig config,
  }) {
    final rule = config.rules[platform];
    if (rule == null) return const UpdateOk(OkReason.noRuleForPlatform);

    if (current < rule.minSupported) {
      return UpdateForce(
        minSupported: rule.minSupported,
        message: rule.message ?? defaultForceMessage,
        storeUri: rule.storeUri,
      );
    }
    final recommended = rule.recommended;
    if (recommended != null && current < recommended) {
      return UpdateRecommend(
        recommended: recommended,
        message: rule.message ?? defaultRecommendMessage,
        storeUri: rule.storeUri,
      );
    }
    return const UpdateOk(OkReason.upToDate);
  }
}
