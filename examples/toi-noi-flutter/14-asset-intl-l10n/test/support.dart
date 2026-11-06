import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_14/app.dart';

/// Thời điểm đặt chuyến dùng trong test: 16:30 UTC ngày 6/11/2026.
final requestedAt = DateTime.utc(2026, 11, 6, 16, 30);

/// Độ lệch của Việt Nam so với UTC.
const vietnam = Duration(hours: 7);

/// Đặt danh sách ngôn ngữ ưu tiên của "thiết bị" cho test, tự trả về mặc định
/// khi test kết thúc.
void setDeviceLocales(WidgetTester tester, List<Locale> locales) {
  tester.platformDispatcher.localesTestValue = locales;
  addTearDown(tester.platformDispatcher.clearLocalesTestValue);
}

/// Dựng app song ngữ với giờ cố định ở UTC+7, không phụ thuộc máy chạy test.
Future<void> pumpApp(
  WidgetTester tester, {
  double distanceKm = 5,
  Locale? locale,
}) {
  return tester.pumpWidget(
    ToiNoiL10nApp(
      distanceKm: distanceKm,
      requestedAt: requestedAt,
      utcOffset: vietnam,
      initialLocale: locale,
    ),
  );
}
