import 'package:app_settings/app_settings.dart';
import 'package:flutter/foundation.dart';

/// Ranh giới với plugin mở màn hình cài đặt của hệ điều hành.
abstract interface class SystemSettings {
  /// Nhãn nút, nói đúng nơi nút dẫn tới trên nền tảng hiện tại.
  String get buttonLabel;

  /// Mở màn hình cài đặt mà nền tảng cho phép mở.
  Future<void> openNetworkSettings();
}

/// Cài đặt thật dựa trên `app_settings`.
///
/// - Android: `AppSettingsType.wireless` mở cài đặt không dây (tài liệu của
///   plugin ghi loại này chỉ hỗ trợ Android).
/// - iOS: plugin không có trang mạng. Đọc `AppSettingsPlugin.swift` của
///   `app_settings` 9.0.1, loại không biết đều rơi vào
///   `UIApplication.openSettingsURLString`, tức trang cài đặt **của chính app**,
///   không phải cài đặt chung và không có Wi-Fi. Người dùng iOS vẫn phải tự ra
///   ngoài để bật Wi-Fi, nên nhãn nút nói đúng điều đó. Chưa chạy trên thiết bị.
final class AppSettingsOpener implements SystemSettings {
  const AppSettingsOpener({this.platform});

  /// Nền tảng giả định; `null` là nền tảng đang chạy.
  final TargetPlatform? platform;

  bool get _isAndroid =>
      (platform ?? defaultTargetPlatform) == TargetPlatform.android;

  @override
  String get buttonLabel =>
      _isAndroid ? 'Mở cài đặt mạng' : 'Mở cài đặt ứng dụng';

  @override
  Future<void> openNetworkSettings() =>
      AppSettings.openAppSettings(type: AppSettingsType.wireless);
}
