import Flutter
import UIKit
import workmanager_apple

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    // Template Flutter 3.47 dùng UIScene: plugin được đăng ký SAU khi hàm này
    // trả về, mà BGTaskScheduler chỉ giao task cho handler đã đăng ký trong
    // didFinishLaunching. Vì vậy phải đăng ký lại tại đây (chú thích của
    // WorkmanagerPlugin.registerLaunchHandlers trong workmanager_apple).
    WorkmanagerPlugin.registerLaunchHandlers()
    // Engine nền của task cũng cần các plugin khác (path_provider...).
    WorkmanagerPlugin.setPluginRegistrantCallback { registry in
      GeneratedPluginRegistrant.register(with: registry)
    }
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }

  func didInitializeImplicitFlutterEngine(_ engineBridge: FlutterImplicitEngineBridge) {
    GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)
  }
}
