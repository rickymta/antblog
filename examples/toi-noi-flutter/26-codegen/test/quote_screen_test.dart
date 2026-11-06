import 'package:dio/dio.dart' hide Response;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shelf/shelf.dart' show Handler, Response;
import 'package:toi_noi_26/providers.dart';
import 'package:toi_noi_26/quote_screen.dart';
import 'package:toi_noi_26/quote_source.dart';
import 'package:toi_noi_api/toi_noi_api.dart'
    show ToiNoiApiClient, TimeoutFailure, TimeoutPhase;
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/testing.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support/fake_source.dart';

Future<void> _pump(WidgetTester tester, QuoteSource source) =>
    tester.pumpWidget(
      ProviderScope(
        overrides: [quoteSourceProvider.overrideWithValue(source)],
        child: const MaterialApp(home: QuoteScreen()),
      ),
    );

Dio _fakeApiDio(Handler handler) => ToiNoiApiClient(
  baseUrl: Uri.parse('http://fake.test'),
  adapter: InProcessAdapter(handler),
).dio;

void main() {
  testWidgets('idle, đang tải rồi hiện giá 25.500 VND', (tester) async {
    final source = FakeQuoteSource();
    await _pump(tester, source);
    expect(find.text('Chọn dịch vụ rồi bấm báo giá.'), findsOneWidget);

    await tester.tap(find.text('Báo giá 5 km'));
    await tester.pump();
    expect(find.byType(LinearProgressIndicator), findsOneWidget);
    // Đang tải: nút bị khoá.
    expect(
      tester.widget<FilledButton>(find.byType(FilledButton)).onPressed,
      isNull,
    );

    source.answerQuote(0, calculateFare(5));
    await tester.pump();
    expect(find.text('25.500 VND'), findsOneWidget);
    expect(find.byType(LinearProgressIndicator), findsNothing);
  });

  testWidgets('đổi dịch vụ rồi báo giá: lời gọi mang đúng dịch vụ', (
    tester,
  ) async {
    final source = FakeQuoteSource();
    await _pump(tester, source);

    await tester.tap(find.text(ServiceKind.food.label));
    await tester.pump();
    await tester.tap(find.text('Báo giá 5 km'));
    await tester.pump();

    expect(source.quoteCalls, hasLength(1));
  });

  testWidgets('lỗi quá chậm hiện thông báo; bấm lại thì hỏi lại', (
    tester,
  ) async {
    final source = FakeQuoteSource();
    await _pump(tester, source);

    await tester.tap(find.text('Báo giá 5 km'));
    await tester.pump();
    source.quoteCalls[0].completeError(
      const TimeoutFailure(phase: TimeoutPhase.receive),
    );
    await tester.pump();
    expect(
      find.text('Máy chủ phản hồi quá chậm. Hãy thử lại.'),
      findsOneWidget,
    );

    await tester.tap(find.text('Báo giá 5 km'));
    await tester.pump();
    expect(find.byType(LinearProgressIndicator), findsOneWidget);
    expect(source.quoteCalls, hasLength(2));
  });

  testWidgets('đi qua fake API thật bằng DioQuoteSource: 25.500 VND', (
    tester,
  ) async {
    final source = DioQuoteSource(_fakeApiDio(createFakeApiHandler()));
    await _pump(tester, source);

    await tester.tap(find.text('Báo giá 5 km'));
    await tester.pumpAndSettle();

    expect(find.text('25.500 VND'), findsOneWidget);
  });

  testWidgets(
    'server trả cước lẻ: người dùng thấy lỗi dữ liệu, không thấy giá sai',
    (tester) async {
      final source = DioQuoteSource(
        _fakeApiDio(
          (request) => Response.ok(
            '{"serviceKind":"ride","distanceMeters":5000,"fare":25500.5,"currency":"VND"}',
            headers: {'content-type': 'application/json'},
          ),
        ),
      );
      await _pump(tester, source);

      await tester.tap(find.text('Báo giá 5 km'));
      await tester.pumpAndSettle();

      expect(find.text('Dữ liệu từ máy chủ không đọc được.'), findsOneWidget);
      expect(find.textContaining('VND'), findsNothing);
    },
  );
}
