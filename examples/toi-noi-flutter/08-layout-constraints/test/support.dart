import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

/// Đặt cửa sổ test thành [width] x [height] pixel logic (devicePixelRatio = 1)
/// và tự khôi phục khi test kết thúc.
void setScreen(WidgetTester tester, double width, [double height = 900]) {
  tester.view.devicePixelRatio = 1;
  tester.view.physicalSize = Size(width, height);
  addTearDown(tester.view.reset);
}

/// Bọc [child] trong `MaterialApp` + `Scaffold` có cuộn dọc và lề 16, giống
/// màn hình thật. [textScale] mô phỏng người dùng chỉnh cỡ chữ lớn.
Widget app(Widget child, {double textScale = 1}) {
  return MaterialApp(
    theme: ThemeData(
      colorScheme: ColorScheme.fromSeed(seedColor: Colors.teal),
      useMaterial3: true,
    ),
    builder: (context, home) => MediaQuery(
      data: MediaQuery.of(context)
          .copyWith(textScaler: TextScaler.linear(textScale)),
      child: home!,
    ),
    home: Scaffold(
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: child,
      ),
    ),
  );
}

/// Dựng [widget] và gom mọi lỗi framework báo ra, theo thứ tự.
///
/// Dùng khi một lỗi layout kéo theo nhiều lỗi phụ (ví dụ viewport không có
/// chiều cao rồi render object con báo thêm): `takeException` chỉ trả về một
/// đối tượng gói "Multiple exceptions", còn danh sách này cho phép kiểm lỗi
/// gốc, là lỗi đầu tiên.
Future<List<FlutterErrorDetails>> pumpCollectingErrors(
  WidgetTester tester,
  Widget widget,
) async {
  final errors = <FlutterErrorDetails>[];
  final original = FlutterError.onError;
  FlutterError.onError = errors.add;
  try {
    await tester.pumpWidget(widget);
  } finally {
    FlutterError.onError = original;
  }
  return errors;
}
