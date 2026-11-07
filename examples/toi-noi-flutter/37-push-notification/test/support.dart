import 'dart:async';

import 'package:toi_noi_37/device_api.dart';
import 'package:toi_noi_37/local_notifier.dart';
import 'package:toi_noi_37/push_message.dart';
import 'package:toi_noi_37/push_source.dart';

PushMessage tripMessage(String tripId, {Map<String, Object?>? extra}) =>
    PushMessage(
      id: 'm-$tripId',
      title: 'Tài xế đã nhận chuyến',
      body: 'Mở app để xem.',
      data: {'type': 'trip_update', 'tripId': tripId, ...?extra},
    );

/// Nguồn push giả. Ba trạng thái của app tương ứng ba cách phát tin:
/// [emitForeground], [tapFromBackground], [initialMessage].
class FakePushSource implements PushSource {
  FakePushSource({
    this.permissionValue = PushPermission.notAsked,
    this.nextPermissionAnswer = PushPermission.granted,
    this.initialMessage,
    this.fidOnRegister,
    this.timeline,
  });

  /// Nhật ký chung với [FakeDeviceApi] để kiểm thứ tự giữa hai bên.
  final List<String>? timeline;

  PushPermission permissionValue;
  PushPermission nextPermissionAnswer;
  PushMessage? initialMessage;

  /// Mỗi lần `register()` phát FID này qua `onRegistered` (ngay trong lúc
  /// register chạy), như tài liệu mô tả.
  String? fidOnRegister;

  final _foreground = StreamController<PushMessage>.broadcast();
  final _opened = StreamController<PushMessage>.broadcast();
  final _registered = StreamController<String>.broadcast();

  final calls = <String>[];
  bool failRegister = false;
  bool failUnregister = false;
  int takeInitialCalls = 0;

  void emitForeground(PushMessage m) => _foreground.add(m);
  void tapFromBackground(PushMessage m) => _opened.add(m);
  void emitRegistered(String fid) => _registered.add(fid);

  bool get hasRegisteredListener => _registered.hasListener;

  @override
  Stream<PushMessage> get onForeground => _foreground.stream;

  @override
  Stream<PushMessage> get onOpenedFromBackground => _opened.stream;

  @override
  Future<PushMessage?> takeInitialMessage() async {
    takeInitialCalls++;
    final message = initialMessage;
    initialMessage = null; // plugin chỉ trả một lần
    return message;
  }

  @override
  Stream<String> get onRegistered => _registered.stream;

  @override
  Future<PushPermission> permission() async => permissionValue;

  @override
  Future<PushPermission> requestPermission() async {
    calls.add('requestPermission');
    permissionValue = nextPermissionAnswer;
    return permissionValue;
  }

  @override
  Future<void> register() async {
    calls.add('register(listener=${_registered.hasListener})');
    if (failRegister) throw Exception('register lỗi');
    final fid = fidOnRegister;
    if (fid != null) _registered.add(fid);
  }

  @override
  Future<void> unregister() async {
    calls.add('unregister');
    timeline?.add('fcm:unregister');
    if (failUnregister) throw Exception('unregister lỗi');
  }
}

class FakeLocalNotifier implements LocalNotifier {
  FakeLocalNotifier({this.launch});

  String? launch;
  final shown = <({int id, String title, String body, String payload})>[];
  final _taps = StreamController<String>.broadcast();

  void tap(String payload) => _taps.add(payload);

  @override
  Stream<String> get taps => _taps.stream;

  @override
  Future<void> show({
    required int id,
    required String title,
    required String body,
    required String payload,
  }) async {
    shown.add((id: id, title: title, body: body, payload: payload));
  }

  @override
  Future<String?> launchPayload() async => launch;
}

class FakeDeviceApi implements DeviceApi {
  FakeDeviceApi({this.timeline});

  final List<String>? timeline;
  final calls = <String>[];
  final upserts = <({String deviceId, String pushId, String platform})>[];
  Object? failUpsert;

  /// Khi khác null, `upsert` chờ tới khi gate hoàn thành (mô phỏng mạng chậm).
  Completer<void>? gate;
  Object? failRemove;

  @override
  Future<void> upsert({
    required String deviceId,
    required String pushId,
    required String platform,
  }) async {
    calls.add('upsert:$pushId');
    timeline?.add('api:upsert:$pushId');
    await gate?.future;
    if (failUpsert != null) throw failUpsert!;
    upserts.add((deviceId: deviceId, pushId: pushId, platform: platform));
  }

  @override
  Future<void> remove(String deviceId) async {
    calls.add('remove:$deviceId');
    timeline?.add('api:remove:$deviceId');
    if (failRemove != null) throw failRemove!;
  }
}
