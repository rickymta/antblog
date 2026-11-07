import 'dart:async';
import 'dart:math';

import 'package:flutter/foundation.dart';

import 'device_api.dart';
import 'push_message.dart';
import 'push_source.dart';

/// Tình trạng đăng ký nhận push của thiết bị này.
enum PushStatus {
  /// Chưa bật, bị từ chối, hoặc đã đăng xuất.
  off,

  /// Đã được phép và đã gọi `register`, chờ FCM trả định danh.
  waiting,

  /// Backend đã giữ định danh hiện tại.
  synced,

  /// Có định danh nhưng chưa báo được cho backend; gọi [PushRegistrar.retry].
  failed,
}

/// Giữ backend luôn biết đúng định danh push của thiết bị.
final class PushRegistrar {
  PushRegistrar({
    required this._source,
    required this._api,
    required this._deviceId,
    required this._platform,
  });

  final PushSource _source;
  final DeviceApi _api;
  final String _deviceId;
  final String _platform;

  final status = ValueNotifier<PushStatus>(PushStatus.off);

  StreamSubscription<String>? _subscription;
  Future<void> _queue = Future<void>.value();
  String? _uploaded;
  String? _pending;
  bool _registerFailed = false;

  /// Người dùng bấm "Bật thông báo": hỏi quyền, được phép thì đăng ký.
  Future<PushPermission> enable() async {
    final permission = await _source.requestPermission();
    if (permission.isGranted) await _begin();
    return permission;
  }

  /// Mỗi lần mở app khi đã đăng nhập: chỉ đăng ký nếu quyền đã có sẵn, và
  /// không bao giờ tự hiện hộp thoại.
  Future<void> resume() async {
    if ((await _source.permission()).isGranted) await _begin();
  }

  Future<void> _begin() async {
    // Nghe trước khi gọi register: tài liệu dặn vậy, vì sự kiện có thể tới
    // ngay trong lúc register còn chạy.
    _subscription ??= _source.onRegistered.listen(_onRegistered);
    if (status.value != PushStatus.synced) status.value = PushStatus.waiting;
    try {
      await _source.register();
      _registerFailed = false;
    } on Exception {
      _registerFailed = true;
      status.value = PushStatus.failed;
    }
  }

  void _onRegistered(String pushId) {
    _pending = pushId;
    _queue = _queue.then((_) => _upload(pushId));
  }

  Future<void> _upload(String pushId) async {
    if (pushId == _uploaded) {
      status.value = PushStatus.synced;
      return;
    }
    try {
      await _api.upsert(
        deviceId: _deviceId,
        pushId: pushId,
        platform: _platform,
      );
      _uploaded = pushId;
      if (_pending == pushId) _pending = null;
      status.value = PushStatus.synced;
    } on DeviceApiException {
      status.value = PushStatus.failed;
    }
  }

  /// Thử lại phần đang hỏng (ví dụ khi app quay lại foreground): nếu
  /// `register` từng lỗi thì gọi lại nó, nếu không thì gửi lại định danh chưa
  /// báo được cho backend.
  Future<void> retry() async {
    if (_registerFailed) {
      await _begin();
      return;
    }
    final pushId = _pending;
    if (pushId == null) return;
    _queue = _queue.then((_) => _upload(pushId));
    await _queue;
  }

  /// Đăng xuất. Thứ tự có chủ đích: xoá ở backend TRƯỚC, khi phiên còn hợp
  /// lệ; rồi mới huỷ đăng ký ở FCM. Backend lỗi thì vẫn huỷ ở máy: tin gửi
  /// tới định danh cũ sẽ lỗi và backend dọn dòng đó khi gặp.
  Future<({bool serverRemoved, bool unregistered})> signOut() async {
    await _subscription?.cancel();
    _subscription = null;
    await _queue;

    var serverRemoved = true;
    try {
      await _api.remove(_deviceId);
    } on DeviceApiException {
      serverRemoved = false;
    }
    var unregistered = true;
    try {
      await _source.unregister();
    } on Exception {
      unregistered = false;
    }
    _uploaded = null;
    _pending = null;
    _registerFailed = false;
    status.value = PushStatus.off;
    return (serverRemoved: serverRemoved, unregistered: unregistered);
  }
}

/// Mã cài đặt ngẫu nhiên 128 bit. Bản thật lưu bằng `shared_preferences` (bài
/// 22) để giữ nguyên giữa các lần mở app; lab chỉ sinh trong bộ nhớ.
String generateInstallationId([Random? random]) {
  final rng = random ?? Random.secure();
  final bytes = List<int>.generate(16, (_) => rng.nextInt(256));
  return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
}
