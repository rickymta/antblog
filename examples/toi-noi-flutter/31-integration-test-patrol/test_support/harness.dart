import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/misc.dart' show Override;
import 'package:shelf/shelf.dart' show Handler;
import 'package:toi_noi_31/bootstrap.dart';
import 'package:toi_noi_31/providers.dart';
import 'package:toi_noi_fake_api/testing.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'driver_simulator.dart';
import 'fake_notification_permission.dart';

final fakeBaseUrl = Uri.parse('http://fake.test');

/// Server giả chạy trong tiến trình cho `flutter test`: không mạng, không
/// thiết bị. App và [DriverSimulator] dùng chung một adapter nên thấy cùng
/// một kho chuyến.
final class Harness {
  Harness({
    Handler? handler,
    FakeNotificationPermission? permission,
    this.pollInterval = const Duration(milliseconds: 500),
  }) : adapter = InProcessAdapter(handler ?? _defaultHandler()),
       permission = permission ?? FakeNotificationPermission();

  static Handler _defaultHandler() {
    var seq = 0;
    return createFakeApiHandler(
      clock: () => DateTime.utc(2026, 11, 6, 3, 30),
      newId: () => 'T-${++seq}',
    );
  }

  final InProcessAdapter adapter;
  final FakeNotificationPermission permission;
  final Duration pollInterval;

  late final DriverSimulator driver = DriverSimulator(
    baseUrl: fakeBaseUrl,
    adapter: adapter,
  );

  List<Override> get overrides => [
    apiBaseUrlProvider.overrideWithValue(fakeBaseUrl),
    httpAdapterProvider.overrideWithValue(adapter),
    notificationPermissionProvider.overrideWithValue(permission),
    pollIntervalProvider.overrideWithValue(pollInterval),
    idempotencyKeyProvider.overrideWithValue(() => 'test-key-1'),
  ];

  Widget get app => buildApp(overrides: overrides);
}
