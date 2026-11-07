import 'dart:io';

import 'package:toi_noi_56/toi_noi_56.dart';

/// Đường dẫn tính từ thư mục lab, nơi `dart test` chạy.
String readText(String path) => File(path).readAsStringSync();

Map<String, Object?> readPlist(String path) => parsePlist(readText(path));

const customerPlist = 'native-config/customer/ios/Info.plist';
const driverPlist = 'native-config/driver/ios/Info.plist';
const entitlementsFile = 'native-config/ios/Runner.entitlements.example';

/// Biến build mà `flutter build ipa --build-name 1.4.0 --build-number 43` cấp.
const settings143 = {
  'FLUTTER_BUILD_NAME': '1.4.0',
  'FLUTTER_BUILD_NUMBER': '43',
};

/// Bản sao có thể sửa của một plist mẫu (sao chép sâu mức một tầng là đủ).
Map<String, Object?> editable(String path) =>
    Map<String, Object?>.of(readPlist(path));

BetaChecklist customerChecklist({
  String? last,
  Map<String, String> settings = settings143,
}) => BetaChecklist(
  role: AppRole.customer,
  lastUploadedBuild: last,
  buildSettings: settings,
);

BetaChecklist driverChecklist({
  String? last,
  Map<String, String> settings = settings143,
}) => BetaChecklist(
  role: AppRole.driver,
  lastUploadedBuild: last,
  buildSettings: settings,
);
