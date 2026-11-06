import UIKit

struct BatteryUnavailable: Error {
  let message: String
}

/// Đọc pin qua UIDevice và ProcessInfo. Gọi trên main thread.
final class BatteryReader {
  func read() throws -> BatteryStatusMessage {
    let device = UIDevice.current
    device.isBatteryMonitoringEnabled = true
    // Simulator luôn báo .unknown, nên đây là chỗ PlatformException xuất hiện
    // khi chạy thử trên simulator.
    if device.batteryState == .unknown || device.batteryLevel < 0 {
      throw BatteryUnavailable(message: "Battery level not available.")
    }
    return BatteryStatusMessage(
      levelPercent: Int64((device.batteryLevel * 100).rounded()),
      isCharging: device.batteryState == .charging || device.batteryState == .full,
      isPowerSaveMode: ProcessInfo.processInfo.isLowPowerModeEnabled
    )
  }
}
