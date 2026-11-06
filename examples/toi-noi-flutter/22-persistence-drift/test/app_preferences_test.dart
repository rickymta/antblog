import 'package:flutter/material.dart' show ThemeMode;
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:shared_preferences_platform_interface/in_memory_shared_preferences_async.dart';
import 'package:shared_preferences_platform_interface/shared_preferences_async_platform_interface.dart';
import 'package:toi_noi_22/preferences/app_preferences.dart';

void main() {
  // Cơ chế mock chính thức của SharedPreferencesAsync: thay bản cài đặt nền
  // tảng bằng bản trong bộ nhớ, không gọi plugin.
  void useStore([Map<String, Object> data = const {}]) {
    SharedPreferencesAsyncPlatform.instance =
        InMemorySharedPreferencesAsync.withData(data);
  }

  AppPreferences newPreferences() => AppPreferences(SharedPreferencesAsync());

  test('chưa chọn gì thì theo hệ thống', () async {
    useStore();
    expect(await newPreferences().readThemeMode(), ThemeMode.system);
  });

  test('ghi rồi đọc lại đúng từng chế độ', () async {
    useStore();
    final prefs = newPreferences();
    for (final mode in ThemeMode.values) {
      await prefs.writeThemeMode(mode);
      expect(await prefs.readThemeMode(), mode);
    }
  });

  test('lưu bằng tên enum dưới đúng khoá', () async {
    useStore();
    await newPreferences().writeThemeMode(ThemeMode.dark);

    expect(
      await SharedPreferencesAsync().getString(AppPreferences.themeModeKey),
      'dark',
    );
  });

  test('giá trị lạ trong kho thì quay về hệ thống, không ném lỗi', () async {
    useStore({AppPreferences.themeModeKey: 'solarized'});
    expect(await newPreferences().readThemeMode(), ThemeMode.system);
  });

  test('SharedPreferencesAsync chỉ chạy được khi đã gắn nền tảng', () {
    SharedPreferencesAsyncPlatform.instance = null;
    expect(SharedPreferencesAsync.new, throwsStateError);
  });
}
