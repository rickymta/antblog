import 'package:flutter/foundation.dart';

import 'location_access.dart';
import 'location_models.dart';
import 'location_source.dart';

/// Nơi app nhớ "đã hỏi chưa", vì plugin không phân biệt chưa hỏi với từ chối.
///
/// Bản thật lưu vào `shared_preferences` (bài 22); lab dùng bản trong bộ nhớ.
abstract interface class AskedStore {
  Future<bool> asked(AskTarget target);
  Future<void> markAsked(AskTarget target);

  /// Lần xin gần nhất bị hệ điều hành chặn vĩnh viễn (xem `blockedByRequest`).
  Future<bool> blocked();
  Future<void> setBlocked(bool value);
}

final class MemoryAskedStore implements AskedStore {
  final Set<AskTarget> _asked = {};
  bool _blocked = false;

  @override
  Future<bool> asked(AskTarget target) async => _asked.contains(target);

  @override
  Future<void> markAsked(AskTarget target) async => _asked.add(target);

  @override
  Future<bool> blocked() async => _blocked;

  @override
  Future<void> setBlocked(bool value) async => _blocked = value;
}

/// Nối máy trạng thái thuần [reduce] với [LocationSource] và [AskedStore].
final class LocationAccessController
    extends ValueNotifier<LocationAccessState> {
  LocationAccessController({
    required this._source,
    required this._store,
    required this.role,
  }) : super(const NotAsked());

  final LocationSource _source;
  final AskedStore _store;
  final AppRole role;

  /// Đọc lại trạng thái từ hệ điều hành (gọi khi mở màn hình và khi app
  /// quay lại từ Cài đặt). Bỏ qua khi hộp thoại đang hiện.
  Future<void> refresh() async {
    if (value is Asking) {
      return;
    }
    await _observe();
  }

  /// Xin quyền "khi dùng app". Chỉ gọi sau khi người dùng đã đọc lời giải
  /// thích. Không làm gì nếu trạng thái hiện tại không cho phép hỏi.
  Future<void> requestForeground() async {
    final next = reduce(value, const RequestStarted(AskTarget.foreground));
    if (value is Asking || next is! Asking) {
      return;
    }
    value = next;
    try {
      final result = await _source.requestForeground();
      await _store.markAsked(AskTarget.foreground);
      // Android: chỉ kết quả của lần xin này mới biết là bị chặn vĩnh viễn.
      await _store.setBlocked(result == SystemPermission.deniedForever);
    } on LocationRequestInProgressException {
      // Hộp thoại khác đang hiện: không đánh dấu đã hỏi, đọc lại bên dưới.
    } finally {
      await _observe(force: true);
    }
  }

  /// Xin nâng lên "luôn cho phép". App khách không bao giờ làm được việc này.
  Future<void> requestBackground() async {
    if (role != AppRole.driver || !_source.canRequestBackground) {
      return;
    }
    final next = reduce(value, const RequestStarted(AskTarget.background));
    if (value is Asking || next is! Asking) {
      return;
    }
    value = next;
    try {
      await _source.requestBackground();
      await _store.markAsked(AskTarget.background);
    } on LocationRequestInProgressException {
      // như requestForeground
    } finally {
      await _observe(force: true);
    }
  }

  /// Xin nâng từ vị trí gần đúng lên chính xác. Chỉ có nghĩa khi đã có quyền
  /// và đang ở mức `reduced`.
  Future<void> requestPrecise() async {
    if (value.precision != LocationPrecision.reduced) {
      return;
    }
    await _source.requestPrecise();
    await _observe();
  }

  /// Nền tảng có bước xin quyền nền không (để giao diện quyết định có mời không).
  bool get backgroundSupported => _source.canRequestBackground;

  Future<bool> openAppSettings() => _source.openAppSettings();

  Future<bool> openLocationSettings() => _source.openLocationSettings();

  Future<void> _observe({bool force = false}) async {
    final enabled = await _source.isServiceEnabled();
    final permission = await _source.checkPermission();
    final granted =
        permission == SystemPermission.whileInUse ||
        permission == SystemPermission.always;
    final precision = granted
        ? await _source.precision()
        : LocationPrecision.precise;
    if (granted) {
      // Người dùng đã mở khoá trong Cài đặt: bỏ cờ bị chặn.
      await _store.setBlocked(false);
    }
    final snapshot = LocationSnapshot(
      serviceEnabled: enabled,
      permission: permission,
      precision: precision,
      askedForeground: await _store.asked(AskTarget.foreground),
      askedBackground: await _store.asked(AskTarget.background),
      blockedByRequest: await _store.blocked(),
    );
    // Một lần refresh() chậm có thể về sau khi hộp thoại đã mở: bỏ kết quả cũ.
    if (!force && value is Asking) {
      return;
    }
    value = reduce(value, Observed(snapshot));
  }
}
