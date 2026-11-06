import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_08/broken_examples.dart';
import 'package:toi_noi_08/sample_trips.dart';
import 'package:toi_noi_08/trip_card.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

void main() {
  group('tái hiện lỗi layout có chủ đích', () {
    testWidgets('BrokenTripCard ở 320 px: RenderFlex overflowed', (
      tester,
    ) async {
      setScreen(tester, 320);
      await tester.pumpWidget(
        app(
          BrokenTripCard(
            trip: tripAt(TripStatus.arriving),
            pickup: longAddress,
          ),
        ),
      );

      // Flutter báo overflow qua FlutterError.onError; flutter_test giữ lại
      // để takeException lấy ra. Lấy một lần: gọi lần hai sẽ trả null.
      final error = tester.takeException();
      expect(error, isA<FlutterError>());
      expect(error.toString(), contains('A RenderFlex overflowed by'));
      expect(error.toString(), contains('on the right'));
      expect(tester.takeException(), isNull);
    });

    testWidgets('cùng dữ liệu, cùng màn hình: TripCard không overflow', (
      tester,
    ) async {
      setScreen(tester, 320);
      await tester.pumpWidget(
        app(
          TripCard(
            trip: tripAt(TripStatus.arriving),
            pickup: longAddress,
            dropoff: shortAddress,
          ),
        ),
      );
      expect(tester.takeException(), isNull);
    });

    testWidgets('BrokenTripCard với địa chỉ ngắn trên màn hình rộng vẫn chạy', (
      tester,
    ) async {
      // Bản sai chỉ hỏng khi nội dung không vừa: dễ lọt qua lúc thử bằng
      // dữ liệu ngắn trên máy lớn.
      setScreen(tester, 800);
      await tester.pumpWidget(
        app(BrokenTripCard(trip: tripAt(TripStatus.arriving), pickup: 'Số 1')),
      );
      expect(tester.takeException(), isNull);
    });

    testWidgets('ListView trong Column: unbounded height', (tester) async {
      setScreen(tester, 390);
      final errors = await pumpCollectingErrors(
        tester,
        const MaterialApp(home: Scaffold(body: ListInColumnBroken())),
      );
      expect(errors, isNotEmpty);
      expect(errors.first.exception, isA<FlutterError>());
      expect(
        errors.first.exception.toString(),
        contains('Vertical viewport was given unbounded height'),
      );
    });

    testWidgets('bọc ListView bằng Expanded thì hết lỗi và cuộn được', (
      tester,
    ) async {
      setScreen(tester, 390);
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: ListInColumnFixed())),
      );
      expect(tester.takeException(), isNull);
      expect(find.text('T-1042'), findsOneWidget);
    });

    testWidgets(
      'Expanded trong Row cuộn ngang: non-zero flex, unbounded width',
      (tester) async {
        setScreen(tester, 390);
        final errors = await pumpCollectingErrors(
          tester,
          const MaterialApp(
            home: Scaffold(body: ExpandedInHorizontalScrollBroken()),
          ),
        );
        expect(errors, isNotEmpty);
        expect(errors.first.exception, isA<FlutterError>());
        expect(
          errors.first.exception.toString(),
          contains(
            'RenderFlex children have non-zero flex but incoming width '
            'constraints are unbounded',
          ),
        );
      },
    );
  });
}
