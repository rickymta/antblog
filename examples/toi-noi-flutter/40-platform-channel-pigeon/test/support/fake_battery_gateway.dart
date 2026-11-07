import 'dart:async';

import 'package:toi_noi_40/toi_noi_40.dart';

/// Gateway giả cho test logic: không có channel nào.
class FakeBatteryGateway implements BatteryGateway {
  final StreamController<BatteryStatus> _controller =
      StreamController<BatteryStatus>.broadcast();
  Completer<BatteryStatus>? _pendingRead;
  int readCount = 0;

  bool get hasListener => _controller.hasListener;

  @override
  Stream<BatteryStatus> get changes => _controller.stream;

  @override
  Future<BatteryStatus> read() {
    readCount++;
    return (_pendingRead = Completer<BatteryStatus>()).future;
  }

  void completeRead(BatteryStatus status) => _pendingRead!.complete(status);

  void failRead([String code = 'UNAVAILABLE']) =>
      _pendingRead!.completeError(BatteryUnavailableException(code));

  void emit(BatteryStatus status) => _controller.add(status);

  @override
  void dispose() => unawaited(_controller.close());
}

/// Cho các luồng async của test chạy hết các microtask đang chờ.
Future<void> settle() => Future<void>.delayed(Duration.zero);
