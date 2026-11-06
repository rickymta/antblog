import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_39/payment_flow.dart';
import 'package:toi_noi_39/payment_models.dart';
import 'package:toi_noi_39/payment_providers.dart';
import 'package:toi_noi_39/payment_screen.dart';
import 'package:toi_noi_39/payment_store.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/harness.dart';
import 'support/memory_gateway.dart';

const _amount = Money(25500);

void main() {
  late MemoryGateway gateway;
  late FakeLauncher launcher;
  late InMemoryPaymentStore store;

  setUp(() {
    gateway = MemoryGateway();
    launcher = FakeLauncher();
    store = InMemoryPaymentStore();
  });

  Future<ProviderContainer> pumpScreen(WidgetTester tester) async {
    final container = ProviderContainer(
      overrides: [
        paymentGatewayProvider.overrideWithValue(gateway),
        paymentStoreProvider.overrideWithValue(store),
        checkoutLauncherProvider.overrideWithValue(launcher),
        paymentNowProvider.overrideWithValue(
          () => DateTime.utc(2026, 11, 6, 9),
        ),
        paymentDelayProvider.overrideWithValue((_) async {}),
        paymentPollProvider.overrideWithValue(
          const PollPolicy(waits: [Duration(seconds: 1)]),
        ),
      ],
    );
    addTearDown(container.dispose);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const MaterialApp(
          home: PaymentScreen(tripId: 'trip-1', amount: _amount),
        ),
      ),
    );
    return container;
  }

  String message(WidgetTester tester) =>
      tester.widget<Text>(find.byKey(const Key('payment-message'))).data!;

  testWidgets('ban đầu hiện số tiền và nút Thanh toán', (tester) async {
    await pumpScreen(tester);
    expect(message(tester), 'Thanh toán 25.500 VND bằng ví VíXanh.');
    expect(find.widgetWithText(FilledButton, 'Thanh toán'), findsOneWidget);
  });

  testWidgets('bấm Thanh toán: tạo phiên và mở cổng', (tester) async {
    await pumpScreen(tester);

    await tester.tap(find.widgetWithText(FilledButton, 'Thanh toán'));
    await tester.pump();

    expect(gateway.created, 1);
    expect(launcher.opened, hasLength(1));
    expect(message(tester), startsWith('Hoàn tất thanh toán ở trang VíXanh'));
  });

  testWidgets('link nói success nhưng server chưa: không hiện thành công', (
    tester,
  ) async {
    final container = await pumpScreen(tester);
    await tester.tap(find.widgetWithText(FilledButton, 'Thanh toán'));
    await tester.pump();

    await container
        .read(paymentControllerProvider.notifier)
        .onDeepLink(returnLink('mem-1', status: 'success'));
    await tester.pump();

    expect(message(tester), startsWith('Đang chờ xác nhận.'));
    expect(find.text('Thanh toán thành công.'), findsNothing);
    expect(find.text('Kiểm tra lại'), findsOneWidget);

    gateway.status = PaymentStatus.succeeded;
    await tester.tap(find.text('Kiểm tra lại'));
    await tester.pump();

    expect(message(tester), 'Thanh toán thành công.');
  });

  testWidgets('mất mạng: nói rõ đang chờ, không báo thất bại', (tester) async {
    final container = await pumpScreen(tester);
    await tester.tap(find.widgetWithText(FilledButton, 'Thanh toán'));
    await tester.pump();
    gateway.offline = true;

    await container
        .read(paymentControllerProvider.notifier)
        .onDeepLink(returnLink('mem-1'));
    await tester.pump();

    expect(message(tester), contains('Chưa kết nối được'));
    expect(find.textContaining('thất bại'), findsNothing);
  });

  testWidgets('đóng tab bằng nút Done: app trở lại foreground thì hỏi server', (
    tester,
  ) async {
    await pumpScreen(tester);
    await tester.tap(find.widgetWithText(FilledButton, 'Thanh toán'));
    await tester.pump();
    expect(message(tester), startsWith('Hoàn tất thanh toán'));

    gateway.status = PaymentStatus.cancelled;
    for (final state in [
      AppLifecycleState.inactive,
      AppLifecycleState.hidden,
      AppLifecycleState.paused,
      AppLifecycleState.hidden,
      AppLifecycleState.inactive,
      AppLifecycleState.resumed,
    ]) {
      tester.binding.handleAppLifecycleStateChanged(state);
    }
    await tester.pump();

    expect(message(tester), 'Bạn đã huỷ thanh toán.');
    expect(find.text('Thanh toán lại'), findsOneWidget);
  });

  testWidgets('chỉ chuyển sang paused thì không hỏi server', (tester) async {
    await pumpScreen(tester);
    await tester.tap(find.widgetWithText(FilledButton, 'Thanh toán'));
    await tester.pump();
    gateway.status = PaymentStatus.succeeded;

    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.hidden);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    await tester.pump();

    expect(message(tester), startsWith('Hoàn tất thanh toán'));
  });

  testWidgets('nút Kiểm tra lại có ngay khi đang đợi khách', (tester) async {
    await pumpScreen(tester);
    await tester.tap(find.widgetWithText(FilledButton, 'Thanh toán'));
    await tester.pump();
    gateway.status = PaymentStatus.succeeded;

    await tester.tap(find.text('Kiểm tra lại'));
    await tester.pump();

    expect(message(tester), 'Thanh toán thành công.');
  });

  testWidgets('mở app khi có thanh toán dang dở: tự nạp và hỏi server', (
    tester,
  ) async {
    // Lần mở trước: tạo phiên rồi bị kill.
    final first = PaymentFlow(
      gateway: gateway,
      launcher: launcher,
      store: store,
      newKey: () => 'key-1',
      now: () => DateTime.utc(2026, 11, 6, 9),
    );
    await first.start(tripId: 'trip-1', amount: _amount);
    first.dispose();
    gateway.status = PaymentStatus.succeeded;

    await pumpScreen(tester);
    await tester.pump();

    expect(message(tester), 'Thanh toán thành công.');
  });
}
