import 'dart:async';

import 'package:flutter/services.dart';

import 'battery_gateway.dart';
import 'battery_status.dart';

/// Tên channel và method: phải khớp từng ký tự với code Kotlin và Swift.
const String batteryChannelName = 'vn.toinoi.driver/battery';
const String getBatteryStatusMethod = 'getBatteryStatus';
const String onBatteryChangedMethod = 'onBatteryChanged';

/// Hai chiều trên một [MethodChannel]: Dart gọi `getBatteryStatus`, native gọi
/// `onBatteryChanged` khi pin đổi.
final class MethodChannelBatteryGateway implements BatteryGateway {
  MethodChannelBatteryGateway({BinaryMessenger? binaryMessenger})
    : _channel = MethodChannel(
        batteryChannelName,
        const StandardMethodCodec(),
        binaryMessenger,
      ) {
    _channel.setMethodCallHandler(_onNativeCall);
  }

  final MethodChannel _channel;
  final StreamController<BatteryStatus> _changes =
      StreamController<BatteryStatus>.broadcast();

  @override
  Stream<BatteryStatus> get changes => _changes.stream;

  @override
  Future<BatteryStatus> read() async {
    try {
      // invokeMapMethod ép kiểu bên trong framework và ném TypeError khi
      // native trả String, List hoặc int; nhận Object? rồi tự kiểm.
      final reply = await _channel.invokeMethod<Object?>(
        getBatteryStatusMethod,
      );
      if (reply == null) {
        throw const BatteryUnavailableException('null-reply');
      }
      if (reply is! Map<Object?, Object?>) {
        throw BatteryUnavailableException(
          'bad-payload',
          'Cần Map, nhận ${reply.runtimeType}',
        );
      }
      return BatteryStatus.fromMap(reply);
    } on PlatformException catch (e) {
      throw BatteryUnavailableException(e.code, e.message);
    } on MissingPluginException catch (e) {
      throw BatteryUnavailableException('missing-plugin', e.message);
    } on FormatException catch (e) {
      throw BatteryUnavailableException('bad-payload', e.message);
    }
  }

  Future<Object?> _onNativeCall(MethodCall call) async {
    switch (call.method) {
      case onBatteryChangedMethod:
        final arguments = call.arguments;
        if (arguments is! Map<Object?, Object?>) {
          throw PlatformException(
            code: 'bad-payload',
            message: 'onBatteryChanged cần một Map',
          );
        }
        try {
          _changes.add(BatteryStatus.fromMap(arguments));
        } on FormatException catch (e) {
          throw PlatformException(code: 'bad-payload', message: e.message);
        }
        return null;
      default:
        // Native gọi method Dart chưa có: trả "chưa cài đặt" thay vì lỗi.
        throw MissingPluginException('Dart không có method ${call.method}');
    }
  }

  @override
  void dispose() {
    _channel.setMethodCallHandler(null);
    unawaited(_changes.close());
  }
}
