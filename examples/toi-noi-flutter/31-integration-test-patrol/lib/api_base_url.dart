import 'package:flutter/foundation.dart';

/// Địa chỉ API truyền lúc build: `--dart-define=API_BASE_URL=http://...`.
const _definedBaseUrl = String.fromEnvironment('API_BASE_URL');

/// Android emulator thấy máy dev qua địa chỉ này, không phải `localhost`
/// (`localhost` trong emulator là chính emulator).
const androidEmulatorHost = '10.0.2.2';

/// Cổng mặc định của `tools/fake_api`.
const fakeApiPort = 8787;

/// Chọn địa chỉ API.
///
/// Có `API_BASE_URL` thì dùng nguyên giá trị đó (thiết bị thật phải làm vậy,
/// với địa chỉ LAN của máy dev). Không có thì đoán theo nền tảng: Android là
/// emulator, các nền tảng còn lại (iOS Simulator) gọi `localhost`.
Uri resolveApiBaseUrl({
  String defined = _definedBaseUrl,
  TargetPlatform? platform,
}) {
  if (defined.isNotEmpty) {
    return Uri.parse(defined);
  }
  final host = (platform ?? defaultTargetPlatform) == TargetPlatform.android
      ? androidEmulatorHost
      : 'localhost';
  return Uri(scheme: 'http', host: host, port: fakeApiPort);
}
