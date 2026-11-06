import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_28/providers.dart';
import 'package:toi_noi_28/quote.dart';
import 'package:toi_noi_28/quote_repository.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

const _loading = Key('quote-loading');
const _error = Key('quote-error');
const _card = Key('quote-card');

void main() {
  final quote = Quote(
    serviceKind: ServiceKind.ride,
    distanceKm: 5,
    fare: calculateFare(5),
  );

  group('ghi đè quoteProvider bằng giá trị: kiểm từng state', () {
    testWidgets('loading: vòng quay, không có cước, không có nút đặt', (
      tester,
    ) async {
      await pumpQuoteScreen(
        tester,
        overrides: [quoteProvider.overrideWithValue(const AsyncLoading())],
      );

      expect(find.byKey(_loading), findsOneWidget);
      expect(find.byKey(_card), findsNothing);
      expect(find.byKey(const Key('place-trip')), findsNothing);
    });

    testWidgets('error: thông báo, nút Thử lại, không có cước', (tester) async {
      await pumpQuoteScreen(
        tester,
        overrides: [
          quoteProvider.overrideWithValue(
            AsyncError(const QuoteUnavailable(), StackTrace.empty),
          ),
        ],
      );

      expect(find.byKey(_error), findsOneWidget);
      expect(find.text('Dịch vụ báo giá đang bận'), findsOneWidget);
      expect(find.widgetWithText(OutlinedButton, 'Thử lại'), findsOneWidget);
      expect(find.byKey(_loading), findsNothing);
      expect(find.byKey(_card), findsNothing);
    });

    testWidgets('data: cước 25.500 VND và bảng chi tiết cộng đúng', (
      tester,
    ) async {
      await pumpQuoteScreen(
        tester,
        overrides: [quoteProvider.overrideWithValue(AsyncData(quote))],
      );

      expect(find.byKey(_card), findsOneWidget);
      expect(find.text('Đặt xe'), findsWidgets);
      final breakdown = find.byKey(const Key('fare-breakdown'));
      // 12.000 (mở cửa) + 13.500 (3 km vượt x 4.500) = 25.500.
      expect(
        find.descendant(of: breakdown, matching: find.text('12.000 VND')),
        findsOneWidget,
      );
      expect(
        find.descendant(of: breakdown, matching: find.text('13.500 VND')),
        findsOneWidget,
      );
      expect(find.byKey(const Key('breakdown-total')), findsOneWidget);
      expect(find.byKey(_loading), findsNothing);
      expect(find.byKey(_error), findsNothing);
    });

    testWidgets('data dưới 2 km: phần vượt là 0 VND', (tester) async {
      final short = Quote(
        serviceKind: ServiceKind.parcel,
        distanceKm: 1,
        fare: calculateFare(1),
      );
      await pumpQuoteScreen(
        tester,
        overrides: [quoteProvider.overrideWithValue(AsyncData(short))],
      );

      final breakdown = find.byKey(const Key('fare-breakdown'));
      expect(
        find.descendant(of: breakdown, matching: find.text('0 VND')),
        findsOneWidget,
      );
      expect(
        find.descendant(of: breakdown, matching: find.text('12.000 VND')),
        findsNWidgets(2),
      );
    });
  });

  group('ghi đè repository: chạy qua provider thật', () {
    testWidgets('loading, rồi lỗi, rồi Thử lại, rồi dữ liệu', (tester) async {
      final repository = ControlledQuoteRepository();
      await pumpQuoteScreen(
        tester,
        overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
      );
      expect(find.byKey(_loading), findsOneWidget);

      repository.failQuote();
      await tester.pump();
      expect(find.byKey(_error), findsOneWidget);
      expect(find.byKey(_loading), findsNothing);

      await tester.tap(find.byKey(const Key('retry')));
      await tester.pump();
      expect(find.byKey(_loading), findsOneWidget);
      expect(repository.fetchCount, 2);

      repository.completeQuote();
      await tester.pump();
      expect(find.byKey(_card), findsOneWidget);
      expect(find.byKey(_error), findsNothing);
    });

    testWidgets('lỗi không bị Riverpod tự thử lại', (tester) async {
      final repository = ControlledQuoteRepository();
      await pumpQuoteScreen(
        tester,
        overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
      );

      repository.failQuote();
      // Mặc định của Riverpod 3 sẽ thử lại sau 200 ms; provider của lab tắt.
      await tester.pump(const Duration(seconds: 30));

      expect(repository.fetchCount, 1);
      expect(find.byKey(_error), findsOneWidget);
    });

    testWidgets('provider trong ProviderScope khác nhau không dùng chung', (
      tester,
    ) async {
      final first = ControlledQuoteRepository();
      await pumpQuoteScreen(
        tester,
        overrides: [quoteRepositoryProvider.overrideWithValue(first)],
      );
      first.completeQuote();
      await tester.pump();
      expect(find.byKey(_card), findsOneWidget);

      // Dựng lại với ProviderScope mới (khoá khác): state cũ bị bỏ.
      final second = ControlledQuoteRepository();
      await tester.pumpWidget(
        ProviderScope(
          key: UniqueKey(),
          overrides: [quoteRepositoryProvider.overrideWithValue(second)],
          child: const MaterialApp(home: Scaffold(body: _Probe())),
        ),
      );

      expect(find.byKey(_card), findsNothing);
      expect(second.fetchCount, 1);
    });
  });
}

/// Đọc `quoteProvider` để kích hoạt yêu cầu mà không cần cả màn hình.
class _Probe extends ConsumerWidget {
  const _Probe();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.watch(quoteProvider);
    return const SizedBox();
  }
}
