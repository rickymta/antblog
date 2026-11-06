import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_18/booking_screen.dart';
import 'package:toi_noi_18/providers.dart';
import 'package:toi_noi_18/quote.dart';
import 'package:toi_noi_18/quote_repository.dart';
import 'package:toi_noi_18/trip_draft.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

FilledButton _bookButton(WidgetTester tester) => tester.widget<FilledButton>(
  find.widgetWithText(FilledButton, 'Đặt chuyến'),
);

void main() {
  group('BookingScreen: ba trạng thái của báo giá', () {
    testWidgets('loading rồi data: spinner, sau đó cước 25.500 VND của 5 km', (
      tester,
    ) async {
      final reply = Completer<Quote>();
      final repository = StubQuoteRepository(onQuote: (_, _) => reply.future);
      await pumpApp(tester, repository: repository);

      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      expect(find.text('Đang báo giá…'), findsOneWidget);

      reply.complete(quoteFor(5));
      await tester.pump();

      expect(find.byType(CircularProgressIndicator), findsNothing);
      expect(find.text('25.500 VND'), findsOneWidget);
      expect(find.text('Đặt xe'), findsWidgets);
    });

    testWidgets(
      'ghi đè quoteProvider bằng AsyncLoading, AsyncData, AsyncError',
      (tester) async {
        Future<void> pumpWith(AsyncValue<Quote> value) async {
          await tester.pumpWidget(
            ProviderScope(
              overrides: [quoteProvider.overrideWithValue(value)],
              child: const MaterialApp(home: BookingScreen()),
            ),
          );
        }

        await pumpWith(const AsyncLoading<Quote>());
        expect(find.byType(CircularProgressIndicator), findsOneWidget);

        await pumpWith(AsyncData(quoteFor(100)));
        expect(find.text('453.000 VND'), findsOneWidget);

        await pumpWith(
          AsyncError<Quote>(const QuoteUnavailable(), StackTrace.empty),
        );
        expect(find.text('Dịch vụ báo giá đang bận'), findsOneWidget);
        expect(find.text('Thử lại'), findsOneWidget);
      },
    );

    testWidgets(
      'trạng thái lỗi không hiện giá trị cũ dù AsyncError còn mang nó',
      (tester) async {
        // Sau một lần thành công, AsyncError vẫn giữ `value` của lần trước.
        final container = await pumpApp(
          tester,
          repository: StubQuoteRepository(
            onQuote: (kind, km) => km > 5
                ? Future<Quote>.error(StateError('hỏng'))
                : Future.value(quoteFor(km, kind: kind)),
          ),
        );
        await tester.pump();
        expect(find.text('25.500 VND'), findsOneWidget);

        await tester.tap(find.byKey(const Key('more')));
        await tester.pump();
        await tester.pump();

        final state = container.read(quoteProvider);
        expect(state, isA<AsyncError<Quote>>());
        expect(state.value?.fare, const Money(25500)); // giá trị cũ vẫn nằm đó
        expect(find.text('25.500 VND'), findsNothing); // nhưng không hiện ra
        expect(find.text('Có lỗi không mong đợi'), findsOneWidget);
      },
    );
  });

  group('BookingScreen: bản nháp điều khiển báo giá', () {
    testWidgets('bấm + tăng 1 km: hỏi lại và hiện cước của 6 km', (
      tester,
    ) async {
      final repository = StubQuoteRepository();
      await pumpApp(tester, repository: repository);
      await tester.pump();
      expect(find.text('5 km'), findsOneWidget);

      await tester.tap(find.byKey(const Key('more')));
      await tester.pump();
      await tester.pump();

      expect(find.text('6 km'), findsOneWidget);
      expect(find.text('${calculateFare(6)}'), findsOneWidget); // 30.000 VND
      expect(repository.quoteRequests.map((r) => r.km), [5, 6]);
    });

    testWidgets('chọn Giao đồ ăn: hỏi lại với dịch vụ mới', (tester) async {
      final repository = StubQuoteRepository();
      await pumpApp(tester, repository: repository);
      await tester.pump();

      await tester.tap(find.text('Giao đồ ăn'));
      await tester.pump();
      await tester.pump();

      expect(repository.quoteRequests.last.kind, ServiceKind.food);
      expect(find.text('Giao đồ ăn'), findsNWidgets(2)); // chip và thẻ báo giá
    });

    testWidgets('overrideWithBuild đặt bản nháp khởi đầu cho test', (
      tester,
    ) async {
      await pumpApp(
        tester,
        repository: StubQuoteRepository(),
        overrides: [
          tripDraftProvider.overrideWithBuild(
            (ref, notifier) => const TripDraft(
              serviceKind: ServiceKind.parcel,
              distanceKm: 12,
            ),
          ),
        ],
      );
      await tester.pump();

      expect(find.text('12 km'), findsOneWidget);
      expect(find.text('${calculateFare(12)}'), findsOneWidget);
    });

    testWidgets('nút - tắt ở 1 km, nút + tắt ở 100 km', (tester) async {
      Future<void> pumpAt(double km) async {
        await tester.pumpWidget(
          ProviderScope(
            key: ValueKey(km),
            overrides: [
              quoteRepositoryProvider.overrideWithValue(StubQuoteRepository()),
              tripDraftProvider.overrideWithBuild(
                (ref, notifier) =>
                    TripDraft(serviceKind: ServiceKind.ride, distanceKm: km),
              ),
            ],
            child: const MaterialApp(home: BookingScreen()),
          ),
        );
        await tester.pump();
      }

      await pumpAt(1);
      expect(
        tester.widget<IconButton>(find.byKey(const Key('less'))).onPressed,
        isNull,
      );
      expect(
        tester.widget<IconButton>(find.byKey(const Key('more'))).onPressed,
        isNotNull,
      );

      await pumpAt(100);
      expect(
        tester.widget<IconButton>(find.byKey(const Key('more'))).onPressed,
        isNull,
      );
      expect(find.text('453.000 VND'), findsOneWidget);
    });
  });

  group('BookingScreen: lỗi, thử lại và ref.listen', () {
    testWidgets(
      'lỗi tạm thời: spinner "đang thử lại", rồi đúng một SnackBar khi hết lượt',
      (tester) async {
        final repository = StubQuoteRepository(
          onQuote: (_, _) => Future<Quote>.error(const QuoteUnavailable()),
        );
        await pumpApp(tester, repository: repository);
        await tester.pump();

        // Lần đầu lỗi, chờ 1 giây để thử lại: chưa phải lỗi với người dùng.
        expect(find.text('Mạng chập chờn, đang thử lại…'), findsOneWidget);
        expect(find.byType(SnackBar), findsNothing);

        await tester.pump(const Duration(seconds: 1));
        await tester.pump();

        expect(repository.quoteRequests, hasLength(2));
        expect(find.byType(SnackBar), findsOneWidget);
        expect(
          find.text('Không lấy được báo giá: Dịch vụ báo giá đang bận'),
          findsOneWidget,
        );
        expect(find.text('Thử lại'), findsOneWidget);
      },
    );

    testWidgets('bấm Thử lại: invalidate, hiện spinner, rồi ra cước', (
      tester,
    ) async {
      var calls = 0;
      final repository = StubQuoteRepository(
        onQuote: (kind, km) {
          calls++;
          return calls <=
                  2 // lần đầu và lần tự thử lại đều lỗi
              ? Future<Quote>.error(const QuoteUnavailable())
              : Future.value(quoteFor(km, kind: kind));
        },
      );
      await pumpApp(tester, repository: repository);
      await tester.pump();
      await tester.pump(const Duration(seconds: 1));
      await tester.pump();
      expect(find.text('Thử lại'), findsOneWidget);

      await tester.tap(find.text('Thử lại'));
      await tester.pump(); // invalidate -> build lại, đang tải
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      expect(find.text('Thử lại'), findsNothing);

      await tester.pump();
      expect(find.text('25.500 VND'), findsOneWidget);
      expect(repository.quoteRequests, hasLength(3));
    });

    testWidgets('lỗi lập trình (Error) báo ngay, không qua bước thử lại', (
      tester,
    ) async {
      final repository = StubQuoteRepository(
        onQuote: (_, _) => Future<Quote>.error(StateError('bug')),
      );
      await pumpApp(tester, repository: repository);
      await tester.pump();

      expect(find.byType(SnackBar), findsOneWidget);
      expect(repository.quoteRequests, hasLength(1));
      expect(find.text('Mạng chập chờn, đang thử lại…'), findsNothing);
    });

    testWidgets('ref.listen không chạy ở lần build đầu, chỉ khi state đổi', (
      tester,
    ) async {
      final seen = <String>[];
      await tester.pumpWidget(
        ProviderScope(
          child: Consumer(
            builder: (context, ref, _) {
              ref.listen(tripDraftProvider, (prev, next) {
                seen.add('${prev?.distanceKm} -> ${next.distanceKm}');
              });
              return const SizedBox();
            },
          ),
        ),
      );
      expect(seen, isEmpty);

      final container = ProviderScope.containerOf(
        tester.element(find.byType(Consumer)),
      );
      container.read(tripDraftProvider.notifier).setDistanceKm(8);
      await tester.pump();

      expect(seen, ['5.0 -> 8.0']);
    });
  });

  group('BookingScreen: đặt chuyến', () {
    testWidgets('đang đặt thì nút tắt; xong thì mở màn hình chuyến', (
      tester,
    ) async {
      final reply = Completer<Trip>();
      final repository = StubQuoteRepository(onPlace: (_) => reply.future);
      await pumpApp(tester, repository: repository);
      await tester.pump();

      await tester.tap(find.widgetWithText(FilledButton, 'Đặt chuyến'));
      await tester.pump();
      expect(_bookButton(tester).onPressed, isNull);
      expect(repository.placed.single.fare, const Money(25500));

      reply.complete(tripFor('T-0001'));
      await tester.pumpAndSettle();

      expect(find.text('Chuyến T-0001'), findsOneWidget);
      expect(
        find.text('Đã đặt'),
        findsOneWidget,
      ); // TripStatusChip của requested
      expect(find.text('25.500 VND'), findsOneWidget);
      expect(repository.tripRequests, ['T-0001']);
    });

    testWidgets('đặt lỗi: SnackBar và nút mở lại', (tester) async {
      final repository = StubQuoteRepository(
        onPlace: (_) => Future<Trip>.error(const QuoteUnavailable()),
      );
      await pumpApp(tester, repository: repository);
      await tester.pump();

      await tester.tap(find.widgetWithText(FilledButton, 'Đặt chuyến'));
      await tester.pump();
      await tester.pump();

      expect(
        find.text('Không đặt được chuyến: Dịch vụ báo giá đang bận'),
        findsOneWidget,
      );
      expect(_bookButton(tester).onPressed, isNotNull);
      expect(repository.placed, hasLength(1)); // không tự thử lại
    });

    testWidgets('rời màn hình chuyến thì tripByIdProvider(id) bị bỏ', (
      tester,
    ) async {
      final repository = StubQuoteRepository();
      final container = await pumpApp(tester, repository: repository);
      await tester.pump();
      await tester.tap(find.widgetWithText(FilledButton, 'Đặt chuyến'));
      await tester.pumpAndSettle();
      expect(container.read(tripByIdProvider('T-0001').exists), isTrue);

      await tester.pageBack();
      await tester.pumpAndSettle();
      await container.pump();

      expect(container.read(tripByIdProvider('T-0001').exists), isFalse);
    });

    testWidgets('chuyến không tồn tại: màn hình chuyến hiện lỗi', (
      tester,
    ) async {
      final repository = StubQuoteRepository(
        onTrip: (id) => Future<Trip>.error(TripNotFound(id)),
      );
      await pumpApp(tester, repository: repository);
      await tester.pump();

      await tester.tap(find.widgetWithText(FilledButton, 'Đặt chuyến'));
      await tester.pumpAndSettle();

      expect(find.text('Không tìm thấy chuyến T-0001'), findsOneWidget);
    });
  });

  group('FakeQuoteRepository trong app thật (đường vui)', () {
    testWidgets(
      'độ trễ 400 ms: spinner rồi cước, đặt xong thấy chuyến T-0001',
      (tester) async {
        final repository = FakeQuoteRepository(
          latency: const Duration(milliseconds: 400),
          now: () => DateTime.utc(2026, 11, 6, 8),
        );
        await pumpApp(tester, repository: repository);
        expect(find.byType(CircularProgressIndicator), findsOneWidget);

        await tester.pump(const Duration(milliseconds: 400));
        await tester.pump();
        expect(find.text('25.500 VND'), findsOneWidget);

        await tester.tap(find.widgetWithText(FilledButton, 'Đặt chuyến'));
        await tester.pump(const Duration(milliseconds: 400));
        await tester.pumpAndSettle(const Duration(milliseconds: 400));

        expect(find.text('Chuyến T-0001'), findsOneWidget);
      },
    );
  });
}
