import 'dart:io';

import 'metadata.dart';
import 'privacy_manifest.dart';
import 'release_checklist.dart';
import 'screenshot.dart';

/// Đọc gói phát hành của [app] từ thư mục lab [root]:
///
/// - `native-config/<app>/ios/PrivacyInfo.xcprivacy` và `Info.plist`;
/// - `store/<app>/privacy-label.json`;
/// - `store/<app>/metadata/` (bố cục của `fastlane deliver`).
ReleaseInput loadReleaseInput(
  Directory root,
  StoreApp app, {
  bool isFirstVersion = true,
  int deploymentTarget = 15,
}) {
  String read(String path) => File('${root.path}/$path').readAsStringSync();
  return ReleaseInput(
    app: app,
    manifest: PrivacyManifest.parse(
      read('native-config/${app.name}/ios/PrivacyInfo.xcprivacy'),
    ),
    info: InfoPlist.parse(read('native-config/${app.name}/ios/Info.plist')),
    label: PrivacyLabel.parse(read('store/${app.name}/privacy-label.json')),
    metadata: AppMetadata.read(
      Directory('${root.path}/store/${app.name}/metadata'),
    ),
    isFirstVersion: isFirstVersion,
    deploymentTarget: deploymentTarget,
  );
}

/// Đọc mọi `.png` trong [dir]; thư mục không có thì trả danh sách rỗng.
List<PngInfo> loadScreenshots(Directory dir) => [
  if (dir.existsSync())
    for (final f
        in dir.listSync().whereType<File>().toList()
          ..sort((a, b) => a.path.compareTo(b.path)))
      if (f.path.endsWith('.png')) PngInfo.parse(f.readAsBytesSync()),
];
