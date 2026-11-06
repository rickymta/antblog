import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_16/broken_examples.dart';
import 'package:toi_noi_16/sample_trips.dart';
import 'package:toi_noi_16/trip_detail.dart';
import 'package:toi_noi_16/trip_tile.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'support.dart';

void main() {
  group('cỡ chữ hệ thống tới được widget', () {
    testWidgets(
      'textScaleFactorTestValue = 2 thành textScaler của MediaQuery',
      (tester) async {
        setTextScale(tester, 2);
        late TextScaler scaler;
        await pumpThemed(
          tester,
          Builder(
            builder: (context) {
              scaler = MediaQuery.textScalerOf(context);
              return const SizedBox();
            },
          ),
        );
        expect(scaler.scale(14), 28);
      },
    );
  });

  group('TripTile ở 320 px', () {
    for (final scale in [1.0, 1.5, 2.0, 3.0]) {
      testWidgets('cỡ chữ x$scale: cả 8 trạng thái không overflow', (
        tester,
      ) async {
        setScreen(tester, 320, 700);
        setTextScale(tester, scale);
        await pumpThemed(
          tester,
          Scaffold(
            body: ListView(
              padding: const EdgeInsets.all(ToiNoiSpacing.md),
              children: [
                for (final status in TripStatus.values)
                  TripTile(
                    trip: tripAt(status, id: 'T-${status.index}'),
                    onTap: () {},
                  ),
              ],
            ),
          ),
        );
        // takeException trả lỗi layout đầu tiên (ví dụ RenderFlex overflowed).
        expect(tester.takeException(), isNull);
      });
    }

    testWidgets('chữ x2: chip trạng thái xuống dòng thay vì tràn', (
      tester,
    ) async {
      setScreen(tester, 390, 800);
      final trip = tripAt(TripStatus.cancelled, id: 'T-1');
      Future<void> pumpTile() => pumpThemed(
        tester,
        Scaffold(
          body: Center(
            child: TripTile(trip: trip, onTap: () {}),
          ),
        ),
      );

      await pumpTile();
      final title = find.text('Đặt xe');
      final chip = find.byType(TripStatusChip);
      // Cỡ chữ 1: tiêu đề và chip cùng một hàng.
      expect(tester.getRect(chip).top, lessThan(tester.getRect(title).bottom));

      setTextScale(tester, 2);
      await tester.pump();
      // Cỡ chữ 2: chip nằm dưới tiêu đề.
      expect(
        tester.getRect(chip).top,
        greaterThanOrEqualTo(tester.getRect(title).bottom),
      );
      expect(tester.takeException(), isNull);
    });
  });

  group('ca sai có chủ đích: chiều cao cố định', () {
    testWidgets('cao 72 vừa ở cỡ chữ 1', (tester) async {
      setScreen(tester, 390, 800);
      await pumpThemed(
        tester,
        Scaffold(
          body: FixedHeightTripTile(
            trip: tripAt(TripStatus.completed, id: 'T-1'),
          ),
        ),
      );
      expect(tester.takeException(), isNull);
    });

    testWidgets('cao 72 tràn ở cỡ chữ 2: RenderFlex overflowed', (
      tester,
    ) async {
      setScreen(tester, 390, 800);
      setTextScale(tester, 2);
      await pumpThemed(
        tester,
        Scaffold(
          body: FixedHeightTripTile(
            trip: tripAt(TripStatus.completed, id: 'T-1'),
          ),
        ),
      );
      final error = tester.takeException();
      expect(error, isA<FlutterError>());
      expect(error.toString(), contains('A RenderFlex overflowed by'));
      expect(error.toString(), contains('on the bottom'));
    });
  });

  group('app đầy đủ ở 320 px, chữ x2', () {
    for (final destination in ['Báo giá', 'Lịch sử', 'Cài đặt']) {
      testWidgets('màn $destination không overflow', (tester) async {
        setScreen(tester, 320, 568);
        setTextScale(tester, 2);
        await pumpApp(tester);
        await tester.tap(find.text(destination).last);
        await tester.pumpAndSettle();

        expect(tester.takeException(), isNull);
      });
    }

    testWidgets('trang chi tiết và hộp thoại huỷ không overflow', (
      tester,
    ) async {
      setScreen(tester, 320, 568);
      setTextScale(tester, 2);
      await pumpApp(tester);
      await tester.tap(find.text('Lịch sử').last);
      await tester.pumpAndSettle();
      await tester.tap(find.textContaining('T-1045'));
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);

      // Chữ to đẩy nút xuống dưới màn hình: phải cuộn tới mới chạm được.
      final cancel = find.widgetWithText(OutlinedButton, 'Huỷ chuyến');
      await tester.scrollUntilVisible(
        cancel,
        100,
        scrollable: find.descendant(
          of: find.byType(TripDetailPage),
          matching: find.byType(Scrollable),
        ),
      );
      await tester.ensureVisible(cancel);
      await tester.pumpAndSettle();
      await tester.tap(cancel);
      await tester.pumpAndSettle();
      expect(find.text('Huỷ chuyến T-1045?'), findsOneWidget);
      expect(tester.takeException(), isNull);
    });

    testWidgets('màn báo giá cuộn được: nút Đặt chuyến vẫn ghim ở đáy', (
      tester,
    ) async {
      setScreen(tester, 320, 568);
      setTextScale(tester, 2);
      await pumpApp(tester);

      final button = find.widgetWithText(FilledButton, 'Đặt chuyến');
      expect(button, findsOneWidget);
      expect(tester.getRect(button).bottom, lessThanOrEqualTo(568));
      expect(tester.takeException(), isNull);
    });
  });
}
