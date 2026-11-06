import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/misc.dart' show Override;
import 'package:shelf/shelf.dart';
import 'package:toi_noi_25/config/app_config.dart';
import 'package:toi_noi_25/providers.dart';
import 'package:toi_noi_fake_api/testing.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

/// Một "máy chủ" trong tiến trình cho test: fake API (hoặc handler tự viết),
/// kèm các dòng log HTTP đã ghi.
final class Harness {
  Harness([Handler? handler])
    : adapter = InProcessAdapter(handler ?? createFakeApiHandler()),
      logs = [];

  final InProcessAdapter adapter;
  final List<String> logs;

  List<Override> overridesFor(AppConfig config) => [
    appConfigProvider.overrideWithValue(config),
    httpAdapterProvider.overrideWithValue(adapter),
    logSinkProvider.overrideWithValue(logs.add),
  ];

  ProviderContainer containerFor(AppConfig config) =>
      ProviderContainer.test(overrides: overridesFor(config));
}
