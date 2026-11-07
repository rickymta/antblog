import 'location_models.dart';

/// Việc đang xin: quyền "khi dùng app" hay nâng lên "luôn cho phép".
enum AskTarget { foreground, background }

/// Ảnh chụp những gì ta biết về vị trí tại một thời điểm.
final class LocationSnapshot {
  const LocationSnapshot({
    required this.serviceEnabled,
    required this.permission,
    required this.precision,
    required this.askedForeground,
    required this.askedBackground,
    this.blockedByRequest = false,
  });

  final bool serviceEnabled;
  final SystemPermission permission;
  final LocationPrecision precision;

  /// App đã từng hiện hộp thoại xin quyền foreground chưa (app tự nhớ).
  final bool askedForeground;

  /// App đã từng xin quyền nền chưa (app tự nhớ).
  final bool askedBackground;

  /// Lần xin gần nhất trả về `deniedForever` (app tự nhớ). Android cần cờ này
  /// vì `checkPermission` không bao giờ báo `deniedForever`, chỉ kết quả của
  /// chính lần xin mới báo.
  final bool blockedByRequest;
}

/// Bảy trạng thái truy cập vị trí. Sealed để `switch` vét cạn.
sealed class LocationAccessState {
  const LocationAccessState();
}

/// Chưa từng hỏi: chưa có hộp thoại nào hiện ra.
final class NotAsked extends LocationAccessState {
  const NotAsked();
}

/// Hộp thoại của hệ điều hành đang hiện.
final class Asking extends LocationAccessState {
  const Asking(this.target);
  final AskTarget target;

  @override
  bool operator ==(Object other) => other is Asking && other.target == target;

  @override
  int get hashCode => target.hashCode;
}

/// Được dùng khi app đang mở (hoặc đang chạy foreground service).
final class WhileInUse extends LocationAccessState {
  const WhileInUse(this.precision, {this.backgroundAsked = false});
  final LocationPrecision precision;
  final bool backgroundAsked;

  @override
  bool operator ==(Object other) =>
      other is WhileInUse &&
      other.precision == precision &&
      other.backgroundAsked == backgroundAsked;

  @override
  int get hashCode => Object.hash(precision, backgroundAsked);
}

/// Được dùng cả khi app ở nền ("luôn cho phép").
final class Always extends LocationAccessState {
  const Always(this.precision);
  final LocationPrecision precision;

  @override
  bool operator ==(Object other) =>
      other is Always && other.precision == precision;

  @override
  int get hashCode => precision.hashCode;
}

/// Đã từ chối nhưng còn hỏi lại được (chủ yếu Android).
final class Denied extends LocationAccessState {
  const Denied();
}

/// Đã từ chối và hệ điều hành sẽ không hiện hộp thoại nữa: chỉ còn cài đặt.
final class DeniedForever extends LocationAccessState {
  const DeniedForever();
}

/// Dịch vụ vị trí của thiết bị đang tắt, bất kể quyền của app.
final class ServiceOff extends LocationAccessState {
  const ServiceOff();
}

sealed class LocationAccessEvent {
  const LocationAccessEvent();
}

/// Đọc lại từ hệ điều hành.
final class Observed extends LocationAccessEvent {
  const Observed(this.snapshot);
  final LocationSnapshot snapshot;
}

/// Người dùng đã đọc lời giải thích và bấm tiếp tục.
final class RequestStarted extends LocationAccessEvent {
  const RequestStarted(this.target);
  final AskTarget target;
}

/// Hàm thuần: cùng (state, event) luôn ra cùng kết quả, không gọi plugin.
///
/// Sự kiện không hợp lệ trả về nguyên trạng thái cũ, không ném lỗi: ví dụ
/// `RequestStarted(foreground)` khi đã `DeniedForever` không được phép mở hộp
/// thoại (hệ điều hành cũng sẽ không hiện).
LocationAccessState reduce(
  LocationAccessState state,
  LocationAccessEvent event,
) {
  return switch (event) {
    Observed(:final snapshot) => _fromSnapshot(snapshot),
    RequestStarted(target: AskTarget.foreground) => switch (state) {
      NotAsked() || Denied() => const Asking(AskTarget.foreground),
      Asking() ||
      WhileInUse() ||
      Always() ||
      DeniedForever() ||
      ServiceOff() => state,
    },
    RequestStarted(target: AskTarget.background) => switch (state) {
      WhileInUse() => const Asking(AskTarget.background),
      NotAsked() ||
      Asking() ||
      Always() ||
      Denied() ||
      DeniedForever() ||
      ServiceOff() => state,
    },
  };
}

LocationAccessState _fromSnapshot(LocationSnapshot s) {
  if (!s.serviceEnabled) {
    return const ServiceOff();
  }
  return switch (s.permission) {
    SystemPermission.always => Always(s.precision),
    SystemPermission.whileInUse => WhileInUse(
      s.precision,
      backgroundAsked: s.askedBackground,
    ),
    SystemPermission.deniedForever => const DeniedForever(),
    SystemPermission.denied =>
      !s.askedForeground
          ? const NotAsked()
          : s.blockedByRequest
          ? const DeniedForever()
          : const Denied(),
  };
}

/// Việc giao diện nên làm tiếp theo.
enum LocationAdvice {
  /// Đang chờ hộp thoại của hệ điều hành.
  wait,

  /// Giải thích rồi mới xin quyền foreground.
  explainThenRequest,

  /// Đã đủ quyền cho việc cần làm.
  ready,

  /// Giải thích rồi mới xin nâng lên "luôn cho phép" (chỉ app tài xế, có chuyến).
  offerBackground,

  openAppSettings,

  openLocationSettings,
}

/// Gợi ý cho giao diện theo [state], vai trò, việc có chuyến đang chạy và việc
/// nền tảng có bước xin quyền nền hay không.
LocationAdvice adviceFor(
  LocationAccessState state,
  AppRole role, {
  bool tripActive = false,
  bool backgroundAvailable = true,
}) {
  return switch (state) {
    Asking() => LocationAdvice.wait,
    NotAsked() || Denied() => LocationAdvice.explainThenRequest,
    DeniedForever() => LocationAdvice.openAppSettings,
    ServiceOff() => LocationAdvice.openLocationSettings,
    WhileInUse(:final backgroundAsked) =>
      role == AppRole.driver &&
              tripActive &&
              backgroundAvailable &&
              !backgroundAsked
          ? LocationAdvice.offerBackground
          : LocationAdvice.ready,
    Always() => LocationAdvice.ready,
  };
}

extension LocationAccessStateX on LocationAccessState {
  /// Có thể mở luồng vị trí không.
  bool get canTrack => this is WhileInUse || this is Always;

  /// Độ chính xác khi đã có quyền; `null` nếu chưa có.
  LocationPrecision? get precision => switch (this) {
    WhileInUse(:final precision) || Always(:final precision) => precision,
    NotAsked() ||
    Asking() ||
    Denied() ||
    DeniedForever() ||
    ServiceOff() => null,
  };
}
