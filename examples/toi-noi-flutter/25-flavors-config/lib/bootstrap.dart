import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/misc.dart' show Override;

import 'app.dart';
import 'config/app_config.dart';
import 'config_error_app.dart';
import 'providers.dart';

/// Dựng widget gốc: đọc và kiểm tra cấu hình trước, cấu hình hợp lệ thì ghi đè
/// [appConfigProvider] rồi chạy app, sai thì chạy màn hình báo lỗi.
///
/// [loadConfig] và [overrides] chỉ để test thay đầu vào.
Widget buildRoot({
  AppConfig Function() loadConfig = AppConfig.fromEnvironment,
  List<Override> overrides = const [],
}) {
  try {
    final config = loadConfig();
    return ProviderScope(
      overrides: [appConfigProvider.overrideWithValue(config), ...overrides],
      child: const ToiNoiApp(),
    );
  } on ConfigException catch (error) {
    return ConfigErrorApp(problems: error.problems);
  }
}
