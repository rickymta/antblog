import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_28/providers.dart';
import 'package:toi_noi_28/quote.dart';
import 'package:toi_noi_28/quote_repository.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

const _loading = Key('quote-loading');
const _latency = Duration(milliseconds: 400);

Future<void> _pumpWithLatency(WidgetTester tester) => pumpQuoteScreen(
  tester,
  overrides: [
    quoteRepositoryProvider.overrideWithValue(
      FakeQuoteRepository(latency: _latency),
    ),
  ],
);

void main() {
  group('pump, pump(Duration), pumpAndSettle', () {
    testWidgets('pumpWidget dựng khung đầu: báo giá đang tải', (tester) async {
      await _pumpWithLatency(tester);

      expect(find.byKey(_loading), findsOneWidget);
      expect(find.text('25.500 VND'), findsNothing);

      // Timer 400 ms của repository giả còn treo: test mà kết thúc lúc này
      // sẽ bị framework báo "A Timer is still pending". Tua qua hạn.
      await tester.pump(_latency);
    });

    testWidgets('pump() dựng khung mới nhưng không cho thời gian trôi', (
      tester,
    ) async {
      await _pumpWithLatency(tester);

      await tester.pump();
      await tester.pump();

      expect(find.byKey(_loading), findsOneWidget);
      await tester.pump(_latency);
    });

    testWidgets('pump(Duration) tua đồng hồ giả: 399 ms chưa, 400 ms rồi', (
      tester,
    ) async {
      await _pumpWithLatency(tester);

      await tester.pump(const Duration(milliseconds: 399));
      expect(find.byKey(_loading), findsOneWidget);

      await tester.pump(const Duration(milliseconds: 1));
      expect(find.byKey(_loading), findsNothing);
      expect(find.byKey(const Key('quote-fare')), findsOneWidget);
    });

    testWidgets('pumpAndSettle tua từng 100 ms cho tới khi không còn khung', (
      tester,
    ) async {
      await _pumpWithLatency(tester);

      final pumps = await tester.pumpAndSettle();

      // 4 bước x 100 ms tới 400 ms thì báo giá về; con số chính xác còn
      // tuỳ hoạt ảnh chuyển trạng thái, nên chỉ kiểm cận dưới.
      expect(pumps, greaterThanOrEqualTo(4));
      expect(find.byKey(const Key('quote-fare')), findsOneWidget);
    });

    testWidgets('pumpAndSettle hết hạn khi vòng quay chạy mãi', (tester) async {
      // Báo giá không bao giờ về: CircularProgressIndicator xin khung mới
      // liên tục, `hasScheduledFrame` luôn true.
      final repository = ControlledQuoteRepository();
      await pumpQuoteScreen(
        tester,
        overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
      );

      Object? error;
      try {
        await tester.pumpAndSettle(
          const Duration(milliseconds: 100),
          EnginePhase.sendSemanticsUpdate,
          const Duration(seconds: 2),
        );
      } on FlutterError catch (e) {
        error = e;
      }

      expect(
        error,
        isA<FlutterError>().having(
          (e) => e.message,
          'message',
          'pumpAndSettle timed out',
        ),
      );
    });

    testWidgets('state loading ghi đè bằng giá trị cũng không bao giờ yên', (
      tester,
    ) async {
      await pumpQuoteScreen(
        tester,
        overrides: [quoteProvider.overrideWithValue(const AsyncLoading())],
      );

      Object? error;
      try {
        await tester.pumpAndSettle(
          const Duration(milliseconds: 100),
          EnginePhase.sendSemanticsUpdate,
          const Duration(seconds: 1),
        );
      } on FlutterError catch (e) {
        error = e;
      }

      expect(error, isA<FlutterError>());
    });

    testWidgets('pump(Duration) đi qua vòng quay mà không hết hạn', (
      tester,
    ) async {
      final repository = ControlledQuoteRepository();
      await pumpQuoteScreen(
        tester,
        overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
      );

      await tester.pump(const Duration(seconds: 5));

      expect(find.byKey(_loading), findsOneWidget);
    });

    testWidgets('không có hoạt ảnh thì pumpAndSettle trả về sau một khung', (
      tester,
    ) async {
      // State đã có dữ liệu, không có vòng quay: không còn khung nào được xin.
      final quote = Quote(
        serviceKind: ServiceKind.ride,
        distanceKm: 5,
        fare: calculateFare(5),
      );
      await pumpQuoteScreen(
        tester,
        overrides: [quoteProvider.overrideWithValue(AsyncData(quote))],
      );

      expect(await tester.pumpAndSettle(), 1);
    });

    testWidgets('complete một Completer rồi pump(): có thể thiếu một nhịp', (
      tester,
    ) async {
      // `pump()` chỉ dựng khung nếu lúc bắt đầu đã có khung được xin. Lúc đã
      // yên, Future xong chạy microtask (gọi setState) SAU bước kiểm đó.
      final done = Completer<void>();
      await tester.pumpWidget(
        MaterialApp(
          home: FutureBuilder<void>(
            future: done.future,
            builder: (context, snapshot) => Text(
              snapshot.connectionState == ConnectionState.done
                  ? 'xong'
                  : 'đang chờ',
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();
      done.complete();
      await tester.pump();
      expect(find.text('đang chờ'), findsOneWidget);

      await tester.pump();
      expect(find.text('xong'), findsOneWidget);
    });

    testWidgets('pump(Duration.zero) chạy microtask trước khi dựng khung', (
      tester,
    ) async {
      final done = Completer<void>();
      await tester.pumpWidget(
        MaterialApp(
          home: FutureBuilder<void>(
            future: done.future,
            builder: (context, snapshot) => Text(
              snapshot.connectionState == ConnectionState.done
                  ? 'xong'
                  : 'đang chờ',
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();
      done.complete();
      await tester.pump(Duration.zero);

      expect(find.text('xong'), findsOneWidget);
    });

    testWidgets('khi vòng quay còn chạy, một pump() là đủ', (tester) async {
      // Vòng quay luôn xin khung mới nên khung đã sẵn sàng được dựng.
      final repository = ControlledQuoteRepository();
      await pumpQuoteScreen(
        tester,
        overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
      );

      repository.completeQuote();
      await tester.pump();

      expect(find.byKey(const Key('quote-fare')), findsOneWidget);
    });
  });
}
