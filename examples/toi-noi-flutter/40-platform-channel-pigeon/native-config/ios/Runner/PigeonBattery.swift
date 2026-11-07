import Flutter

/// Cầu nối Pigeon: tên channel, codec và kiểu dữ liệu đến từ `BatteryApi.g.swift`.
final class PigeonBattery: BatteryHostApi {
  private let reader: BatteryReader
  private let flutterApi: BatteryFlutterApi

  init(reader: BatteryReader, messenger: FlutterBinaryMessenger) {
    self.reader = reader
    self.flutterApi = BatteryFlutterApi(binaryMessenger: messenger)
    BatteryHostApiSetup.setUp(binaryMessenger: messenger, api: self)
  }

  func getBatteryStatus() throws -> BatteryStatusMessage {
    do {
      return try reader.read()
    } catch let error as BatteryUnavailable {
      throw PigeonError(code: "UNAVAILABLE", message: error.message, details: nil)
    }
  }

  /// Native gọi Dart. Phải gọi trên main thread.
  func push(_ status: BatteryStatusMessage) {
    flutterApi.onBatteryChanged(status: status) { result in
      if case .failure(let error) = result {
        NSLog("PigeonBattery: Dart từ chối onBatteryChanged: \(error)")
      }
    }
  }
}
