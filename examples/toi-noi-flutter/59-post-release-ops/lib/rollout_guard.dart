import 'store_platform.dart';

/// Số liệu một bản đang phát hành dần. Tỷ lệ crash-free tính bằng **điểm cơ
/// bản** (basis point, 9.920 là 99,20%) để so sánh bằng số nguyên.
final class RolloutStats {
  const RolloutStats({
    required this.version,
    required this.platform,
    required this.rolloutPercent,
    required this.sessions,
    required this.crashFreeBp,
    required this.baselineCrashFreeBp,
  });

  final String version;
  final StorePlatform platform;

  /// Phần trăm người dùng đã nhận bản này (0..100).
  final int rolloutPercent;

  /// Số phiên của bản này trong cửa sổ đo.
  final int sessions;

  final int crashFreeBp;

  /// Crash-free của bản ổn định trước đó, cùng cửa sổ đo.
  final int baselineCrashFreeBp;

  int get dropBp => baselineCrashFreeBp - crashFreeBp;
}

enum RolloutAction { wait, proceed, pause, halt }

/// Việc cụ thể trên cửa hàng.
enum StoreAction {
  /// Android: tăng tỷ lệ staged rollout. iOS: phased release tự tăng.
  continueRollout,

  /// Android: giữ nguyên tỷ lệ, không tăng.
  holdAndroidPercent,

  /// Android: Manage rollout, Halt rollout.
  haltAndroidRollout,

  /// iOS: tạm dừng phased release (tổng tối đa 30 ngày theo tài liệu Apple).
  pauseIosPhasedRelease,
}

final class RolloutDecision {
  const RolloutDecision(this.action, this.storeAction, this.reason);
  final RolloutAction action;
  final StoreAction storeAction;
  final String reason;
}

/// Ngưỡng quyết định. Mặc định là số minh hoạ, mỗi đội tự hiệu chỉnh theo
/// lịch sử của mình.
final class RolloutThresholds {
  const RolloutThresholds({
    this.minSessions = 2000,
    this.floorMinSessions = 300,
    this.pauseDropBp = 30,
    this.haltDropBp = 100,
    this.floorBp = 9700,
  });

  /// Số phiên tối thiểu để xét mức giảm so với bản trước.
  final int minSessions;

  /// Số phiên tối thiểu để xét sàn tuyệt đối. Thấp hơn [minSessions] vì
  /// crash-free dưới sàn là tín hiệu đủ nặng để dừng sớm.
  final int floorMinSessions;
  final int pauseDropBp;
  final int haltDropBp;
  final int floorBp;
}

abstract final class RolloutGuard {
  static RolloutDecision decide(
    RolloutStats stats, [
    RolloutThresholds t = const RolloutThresholds(),
  ]) {
    final isAndroid = stats.platform == StorePlatform.android;
    // iOS không có "giữ nguyên %": phased release chạy theo ngày, chỉ có tạm dừng.
    final hold = isAndroid
        ? StoreAction.holdAndroidPercent
        : StoreAction.continueRollout;
    final pause = isAndroid
        ? StoreAction.holdAndroidPercent
        : StoreAction.pauseIosPhasedRelease;
    final halt = isAndroid
        ? StoreAction.haltAndroidRollout
        : StoreAction.pauseIosPhasedRelease;

    if (stats.sessions < t.floorMinSessions) {
      return RolloutDecision(
        RolloutAction.wait,
        hold,
        'Mới ${stats.sessions} phiên, cần ít nhất ${t.floorMinSessions} để xét',
      );
    }
    if (stats.crashFreeBp < t.floorBp) {
      return RolloutDecision(
        RolloutAction.halt,
        halt,
        'Crash-free ${_pct(stats.crashFreeBp)} thấp hơn sàn ${_pct(t.floorBp)}',
      );
    }
    if (stats.sessions < t.minSessions) {
      return RolloutDecision(
        RolloutAction.wait,
        hold,
        'Mới ${stats.sessions} phiên, cần ít nhất ${t.minSessions} để tin mức giảm',
      );
    }
    if (stats.dropBp >= t.haltDropBp) {
      return RolloutDecision(
        RolloutAction.halt,
        halt,
        'Crash-free giảm ${_pts(stats.dropBp)} so với bản trước',
      );
    }
    if (stats.dropBp >= t.pauseDropBp) {
      return RolloutDecision(
        RolloutAction.pause,
        pause,
        'Crash-free giảm ${_pts(stats.dropBp)}, giữ nguyên để điều tra',
      );
    }
    return RolloutDecision(
      RolloutAction.proceed,
      StoreAction.continueRollout,
      'Crash-free ${_pct(stats.crashFreeBp)}, giảm ${_pts(stats.dropBp)}',
    );
  }

  /// Nội dung thông báo gửi kênh vận hành.
  static String notice(RolloutStats stats, RolloutDecision d) {
    final platform = stats.platform == StorePlatform.android
        ? 'Android'
        : 'iOS';
    return '[${d.action.name.toUpperCase()}] Tới Nơi ${stats.version} ($platform, '
        '${stats.rolloutPercent}% người dùng, ${stats.sessions} phiên): ${d.reason}.';
  }

  static String _pct(int bp) =>
      '${(bp ~/ 100)},${(bp % 100).toString().padLeft(2, '0')}%';

  static String _pts(int bp) =>
      '${bp ~/ 100},${(bp % 100).toString().padLeft(2, '0')} điểm %';
}
