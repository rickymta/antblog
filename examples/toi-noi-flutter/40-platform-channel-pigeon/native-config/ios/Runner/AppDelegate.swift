import Flutter
import UIKit

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  // Giữ tham chiếu: bỏ đi thì cầu nối bị giải phóng và channel ngừng trả lời.
  private var methodChannelBattery: MethodChannelBattery?
  private var pigeonBattery: PigeonBattery?

  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }

  func didInitializeImplicitFlutterEngine(_ engineBridge: FlutterImplicitEngineBridge) {
    GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)

    // Với vòng đời UIScene, `window` còn nil ở đây: lấy messenger từ engineBridge.
    let messenger = engineBridge.applicationRegistrar.messenger()
    let reader = BatteryReader()
    // Lab nối cả hai cầu để so sánh; app thật chọn một.
    let channelBridge = MethodChannelBattery(reader: reader, messenger: messenger)
    let pigeonBridge = PigeonBattery(reader: reader, messenger: messenger)
    methodChannelBattery = channelBridge
    pigeonBattery = pigeonBridge

    UIDevice.current.isBatteryMonitoringEnabled = true
    let names: [Notification.Name] = [
      UIDevice.batteryLevelDidChangeNotification,
      UIDevice.batteryStateDidChangeNotification,
      Notification.Name.NSProcessInfoPowerStateDidChange,
    ]
    for name in names {
      // queue: .main để gọi channel về Dart trên main thread.
      NotificationCenter.default.addObserver(forName: name, object: nil, queue: .main) { _ in
        guard let status = try? reader.read() else { return }
        channelBridge.push(status)
        pigeonBridge.push(status)
      }
    }
  }
}
