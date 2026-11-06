import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_08/sample_trips.dart';
import 'package:toi_noi_08/trip_card.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

void main() {
  group('dữ liệu lấy từ Trip', () {
    testWidgets('5 km hiện 25.500 VND, nhãn trạng thái và mã chuyến', (
      tester,
    ) async {
      setScreen(tester, 390);
      final trip = tripAt(TripStatus.inTransit);
      await tester.pumpWidget(
        app(TripCard(trip: trip, pickup: shortAddress, dropoff: shortAddress)),
      );

      expect(find.text('25.500 VND'), findsOneWidget);
      expect(find.text('5 km'), findsOneWidget);
      expect(find.text('Đang chở / giao'), findsOneWidget);
      expect(find.text('Đặt xe · T-1042'), findsOneWidget);
    });

    testWidgets('2,001 km hiện 12.005 VND và 100 km hiện 453.000 VND', (
      tester,
    ) async {
      setScreen(tester, 390);
      await tester.pumpWidget(
        app(
          Column(
            children: [
              TripCard(
                trip: tripAt(TripStatus.requested, distanceKm: 2.001),
                pickup: shortAddress,
                dropoff: shortAddress,
              ),
              TripCard(
                trip: tripAt(TripStatus.requested, distanceKm: 100),
                pickup: shortAddress,
                dropoff: shortAddress,
              ),
            ],
          ),
        ),
      );

      expect(find.text('12.005 VND'), findsOneWidget);
      expect(find.text('2,001 km'), findsOneWidget);
      expect(find.text('453.000 VND'), findsOneWidget);
      expect(find.text('100 km'), findsOneWidget);
    });

    testWidgets('mỗi trong 8 trạng thái hiện đúng nhãn, không lỗi layout', (
      tester,
    ) async {
      setScreen(tester, 320);
      for (final status in TripStatus.values) {
        await tester.pumpWidget(
          app(
            TripCard(
              trip: tripAt(status),
              pickup: shortAddress,
              dropoff: shortAddress,
            ),
          ),
        );
        expect(find.text(status.label), findsOneWidget, reason: status.name);
        expect(tester.takeException(), isNull, reason: status.name);
      }
    });

    testWidgets('ba dịch vụ dùng ba biểu tượng khác nhau', (tester) async {
      setScreen(tester, 390);
      final icons = <IconData>{};
      for (final kind in ServiceKind.values) {
        await tester.pumpWidget(
          app(
            TripCard(
              trip: tripAt(TripStatus.requested, serviceKind: kind),
              pickup: shortAddress,
              dropoff: shortAddress,
            ),
          ),
        );
        expect(find.text('${kind.label} · T-1042'), findsOneWidget);
        icons.add(tester.widget<Icon>(find.byType(Icon).first).icon!);
      }
      expect(icons, hasLength(3));
    });

    test('formatKm bỏ phần .0 và dùng dấu phẩy', () {
      expect(formatKm(5), '5 km');
      expect(formatKm(2.001), '2,001 km');
      expect(formatKm(12.4), '12,4 km');
    });
  });

  group('màn hình hẹp 320 px với địa chỉ dài', () {
    testWidgets(
      'không overflow, thẻ rộng 288 px và mọi chữ nằm trong màn hình',
      (tester) async {
        setScreen(tester, 320);
        await tester.pumpWidget(
          app(
            TripCard(
              trip: tripAt(TripStatus.arriving, distanceKm: 12.4),
              pickup: longAddress,
              dropoff: longAddress,
            ),
          ),
        );

        // Overflow được báo qua FlutterError; không có gì để lấy nghĩa là sạch.
        expect(tester.takeException(), isNull);
        expect(tester.getSize(find.byType(TripCard)).width, 288);
        expect(find.byKey(TripCard.compactKey), findsOneWidget);

        for (final element in find.byType(Text).evaluate()) {
          final rect = tester.getRect(find.byWidget(element.widget));
          expect(rect.left, greaterThanOrEqualTo(0));
          expect(rect.right, lessThanOrEqualTo(320));
        }
      },
    );

    testWidgets('địa chỉ dài bị cắt ở hai dòng với dấu ba chấm', (
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

      final address = tester.widget<Text>(find.text(longAddress));
      expect(address.maxLines, 2);
      expect(address.overflow, TextOverflow.ellipsis);
      final paragraph = tester.renderObject<RenderParagraph>(
        find.text(longAddress),
      );
      expect(paragraph.didExceedMaxLines, isTrue);
    });

    testWidgets('cỡ chữ hệ thống 1,5 lần vẫn không overflow ở 320 px', (
      tester,
    ) async {
      setScreen(tester, 320);
      await tester.pumpWidget(
        app(
          TripCard(
            trip: tripAt(TripStatus.pickedUp, distanceKm: 100),
            pickup: longAddress,
            dropoff: longAddress,
          ),
          textScale: 1.5,
        ),
      );

      expect(tester.takeException(), isNull);
      expect(find.text('453.000 VND'), findsOneWidget);
    });
  });

  group('LayoutBuilder đổi bố cục theo constraints', () {
    testWidgets('layoutFor: 599 là compact, 600 là wide', (tester) async {
      expect(TripCard.layoutFor(599.9), TripCardLayout.compact);
      expect(TripCard.layoutFor(600), TripCardLayout.wide);
      expect(TripCard.wideBreakpoint, 600);
    });

    testWidgets('màn hình 360 px dùng compact', (tester) async {
      setScreen(tester, 360);
      await tester.pumpWidget(
        app(
          TripCard(
            trip: tripAt(TripStatus.accepted),
            pickup: shortAddress,
            dropoff: shortAddress,
          ),
        ),
      );
      expect(find.byKey(TripCard.compactKey), findsOneWidget);
      expect(find.byKey(TripCard.wideKey), findsNothing);
    });

    testWidgets('màn hình 632 px (thẻ rộng 600) đổi sang wide', (tester) async {
      setScreen(tester, 632);
      await tester.pumpWidget(
        app(
          TripCard(
            trip: tripAt(TripStatus.accepted),
            pickup: shortAddress,
            dropoff: shortAddress,
          ),
        ),
      );
      expect(tester.getSize(find.byType(TripCard)).width, 600);
      expect(find.byKey(TripCard.wideKey), findsOneWidget);
      expect(find.byKey(TripCard.compactKey), findsNothing);
      expect(tester.takeException(), isNull);
    });

    testWidgets('màn hình 631 px (thẻ rộng 599) vẫn compact', (tester) async {
      setScreen(tester, 631);
      await tester.pumpWidget(
        app(
          TripCard(
            trip: tripAt(TripStatus.accepted),
            pickup: shortAddress,
            dropoff: shortAddress,
          ),
        ),
      );
      expect(tester.getSize(find.byType(TripCard)).width, 599);
      expect(find.byKey(TripCard.compactKey), findsOneWidget);
    });

    testWidgets('xoay màn hình 390 -> 844 làm bố cục đổi trong cùng một cây', (
      tester,
    ) async {
      final card = TripCard(
        trip: tripAt(TripStatus.accepted),
        pickup: shortAddress,
        dropoff: longAddress,
      );
      setScreen(tester, 390, 844);
      await tester.pumpWidget(app(card));
      expect(find.byKey(TripCard.compactKey), findsOneWidget);

      tester.view.physicalSize = const Size(844, 390);
      await tester.pump();
      expect(find.byKey(TripCard.wideKey), findsOneWidget);
      expect(tester.takeException(), isNull);
    });

    testWidgets('bố cục compact: cước canh sát mép phải của thẻ', (
      tester,
    ) async {
      // Màn hình 560 px: thẻ rộng 528 (< 600 nên vẫn compact).
      setScreen(tester, 560);
      await tester.pumpWidget(
        app(
          TripCard(
            trip: tripAt(TripStatus.accepted),
            pickup: shortAddress,
            dropoff: shortAddress,
          ),
        ),
      );
      expect(find.byKey(TripCard.compactKey), findsOneWidget);
      final card = tester.getRect(find.byType(Card));
      final fare = tester.getRect(find.text('25.500 VND'));
      // Lề trong của thẻ là 16. Nếu Column dùng crossAxisAlignment.start thì
      // Wrap co theo nội dung và cước dừng sớm hơn rất nhiều.
      expect(fare.right, card.right - 16);
      // Nhãn trạng thái bắt đầu sau lề thẻ 16 và đệm của nhãn 8.
      expect(tester.getRect(find.text('Tài xế đã nhận')).left, card.left + 24);
    });

    testWidgets('physicalSize chia devicePixelRatio ra pixel logic', (
      tester,
    ) async {
      // 960 pixel vật lý ở tỉ lệ 3 là 320 pixel logic: cùng kết quả như 320 x 1.
      tester.view.devicePixelRatio = 3;
      tester.view.physicalSize = const Size(960, 2700);
      addTearDown(tester.view.reset);
      await tester.pumpWidget(
        app(
          TripCard(
            trip: tripAt(TripStatus.accepted),
            pickup: longAddress,
            dropoff: longAddress,
          ),
        ),
      );
      expect(tester.getSize(find.byType(TripCard)).width, 288);
      expect(find.byKey(TripCard.compactKey), findsOneWidget);
      expect(tester.takeException(), isNull);
    });

    testWidgets('hai thẻ cạnh nhau trên màn hình 1000 px đều là compact', (
      tester,
    ) async {
      setScreen(tester, 1000);
      TripCard card() => TripCard(
        trip: tripAt(TripStatus.accepted),
        pickup: shortAddress,
        dropoff: longAddress,
      );
      await tester.pumpWidget(
        app(
          Row(
            children: [
              Expanded(child: card()),
              Expanded(child: card()),
            ],
          ),
        ),
      );
      // (1000 - 2 x 16) / 2 = 484 < 600, dù màn hình rộng.
      expect(tester.getSize(find.byType(TripCard).first).width, 484);
      expect(find.byKey(TripCard.compactKey), findsNWidgets(2));
      expect(find.byKey(TripCard.wideKey), findsNothing);
    });

    testWidgets('bố cục theo bề rộng cha chứ không theo màn hình', (
      tester,
    ) async {
      setScreen(tester, 1000);
      await tester.pumpWidget(
        app(
          Align(
            alignment: Alignment.topLeft,
            child: SizedBox(
              width: 320,
              child: TripCard(
                trip: tripAt(TripStatus.accepted),
                pickup: shortAddress,
                dropoff: longAddress,
              ),
            ),
          ),
        ),
      );
      expect(tester.getSize(find.byType(TripCard)).width, 320);
      expect(find.byKey(TripCard.compactKey), findsOneWidget);
      expect(tester.takeException(), isNull);
    });
  });

  group('Stack và Positioned', () {
    testWidgets('chấm trạng thái nằm ở góc dưới phải của biểu tượng', (
      tester,
    ) async {
      setScreen(tester, 390);
      await tester.pumpWidget(
        app(
          TripCard(
            trip: tripAt(TripStatus.accepted),
            pickup: shortAddress,
            dropoff: shortAddress,
          ),
        ),
      );
      final dot = tester.getRect(find.byKey(const Key('status-dot')));
      final badge = tester.getRect(
        find
            .ancestor(
              of: find.byKey(const Key('status-dot')),
              matching: find.byType(SizedBox),
            )
            .first,
      );
      expect(badge.size, const Size(44, 44));
      expect(dot.bottomRight, badge.bottomRight);
      expect(dot.size, const Size(12, 12));
    });
  });
}
