import 'dart:io' show Platform;
import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

/// Hệ điều hành đã sinh ảnh chuẩn trong `test/goldens/`.
///
/// Ảnh chuẩn chỉ đáng tin trên hệ điều hành này; chỗ khác test golden bị bỏ
/// qua kèm lý do (xem [goldenSkip]).
const goldenBaselineOs = 'macos';

/// Lý do bỏ qua test golden, hoặc `false` để chạy.
///
/// Khi chạy `flutter test --update-goldens` thì luôn chạy: người đó đang chủ
/// động sinh lại ảnh chuẩn (nếu sinh trên hệ điều hành khác thì đổi
/// [goldenBaselineOs] cùng commit).
Object get goldenSkip =>
    autoUpdateGoldenFiles || Platform.operatingSystem == goldenBaselineOs
    ? false
    : 'Ảnh chuẩn sinh trên $goldenBaselineOs, máy này là '
          '${Platform.operatingSystem}. Chạy golden ở job $goldenBaselineOs.';

/// Nạp font icon thật của Material để golden vẽ đúng icon.
///
/// Mặc định `flutter test` không nạp `MaterialIcons`: icon hiện thành ô vuông
/// rỗng, không phân biệt được icon nào. Font nằm sẵn trong asset của
/// lab (do `uses-material-design: true`), ta chỉ đọc và đăng ký, không commit
/// file font nào. Chữ thường vẫn dùng font kiểm thử mặc định của `flutter test`.
Future<void> loadMaterialIcons() async {
  final loader = FontLoader('MaterialIcons')
    ..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'));
  await loader.load();
}

/// Như [goldenSkip], nhưng còn bỏ qua khi đang `--update-goldens`.
///
/// Dành cho test tự dựng ảnh chuẩn tạm rồi cố tình làm golden trượt: ở chế độ
/// cập nhật, `matchesGoldenFile` luôn "qua" bằng cách ghi đè, nên test sẽ sai.
Object get goldenCompareSkip => autoUpdateGoldenFiles
    ? 'Đang chạy --update-goldens: matchesGoldenFile ghi đè thay vì so sánh.'
    : goldenSkip;

/// Khoá của vùng được chụp.
const goldenKey = Key('golden-area');

/// Bề rộng cố định của vùng chụp, pixel logic.
const goldenWidth = 390.0;

/// Đặt cửa sổ test thành [width] x [height] pixel logic, devicePixelRatio 1,
/// và tự khôi phục khi test kết thúc.
void setGoldenView(
  WidgetTester tester, {
  double width = goldenWidth,
  double height = 844,
}) {
  tester.view.devicePixelRatio = 1;
  tester.view.physicalSize = Size(width, height);
  addTearDown(tester.view.reset);
}

/// Dựng [child] trong khung chụp cố định [goldenWidth] với theme sáng hoặc tối.
///
/// Theme chọn bằng `themeMode`, không bằng cài đặt hệ thống, nên kết quả
/// không đổi theo máy chạy test.
Future<void> pumpGolden(
  WidgetTester tester,
  Widget child, {
  Brightness brightness = Brightness.light,
  double textScale = 1,
  double viewHeight = 844,
}) async {
  setGoldenView(tester, height: viewHeight);
  tester.platformDispatcher.textScaleFactorTestValue = textScale;
  addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);

  await tester.pumpWidget(
    MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: ToiNoiTheme.light(),
      darkTheme: ToiNoiTheme.dark(),
      themeMode: brightness == Brightness.dark
          ? ThemeMode.dark
          : ThemeMode.light,
      home: Scaffold(
        body: Align(
          alignment: Alignment.topLeft,
          child: RepaintBoundary(
            key: goldenKey,
            child: SizedBox(
              width: goldenWidth,
              child: Builder(
                builder: (context) => ColoredBox(
                  color: Theme.of(context).colorScheme.surface,
                  child: Padding(
                    padding: const EdgeInsets.all(16),
                    child: child,
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    ),
  );
}

/// Chụp vùng [goldenKey] thành byte PNG, đúng thứ `matchesGoldenFile` so.
Future<Uint8List> capturePng(WidgetTester tester) async {
  final element = tester.element(find.byKey(goldenKey));
  final bytes = await tester.runAsync(() async {
    final image = await captureImage(element);
    try {
      final data = await image.toByteData(format: ui.ImageByteFormat.png);
      return data!.buffer.asUint8List();
    } finally {
      image.dispose();
    }
  });
  return bytes!;
}
