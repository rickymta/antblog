/// Kế hoạch phased release 7 ngày của App Store.
///
/// Bảng phần trăm và hạn tạm dừng lấy từ "Release a version update in phases"
/// (App Store Connect Help, kiểm tại 11/2026). Apple không nói mốc đổi ngày
/// chính xác, nên lab mô hình hoá mỗi "ngày" là 24 giờ kể từ lúc bắt đầu;
/// đừng dùng nó để đoán giờ chuyển đợt ngoài đời.
library;

/// Quyết định sau khi xem số liệu của đợt hiện tại.
enum RolloutAction { wait, proceed, pause }

/// Số liệu phiên của bản đang phát hành (minh hoạ: lấy từ Crashlytics, bài 48).
final class RolloutMetrics {
  const RolloutMetrics({required this.sessions, required this.crashedSessions});

  final int sessions;
  final int crashedSessions;

  @override
  String toString() => 'RolloutMetrics($crashedSessions/$sessions phiên crash)';
}

/// Ngưỡng quyết định. Số mặc định chỉ để minh hoạ, không phải chuẩn ngành.
final class RolloutThresholds {
  const RolloutThresholds({
    this.minSessions = 500,
    this.minCrashFreeBasisPoints = 9950,
  });

  /// Dưới mức này số liệu chưa đủ tin: chờ thêm, không kết luận.
  final int minSessions;

  /// Phiên không crash tối thiểu, tính bằng điểm cơ bản (9950 = 99,50%).
  /// Dùng số nguyên để so sánh biên không bị sai số `double`.
  final int minCrashFreeBasisPoints;
}

final class PhasedReleasePlan {
  PhasedReleasePlan({required this.startedAt})
    : _pauses = const [],
      assert(startedAt.isUtc, 'startedAt phải là UTC');

  PhasedReleasePlan._(this.startedAt, this._pauses);

  /// Phần trăm người dùng bật tự động cập nhật được nhận bản mới theo ngày.
  static const List<int> percentByDay = [1, 2, 5, 10, 20, 50, 100];

  static const Duration day = Duration(hours: 24);

  /// Tổng thời gian được tạm dừng trong một đợt phát hành.
  static const Duration pauseBudget = Duration(days: 30);

  final DateTime startedAt;
  final List<({DateTime from, DateTime? to})> _pauses;

  /// Phần trăm của ngày thứ [dayNumber] (1..7); sau ngày 7 là 100.
  static int percentOnDay(int dayNumber) {
    if (dayNumber < 1) {
      throw ArgumentError.value(dayNumber, 'dayNumber', 'phải từ 1 trở lên');
    }
    return dayNumber > percentByDay.length ? 100 : percentByDay[dayNumber - 1];
  }

  bool get isPaused => _pauses.isNotEmpty && _pauses.last.to == null;

  /// Tổng thời gian đã tạm dừng tính đến [now].
  Duration pausedFor(DateTime now) {
    var total = Duration.zero;
    for (final p in _pauses) {
      total += (p.to ?? now).difference(p.from);
    }
    return total;
  }

  /// Hạn mức tạm dừng còn lại, không bao giờ âm.
  Duration pauseBudgetLeft(DateTime now) {
    final left = pauseBudget - pausedFor(now);
    return left.isNegative ? Duration.zero : left;
  }

  /// Tạm dừng tại [now]. Ném [StateError] nếu đang dừng hoặc hết hạn mức.
  PhasedReleasePlan pause(DateTime now) {
    if (isPaused) throw StateError('Đợt phát hành đang tạm dừng.');
    if (pauseBudgetLeft(now) <= Duration.zero) {
      throw StateError('Đã dùng hết ${pauseBudget.inDays} ngày tạm dừng.');
    }
    return PhasedReleasePlan._(startedAt, [..._pauses, (from: now, to: null)]);
  }

  /// Tiếp tục tại [now]; đợt chạy tiếp từ ngày đã dừng.
  PhasedReleasePlan resume(DateTime now) {
    if (!isPaused) throw StateError('Đợt phát hành không đang tạm dừng.');
    return PhasedReleasePlan._(startedAt, [
      ..._pauses.sublist(0, _pauses.length - 1),
      (from: _pauses.last.from, to: now),
    ]);
  }

  /// Ngày thứ mấy của đợt (1 là ngày đầu); 0 nếu [now] trước lúc bắt đầu.
  /// Đang tạm dừng thì giữ nguyên ngày tại lúc dừng.
  int dayAt(DateTime now) {
    final clock = isPaused ? _pauses.last.from : now;
    final elapsed = clock.difference(startedAt) - _closedPauses();
    if (elapsed.isNegative) return 0;
    return elapsed.inMicroseconds ~/ day.inMicroseconds + 1;
  }

  Duration _closedPauses() {
    var total = Duration.zero;
    for (final p in _pauses) {
      if (p.to != null) total += p.to!.difference(p.from);
    }
    return total;
  }

  int percentAt(DateTime now) {
    final d = dayAt(now);
    return d == 0 ? 0 : percentOnDay(d);
  }

  /// Số người dùng tối đa nhận bản mới tự động, làm tròn xuống.
  /// [autoUpdateUsers] chỉ gồm người bật tự động cập nhật; ai tự vào App
  /// Store bấm Cập nhật thì nhận được bất cứ lúc nào.
  int exposedUsers(int autoUpdateUsers, DateTime now) {
    if (autoUpdateUsers < 0) {
      throw ArgumentError.value(autoUpdateUsers, 'autoUpdateUsers', 'không âm');
    }
    return autoUpdateUsers * percentAt(now) ~/ 100;
  }

  /// Đợt hiện tại nên chờ, đi tiếp hay dừng, theo [metrics] và [thresholds].
  ///
  /// Tạm dừng chỉ chặn người nhận *tự động* từ giờ. Ai đã nhận bản lỗi vẫn giữ
  /// nó: tài liệu chỉ mô tả dừng đợt, không có thao tác đưa họ về bản cũ.
  /// Sửa lỗi nghĩa là gửi một bản mới (bài 59).
  RolloutAction decide(
    RolloutMetrics metrics, {
    RolloutThresholds thresholds = const RolloutThresholds(),
  }) {
    if (metrics.sessions < 0 ||
        metrics.crashedSessions < 0 ||
        metrics.crashedSessions > metrics.sessions) {
      throw ArgumentError('Số phiên không hợp lệ: $metrics');
    }
    if (metrics.sessions < thresholds.minSessions) return RolloutAction.wait;
    final crashFree = (metrics.sessions - metrics.crashedSessions) * 10000;
    return crashFree < thresholds.minCrashFreeBasisPoints * metrics.sessions
        ? RolloutAction.pause
        : RolloutAction.proceed;
  }
}
