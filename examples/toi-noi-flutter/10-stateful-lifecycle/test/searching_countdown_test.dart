import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_10/searching_countdown.dart';

Widget _app(Widget child) => MaterialApp(home: Scaffold(body: child));

Finder _remaining(int seconds) =>
    find.text('Đang tìm tài xế, còn $seconds giây');

/// Phiên bản SAI cố ý: quên huỷ Timer ở dispose. Chỉ dùng để chứng minh lỗi.
class _LeakyCountdown extends StatefulWidget {
  const _LeakyCountdown();

  @override
  State<_LeakyCountdown> createState() => _LeakyCountdownState();
}

class _LeakyCountdownState extends State<_LeakyCountdown> {
  int _remaining = 5;
  late final Timer timer;

  @override
  void initState() {
    super.initState();
    timer = Timer.periodic(const Duration(seconds: 1), (_) => tick());
  }

  /// Việc timer sẽ làm mỗi giây. Public để test gọi trực tiếp thay vì để
  /// lỗi trôi ra ngoài test zone.
  void tick() => setState(() => _remaining--);

  @override
  Widget build(BuildContext context) => Text('còn $_remaining');
}

/// Phiên bản SAI cố ý: setState sau await mà không kiểm tra mounted.
class _NoMountedCheck extends StatefulWidget {
  const _NoMountedCheck(this.work);

  final Future<void> work;

  @override
  State<_NoMountedCheck> createState() => _NoMountedCheckState();
}

class _NoMountedCheckState extends State<_NoMountedCheck> {
  var _done = false;

  Future<void> run() async {
    await widget.work;
    setState(() => _done = true);
  }

  @override
  Widget build(BuildContext context) => Text(_done ? 'xong' : 'chạy');
}

