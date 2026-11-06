import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_01/main.dart';

void main() {
  testWidgets('màn hình báo giá hiển thị tiêu đề và cước 5 km',
      (WidgetTester tester) async {
    await tester.pumpWidget(const ToiNoiApp());

    expect(find.text('Tới Nơi'), findsOneWidget);
    expect(find.text('Quãng đường: 5 km'), findsOneWidget);
    expect(find.text('Cước minh hoạ: 25.500 VND'), findsOneWidget);
    expect(find.byType(Scaffold), findsOneWidget);
  });

  group('formatVnd', () {
    test('chèn dấu chấm mỗi ba chữ số', () {
      expect(formatVnd(0), '0 VND');
      expect(formatVnd(999), '999 VND');
      expect(formatVnd(12000), '12.000 VND');
      expect(formatVnd(25500), '25.500 VND');
      expect(formatVnd(1234567), '1.234.567 VND');
    });
  });
}
