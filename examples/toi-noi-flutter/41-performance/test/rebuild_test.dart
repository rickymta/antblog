import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_41/build_log.dart';
import 'package:toi_noi_41/rebuild_screens.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

Widget _app(Widget child) => MaterialApp(
  theme: ToiNoiTheme.light(),
  home: Scaffold(body: child),
);

void main() {
  final fare = calculateFare(5); // 25.500 VND, tính từ công thức
  late ValueNotifier<int> ticker;

  setUp(() {
    BuildLog.reset();
    ticker = ValueNotifier<int>(30);
    addTearDown(ticker.dispose);
  });

  /// Đẩy [ticks] nhịp đếm ngược, mỗi nhịp một khung hình.
  Future<void> tick(WidgetTester tester, int ticks) async {
    for (var i = 0; i < ticks; i++) {
      ticker.value -= 1;
      await tester.pump();
    }
  }

  test('cước minh hoạ đúng công thức: 5 km là 25.500 VND', () {
    expect(fare, const Money(25500));
  });

  group('NaiveTripScreen: setState ở gốc', () {
    testWidgets('lần dựng đầu build mỗi widget đúng một lần', (tester) async {
      await tester.pumpWidget(
        _app(NaiveTripScreen(ticker: ticker, fare: fare)),
      );

      expect(BuildLog.of('screen'), 1);
      expect(BuildLog.of('quote-card'), 1);
      expect(BuildLog.of('const-footer'), 1);
      expect(BuildLog.of('plain-footer'), 1);
    });

    testWidgets('5 nhịp: thẻ báo giá và footer thường build lại 5 lần', (
      tester,
    ) async {
      await tester.pumpWidget(
        _app(NaiveTripScreen(ticker: ticker, fare: fare)),
      );
      BuildLog.reset();

      await tick(tester, 5);

      expect(BuildLog.of('screen'), 5);
      // Cước không đổi nhưng widget được tạo mới mỗi lần nên bị gọi lại build.
      expect(BuildLog.of('quote-card'), 5);
      expect(BuildLog.of('plain-footer'), 5);
    });

    testWidgets('widget const không bao giờ build lại', (tester) async {
      await tester.pumpWidget(
        _app(NaiveTripScreen(ticker: ticker, fare: fare)),
      );
      BuildLog.reset();

      await tick(tester, 5);

      expect(BuildLog.of('const-footer'), 0);
    });
  });

  group('SplitTripScreen: tách widget', () {
    testWidgets('5 nhịp: chỉ EtaText build lại', (tester) async {
      await tester.pumpWidget(
        _app(SplitTripScreen(ticker: ticker, fare: fare)),
      );
      BuildLog.reset();

      await tick(tester, 5);

      expect(BuildLog.of('eta'), 5);
      expect(BuildLog.of('screen'), 0);
      expect(BuildLog.of('quote-card'), 0);
    });

    testWidgets('nội dung vẫn cập nhật đúng', (tester) async {
      await tester.pumpWidget(
        _app(SplitTripScreen(ticker: ticker, fare: fare)),
      );

      await tick(tester, 3);

      expect(find.text('Tài xế tới sau 27 giây'), findsOneWidget);
    });
  });

  group('ChildParamTripScreen: tham số child của builder', () {
    testWidgets('builder chạy mỗi nhịp, child dựng sẵn thì không', (
      tester,
    ) async {
      await tester.pumpWidget(
        _app(ChildParamTripScreen(ticker: ticker, fare: fare)),
      );
      BuildLog.reset();

      await tick(tester, 5);

      expect(BuildLog.of('builder'), 5);
      expect(BuildLog.of('quote-card'), 0);
      expect(find.text('Còn 25 giây'), findsOneWidget);
    });
  });

  group('khi cha dựng lại với dữ liệu mới thì thẻ vẫn phải cập nhật', () {
    testWidgets('đổi cước làm QuoteCard build lại và hiện số mới', (
      tester,
    ) async {
      await tester.pumpWidget(
        _app(SplitTripScreen(ticker: ticker, fare: fare)),
      );
      expect(find.text('25.500 VND'), findsOneWidget);
      BuildLog.reset();

      await tester.pumpWidget(
        _app(SplitTripScreen(ticker: ticker, fare: calculateFare(100))),
      );

      expect(BuildLog.of('quote-card'), 1);
      expect(find.text('453.000 VND'), findsOneWidget);
    });
  });

  group('const chỉ có tác dụng khi cả đối số cũng const', () {
    testWidgets('const QuoteCard: cùng instance, không build lại', (
      tester,
    ) async {
      final host = ValueNotifier<int>(0);
      addTearDown(host.dispose);
      await tester.pumpWidget(
        _app(
          ValueListenableBuilder<int>(
            valueListenable: host,
            builder: (context, v, _) => Column(
              children: [
                Text('$v'),
                const QuoteCard(fare: Money(25500)),
              ],
            ),
          ),
        ),
      );
      BuildLog.reset();

      host.value = 1;
      await tester.pump();

      expect(BuildLog.of('quote-card'), 0);
    });

    testWidgets('QuoteCard(fare: fare) cùng giá trị nhưng là instance mới', (
      tester,
    ) async {
      final host = ValueNotifier<int>(0);
      addTearDown(host.dispose);
      await tester.pumpWidget(
        _app(
          ValueListenableBuilder<int>(
            valueListenable: host,
            // ignore: prefer_const_constructors
            builder: (context, v, _) => Column(
              children: [
                Text('$v'),
                QuoteCard(fare: fare),
              ],
            ),
          ),
        ),
      );
      BuildLog.reset();

      host.value = 1;
      await tester.pump();

      expect(BuildLog.of('quote-card'), 1);
    });
  });

  group('debugOnRebuildDirtyWidget', () {
    testWidgets('ghi đúng kiểu widget bị rebuild', (tester) async {
      await tester.pumpWidget(
        _app(NaiveTripScreen(ticker: ticker, fare: fare)),
      );
      final recorder = RebuildRecorder.start();
      addTearDown(recorder.stop);

      await tick(tester, 2);

      expect(
        recorder.count('_NaiveTripScreenState'),
        0,
      ); // State không phải widget
      expect(recorder.count('NaiveTripScreen'), 2);
      expect(recorder.count('QuoteCard'), 2);
      expect(recorder.count('BuildProbe'), 2); // chỉ plain-footer, const không
      expect(recorder.count('MoneyText'), 2);
    });

    testWidgets('bản tách: chỉ ValueListenableBuilder bị rebuild', (
      tester,
    ) async {
      await tester.pumpWidget(
        _app(SplitTripScreen(ticker: ticker, fare: fare)),
      );
      final recorder = RebuildRecorder.start();
      addTearDown(recorder.stop);

      await tick(tester, 2);

      expect(recorder.count('QuoteCard'), 0);
      expect(recorder.count('SplitTripScreen'), 0);
      expect(recorder.count('ValueListenableBuilder<int>'), 2);
    });

    testWidgets('lần dựng đầu cũng được ghi; stop gỡ callback', (tester) async {
      final recorder = RebuildRecorder.start();
      await tester.pumpWidget(
        _app(SplitTripScreen(ticker: ticker, fare: fare)),
      );

      expect(recorder.count('QuoteCard'), 1);

      recorder.stop();
      expect(debugOnRebuildDirtyWidget, isNull);
    });
  });

  test('debugProfileBuildsEnabled mặc định tắt và bật/tắt được', () {
    expect(debugProfileBuildsEnabled, isFalse);
    debugProfileBuildsEnabled = true;
    addTearDown(() => debugProfileBuildsEnabled = false);
    expect(debugProfileBuildsEnabled, isTrue);
  });

  testWidgets('bật debugProfileBuildsEnabled không đổi số lần build', (
    tester,
  ) async {
    // Phải trả cờ về false ngay trong thân test: flutter_test kiểm cờ này ở
    // cuối thân test, trước khi addTearDown chạy.
    debugProfileBuildsEnabled = true;
    try {
      await tester.pumpWidget(
        _app(SplitTripScreen(ticker: ticker, fare: fare)),
      );
      BuildLog.reset();

      await tick(tester, 3);

      expect(BuildLog.of('eta'), 3);
      expect(BuildLog.of('quote-card'), 0);
    } finally {
      debugProfileBuildsEnabled = false;
    }
  });
}
