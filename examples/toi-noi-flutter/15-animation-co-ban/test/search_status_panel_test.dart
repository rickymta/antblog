import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_15/motion.dart';
import 'package:toi_noi_15/search_status_panel.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

Widget _panel({bool reduceMotion = false}) => harness(
  Scaffold(
    body: Host<TripStatus>(
      initial: TripStatus.searching,
      builder: (context, status) => SearchStatusPanel(status: status),
    ),
  ),
  reduceMotion: reduceMotion,
);

/// Độ mờ của nội dung chứa [text] bên trong AnimatedSwitcher.
double _opacityOf(WidgetTester tester, String text) {
  final fade = find.descendant(
    of: find.byType(AnimatedSwitcher),
    matching: find.ancestor(
      of: find.text(text),
      matching: find.byType(FadeTransition),
    ),
  );
  return tester.widget<FadeTransition>(fade).opacity.value;
}

void main() {
  final searching = messageOf(TripStatus.searching);
  final accepted = messageOf(TripStatus.accepted);
  const half = Duration(milliseconds: 150);

  group('SearchStatusPanel', () {
    testWidgets('trước khi đổi: chỉ có nội dung của searching, đã hiện rõ', (
      tester,
    ) async {
      await tester.pumpWidget(_panel());
      expect(find.text(searching), findsOneWidget);
      expect(find.text(accepted), findsNothing);
      expect(_opacityOf(tester, searching), 1);
    });

    testWidgets('giữa chừng: nội dung cũ và mới cùng có, mờ dở dang', (
      tester,
    ) async {
      await tester.pumpWidget(_panel());
      Host.of<TripStatus>(tester).value = TripStatus.accepted;
      await tester.pump(); // khung bắt đầu chuyển
      await tester.pump(half);

      expect(find.text(searching), findsOneWidget);
      expect(find.text(accepted), findsOneWidget);

      final outgoing = _opacityOf(tester, searching);
      final incoming = _opacityOf(tester, accepted);
      expect(outgoing, inExclusiveRange(0, 1));
      expect(incoming, inExclusiveRange(0, 1));
      // Giữa chặng của 300 ms: cái cũ còn mờ dần, cái mới đang hiện dần.
      expect(outgoing + incoming, closeTo(1, 0.5));
    });

    testWidgets('sau 300 ms: nội dung mới đã rõ, nội dung cũ được gỡ khi '
        'animation báo hoàn tất', (tester) async {
      await tester.pumpWidget(_panel());
      Host.of<TripStatus>(tester).value = TripStatus.accepted;
      await tester.pump();
      await tester.pump(ToiNoiMotion.standard);

      // Đúng 300 ms: giá trị đã tới đích nhưng controller chỉ báo hoàn tất
      // khi thời gian trôi quá duration, nên nội dung cũ chưa bị gỡ.
      expect(_opacityOf(tester, accepted), 1);
      expect(find.text(searching), findsOneWidget);

      await tester.pump(const Duration(milliseconds: 1));
      expect(find.text(searching), findsNothing);
      expect(find.text(accepted), findsOneWidget);
    });

    testWidgets('bật giảm chuyển động: đổi ngay trong khung đầu, không có '
        'giai đoạn chuyển', (tester) async {
      await tester.pumpWidget(_panel(reduceMotion: true));
      Host.of<TripStatus>(tester).value = TripStatus.accepted;
      await tester.pump();

      expect(find.text(searching), findsNothing);
      expect(find.text(accepted), findsOneWidget);
      expect(_opacityOf(tester, accepted), 1);
    });

    testWidgets('tám trạng thái đều có câu mô tả riêng', (tester) async {
      final messages = {for (final s in TripStatus.values) messageOf(s)};
      expect(messages, hasLength(TripStatus.values.length));
      expect(messages, everyElement(isNotEmpty));
    });
  });

  group('AnimatedSwitcher thiếu key', () {
    Widget switcher(Widget Function(String value) child) => harness(
      Scaffold(
        body: Host<String>(
          initial: 'a',
          builder: (context, value) => AnimatedSwitcher(
            duration: ToiNoiMotion.standard,
            child: child(value),
          ),
        ),
      ),
    );

    testWidgets(
      'cùng kiểu widget, không key: chữ đổi ngay, không chuyển cảnh',
      (tester) async {
        await tester.pumpWidget(switcher((value) => Text(value)));
        Host.of<String>(tester).value = 'b';
        await tester.pump();
        await tester.pump(half);

        expect(find.text('a'), findsNothing);
        expect(find.text('b'), findsOneWidget);
      },
    );

    testWidgets('có ValueKey khác nhau: hai nội dung cùng có giữa chừng', (
      tester,
    ) async {
      await tester.pumpWidget(
        switcher((value) => Text(value, key: ValueKey(value))),
      );
      Host.of<String>(tester).value = 'b';
      await tester.pump();
      await tester.pump(half);

      expect(find.text('a'), findsOneWidget);
      expect(find.text('b'), findsOneWidget);
    });
  });
}
