import 'build_status.dart';
import 'tester_groups.dart';

enum ReviewExpectation {
  /// Nhóm internal không qua TestFlight App Review.
  notRequired,

  /// Bản đầu của một version gửi nhóm external: review đầy đủ.
  full,

  /// Các bản sau của cùng version: Apple nói "có thể" không cần review đầy đủ.
  mayBeLighter,
}

/// Mức review mong đợi khi gán build [version] cho nhóm loại [kind].
///
/// [versionsAlreadySubmitted] là các version đã từng có build gửi review.
/// Hàm không hứa các bản sau chắc chắn được bỏ qua: Apple chỉ nói "might not".
ReviewExpectation expectedReview({
  required GroupKind kind,
  required String version,
  required Set<String> versionsAlreadySubmitted,
}) {
  if (kind == GroupKind.internal) return ReviewExpectation.notRequired;
  return versionsAlreadySubmitted.contains(version)
      ? ReviewExpectation.mayBeLighter
      : ReviewExpectation.full;
}

/// Một lần gửi build lên TestFlight App Review.
class BetaSubmission {
  const BetaSubmission({
    required this.version,
    required this.build,
    required this.submittedAt,
    required this.status,
  });

  final String version;
  final String build;

  /// UTC.
  final DateTime submittedAt;
  final BuildStatus status;
}

class SubmissionDecision {
  const SubmissionDecision.allowed() : reasons = const [], retryAt = null;

  const SubmissionDecision.blocked(this.reasons, {this.retryAt});

  final List<String> reasons;

  /// Sớm nhất có thể thử lại vì giới hạn 24 giờ; `null` nếu phải chờ một build
  /// đang review xong.
  final DateTime? retryAt;

  bool get isAllowed => reasons.isEmpty;
}

/// Hai giới hạn gửi Beta App Review mà trang "Invite external testers" của
/// Apple nêu (kiểm tại 11/2026): tối đa sáu build trong 24 giờ, và mỗi version
/// chỉ một build đang review tại một thời điểm.
const maxBetaSubmissionsPer24h = 6;

SubmissionDecision checkBetaSubmission({
  required DateTime now,
  required String version,
  required List<BetaSubmission> history,
}) {
  final reasons = <String>[];
  DateTime? retryAt;

  final windowStart = now.subtract(const Duration(hours: 24));
  final recent =
      history.where((s) => s.submittedAt.isAfter(windowStart)).toList()
        ..sort((a, b) => a.submittedAt.compareTo(b.submittedAt));
  if (recent.length >= maxBetaSubmissionsPer24h) {
    // Mốc giải phóng một chỗ: bản cũ nhất cần thiết để còn dưới giới hạn.
    final oldestBlocking =
        recent[recent.length - maxBetaSubmissionsPer24h].submittedAt;
    retryAt = oldestBlocking.add(const Duration(hours: 24));
    reasons.add(
      'Đã gửi ${recent.length} build trong 24 giờ qua, giới hạn '
      '$maxBetaSubmissionsPer24h.',
    );
  }

  final inReview = history.where(
    (s) =>
        s.version == version &&
        (s.status == BuildStatus.waitingForReview ||
            s.status == BuildStatus.inBetaReview),
  );
  if (inReview.isNotEmpty) {
    reasons.add(
      'Version $version đã có build ${inReview.first.build} đang chờ hoặc '
      'trong review; chỉ một build mỗi version được review cùng lúc.',
    );
  }

  return reasons.isEmpty
      ? const SubmissionDecision.allowed()
      : SubmissionDecision.blocked(reasons, retryAt: retryAt);
}

/// Cửa sổ 90 ngày của một build trên TestFlight.
///
/// Trang Apple nói build "unavailable after 90 days" nhưng bản ta đọc không nói
/// mốc tính từ đâu. Lab tính từ lúc tải lên, tức mốc sớm nhất có thể, để cảnh
/// báo sớm. Số ngày còn lại thật hiện trong App Store Connect.
abstract final class BuildExpiry {
  static const window = Duration(days: 90);

  static DateTime expiresAt(DateTime uploadedAt) => uploadedAt.add(window);

  /// Số ngày còn lại, làm tròn lên; 0 khi đã hết hạn.
  static int daysLeft(DateTime uploadedAt, DateTime now) {
    final left = expiresAt(uploadedAt).difference(now);
    if (left <= Duration.zero) return 0;
    return (left.inSeconds / Duration.secondsPerDay).ceil();
  }

  static bool isExpired(DateTime uploadedAt, DateTime now) =>
      daysLeft(uploadedAt, now) == 0;

  /// Build sắp hết hạn: còn không quá [warnDays] ngày (lab chọn 14) nhưng chưa hết.
  static bool isExpiringSoon(
    DateTime uploadedAt,
    DateTime now, {
    int warnDays = 14,
  }) {
    final d = daysLeft(uploadedAt, now);
    return d > 0 && d <= warnDays;
  }
}
