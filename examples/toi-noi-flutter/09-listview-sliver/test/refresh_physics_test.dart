import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support.dart';

/// Danh sách hai dòng, ngắn hơn màn hình, bọc trong RefreshIndicator.
/// Trả về số lần `onRefresh` được gọi sau một cú kéo xuống.
Future<int> _pullDown(
  WidgetTester tester, {
  ScrollController? controller,
  ScrollPhysics? physics,
}) async {
  var refreshes = 0;
  await tester.pumpWidget(
    app(
      RefreshIndicator(
        onRefresh: () async => refreshes++,
        child: CustomScrollView(
          controller: controller,
          physics: physics,
          slivers: [
            SliverList.builder(
              itemCount: 2,
              itemBuilder: (context, index) =>
                  SizedBox(height: 72, child: Text('Dòng $index')),
            ),
          ],
        ),
      ),
    ),
  );
  await tester.fling(find.byType(CustomScrollView), const Offset(0, 300), 1000);
  await tester.pump();
  await tester.pump(const Duration(seconds: 1));
  await tester.pumpAndSettle();
  return refreshes;
}

void main() {
  final platforms = TargetPlatformVariant({
    TargetPlatform.android,
    TargetPlatform.iOS,
  });

  group('danh sách ngắn hơn màn hình', () {
    testWidgets('không controller: ScrollView dọc tự dùng '
        'AlwaysScrollableScrollPhysics nên kéo làm mới được', (tester) async {
      expect(await _pullDown(tester), 1);
    }, variant: platforms);

    testWidgets('có controller mà không đặt physics: dùng physics mặc định của '
        'nền tảng và từ chối thao tác kéo', (tester) async {
      final controller = ScrollController();
      addTearDown(controller.dispose);

      expect(await _pullDown(tester, controller: controller), 0);
    }, variant: platforms);

    testWidgets('có controller và AlwaysScrollableScrollPhysics: kéo làm mới '
        'được', (tester) async {
      final controller = ScrollController();
      addTearDown(controller.dispose);

      expect(
        await _pullDown(
          tester,
          controller: controller,
          physics: const AlwaysScrollableScrollPhysics(),
        ),
        1,
      );
    }, variant: platforms);
  });
}
