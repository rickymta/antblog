import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_07/quote_screen.dart';
import 'package:toi_noi_07/toi_noi_app.dart';

/// Dựng `QuoteScreen` trong `MaterialApp` như app thật: `Scaffold` cần
/// `Directionality` và `MaterialLocalizations` từ tổ tiên (`Theme.of` có
/// giá trị dự phòng).
Widget _host(double distanceKm) =>
    MaterialApp(home: QuoteScreen(distanceKm: distanceKm));

void main() {
  testWidgets('5 km hiện quãng đường và cước 25.500 VND', (tester) async {
    await tester.pumpWidget(const ToiNoiApp());

    expect(find.text('Tới Nơi'), findsOneWidget); // tiêu đề AppBar
    expect(find.text('Quãng đường: 5 km'), findsOneWidget);
    expect(find.text('25.500 VND'), findsOneWidget);
    expect(find.byIcon(Icons.directions_car), findsOneWidget);
    expect(find.textContaining('không phải bảng giá thực tế'), findsOneWidget);
  });

  group('cước theo quãng đường', () {
    // km, chuỗi cước kỳ vọng. Số minh hoạ, cùng bảng với toi_noi_core.
    const cases = <(double, String)>[
      (0, '12.000 VND'),
      (2, '12.000 VND'),
      (2.001, '12.005 VND'),
      (5, '25.500 VND'),
      (100, '453.000 VND'),
    ];

    for (final (km, expected) in cases) {
      testWidgets('$km km hiện $expected', (tester) async {
        await tester.pumpWidget(_host(km));

        expect(find.text(expected), findsOneWidget);
        expect(tester.takeException(), isNull);
      });
    }
  });

  group('ngoài miền 0..100 km', () {
    const invalid = <double>[
      -1,
      100.5,
      double.nan,
      double.infinity,
      double.negativeInfinity,
    ];

    for (final km in invalid) {
      testWidgets('$km km hiện thông báo thay vì cước', (tester) async {
        await tester.pumpWidget(_host(km));

        expect(find.text('Quãng đường phải từ 0 đến 100 km'), findsOneWidget);
        expect(find.textContaining('VND'), findsNothing);
        // build không ném: không có ErrorWidget đỏ trên màn hình.
        expect(tester.takeException(), isNull);
        expect(find.byType(ErrorWidget), findsNothing);
      });
    }
  });

  group('ToiNoiApp', () {
    testWidgets('dựng MaterialApp Material 3 với màn hình báo giá', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp());

      final app = tester.widget<MaterialApp>(find.byType(MaterialApp));
      expect(app.title, 'Tới Nơi');
      expect(app.theme?.useMaterial3, isTrue);
      expect(find.byType(QuoteScreen), findsOneWidget);
    });

    testWidgets('distanceKm đi từ ToiNoiApp xuống QuoteScreen', (tester) async {
      await tester.pumpWidget(const ToiNoiApp(distanceKm: 10));

      expect(find.text('Quãng đường: 10 km'), findsOneWidget);
      expect(find.text('48.000 VND'), findsOneWidget);
    });
  });

  group('formatKm', () {
    test('số nguyên bỏ phần thập phân', () {
      expect(formatKm(5), '5');
      expect(formatKm(0), '0');
      expect(formatKm(100), '100');
    });

    test('số lẻ dùng dấu phẩy', () {
      expect(formatKm(2.001), '2,001');
      expect(formatKm(7.5), '7,5');
    });

    test('NaN và vô cực không ném', () {
      expect(formatKm(double.nan), 'NaN');
      expect(formatKm(double.infinity), 'Infinity');
    });
  });
}
