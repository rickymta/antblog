import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

/// Đặt độ sáng của hệ thống (platformBrightness) cho test, tự trả về mặc định
/// khi test kết thúc.
void setPlatformBrightness(WidgetTester tester, Brightness brightness) {
  tester.platformDispatcher.platformBrightnessTestValue = brightness;
  addTearDown(tester.platformDispatcher.clearPlatformBrightnessTestValue);
}

/// Độ sáng của theme đang áp dụng cho phần thân màn hình.
Brightness themeBrightness(WidgetTester tester) {
  return Theme.of(tester.element(find.byType(Scaffold))).brightness;
}
