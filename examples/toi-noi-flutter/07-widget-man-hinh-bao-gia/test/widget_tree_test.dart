import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_07/quote_screen.dart';
import 'package:toi_noi_07/toi_noi_app.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Số lần `build` của [_Probe], đếm bằng biến toàn cục để dùng được với
/// widget `const`. Mỗi test tự đặt lại về 0.
int _probeBuilds = 0;

class _Probe extends StatelessWidget {
  const _Probe();

  @override
  Widget build(BuildContext context) {
    _probeBuilds++;
    return const Directionality(
      textDirection: TextDirection.ltr,
      child: Text('probe'),
    );
  }
}

/// Widget cố ý hỏng: tính cước 101 km ngay trong `build`.
class _NaiveQuote extends StatelessWidget {
  const _NaiveQuote();

  @override
  Widget build(BuildContext context) {
    return Directionality(
      textDirection: TextDirection.ltr,
      child: Text(calculateFare(101).toString()),
    );
  }
}

void main() {
  group('ba cây: widget, element, render object', () {
    testWidgets('QuoteScreen là StatelessWidget, element là StatelessElement', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp());

      final element = tester.element(find.byType(QuoteScreen));
      expect(element, isA<StatelessElement>());
      expect(element.widget, isA<StatelessWidget>());
      // StatelessElement không có render object riêng; getter đi xuống tới
      // render object đầu tiên bên dưới.
      expect(element is RenderObjectElement, isFalse);
    });

    testWidgets('Padding: widget cấu hình, RenderPadding làm việc layout', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp());
      final finder = find.byKey(const ValueKey('quote-body'));

      final element = tester.element(finder);
      expect(element, isA<SingleChildRenderObjectElement>());

      final renderObject = tester.renderObject<RenderPadding>(finder);
      expect(renderObject.padding, const EdgeInsets.all(16));
      // Render object giữ kích thước đã layout; widget thì không có size.
      expect(renderObject.size.width, greaterThan(0));
      expect(identical(element.renderObject, renderObject), isTrue);
    });

    testWidgets('Text là StatelessWidget; chữ được vẽ bởi RenderParagraph', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp());
      final text = find.text('Quãng đường: 5 km');

      expect(tester.element(text), isA<StatelessElement>());
      // Text dựng ra RichText, và RichText mới có render object.
      expect(
        find.descendant(of: text, matching: find.byType(RichText)),
        findsOneWidget,
      );
      expect(tester.renderObject(text), isA<RenderParagraph>());
    });
  });

  group('dựng lại: widget mới, element và render object sống tiếp', () {
    testWidgets('cùng kiểu và cùng key thì Element được giữ lại', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp(distanceKm: 5));
      final text = find.text('25.500 VND');
      final elementBefore = tester.element(text);
      final renderBefore = tester.renderObject(text);

      await tester.pumpWidget(const ToiNoiApp(distanceKm: 10));

      expect(find.text('25.500 VND'), findsNothing);
      final after = find.text('48.000 VND');
      expect(after, findsOneWidget);
      // Widget cũ bị bỏ, nhưng element và render object vẫn là đối tượng cũ.
      expect(identical(tester.element(after), elementBefore), isTrue);
      expect(identical(tester.renderObject(after), renderBefore), isTrue);
    });

    testWidgets('khác kiểu thì Element cũ bị bỏ, dựng element mới', (
      tester,
    ) async {
      await tester.pumpWidget(
        const Directionality(
          textDirection: TextDirection.ltr,
          child: Text('a'),
        ),
      );
      final before = tester.element(find.text('a'));

      // Con của Directionality đổi từ Text sang Padding: runtimeType khác nên
      // canUpdate là false và Text mới được dựng thành element mới.
      await tester.pumpWidget(
        const Directionality(
          textDirection: TextDirection.ltr,
          child: Padding(padding: EdgeInsets.zero, child: Text('a')),
        ),
      );
      final after = tester.element(find.text('a'));

      expect(identical(before, after), isFalse);
    });

    test('Widget.canUpdate so runtimeType và key', () {
      const a = Text('a');
      const b = Text('b');
      expect(Widget.canUpdate(a, b), isTrue); // cùng kiểu, cùng key null
      expect(
        Widget.canUpdate(const Text('a', key: ValueKey(1)), const Text('a')),
        isFalse,
      );
      expect(Widget.canUpdate(a, const Icon(Icons.add)), isFalse);
    });
  });

  group('mô phỏng hot reload bằng reassembleApplication', () {
    setUp(() => _probeBuilds = 0);

    testWidgets('widget const giống hệt thì không build lại', (tester) async {
      const probe = _Probe();
      await tester.pumpWidget(probe);
      expect(_probeBuilds, 1);

      await tester.pumpWidget(probe);
      expect(_probeBuilds, 1); // cùng instance: framework bỏ qua
    });

    testWidgets('reassembleApplication ép build lại cả widget const', (
      tester,
    ) async {
      await tester.pumpWidget(const _Probe());
      final elementBefore = tester.element(find.byType(_Probe));
      expect(_probeBuilds, 1);

      // Giống bước 3 của hot reload: dựng lại mọi widget đang có. Future chỉ
      // hoàn tất sau một frame nên phải pump trước khi await.
      final reassemble = tester.binding.reassembleApplication();
      await tester.pump();
      await reassemble;

      expect(_probeBuilds, 2);
      // Hot reload dựng lại nhưng không tạo element mới.
      expect(
        identical(tester.element(find.byType(_Probe)), elementBefore),
        isTrue,
      );
    });
  });

  group('build không được ném lỗi', () {
    testWidgets('tính cước 101 km trong build ném ArgumentError', (
      tester,
    ) async {
      await tester.pumpWidget(const _NaiveQuote());

      expect(tester.takeException(), isA<ArgumentError>());
      // Framework thay widget hỏng bằng ErrorWidget (màn hình đỏ ở debug).
      expect(find.byType(ErrorWidget), findsOneWidget);
    });
  });
}
