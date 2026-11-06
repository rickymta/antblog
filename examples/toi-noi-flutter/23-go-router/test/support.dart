import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:toi_noi_23/app.dart';
import 'package:toi_noi_23/auth.dart';
import 'package:toi_noi_23/router.dart';
import 'package:toi_noi_23/trips.dart';

const alice = Session(userName: 'An');

/// App thật với kho phiên và repository giả.
Widget pumpAppWidget({
  Session? signedInAs,
  Duration restoreLatency = Duration.zero,
  bool storeFails = false,
  TripRepository? trips,
}) => ProviderScope(
  overrides: [
    sessionStoreProvider.overrideWithValue(
      InMemorySessionStore(
        initial: signedInAs,
        latency: restoreLatency,
        failOnRead: storeFails,
      ),
    ),
    tripRepositoryProvider.overrideWithValue(trips ?? FakeTripRepository()),
  ],
  child: const ToiNoiApp(),
);

/// Dựng app rồi chờ yên. Trả về `ProviderContainer` của `ProviderScope` để
/// test đọc router và `authProvider`.
Future<ProviderContainer> pumpApp(
  WidgetTester tester, {
  Session? signedInAs,
  Duration restoreLatency = Duration.zero,
  bool storeFails = false,
  TripRepository? trips,
}) async {
  await tester.pumpWidget(
    pumpAppWidget(
      signedInAs: signedInAs,
      restoreLatency: restoreLatency,
      storeFails: storeFails,
      trips: trips,
    ),
  );
  await tester.pumpAndSettle();
  return ProviderScope.containerOf(tester.element(find.byType(MaterialApp)));
}

extension AppContainer on ProviderContainer {
  GoRouter get router => read(routerProvider);
  AuthController get auth => read(authProvider.notifier);
}

/// Vị trí của màn hình trên cùng (đã qua redirect), kể cả khi vừa `push`.
/// `routeInformationProvider.value` và `currentConfiguration.uri` chỉ phản
/// ánh phần khai báo (go), không phản ánh route đẩy bằng `push`.
String locationOf(GoRouter router) => router.state.uri.toString();

/// Giả lập đúng thông điệp mà engine gửi khi app đang chạy và nhận deep link:
/// kênh `flutter/navigation`, method `pushRouteInformation`, tham số
/// `location` là URI đầy đủ (Android: `Intent.getData().toString()`,
/// iOS: `NSURL.absoluteString`).
Future<void> receiveLink(WidgetTester tester, Uri link) async {
  final message = const JSONMethodCodec().encodeMethodCall(
    MethodCall('pushRouteInformation', {'location': link.toString()}),
  );
  await tester.binding.defaultBinaryMessenger.handlePlatformMessage(
    'flutter/navigation',
    message,
    (_) {},
  );
  await tester.pumpAndSettle();
}

/// Giả lập khởi động nguội kiểu Android: engine đặt `defaultRouteName` thành
/// URI trước khi `GoRouter` được dựng. (iOS mở ở `/` rồi gửi link bằng
/// `pushRouteInformation`, tức là đường của [receiveLink].)
void launchWithLink(WidgetTester tester, Uri link) {
  tester.binding.platformDispatcher.defaultRouteNameTestValue = link.toString();
  addTearDown(tester.binding.platformDispatcher.clearDefaultRouteNameTestValue);
}

/// Nhãn trên thanh tab (tránh trùng với tiêu đề AppBar cùng chữ).
Finder tab(String label) =>
    find.descendant(of: find.byType(NavigationBar), matching: find.text(label));
