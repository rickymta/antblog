import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_59/force_update_screen.dart';

import 'support.dart';

final storeUri = Uri.parse(
  'https://play.google.com/store/apps/details?id=vn.toinoi.customer',
);

Widget screen(FakeOpener opener, {String message = 'Cần bản mới'}) =>
    MaterialApp(
      home: ForceUpdateScreen(
        message: message,
        storeUri: storeUri,
        opener: opener,
      ),
    );

void main() {
  testWidgets('hiện thông điệp và đúng một nút hành động', (tester) async {
    await tester.pumpWidget(screen(FakeOpener(), message: 'Bản này đã cũ'));

    expect(find.text('Cần cập nhật Tới Nơi'), findsOneWidget);
    expect(find.text('Bản này đã cũ'), findsOneWidget);
    expect(find.byType(FilledButton), findsOneWidget);
    expect(find.byType(TextButton), findsNothing);
  });

  testWidgets('không có AppBar hay nút quay lại để thoát', (tester) async {
    await tester.pumpWidget(screen(FakeOpener()));

    expect(find.byType(AppBar), findsNothing);
    expect(find.byType(BackButton), findsNothing);
    expect(find.byType(CloseButton), findsNothing);
  });

  testWidgets('bấm nút mở đúng link cửa hàng', (tester) async {
    final opener = FakeOpener();
    await tester.pumpWidget(screen(opener));

    await tester.tap(find.text('Mở cửa hàng'));
    await tester.pump();

    expect(opener.opened, [storeUri]);
    expect(find.byType(SnackBar), findsNothing);
  });

  testWidgets('hệ điều hành từ chối mở thì báo cách tự tìm app', (
    tester,
  ) async {
    final opener = FakeOpener()..result = false;
    await tester.pumpWidget(screen(opener));

    await tester.tap(find.text('Mở cửa hàng'));
    await tester.pump();

    expect(find.text(ForceUpdateScreen.openFailedText), findsOneWidget);
  });

  testWidgets('opener ném lỗi cũng chỉ hiện thông báo, không văng', (
    tester,
  ) async {
    final opener = FakeOpener()..error = StateError('không có app cửa hàng');
    await tester.pumpWidget(screen(opener));

    await tester.tap(find.text('Mở cửa hàng'));
    await tester.pump();

    expect(find.text(ForceUpdateScreen.openFailedText), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  group('nút Back của hệ thống', () {
    Widget host(Widget pushed) => MaterialApp(
      home: Builder(
        builder: (context) => Scaffold(
          body: TextButton(
            onPressed: () =>
                Navigator.of(context)
                    .push(MaterialPageRoute<void>(builder: (_) => pushed)),
            child: const Text('Mở'),
          ),
        ),
      ),
    );

    testWidgets('đối chứng: một route thường bị đóng', (tester) async {
      await tester.pumpWidget(host(const Scaffold(body: Text('Route thường'))));
      await tester.tap(find.text('Mở'));
      await tester.pumpAndSettle();
      expect(find.text('Route thường'), findsOneWidget);

      await tester.binding.handlePopRoute();
      await tester.pumpAndSettle();

      expect(find.text('Route thường'), findsNothing);
    });

    testWidgets('màn hình chặn không bị đóng', (tester) async {
      await tester.pumpWidget(
        host(
          ForceUpdateScreen(
            message: 'Cần bản mới',
            storeUri: storeUri,
            opener: FakeOpener(),
          ),
        ),
      );
      await tester.tap(find.text('Mở'));
      await tester.pumpAndSettle();

      await tester.binding.handlePopRoute();
      await tester.pumpAndSettle();

      expect(find.byType(ForceUpdateScreen), findsOneWidget);
    });
  });

  testWidgets('nút có nhãn cho trình đọc màn hình', (tester) async {
    final handle = tester.ensureSemantics();
    await tester.pumpWidget(screen(FakeOpener()));

    expect(
      tester.getSemantics(find.text('Mở cửa hàng')),
      matchesSemantics(
        label: 'Mở cửa hàng',
        isButton: true,
        hasTapAction: true,
        isEnabled: true,
        hasEnabledState: true,
        isFocusable: true,
        hasFocusAction: true,
      ),
    );
    handle.dispose();
  });
}
