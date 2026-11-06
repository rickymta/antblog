import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_40/main.dart';
import 'package:toi_noi_40/toi_noi_40.dart';

import 'support/fake_battery_gateway.dart';

void main() {
  testWidgets('hiện pin đọc được và đổi tần suất khi native báo pin yếu', (
    tester,
  ) async {
    final gateway = FakeBatteryGateway();
    await tester.pumpWidget(MaterialApp(home: BatteryPanel(gateway: gateway)));
    expect(find.text('Đang đọc pin...'), findsOneWidget);

    gateway.completeRead(
      const BatteryStatus(
        levelPercent: 80,
        isCharging: false,
        isPowerSaveMode: false,
      ),
    );
    await tester.pump();
    expect(find.text('Mức pin: 80%'), findsOneWidget);
    expect(find.text('Gửi vị trí mỗi 5 giây'), findsOneWidget);

    gateway.emit(
      const BatteryStatus(
        levelPercent: 15,
        isCharging: false,
        isPowerSaveMode: false,
      ),
    );
    // Stream phát qua microtask, nên cần pump lần nữa để dựng khung hình mới.
    await tester.pump();
    await tester.pump();
    expect(find.text('Gửi vị trí mỗi 30 giây'), findsOneWidget);
  });

  testWidgets('không đọc được pin: báo mã lỗi và dùng mốc giữa 10 giây', (
    tester,
  ) async {
    final gateway = FakeBatteryGateway();
    await tester.pumpWidget(MaterialApp(home: BatteryPanel(gateway: gateway)));

    gateway.failRead('missing-plugin');
    await tester.pump();

    expect(find.text('Không đọc được pin (missing-plugin)'), findsOneWidget);
    expect(find.text('Gửi vị trí mỗi 10 giây'), findsOneWidget);
  });
}
