import Flutter
import UIKit
import UserNotifications
import firebase_messaging

// Dựa trên AppDelegate.swift của template Flutter 3.47.4 (UIScene); hai dòng
// thêm được đánh dấu "PUSH". Chưa build bằng Xcode.
@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    // PUSH 1: README của flutter_local_notifications yêu cầu đặt delegate này.
    UNUserNotificationCenter.current().delegate = self
    // PUSH 2: README của firebase_messaging: với UIScene, Apple đòi delegate
    // phải được cấu hình trước khi hàm này trả về. Gọi SAU dòng trên để
    // plugin giữ delegate đã đặt và chuyển tiếp cho nó.
    FLTFirebaseMessagingPlugin.configureNotificationCenterDelegate()
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }

  func didInitializeImplicitFlutterEngine(_ engineBridge: FlutterImplicitEngineBridge) {
    GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)
  }
}
