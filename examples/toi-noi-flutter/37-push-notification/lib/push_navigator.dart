import 'dart:async';

import 'local_notifier.dart';
import 'push_message.dart';
import 'push_router.dart';
import 'push_source.dart';

/// Nối bốn đường một thông báo có thể tới app với một hàm `open(location)`:
///
/// | App đang ở | Nguồn | Hành động |
/// |---|---|---|
/// | foreground | `onForeground` | hiện thông báo cục bộ, KHÔNG tự chuyển màn hình |
/// | background | `onOpenedFromBackground` | mở màn hình |
/// | terminated | `takeInitialMessage` | mở màn hình |
/// | chạm thông báo cục bộ | `taps` hoặc `launchPayload` | mở màn hình |
final class PushNavigator {
  PushNavigator({
    required this._source,
    required this._notifier,
    required this._open,
    this._router = const PushRouter(),
    this._onRejected,
  });

  final PushSource _source;
  final LocalNotifier _notifier;
  final void Function(String location) _open;
  final PushRouter _router;
  final void Function(String reason)? _onRejected;

  final _subscriptions = <StreamSubscription<Object?>>[];
  var _started = false;
  var _notificationId = 0;

  /// Đăng ký lắng nghe trước, rồi mới hỏi tin khởi chạy: tin tới giữa hai
  /// bước đó vẫn không bị mất.
  Future<void> start() async {
    if (_started) return;
    _started = true;
    _subscriptions
      ..add(_source.onForeground.listen(_showForeground))
      ..add(
        _source.onOpenedFromBackground.listen(
          (message) => _openFrom(_router.decide(message.data)),
        ),
      )
      ..add(
        _notifier.taps.listen(
          (payload) => _openFrom(_router.decideFromPayload(payload)),
        ),
      );

    final initial = await _source.takeInitialMessage();
    if (initial != null) {
      _openFrom(_router.decide(initial.data));
      return;
    }
    final launch = await _notifier.launchPayload();
    if (launch != null) {
      _openFrom(_router.decideFromPayload(launch));
    }
  }

  Future<void> dispose() async {
    for (final subscription in _subscriptions) {
      await subscription.cancel();
    }
    _subscriptions.clear();
    _started = false;
  }

  Future<void> _showForeground(PushMessage message) async {
    final decision = _router.decide(message.data);
    if (decision is! OpenTrip) {
      _reject(decision);
      return;
    }
    await _notifier.show(
      id: _notificationId++,
      title: message.title ?? 'Tới Nơi',
      body: message.body ?? 'Chuyến của bạn có cập nhật.',
      payload: decision.toPayload(),
    );
  }

  void _openFrom(PushDecision decision) {
    switch (decision) {
      case OpenTrip(:final location):
        _open(location);
      case IgnorePush():
        _reject(decision);
    }
  }

  void _reject(PushDecision decision) {
    if (decision is IgnorePush) _onRejected?.call(decision.reason);
  }
}
