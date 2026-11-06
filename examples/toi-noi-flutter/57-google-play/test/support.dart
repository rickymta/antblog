import 'dart:io';

import 'package:toi_noi_57/manifest_permissions.dart';
import 'package:toi_noi_57/track_promotion.dart';

/// Manifest mẫu của app tài xế. Test chạy từ thư mục gốc của lab.
String readSampleManifest() =>
    File('native-config/AndroidManifest.xml').readAsStringSync();

final DeveloperAccount oldPersonal = DeveloperAccount(
  kind: AccountKind.personal,
  createdAt: DateTime.utc(2022, 5, 1),
);

final DeveloperAccount newPersonal = DeveloperAccount(
  kind: AccountKind.personal,
  createdAt: DateTime.utc(2025, 3, 1),
);

/// Đầu vào mà mọi điều kiện đều đạt, với bộ quyền của manifest mẫu. Test sửa
/// đúng một trường để kiểm một điều chặn.
PreflightInput readyInput({
  DeveloperAccount? account,
  int targetSdk = minTargetSdk,
  Set<String>? permissions,
  bool dataSafetyComplete = true,
  bool foregroundServiceDeclared = true,
  bool backgroundLocationDeclared = false,
  ClosedTestProgress? closedTest = const ClosedTestProgress(
    testersOptedIn: requiredClosedTesters,
    consecutiveDays: requiredClosedTestDays,
  ),
}) => PreflightInput(
  account: account ?? newPersonal,
  targetSdk: targetSdk,
  manifestPermissions: permissions ?? readPermissions(readSampleManifest()),
  dataSafetyComplete: dataSafetyComplete,
  foregroundServiceDeclared: foregroundServiceDeclared,
  backgroundLocationDeclared: backgroundLocationDeclared,
  closedTest: closedTest,
);
