import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:shared_preferences_platform_interface/in_memory_shared_preferences_async.dart';
import 'package:shared_preferences_platform_interface/shared_preferences_async_platform_interface.dart';
import 'package:toi_noi_22/app.dart';
import 'package:toi_noi_22/auth/token_store.dart';
import 'package:toi_noi_22/db/app_database.dart';
import 'package:toi_noi_22/preferences/app_preferences.dart';
import 'package:toi_noi_22/providers.dart';
import 'package:toi_noi_22/trip_history_repository.dart';

import 'support.dart';

void main() {
  late AppDatabase db;
  late InMemoryTokenStore tokens;

  setUp(() {
    db = openMemoryDatabase();
    tokens = InMemoryTokenStore();
    SharedPreferencesAsyncPlatform.instance =
        InMemorySharedPreferencesAsync.empty();
  });
  tearDown(() => db.close());

  /// Chờ database và kho tuỳ chọn trả kết quả (I/O thật, ngoài đồng hồ giả
  /// của testWidgets) rồi vẽ lại.
  Future<void> settle(WidgetTester tester) async {
    await tester.runAsync(pumpEventQueue);
    await tester.pump();
  }

  /// Dựng app với kho giả; gọi lại hàm này mô phỏng "mở lại app" vì mọi
  /// provider là mới nhưng database, tuỳ chọn và token vẫn là những kho cũ.
  Future<void> launch(WidgetTester tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          appDatabaseProvider.overrideWithValue(db),
          tokenStoreProvider.overrideWithValue(tokens),
          clockProvider.overrideWithValue(() => t0),
        ],
        child: const ToiNoiApp(),
      ),
    );
    await settle(tester);
  }

  ThemeMode appThemeMode(WidgetTester tester) =>
      tester.widget<MaterialApp>(find.byType(MaterialApp)).themeMode!;

  testWidgets('lịch sử rỗng hiện lời nhắn', (tester) async {
    await launch(tester);
    expect(find.text('Chưa có chuyến nào'), findsOneWidget);
  });

  testWidgets('chuyến đã lưu hiện ngay khi mở màn hình', (tester) async {
    await tester.runAsync(
      () => DriftTripHistoryRepository(db.tripDao).saveAll([
        sampleTrip('T1', km: 5),
        sampleTrip(
          'T2',
          km: 2.001,
          requestedAt: t0.add(const Duration(hours: 1)),
        ),
      ]),
    );

    await launch(tester);

    expect(find.text('25.500 VND'), findsOneWidget);
    expect(find.text('12.005 VND'), findsOneWidget);
    // Chuyến mới hơn nằm trên.
    final first = tester.getTopLeft(find.text('12.005 VND')).dy;
    final second = tester.getTopLeft(find.text('25.500 VND')).dy;
    expect(first, lessThan(second));
  });

  testWidgets('thêm chuyến mẫu: bảng đổi, stream query cập nhật màn hình', (
    tester,
  ) async {
    await launch(tester);

    await tester.tap(find.text('Thêm chuyến mẫu'));
    await settle(tester);

    expect(find.text('Chưa có chuyến nào'), findsNothing);
    // Chuyến mẫu đầu tiên: 5 km, cước do calculateFare tính.
    expect(find.text('25.500 VND'), findsOneWidget);
    expect(find.text('Đặt xe · 5.0 km'), findsOneWidget);
  });

  testWidgets(
    'dòng hỏng trong database: màn hình báo lỗi, Xoá bộ nhớ đệm khôi phục',
    (tester) async {
      await tester.runAsync(() async {
        await db.customStatement(
          "INSERT INTO trips (id, status, distance_meters, fare_vnd, "
          "requested_at_ms) VALUES ('BAD', 'completed', 5000, 25500, 1)",
        );
      });

      await launch(tester);
      expect(find.text('Không đọc được lịch sử trên máy.'), findsOneWidget);

      await tester.tap(find.text('Xoá bộ nhớ đệm'));
      await settle(tester);

      expect(find.text('Chưa có chuyến nào'), findsOneWidget);
    },
  );

  testWidgets(
    'chọn chế độ tối: giao diện đổi và còn nguyên sau khi mở lại app',
    (tester) async {
      await launch(tester);
      expect(appThemeMode(tester), ThemeMode.system);

      await tester.tap(find.byTooltip('Cài đặt'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Tối'));
      await settle(tester);
      expect(appThemeMode(tester), ThemeMode.dark);

      // "Mở lại app": gỡ cây widget (mọi provider bị huỷ) rồi dựng lại.
      await tester.pumpWidget(const SizedBox());
      await launch(tester);
      expect(appThemeMode(tester), ThemeMode.dark);
      expect(
        await tester.runAsync(
          () => SharedPreferencesAsync().getString(AppPreferences.themeModeKey),
        ),
        'dark',
      );
    },
  );

  testWidgets('đăng nhập thử lưu token, đăng xuất xoá token', (tester) async {
    await launch(tester);
    await tester.tap(find.byTooltip('Cài đặt'));
    await tester.pumpAndSettle();
    expect(find.text('Chưa đăng nhập'), findsOneWidget);

    await tester.tap(find.text('Đăng nhập thử'));
    await settle(tester);
    expect(find.text('Đã lưu token trên máy'), findsOneWidget);
    expect(await tester.runAsync(tokens.read), isA<AuthTokens>());
    // Giao diện không bao giờ in giá trị token.
    expect(find.textContaining('demo-'), findsNothing);

    await tester.tap(find.text('Đăng xuất'));
    await settle(tester);
    expect(find.text('Chưa đăng nhập'), findsOneWidget);
    expect(await tester.runAsync(tokens.read), isNull);
  });
}
