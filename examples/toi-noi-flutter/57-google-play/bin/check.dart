import 'dart:io';

import 'package:toi_noi_57/data_safety.dart';
import 'package:toi_noi_57/manifest_permissions.dart';
import 'package:toi_noi_57/rollout_plan.dart';

/// In quyền của manifest mẫu, các khai báo Console kéo theo, kết quả đối chiếu
/// Data safety và một lịch rollout minh hoạ. Không gọi Play.
void main(List<String> args) {
  final path = args.isEmpty ? 'native-config/AndroidManifest.xml' : args.first;
  final xml = File(path).readAsStringSync();
  final permissions = readPermissions(xml).toList()..sort();

  stdout.writeln('Quyền trong $path:');
  for (final p in permissions) {
    stdout.writeln('  $p');
  }

  final declarations = consoleDeclarationsFor(permissions.toSet());
  stdout.writeln(
    'Khai báo Console kéo theo: '
    '${declarations.isEmpty ? '(không)' : declarations.map((d) => d.name).join(', ')}',
  );

  final findings = driverDraft.check(xml);
  stdout.writeln(findings.isEmpty ? 'Data safety: khớp.' : findings.join('\n'));

  var plan = RolloutPlan.start(versionCode: 43, previousVersionCode: 42);
  stdout.writeln(
    'Rollout 43 (thay 42), bước: ${plan.policy.steps.join(' > ')}',
  );
  for (final obs in [
    RolloutObservation(users: 120, crashFreeUsersBp: 9990),
    RolloutObservation(users: 800, crashFreeUsersBp: 9960),
    RolloutObservation(users: 4000, crashFreeUsersBp: 9930),
    RolloutObservation(users: 9000, crashFreeUsersBp: 9850),
  ]) {
    final action = plan.decide(obs);
    plan = plan.observe(obs);
    stdout.writeln(
      '  ${obs.users} người, crash-free ${obs.crashFreeUsersBp / 100}% '
      '-> ${action.name}, còn ${plan.percent}% (${plan.status.name})',
    );
  }
}
