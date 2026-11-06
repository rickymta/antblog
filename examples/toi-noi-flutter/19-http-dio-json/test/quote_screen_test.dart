import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shelf/shelf.dart' show Response;
import 'package:toi_noi_19/app.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

Future<void> _pumpApp(WidgetTester tester, Harness harness) async {
  await tester.pumpWidget(
    ProviderScope(overrides: harness.overrides, child: const ToiNoiApp()),
  );
}

void main() {
  testWidgets('màn hình đầu: hướng dẫn, ba dịch vụ và nút Báo giá', (
    tester,
  ) async {
    await _pumpApp(tester, Harness());
    expect(
      find.text('Chọn dịch vụ, kéo quãng đường rồi bấm Báo giá.'),
      findsOneWidget,
    );
    expect(find.text('Đặt xe'), findsOneWidget);
    expect(find.text('Giao đồ ăn'), findsOneWidget);
    expect(find.text('Giao bưu kiện'), findsOneWidget);
    expect(find.text('Quãng đường: 5.0 km'), findsOneWidget);
    expect(find.text('Đặt chuyến'), findsNothing);
  });

  testWidgets('bấm Báo giá: vòng chờ rồi 25.500 VND', (tester) async {
    final pending = Completer<Response>();
    final harness = Harness.custom((request) => pending.future);
    await _pumpApp(tester, harness);

    await tester.tap(find.text('Báo giá'));
    await tester.pump();
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
    // Đang chờ: nút Báo giá bị khoá.
    expect(
      tester
          .widget<FilledButton>(find.widgetWithText(FilledButton, 'Báo giá'))
          .onPressed,
      isNull,
    );

    pending.complete(
      okJson({
        'serviceKind': 'ride',
        'distanceMeters': 5000,
        'fare': 25500,
        'currency': 'VND',
      }),
    );
    await tester.pumpAndSettle();
    expect(find.byType(CircularProgressIndicator), findsNothing);
    expect(find.text('25.500 VND'), findsOneWidget);
    expect(find.text('Đặt chuyến'), findsOneWidget);
  });

  testWidgets('đổi dịch vụ thì báo giá cũ biến mất', (tester) async {
    await _pumpApp(tester, Harness());
    await tester.tap(find.text('Báo giá'));
    await tester.pumpAndSettle();
    expect(find.text('25.500 VND'), findsOneWidget);

    await tester.tap(find.text('Giao đồ ăn'));
    await tester.pumpAndSettle();
    expect(find.text('25.500 VND'), findsNothing);
    expect(
      find.text('Chọn dịch vụ, kéo quãng đường rồi bấm Báo giá.'),
      findsOneWidget,
    );
  });

  testWidgets('kéo thanh trượt đổi quãng đường và xoá báo giá cũ', (
    tester,
  ) async {
    await _pumpApp(tester, Harness());
    await tester.tap(find.text('Báo giá'));
    await tester.pumpAndSettle();
    expect(find.text('25.500 VND'), findsOneWidget);

    await tester.drag(find.byType(Slider), const Offset(60, 0));
    await tester.pumpAndSettle();
    expect(find.text('Quãng đường: 5.0 km'), findsNothing);
    expect(find.text('25.500 VND'), findsNothing);
  });

  testWidgets('lỗi 500: câu thân thiện và mã hỗ trợ, bấm lại thì xong', (
    tester,
  ) async {
    final harness = Harness(faults: const FaultConfig(failFirst: 1));
    await _pumpApp(tester, harness);

    await tester.tap(find.text('Báo giá'));
    await tester.pumpAndSettle();
    expect(
      find.text('Máy chủ đang gặp sự cố. Hãy thử lại sau ít phút.'),
      findsOneWidget,
    );
    expect(find.text('Mã hỗ trợ: fake-000001'), findsOneWidget);
    // Không đưa title tiếng Anh của server lên giao diện.
    expect(find.textContaining('Unexpected'), findsNothing);

    await tester.tap(find.text('Báo giá'));
    await tester.pumpAndSettle();
    expect(find.textContaining('Máy chủ đang gặp sự cố'), findsNothing);
    expect(find.text('25.500 VND'), findsOneWidget);
  });

  testWidgets('đặt chuyến rồi làm mới để thấy tài xế nhận', (tester) async {
    final harness = Harness();
    await _pumpApp(tester, harness);

    await tester.tap(find.text('Báo giá'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Đặt chuyến'));
    await tester.pumpAndSettle();
    expect(find.text('Chuyến T-1'), findsOneWidget);
    expect(find.text('Đã đặt'), findsOneWidget);
    expect(find.text('Chưa có tài xế'), findsOneWidget);

    // Phía tài xế nhận chuyến (app tài xế sẽ làm việc này ở bài sau).
    final accepted = harness
        .driver('D-1')
        .acceptTrip('T-1', expectedVersion: 0);
    await tester.pumpAndSettle();
    await accepted;

    await tester.tap(find.text('Làm mới'));
    await tester.pumpAndSettle();
    expect(find.text('Tài xế đã nhận'), findsOneWidget);
    expect(find.text('Tài xế D-1'), findsOneWidget);
  });

  testWidgets('log của cả luồng không lộ Idempotency-Key', (tester) async {
    final harness = Harness();
    await _pumpApp(tester, harness);
    await tester.tap(find.text('Báo giá'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Đặt chuyến'));
    await tester.pumpAndSettle();

    final all = harness.logs.join('\n');
    expect(all, contains('--> POST /quotes'));
    expect(all, contains('--> POST /trips'));
    expect(all, contains('idempotency-key: [redacted]'));
    expect(all, isNot(contains('book-1')));
    expect(all, isNot(contains('distanceKm')));
  });

  testWidgets('đặt xong thì ẩn nút Đặt chuyến, không ra chuyến thứ hai', (
    tester,
  ) async {
    final harness = Harness();
    await _pumpApp(tester, harness);
    await tester.tap(find.text('Báo giá'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Đặt chuyến'));
    await tester.pumpAndSettle();

    expect(find.text('Chuyến T-1'), findsOneWidget);
    expect(find.text('Đặt chuyến'), findsNothing);
    expect(harness.store.trips, hasLength(1));
  });

  testWidgets('Đặt chuyến khác: bỏ thẻ chuyến và đặt lại được', (tester) async {
    final harness = Harness();
    await _pumpApp(tester, harness);
    await tester.tap(find.text('Báo giá'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Đặt chuyến'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Đặt chuyến khác'));
    await tester.pumpAndSettle();
    expect(find.text('Chuyến T-1'), findsNothing);
    expect(find.text('Đặt chuyến'), findsOneWidget);

    await tester.tap(find.text('Đặt chuyến'));
    await tester.pumpAndSettle();
    expect(find.text('Chuyến T-2'), findsOneWidget);
    expect(harness.store.trips, hasLength(2));
  });

  testWidgets('làm mới lỗi 500: giữ thẻ chuyến, có nút Thử lại', (
    tester,
  ) async {
    // Request thứ 3 (Làm mới) trả 500, request thứ 4 thành công.
    final harness = Harness(faults: const FaultConfig(failEvery: 3));
    await _pumpApp(tester, harness);
    await tester.tap(find.text('Báo giá'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Đặt chuyến'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Làm mới'));
    await tester.pumpAndSettle();
    expect(find.text('Chuyến T-1'), findsOneWidget);
    expect(find.text('Đã đặt'), findsOneWidget);
    expect(find.textContaining('Máy chủ đang gặp sự cố'), findsOneWidget);
    expect(find.text('Thử lại'), findsOneWidget);

    await tester.tap(find.text('Thử lại'));
    await tester.pumpAndSettle();
    expect(find.textContaining('Máy chủ đang gặp sự cố'), findsNothing);
    expect(find.text('Làm mới'), findsOneWidget);
  });

  testWidgets('đặt chuyến lỗi 500: nút đổi thành Thử đặt lại và bấm lại được', (
    tester,
  ) async {
    // Request thứ 2 (Đặt chuyến) trả 500, request thứ 3 thành công.
    final harness = Harness(faults: const FaultConfig(failEvery: 2));
    await _pumpApp(tester, harness);
    await tester.tap(find.text('Báo giá'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Đặt chuyến'));
    await tester.pumpAndSettle();
    expect(find.textContaining('Máy chủ đang gặp sự cố'), findsOneWidget);
    expect(find.text('Chuyến T-1'), findsNothing);

    await tester.tap(find.text('Thử đặt lại'));
    await tester.pumpAndSettle();
    expect(find.text('Chuyến T-1'), findsOneWidget);
    expect(harness.store.trips, hasLength(1));
  });
}
