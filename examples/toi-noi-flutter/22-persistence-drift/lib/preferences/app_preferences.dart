import 'package:flutter/material.dart' show ThemeMode;
import 'package:shared_preferences/shared_preferences.dart';

/// Tuỳ chọn nhỏ của người dùng, lưu bằng `SharedPreferencesAsync`.
///
/// Chỉ để dữ liệu mất đi cũng không sao: nếu khoá thiếu hoặc giá trị lạ, app
/// quay về mặc định thay vì lỗi.
class AppPreferences {
  AppPreferences(this._prefs);

  final SharedPreferencesAsync _prefs;

  static const themeModeKey = 'settings.theme_mode';

  /// Chế độ sáng/tối đã chọn; `ThemeMode.system` khi chưa chọn hoặc giá trị lạ.
  Future<ThemeMode> readThemeMode() async {
    final name = await _prefs.getString(themeModeKey);
    return ThemeMode.values.asNameMap()[name] ?? ThemeMode.system;
  }

  Future<void> writeThemeMode(ThemeMode mode) =>
      _prefs.setString(themeModeKey, mode.name);
}
