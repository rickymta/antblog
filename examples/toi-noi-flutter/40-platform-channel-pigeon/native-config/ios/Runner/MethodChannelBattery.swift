import Flutter

/// Cầu nối MethodChannel viết tay: phải khớp `method_channel_battery_gateway.dart`.
final class MethodChannelBattery {
  static let channelName = "vn.toinoi.driver/battery"

  private let channel: FlutterMethodChannel
  private let reader: BatteryReader

  init(reader: BatteryReader, messenger: FlutterBinaryMessenger) {
    self.reader = reader
    self.channel = FlutterMethodChannel(name: Self.channelName, binaryMessenger: messenger)
    // Capture mạnh self: nếu self bị giải phóng, `result` không bao giờ được gọi
    // và Future phía Dart treo mãi. AppDelegate giữ đối tượng này suốt đời app.
    channel.setMethodCallHandler { call, result in
      self.handle(call, result: result)
    }
  }

  private func handle(_ call: FlutterMethodCall, result: FlutterResult) {
    switch call.method {
    case "getBatteryStatus":
      do {
        result(try reader.read().toMap())
      } catch let error as BatteryUnavailable {
        result(FlutterError(code: "UNAVAILABLE", message: error.message, details: nil))
      } catch {
        result(FlutterError(code: "UNKNOWN", message: "\(error)", details: nil))
      }
    default:
      result(FlutterMethodNotImplemented)
    }
  }

  /// Native gọi Dart. Phải gọi trên main thread.
  func push(_ status: BatteryStatusMessage) {
    channel.invokeMethod("onBatteryChanged", arguments: status.toMap())
  }
}

extension BatteryStatusMessage {
  func toMap() -> [String: Any] {
    [
      "levelPercent": levelPercent,
      "isCharging": isCharging,
      "isPowerSaveMode": isPowerSaveMode,
    ]
  }
}
