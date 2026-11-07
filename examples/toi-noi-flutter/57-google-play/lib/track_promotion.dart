/// Máy trạng thái promote một bản phát hành qua các track của Google Play.
///
/// Thứ tự track và đường đi hợp lệ là quy trình của Tới Nơi. Riêng ba ràng
/// buộc có nguồn từ trang trợ giúp Play Console (kiểm tại 11/2026): yêu cầu
/// closed test cho tài khoản cá nhân mới, Data safety, và target API.
library;

import 'manifest_permissions.dart';

enum Track { internal, closed, open, production }

enum AccountKind { personal, organization }

/// Yêu cầu closed test trước khi xin production (kiểm tại 11/2026): tài khoản
/// cá nhân tạo sau 13/11/2023 cần ít nhất 12 tester, opt-in liên tục 14 ngày.
const int requiredClosedTesters = 12;
const int requiredClosedTestDays = 14;

/// Target API tối thiểu cho bản mới và bản cập nhật (kiểm tại 11/2026: API 36
/// từ 31/8/2026; trang trợ giúp ghi gia hạn tới 1/11/2026).
const int minTargetSdk = 36;

/// Foreground service phải khai loại khi app nhắm Android 14 (API 34) trở lên.
const int foregroundServiceDeclarationSdk = 34;

final DateTime _personalRuleCutoff = DateTime.utc(2023, 11, 13);

final class DeveloperAccount {
  const DeveloperAccount({required this.kind, required this.createdAt});

  final AccountKind kind;
  final DateTime createdAt;

  /// Trang trợ giúp chỉ nêu tài khoản cá nhân tạo "sau 13/11/2023". Lab hiểu
  /// theo ngày UTC: ngày 13/11/2023 chưa bị áp, 14/11/2023 bị áp.
  bool get mustPassClosedTest {
    if (kind != AccountKind.personal) return false;
    final c = createdAt.toUtc();
    return DateTime.utc(c.year, c.month, c.day).isAfter(_personalRuleCutoff);
  }
}

/// Tiến độ closed test, đã lọc: chỉ đếm tester opt-in liên tục suốt kỳ.
final class ClosedTestProgress {
  const ClosedTestProgress({
    required this.testersOptedIn,
    required this.consecutiveDays,
  });

  final int testersOptedIn;
  final int consecutiveDays;

  bool get satisfiesProductionAccess =>
      testersOptedIn >= requiredClosedTesters &&
      consecutiveDays >= requiredClosedTestDays;
}

/// Đầu vào của bước kiểm trước khi promote.
final class PreflightInput {
  const PreflightInput({
    required this.account,
    required this.targetSdk,
    required this.manifestPermissions,
    required this.dataSafetyComplete,
    this.foregroundServiceDeclared = false,
    this.backgroundLocationDeclared = false,
    this.closedTest,
  });

  final DeveloperAccount account;
  final int targetSdk;
  final Set<String> manifestPermissions;
  final bool dataSafetyComplete;
  final bool foregroundServiceDeclared;
  final bool backgroundLocationDeclared;
  final ClosedTestProgress? closedTest;
}

enum Blocker {
  dataSafetyIncomplete,
  targetSdkTooLow,
  backgroundLocationNotDeclared,
  foregroundServiceNotDeclared,
  productionAccessNotEarned,
}

/// Mọi điều chặn khi đưa bản lên [to], theo thứ tự cố định.
///
/// Mọi đích đều là track đã ra khỏi internal, nơi Play không đòi Data safety
/// (trang Data safety: app chỉ có trên internal test thì không cần form).
List<Blocker> preflight(Track to, PreflightInput input) {
  final declarations = consoleDeclarationsFor(input.manifestPermissions);
  return [
    if (!input.dataSafetyComplete) Blocker.dataSafetyIncomplete,
    if (input.targetSdk < minTargetSdk) Blocker.targetSdkTooLow,
    if (declarations.contains(ConsoleDeclaration.backgroundLocation) &&
        !input.backgroundLocationDeclared)
      Blocker.backgroundLocationNotDeclared,
    if (declarations.contains(ConsoleDeclaration.foregroundServiceLocation) &&
        input.targetSdk >= foregroundServiceDeclarationSdk &&
        !input.foregroundServiceDeclared)
      Blocker.foregroundServiceNotDeclared,
    if (to == Track.production &&
        input.account.mustPassClosedTest &&
        !(input.closedTest?.satisfiesProductionAccess ?? false))
      Blocker.productionAccessNotEarned,
  ];
}

class PromotionException implements Exception {
  PromotionException(this.message);
  final String message;
  @override
  String toString() => 'PromotionException: $message';
}

/// Đi từ track này sang track không có trong bảng.
final class IllegalMove extends PromotionException {
  IllegalMove(this.from, this.to)
    : super('không promote trực tiếp ${from.name} -> ${to.name}');
  final Track from;
  final Track to;
}

/// Đường đi hợp lệ, nhưng còn điều chặn.
final class PromotionBlocked extends PromotionException {
  PromotionBlocked(this.to, this.blockers)
    : super(
        'chưa đưa lên ${to.name} được: '
        '${blockers.map((b) => b.name).join(', ')}',
      );
  final Track to;
  final List<Blocker> blockers;
}

/// Một bản phát hành (một versionCode) đang ở một track.
final class TrackPromotion {
  TrackPromotion._(this.versionCode, this.track, List<Track> history)
    : history = List.unmodifiable(history);

  /// Bản vừa tải lên, luôn bắt đầu ở internal.
  TrackPromotion.uploaded({required int versionCode})
    : this._(versionCode, Track.internal, const [Track.internal]);

  final int versionCode;
  final Track track;
  final List<Track> history;

  /// Đường đi hợp lệ. Open testing là tuỳ chọn nên closed có thể đi thẳng
  /// production; không có đường từ internal thẳng lên production và không có
  /// đường đi lùi.
  static const Map<Track, Set<Track>> allowed = {
    Track.internal: {Track.closed},
    Track.closed: {Track.open, Track.production},
    Track.open: {Track.production},
    Track.production: {},
  };

  TrackPromotion promote(Track to, PreflightInput input) {
    if (!allowed[track]!.contains(to)) throw IllegalMove(track, to);
    final blockers = preflight(to, input);
    if (blockers.isNotEmpty) throw PromotionBlocked(to, blockers);
    return TrackPromotion._(versionCode, to, [...history, to]);
  }
}
