import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_10/searching_countdown.dart';

Finder _remaining(int seconds) =>
    find.text('Đang tìm tài xế, còn $seconds giây');

/// Đổi loại widget cha của SearchingCountdown giữa `Padding` và `Center`.
/// Hai loại khác nhau nên Element cũ không dùng lại được, trừ khi có
/// GlobalKey.
class _Reparent extends StatefulWidget {
  const _Reparent({required this.countdownKey});

  final Key? countdownKey;

  @override
  State<_Reparent> createState() => _ReparentState();
}

class _ReparentState extends State<_Reparent> {
  var _padded = true;

  @override
  Widget build(BuildContext context) {
    final countdown = SearchingCountdown(key: widget.countdownKey, seconds: 30);
    return Column(
      children: [
        TextButton(
          onPressed: () => setState(() => _padded = !_padded),
          child: const Text('đổi cha'),
        ),
        if (_padded)
          Padding(padding: const EdgeInsets.all(8), child: countdown)
        else
          Center(child: countdown),
      ],
    );
  }
}

Widget _app(Widget child) => MaterialApp(home: Scaffold(body: child));

void main() {
  group('key quyết định State có được giữ hay không', () {
    testWidgets('không key: đổi loại cha thì State mới, đếm lại từ đầu', (
      tester,
    ) async {
      await tester.pumpWidget(_app(const _Reparent(countdownKey: null)));
      await tester.pump(const Duration(seconds: 10));
      expect(_remaining(20), findsOneWidget);

      await tester.tap(find.text('đổi cha'));
      await tester.pump();

      expect(_remaining(30), findsOneWidget);
    });

    testWidgets('GlobalKey: chuyển sang cha khác vẫn giữ State đang đếm', (
      tester,
    ) async {
      final key = GlobalKey();
      await tester.pumpWidget(_app(_Reparent(countdownKey: key)));
      await tester.pump(const Duration(seconds: 10));
      expect(_remaining(20), findsOneWidget);

      await tester.tap(find.text('đổi cha'));
      await tester.pump();

      expect(_remaining(20), findsOneWidget);
      await tester.pump(const Duration(seconds: 1));
      expect(_remaining(19), findsOneWidget);
    });

    testWidgets('GlobalKey.currentContext trỏ tới Element đang sống', (
      tester,
    ) async {
      final key = GlobalKey();
      await tester.pumpWidget(_app(SearchingCountdown(key: key)));
      expect(key.currentContext, isNotNull);
      expect(key.currentWidget, isA<SearchingCountdown>());

      await tester.pumpWidget(_app(const SizedBox()));
      expect(key.currentContext, isNull);
    });

    testWidgets('đổi ValueKey thì State bị huỷ và tạo lại, đếm lại từ đầu', (
      tester,
    ) async {
      await tester.pumpWidget(
        _app(const SearchingCountdown(key: ValueKey('lần 1'), seconds: 30)),
      );
      await tester.pump(const Duration(seconds: 10));
      expect(_remaining(20), findsOneWidget);

      await tester.pumpWidget(
        _app(const SearchingCountdown(key: ValueKey('lần 2'), seconds: 30)),
      );

      // Cùng seconds nên didUpdateWidget sẽ không reset; chỉ key mới làm được.
      expect(_remaining(30), findsOneWidget);
      await tester.pump(const Duration(seconds: 1));
      expect(_remaining(29), findsOneWidget);
    });
  });
}
