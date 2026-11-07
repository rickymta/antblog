import 'dart:io';

import 'package:toi_noi_52/toi_noi_52.dart';

/// Đường dẫn tính từ gốc lab; `dart test` luôn chạy ở gốc package.
String readLab(String path) => File(path).readAsStringSync();

ProvisioningProfile sample(String name) =>
    ProvisioningProfile.parse(readLab('samples/$name.plist'));

Map<String, Object?> configPlist(String name) =>
    parsePlist(readLab('native-config/ios/$name'));

Map<String, String> xcconfig(String flavor) => parseXcconfig(
  readLab('native-config/ios/Signing-$flavor.xcconfig.example'),
);

/// Team ID giả duy nhất được phép xuất hiện trong lab.
const fakeTeamId = 'ABCDE12345';
