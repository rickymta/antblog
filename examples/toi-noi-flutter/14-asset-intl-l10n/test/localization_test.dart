import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_14/l10n/app_localizations.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'support.dart';

const nbsp = '\u00A0';
const narrowNbsp = '\u202F';

/// Nhãn và số tiền nằm ở hai widget: `Text` cho nhãn, `MoneyText` cho tiền.
void expectFare(String label, String money) {
  expect(find.text(label), findsOneWidget);
  expect(find.widgetWithText(MoneyText, money), findsOneWidget);
}

void main() {
  group('ngôn ngữ theo thiết bị', () {
    testWidgets('thiết bị vi_VN: toàn bộ màn hình bằng tiếng Việt', (
      tester,
    ) async {
      setDeviceLocales(tester, [const Locale('vi', 'VN')]);
      await pumpApp(tester);

      expect(find.text('Báo giá chuyến'), findsOneWidget);
      expect(find.text('Quãng đường: 5 km'), findsOneWidget);
      expectFare('Cước tạm tính', '25.500$nbsp₫');
      expect(find.text('Đặt lúc 6/11/2026 23:30'), findsOneWidget);
      expect(find.text('3 tài xế ở gần'), findsOneWidget);
    });

    testWidgets('thiết bị en_GB: khớp theo mã ngôn ngữ en', (tester) async {
      setDeviceLocales(tester, [const Locale('en', 'GB')]);
      await pumpApp(tester);

      expect(find.text('Trip quote'), findsOneWidget);
      expect(find.text('Distance: 5 km'), findsOneWidget);
      expectFare('Estimated fare', '₫25,500');
      expect(
        find.text('Requested at 11/6/2026 11:30${narrowNbsp}PM'),
        findsOneWidget,
      );
      expect(find.text('3 drivers nearby'), findsOneWidget);
    });

    testWidgets('thiết bị fr: không hỗ trợ nên rơi về vi nhờ '
        'preferred-supported-locales', (tester) async {
      setDeviceLocales(tester, [const Locale('fr', 'FR')]);
      await pumpApp(tester);

      expect(find.text('Báo giá chuyến'), findsOneWidget);
      expect(AppLocalizations.supportedLocales.first, const Locale('vi'));
    });

    testWidgets('thiết bị ưu tiên fr rồi en: lấy en, ngôn ngữ đầu tiên '
        'được hỗ trợ', (tester) async {
      setDeviceLocales(tester, [const Locale('fr'), const Locale('en')]);
      await pumpApp(tester);

      expect(find.text('Trip quote'), findsOneWidget);
    });
  });

  group('đổi ngôn ngữ ngay trong app', () {
    testWidgets('chạm English rồi Tiếng Việt: chữ, tiền và giờ đổi theo', (
      tester,
    ) async {
      setDeviceLocales(tester, [const Locale('vi')]);
      await pumpApp(tester);
      expect(find.text('Báo giá chuyến'), findsOneWidget);

      await tester.tap(find.text('English'));
      await tester.pumpAndSettle();
      expect(find.text('Trip quote'), findsOneWidget);
      expectFare('Estimated fare', '₫25,500');
      expect(find.text('Báo giá chuyến'), findsNothing);

      await tester.tap(find.text('Tiếng Việt'));
      await tester.pumpAndSettle();
      expect(find.text('Báo giá chuyến'), findsOneWidget);
      expectFare('Cước tạm tính', '25.500$nbsp₫');
    });

    testWidgets('locale ghi đè thắng ngôn ngữ của thiết bị', (tester) async {
      setDeviceLocales(tester, [const Locale('vi')]);
      await pumpApp(tester, locale: const Locale('en'));

      expect(find.text('Trip quote'), findsOneWidget);
    });

    testWidgets('tên app của hệ điều hành đổi theo locale', (tester) async {
      setDeviceLocales(tester, [const Locale('vi')]);
      await pumpApp(tester);
      expect(tester.widget<Title>(find.byType(Title)).title, 'Tới Nơi');

      await tester.tap(find.text('English'));
      await tester.pumpAndSettle();
      expect(tester.widget<Title>(find.byType(Title)).title, 'Toi Noi');
    });
  });

  group('số thực và tiền theo locale', () {
    testWidgets('2,001 km: dấu thập phân đổi theo locale, cước 12.005', (
      tester,
    ) async {
      setDeviceLocales(tester, [const Locale('vi')]);
      await pumpApp(tester, distanceKm: 2.001);
      expect(find.text('Quãng đường: 2,001 km'), findsOneWidget);
      expectFare('Cước tạm tính', '12.005$nbsp₫');

      await tester.tap(find.text('English'));
      await tester.pumpAndSettle();
      expect(find.text('Distance: 2.001 km'), findsOneWidget);
      expectFare('Estimated fare', '₫12,005');
    });

    testWidgets('100 km: 453.000 ₫', (tester) async {
      setDeviceLocales(tester, [const Locale('vi')]);
      await pumpApp(tester, distanceKm: 100);
      expectFare('Cước tạm tính', '453.000$nbsp₫');
    });
  });

  group('số nhiều ICU trong ARB', () {
    final vi = lookupAppLocalizations(const Locale('vi'));
    final en = lookupAppLocalizations(const Locale('en'));

    test('en có ba dạng: =0, =1 và other', () {
      expect(en.driversNearby(0), 'No drivers nearby');
      expect(en.driversNearby(1), '1 driver nearby');
      expect(en.driversNearby(2), '2 drivers nearby');
      expect(en.driversNearby(25), '25 drivers nearby');
    });

    test('vi không chia số ít, số nhiều: chỉ =0 và other', () {
      expect(vi.driversNearby(0), 'Chưa có tài xế ở gần');
      expect(vi.driversNearby(1), '1 tài xế ở gần');
      expect(vi.driversNearby(2), '2 tài xế ở gần');
    });

    test('số lớn có dấu phân cách nghìn theo locale', () {
      expect(vi.driversNearby(1000), '1.000 tài xế ở gần');
      expect(en.driversNearby(1000), '1,000 drivers nearby');
    });

    testWidgets('bấm + và − trên màn hình đổi câu theo số lượng', (
      tester,
    ) async {
      setDeviceLocales(tester, [const Locale('en')]);
      await pumpApp(tester);
      expect(find.text('3 drivers nearby'), findsOneWidget);

      final remove = find.byTooltip('Remove a driver');
      await tester.tap(remove);
      await tester.tap(remove);
      await tester.pump();
      expect(find.text('1 driver nearby'), findsOneWidget);

      await tester.tap(remove);
      await tester.pump();
      expect(find.text('No drivers nearby'), findsOneWidget);
      // Về 0 thì nút bớt bị khoá.
      expect(
        tester
            .widget<IconButton>(find.widgetWithIcon(IconButton, Icons.remove))
            .onPressed,
        isNull,
      );

      await tester.tap(find.byTooltip('Add a driver'));
      await tester.pump();
      expect(find.text('1 driver nearby'), findsOneWidget);
    });
  });

  group('widget dùng chung nhận chuỗi đã dịch', () {
    testWidgets('chip trạng thái searching đổi nhãn theo locale', (
      tester,
    ) async {
      setDeviceLocales(tester, [const Locale('vi')]);
      await pumpApp(tester);
      expect(
        find.widgetWithText(TripStatusChip, 'Đang tìm tài xế'),
        findsOneWidget,
      );

      await tester.tap(find.text('English'));
      await tester.pumpAndSettle();
      expect(
        find.widgetWithText(TripStatusChip, 'Finding a driver'),
        findsOneWidget,
      );
    });

    testWidgets('MoneyText nhận formatter, không dùng Money.toString()', (
      tester,
    ) async {
      setDeviceLocales(tester, [const Locale('vi')]);
      await pumpApp(tester);
      expect(find.text('25.500 VND'), findsNothing);
      expect(find.text('25.500$nbsp₫'), findsOneWidget);
    });
  });

  group('chuỗi có sẵn của Material, Cupertino và Widgets', () {
    testWidgets('nhờ GlobalMaterialLocalizations, nút chuẩn đổi theo locale', (
      tester,
    ) async {
      setDeviceLocales(tester, [const Locale('vi')]);
      await pumpApp(tester);
      final viLabels = MaterialLocalizations.of(
        tester.element(find.byType(Scaffold)),
      );
      expect(viLabels.cancelButtonLabel, 'Huỷ');
      expect(viLabels.okButtonLabel, 'OK');

      await tester.tap(find.text('English'));
      await tester.pumpAndSettle();
      final enLabels = MaterialLocalizations.of(
        tester.element(find.byType(Scaffold)),
      );
      expect(enLabels.cancelButtonLabel, 'Cancel');
    });
  });

  group('asset và Semantics theo locale', () {
    testWidgets('logo là Image.asset từ assets/images/logo.png', (
      tester,
    ) async {
      setDeviceLocales(tester, [const Locale('vi')]);
      await pumpApp(tester);

      final image = tester.widget<Image>(find.byType(Image));
      expect((image.image as AssetImage).assetName, 'assets/images/logo.png');
    });

    testWidgets('nhãn Semantics của logo được dịch', (tester) async {
      setDeviceLocales(tester, [const Locale('vi')]);
      final handle = tester.ensureSemantics();
      await pumpApp(tester);
      expect(find.bySemanticsLabel('Biểu tượng Tới Nơi'), findsOneWidget);

      await tester.tap(find.text('English'));
      await tester.pumpAndSettle();
      expect(find.bySemanticsLabel('Toi Noi logo'), findsOneWidget);
      handle.dispose();
    });
  });

  group('lỗi cấu hình', () {
    testWidgets('quên đăng ký delegate: AppLocalizations.of ném lỗi null', (
      tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Text(AppLocalizations.of(context).quoteTitle),
          ),
        ),
      );
      expect(tester.takeException(), isA<TypeError>());
    });
  });
}