void main() {
  group('đếm ngược', () {
    testWidgets('bắt đầu từ seconds và trừ một mỗi giây', (tester) async {
      await tester.pumpWidget(_app(const SearchingCountdown(seconds: 30)));
      expect(_remaining(30), findsOneWidget);

      await tester.pump(const Duration(seconds: 1));
      expect(_remaining(29), findsOneWidget);

      await tester.pump(const Duration(seconds: 9));
      expect(_remaining(20), findsOneWidget);
    });

    testWidgets('chưa đủ một giây thì chưa trừ', (tester) async {
      await tester.pumpWidget(_app(const SearchingCountdown(seconds: 30)));

      await tester.pump(const Duration(milliseconds: 999));
      expect(_remaining(30), findsOneWidget);

      await tester.pump(const Duration(milliseconds: 1));
      expect(_remaining(29), findsOneWidget);
    });

    testWidgets('thanh tiến độ tỉ lệ với số giây còn lại', (tester) async {
      await tester.pumpWidget(_app(const SearchingCountdown(seconds: 30)));
      double value() => tester
          .widget<LinearProgressIndicator>(find.byType(LinearProgressIndicator))
          .value!;
      expect(value(), 1.0);

      await tester.pump(const Duration(seconds: 15));
      expect(value(), 0.5);
    });

    testWidgets('về 0 thì gọi onTimeout đúng một lần và dừng hẳn', (
      tester,
    ) async {
      var timeouts = 0;
      await tester.pumpWidget(
        _app(SearchingCountdown(seconds: 3, onTimeout: () => timeouts++)),
      );

      await tester.pump(const Duration(seconds: 2));
      expect(timeouts, 0);

      await tester.pump(const Duration(seconds: 1));
      expect(timeouts, 1);
      expect(find.text('Chưa tìm được tài xế'), findsOneWidget);

      // Timer đã tự huỷ: đi tiếp một phút vẫn không có gì xảy ra thêm.
      await tester.pump(const Duration(minutes: 1));
      expect(timeouts, 1);
      expect(find.text('Chưa tìm được tài xế'), findsOneWidget);
      expect(tester.takeException(), isNull);
    });

    testWidgets('seconds không dương bị assert chặn', (tester) async {
      expect(() => SearchingCountdown(seconds: 0), throwsAssertionError);
    });
  });

  group('didUpdateWidget', () {
    testWidgets('đổi seconds thì đếm lại từ giá trị mới, chỉ còn một timer', (
      tester,
    ) async {
      await tester.pumpWidget(_app(const SearchingCountdown(seconds: 30)));
      await tester.pump(const Duration(seconds: 10));
      expect(_remaining(20), findsOneWidget);

      await tester.pumpWidget(_app(const SearchingCountdown(seconds: 60)));
      expect(_remaining(60), findsOneWidget);

      // Nếu timer cũ chưa bị huỷ, một giây trôi qua sẽ trừ hai lần (58).
      await tester.pump(const Duration(seconds: 1));
      expect(_remaining(59), findsOneWidget);
      await tester.pump(const Duration(seconds: 5));
      expect(_remaining(54), findsOneWidget);
    });

    testWidgets('widget mới cùng seconds thì không đếm lại', (tester) async {
      await tester.pumpWidget(_app(const SearchingCountdown(seconds: 30)));
      await tester.pump(const Duration(seconds: 10));

      await tester.pumpWidget(
        _app(SearchingCountdown(seconds: 30, onTimeout: () {})),
      );

      expect(_remaining(20), findsOneWidget);
    });

    testWidgets('onTimeout mới được dùng ngay, không cần đếm lại', (
      tester,
    ) async {
      final calls = <String>[];
      await tester.pumpWidget(
        _app(SearchingCountdown(seconds: 3, onTimeout: () => calls.add('cũ'))),
      );
      await tester.pump(const Duration(seconds: 1));

      await tester.pumpWidget(
        _app(SearchingCountdown(seconds: 3, onTimeout: () => calls.add('mới'))),
      );
      await tester.pump(const Duration(seconds: 2));

      expect(calls, ['mới']);
    });

    testWidgets('đã hết giờ rồi đổi seconds thì chạy lại', (tester) async {
      var timeouts = 0;
      await tester.pumpWidget(
        _app(SearchingCountdown(seconds: 2, onTimeout: () => timeouts++)),
      );
      await tester.pump(const Duration(seconds: 2));
      expect(timeouts, 1);

      await tester.pumpWidget(
        _app(SearchingCountdown(seconds: 4, onTimeout: () => timeouts++)),
      );
      expect(_remaining(4), findsOneWidget);

      await tester.pump(const Duration(seconds: 4));
      expect(timeouts, 2);
    });
  });

  group('dispose', () {
    testWidgets('gỡ widget khỏi cây thì Timer bị huỷ, không gọi gì nữa', (
      tester,
    ) async {
      var timeouts = 0;
      await tester.pumpWidget(
        _app(SearchingCountdown(seconds: 5, onTimeout: () => timeouts++)),
      );
      await tester.pump(const Duration(seconds: 2));

      await tester.pumpWidget(_app(const SizedBox()));
      await tester.pump(const Duration(minutes: 1));

      expect(timeouts, 0);
      expect(tester.takeException(), isNull);
      // Nếu dispose quên _timer?.cancel(), flutter_test sẽ làm test này đỏ
      // với "A Timer is still pending even after the widget tree was disposed".
    });

    testWidgets('bản quên cancel: Timer sống sau dispose và setState ném lỗi', (
      tester,
    ) async {
      await tester.pumpWidget(_app(const _LeakyCountdown()));
      final state = tester.state<_LeakyCountdownState>(
        find.byType(_LeakyCountdown),
      );

      await tester.pumpWidget(_app(const SizedBox()));

      expect(state.mounted, isFalse);
      expect(state.timer.isActive, isTrue);
      expect(
        state.tick,
        throwsA(
          isA<FlutterError>().having(
            (e) => e.toString(),
            'message',
            contains('setState() called after dispose()'),
          ),
        ),
      );

      // Dọn timer bị rò, nếu không flutter_test báo pending timer.
      state.timer.cancel();
    });
  });

  group('huỷ tìm và mounted sau await', () {
    testWidgets('onCancel null thì nút Huỷ tìm bị khoá', (tester) async {
      await tester.pumpWidget(_app(const SearchingCountdown()));

      final button = tester.widget<TextButton>(find.byType(TextButton));
      expect(button.onPressed, isNull);
    });

    testWidgets('huỷ xong khi còn mounted: dừng đếm và đổi thông báo', (
      tester,
    ) async {
      final completer = Completer<void>();
      var calls = 0;
      await tester.pumpWidget(
        _app(
          SearchingCountdown(
            seconds: 10,
            onCancel: () {
              calls++;
              return completer.future;
            },
          ),
        ),
      );

      await tester.tap(find.text('Huỷ tìm'));
      await tester.pump();
      // Đang chờ: nút bị khoá, bấm thêm không gọi onCancel lần nữa.
      await tester.tap(find.text('Huỷ tìm'), warnIfMissed: false);
      expect(calls, 1);

      completer.complete();
      await tester.pump();
      expect(find.text('Đã huỷ tìm tài xế'), findsOneWidget);

      await tester.pump(const Duration(seconds: 5));
      expect(find.text('Đã huỷ tìm tài xế'), findsOneWidget);
    });

    testWidgets('widget bị gỡ trong lúc await: mounted chặn setState', (
      tester,
    ) async {
      final completer = Completer<void>();
      await tester.pumpWidget(
        _app(SearchingCountdown(onCancel: () => completer.future)),
      );
      await tester.tap(find.text('Huỷ tìm'));
      await tester.pump();

      await tester.pumpWidget(_app(const SizedBox()));
      completer.complete();
      await tester.pump();

      expect(tester.takeException(), isNull);
    });

    testWidgets('bản thiếu mounted: setState sau await ném FlutterError', (
      tester,
    ) async {
      final completer = Completer<void>();
      await tester.pumpWidget(_app(_NoMountedCheck(completer.future)));
      final state = tester.state<_NoMountedCheckState>(
        find.byType(_NoMountedCheck),
      );

      final outcome = expectLater(
        state.run(),
        throwsA(
          isA<FlutterError>().having(
            (e) => e.toString(),
            'message',
            contains('setState() called after dispose()'),
          ),
        ),
      );
      await tester.pumpWidget(_app(const SizedBox()));
      completer.complete();
      await tester.pump();

      await outcome;
    });

    testWidgets('onCancel lỗi thì báo người dùng và mở lại nút', (
      tester,
    ) async {
      final completer = Completer<void>();
      await tester.pumpWidget(
        _app(SearchingCountdown(onCancel: () => completer.future)),
      );
      await tester.tap(find.text('Huỷ tìm'));
      await tester.pump();

      completer.completeError(StateError('mất mạng'));
      await tester.pump();

      expect(find.text('Chưa huỷ được, hãy thử lại'), findsOneWidget);
      final button = tester.widget<TextButton>(find.byType(TextButton));
      expect(button.onPressed, isNotNull);
      expect(find.text('Đã huỷ tìm tài xế'), findsNothing);
    });

    testWidgets('thông báo lỗi huỷ biến mất khi đếm tiếp', (tester) async {
      final completer = Completer<void>();
      await tester.pumpWidget(
        _app(SearchingCountdown(onCancel: () => completer.future)),
      );
      await tester.tap(find.text('Huỷ tìm'));
      await tester.pump();
      completer.completeError(StateError('mất mạng'));
      await tester.pump();
      expect(find.text('Chưa huỷ được, hãy thử lại'), findsOneWidget);

      await tester.pump(const Duration(seconds: 1));

      expect(find.text('Chưa huỷ được, hãy thử lại'), findsNothing);
      expect(_remaining(29), findsOneWidget);
    });

    testWidgets('đổi seconds xoá thông báo lỗi huỷ', (tester) async {
      final completer = Completer<void>();
      Widget build(int seconds) => _app(
        SearchingCountdown(seconds: seconds, onCancel: () => completer.future),
      );
      await tester.pumpWidget(build(30));
      await tester.tap(find.text('Huỷ tìm'));
      await tester.pump();
      completer.completeError(StateError('mất mạng'));
      await tester.pump();

      await tester.pumpWidget(build(60));

      expect(find.text('Chưa huỷ được, hãy thử lại'), findsNothing);
      expect(_remaining(60), findsOneWidget);
    });
  });
}
