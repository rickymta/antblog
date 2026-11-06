import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_16/app.dart';
import 'package:toi_noi_16/sample_trips.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

/// Giờ cố định để test không phụ thuộc đồng hồ thật.
DateTime fixedClock() => DateTime.utc(2026, 11, 6, 8, 30);

/// Đặt cửa sổ test thành [width] x [height] pixel logic (devicePixelRatio = 1)
/// và tự khôi phục khi test kết thúc.
void setScreen(WidgetTester tester, double width, [double height = 900]) {
  tester.view.devicePixelRatio = 1;
  tester.view.physicalSize = Size(width, height);
  addTearDown(tester.view.reset);
}

/// Giả lập người dùng đặt cỡ chữ hệ thống là [scale].
///
/// Giá trị đi qua `PlatformDispatcher` nên tới widget bằng đúng đường của
/// cài đặt thật: `MediaQuery.textScalerOf(context)`.
void setTextScale(WidgetTester tester, double scale) {
  tester.platformDispatcher.textScaleFactorTestValue = scale;
  addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
}

/// Giả lập vùng không an toàn (tai thỏ, home indicator) bằng pixel logic.
///
/// `SafeArea` đọc `MediaQuery.padding`, nhưng phần đáy của `padding` có thể bị
/// bàn phím "ăn" mất khi nó mở; `viewPadding` giữ nguyên và là thứ
/// `maintainBottomViewPadding` dùng. Thiết bị thật báo cả hai nên ta đặt cùng
/// một giá trị.
void setInsets(
  WidgetTester tester, {
  double top = 0,
  double bottom = 0,
  double left = 0,
  double right = 0,
}) {
  final padding = FakeViewPadding(
    top: top,
    bottom: bottom,
    left: left,
    right: right,
  );
  tester.view.padding = padding;
  tester.view.viewPadding = padding;
  addTearDown(tester.view.resetPadding);
  addTearDown(tester.view.resetViewPadding);
}

/// Dựng app đầy đủ với bốn chuyến mẫu.
Future<void> pumpApp(WidgetTester tester) {
  return tester.pumpWidget(
    ToiNoiApp(initialTrips: sampleTrips(), clock: fixedClock),
  );
}

/// Bọc [child] trong `MaterialApp` dùng theme của `toi_noi_ui`.
Future<void> pumpThemed(WidgetTester tester, Widget child) {
  return tester.pumpWidget(
    MaterialApp(theme: ToiNoiTheme.light(), home: child),
  );
}

/// Chạy [body] với nền tảng giả [platform], luôn trả `debugDefaultTargetPlatformOverride`
/// về `null` ở cuối.
///
/// Phải khôi phục ngay trong thân test (try/finally). Đặt vào `addTearDown`
/// thì quá muộn: kiểm tra cuối test chạy trước tear-down và báo "The value of
/// a foundation debug variable was changed by the test".
Future<void> withPlatform(
  TargetPlatform platform,
  Future<void> Function() body,
) async {
  debugDefaultTargetPlatformOverride = platform;
  try {
    await body();
  } finally {
    debugDefaultTargetPlatformOverride = null;
  }
}

/// Giả lập nút/cử chỉ Back của hệ thống Android: engine gửi `popRoute`
/// qua kênh `flutter/navigation`.
Future<void> systemBack(WidgetTester tester) async {
  final message = const JSONMethodCodec().encodeMethodCall(
    const MethodCall('popRoute'),
  );
  await tester.binding.defaultBinaryMessenger.handlePlatformMessage(
    'flutter/navigation',
    message,
    (_) {},
  );
  await tester.pumpAndSettle();
}
