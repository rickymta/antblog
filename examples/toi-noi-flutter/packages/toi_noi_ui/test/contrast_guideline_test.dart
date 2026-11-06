import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'test_helpers.dart';

/// Xám 3,45:1 trên nền trắng: dưới ngưỡng 4,5 của chữ thường, trên ngưỡng 3
/// của chữ lớn.
const _grey = Color(0xFF8A8A8A);

Widget _app(Widget body) => MaterialApp(
  home: Scaffold(backgroundColor: Colors.white, body: body),
);

TextStyle _style(double size, {Color color = _grey}) =>
    TextStyle(color: color, fontSize: size);

/// Hai dòng chữ cách xa nhau để điểm ảnh của dòng này không lẫn vào dòng kia.
/// Dòng hai trắng trên nền trắng, tương phản 1,00:1.
final _good = Text('Dòng một', style: _style(14, color: Colors.black));
final _invisible = Text('Dòng hai', style: _style(14, color: Colors.white));

void main() {
  test('màu xám của test là 3,45:1 trên nền trắng', () {
    expect(contrastRatio(_grey, Colors.white), inInclusiveRange(3.4, 3.5));
  });

  group('ngưỡng phụ thuộc cỡ chữ (guideline đọc TextStyle)', () {
    testWidgets('chữ 14px 3,45:1 bị báo', (tester) async {
      final handle = tester.ensureSemantics();
      await tester.pumpWidget(
        _app(Center(child: Text('Đang tìm tài xế', style: _style(14)))),
      );
      await expectLater(tester, doesNotMeetGuideline(textContrastGuideline));
      handle.dispose();
    });

    testWidgets('cùng màu ở 18px đạt, vì chữ lớn chỉ cần 3:1', (tester) async {
      final handle = tester.ensureSemantics();
      await tester.pumpWidget(
        _app(Center(child: Text('Đang tìm tài xế', style: _style(18)))),
      );
      await expectLater(tester, meetsGuideline(textContrastGuideline));
      handle.dispose();
    });

    testWidgets('chữ trắng trên nền trắng đứng riêng bị báo (1,00:1)', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await tester.pumpWidget(_app(Center(child: _invisible)));
      await expectLater(tester, doesNotMeetGuideline(textContrastGuideline));
      handle.dispose();
    });
  });

  group('chữ không được kiểm khi bị gộp semantics', () {
    Widget item() => Wrap(spacing: 40, children: [_good, _invisible]);

    testWidgets('hai Text trong một item ListView gộp thành một node', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await tester.pumpWidget(_app(ListView(children: [item()])));
      expect(
        tester.getSemantics(find.text('Dòng một')).label,
        'Dòng một\nDòng hai',
      );
      handle.dispose();
    });

    testWidgets('nhãn ghép không khớp find.text nên chữ vô hình lọt qua', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await tester.pumpWidget(_app(ListView(children: [item()])));
      await expectLater(tester, meetsGuideline(textContrastGuideline));
      handle.dispose();
    });

    testWidgets('addSemanticIndexes: false thì mỗi Text là một node, bị báo', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await tester.pumpWidget(
        _app(ListView(addSemanticIndexes: false, children: [item()])),
      );
      expect(tester.getSemantics(find.text('Dòng một')).label, 'Dòng một');
      await expectLater(tester, doesNotMeetGuideline(textContrastGuideline));
      handle.dispose();
    });
  });
}
