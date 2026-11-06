import 'dart:io';

import 'package:toi_noi_53/release_plan.dart';

/// In ra các lệnh fastlane cho một lần phát hành, không chạy gì cả.
///
/// `dart run bin/plan.dart staging customer 42`
void main(List<String> args) {
  if (args.length != 3) {
    stderr.writeln(
      'Cách dùng: dart run bin/plan.dart <dev|staging|prod> '
      '<customer|driver> <build_number>',
    );
    exitCode = 64;
    return;
  }
  final flavor = Flavor.values.asNameMap()[args[0]];
  final app = AppKind.values.asNameMap()[args[1]];
  final buildNumber = int.tryParse(args[2]);
  if (flavor == null || app == null || buildNumber == null) {
    stderr.writeln('Tham số không hợp lệ: ${args.join(' ')}');
    exitCode = 64;
    return;
  }
  final List<LaneInvocation> plan;
  try {
    plan = planRelease(flavor: flavor, app: app, buildNumber: buildNumber);
  } on ArgumentError catch (e) {
    stderr.writeln(e.message);
    exitCode = 64;
    return;
  }
  if (plan.isEmpty) {
    stdout.writeln(
      '${flavor.name}: không phân phối qua cửa hàng, '
      'chạy bằng flutter run.',
    );
    return;
  }
  stdout.writeln('${applicationId(app, flavor)}:');
  plan.forEach(stdout.writeln);
}
