import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_50/flavor_check.dart';
import 'package:toi_noi_50/flavor_screen.dart';

void main() {
  group('checkFlavors', () {
    test('hai nguồn trùng tên thì khớp', () {
      for (final flavor in ['dev', 'staging', 'prod']) {
        expect(
          checkFlavors(native: flavor, config: flavor),
          FlavorCheck.consistent,
        );
      }
    });

    test('--flavor prod nhưng file cấu hình là dev thì lệch', () {
      expect(checkFlavors(native: 'prod', config: 'dev'), FlavorCheck.mismatch);
    });

    test('không có --flavor: appFlavor là null hoặc rỗng', () {
      expect(
        checkFlavors(native: null, config: 'dev'),
        FlavorCheck.nativeMissing,
      );
      expect(
        checkFlavors(native: '', config: 'dev'),
        FlavorCheck.nativeMissing,
      );
    });

    test('không có cả hai thì vẫn báo thiếu flavor native trước', () {
      expect(
        checkFlavors(native: null, config: null),
        FlavorCheck.nativeMissing,
      );
    });

    test('có --flavor nhưng thiếu FLAVOR trong cấu hình', () {
      expect(
        checkFlavors(native: 'staging', config: ''),
        FlavorCheck.configMissing,
      );
      expect(
        checkFlavors(native: 'staging', config: null),
        FlavorCheck.configMissing,
      );
    });

    test('so khớp phân biệt hoa thường', () {
      expect(
        checkFlavors(native: 'Prod', config: 'prod'),
        FlavorCheck.mismatch,
      );
    });
  });

  group('FlavorScreen', () {
    Future<void> show(WidgetTester tester, String? native, String? config) {
      return tester.pumpWidget(
        MaterialApp(
          home: FlavorScreen(native: native, config: config),
        ),
      );
    }

    String verdict(WidgetTester tester) =>
        tester.widget<Text>(find.byKey(const Key('flavor-verdict'))).data!;

    testWidgets('hiện hai giá trị và lời kết khi khớp', (tester) async {
      await show(tester, 'staging', 'staging');
      expect(find.text('staging'), findsNWidgets(2));
      expect(verdict(tester), FlavorScreen.messages[FlavorCheck.consistent]);
    });

    testWidgets('lệch flavor hiện cảnh báo nêu hai môi trường', (tester) async {
      await show(tester, 'prod', 'dev');
      expect(find.text('prod'), findsOneWidget);
      expect(find.text('dev'), findsOneWidget);
      expect(verdict(tester), FlavorScreen.messages[FlavorCheck.mismatch]);
    });

    testWidgets('giá trị không đặt hiện là (không đặt)', (tester) async {
      await show(tester, null, '');
      expect(find.text('(không đặt)'), findsNWidgets(2));
      expect(verdict(tester), FlavorScreen.messages[FlavorCheck.nativeMissing]);
    });

    test('mỗi kết quả có đúng một thông điệp khác nhau', () {
      expect(FlavorScreen.messages.keys.toSet(), FlavorCheck.values.toSet());
      expect(
        FlavorScreen.messages.values.toSet().length,
        FlavorCheck.values.length,
      );
    });
  });
}
