import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_28/providers.dart';
import 'package:toi_noi_28/quote.dart';
import 'package:toi_noi_28/quote_repository.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

const _fixturePath = 'test/fixtures/quote_ride_5km.json';

Quote _parse(String json) {
  final map = jsonDecode(json) as Map<String, dynamic>;
  return Quote(
    serviceKind: ServiceKind.values.byName(map['serviceKind'] as String),
    distanceKm: (map['distanceKm'] as num).toDouble(),
    fare: Money.fromVnd(map['fareVnd'] as int),
  );
}

/// Repository trả báo giá cố định.
final class _FixedRepository implements QuoteRepository {
  _FixedRepository(this.quote);

  final Quote quote;

  @override
  Future<Quote> fetchQuote({
    required ServiceKind serviceKind,
    required double distanceKm,
  }) async => quote;

  @override
  Future<Trip> placeTrip(Quote quote) => throw UnimplementedError();
}

/// Repository đọc phản hồi mẫu từ file NGAY KHI được hỏi: I/O thật, bắt đầu
/// bên trong cây widget.
final class _FileRepository implements QuoteRepository {
  @override
  Future<Quote> fetchQuote({
    required ServiceKind serviceKind,
    required double distanceKm,
  }) async => _parse(await File(_fixturePath).readAsString());

  @override
  Future<Trip> placeTrip(Quote quote) => throw UnimplementedError();
}

void main() {
  group('runAsync cho I/O thật', () {
    testWidgets('đọc file mẫu trong runAsync, rồi đưa cho widget', (
      tester,
    ) async {
      // I/O thật phải bắt đầu VÀ kết thúc bên trong runAsync.
      final json = await tester.runAsync(
        () => File(_fixturePath).readAsString(),
      );
      final quote = _parse(json!);
      // File mẫu khớp domain: nếu bảng giá đổi, test này nhắc cập nhật file.
      expect(quote.fare, calculateFare(quote.distanceKm));

      await pumpQuoteScreen(
        tester,
        overrides: [
          quoteRepositoryProvider.overrideWithValue(_FixedRepository(quote)),
        ],
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('quote-fare')), findsOneWidget);
      expect(find.text('25.500 VND'), findsWidgets);
    });

    testWidgets(
      'I/O thật bắt đầu trong cây widget: một lượt runAsync chưa đủ',
      (tester) async {
        await pumpQuoteScreen(
          tester,
          overrides: [
            quoteRepositoryProvider.overrideWithValue(_FileRepository()),
          ],
        );

        // Chuỗi I/O bắt đầu trong vùng FakeAsync không tự xong: mỗi bước của nó
        // cần microtask FakeAsync chạy, mà việc đó chỉ xảy ra khi pump hoặc khi
        // một runAsync kết thúc. Tua đồng hồ giả hay đợi thời gian thật một
        // lần đều chưa đủ.
        await tester.pump(const Duration(minutes: 1));
        await tester.runAsync(
          () => Future<void>.delayed(const Duration(milliseconds: 200)),
        );
        await tester.pump(const Duration(minutes: 1));

        expect(find.byKey(const Key('quote-loading')), findsOneWidget);
      },
    );

    testWidgets(
      'lặp runAsync rồi pump thì cuối cùng cũng xong, không tất định',
      (tester) async {
        await pumpQuoteScreen(
          tester,
          overrides: [
            quoteRepositoryProvider.overrideWithValue(_FileRepository()),
          ],
        );

        // Số lượt phụ thuộc số bước của chuỗi I/O, không phải thời gian.
        var rounds = 0;
        while (find.byKey(const Key('quote-loading')).evaluate().isNotEmpty &&
            rounds < 50) {
          await tester.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 20)),
          );
          await tester.pump();
          rounds++;
        }

        expect(find.byKey(const Key('quote-fare')), findsOneWidget);
        expect(rounds, lessThan(50));
      },
    );

    testWidgets('runAsync trả null và đẩy lỗi vào takeException khi hỏng', (
      tester,
    ) async {
      final result = await tester.runAsync(
        () => File('test/fixtures/khong-co.json').readAsString(),
      );

      expect(result, isNull);
      expect(tester.takeException(), isA<FileSystemException>());
    });
  });
}
