import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'account_screen.dart';
import 'auth.dart';
import 'booking_screen.dart';
import 'guard.dart';
import 'history_screen.dart';
import 'login_screen.dart';
import 'not_found_screen.dart';
import 'paths.dart';
import 'shell_scaffold.dart';
import 'trip_screen.dart';

/// Cầu nối Riverpod sang `Listenable`: `GoRouter.refreshListenable` chỉ nhận
/// `Listenable`, còn trạng thái đăng nhập nằm trong provider.
final class AuthRefresh extends ChangeNotifier {
  void ping() => notifyListeners();
}

/// Danh sách route. Tách khỏi [routerProvider] để test dựng router riêng.
List<RouteBase> buildRoutes() => [
  StatefulShellRoute.indexedStack(
    builder: (context, state, shell) => ShellScaffold(shell: shell),
    branches: [
      StatefulShellBranch(
        routes: [
          GoRoute(
            path: AppPaths.booking,
            builder: (context, state) => const BookingScreen(),
          ),
        ],
      ),
      StatefulShellBranch(
        routes: [
          GoRoute(
            path: AppPaths.history,
            builder: (context, state) => const HistoryScreen(),
          ),
        ],
      ),
      StatefulShellBranch(
        routes: [
          GoRoute(
            path: AppPaths.account,
            builder: (context, state) => const AccountScreen(),
          ),
        ],
      ),
    ],
  ),
  GoRoute(
    path: '/trip/:id',
    builder: (context, state) => TripScreen(id: state.pathParameters['id']!),
  ),
  GoRoute(
    path: AppPaths.login,
    builder: (context, state) => const LoginScreen(),
  ),
  GoRoute(
    path: AppPaths.notFound,
    builder: (context, state) =>
        NotFoundScreen(requested: state.uri.queryParameters['uri']),
  ),
];

/// Phiên hiện tại cho `redirect`.
///
/// Đã có giá trị thì đọc ngay từ state. Chỉ khi còn đang đọc kho lúc khởi
/// động mới chờ `.future`. Không dùng `.future` cho mọi trường hợp: ngay sau
/// `state = ...` nó có thể còn trả giá trị cũ, và `redirect` do
/// `refreshListenable` gọi chạy đúng vào lúc đó.
Future<Session?> _currentSession(Ref ref) {
  final auth = ref.read(authProvider);
  if (auth.hasValue) return Future.value(auth.value);
  return ref.read(authProvider.future);
}

/// `GoRouter` duy nhất của app.
///
/// Provider này đọc `authProvider` bằng `ref.listen` và `ref.read`, không
/// bằng `ref.watch`: đổi trạng thái đăng nhập chỉ nhắc router đánh giá lại
/// `redirect`, không dựng router mới (dựng mới là mất ngăn xếp và state tab).
final routerProvider = Provider<GoRouter>((ref) {
  final refresh = AuthRefresh();
  late final GoRouter router;

  // `refreshListenable` chạy lại redirect với vị trí của phần khai báo (go),
  // không thấy trang đẩy bằng push. Mất phiên thì ta tự kiểm trang trên cùng.
  Future<void> recheckTopPage() async {
    if (router.routerDelegate.currentConfiguration.isEmpty) return;
    final target = await guardLocation(
      router.state.uri,
      () => _currentSession(ref),
    );
    if (target != null && ref.mounted) router.go(target);
  }

  ref.listen(authProvider, (_, next) {
    refresh.ping();
    if (next.hasValue && next.value == null) recheckTopPage();
  });

  router = GoRouter(
    initialLocation: AppPaths.booking,
    refreshListenable: refresh,
    redirect: (context, state) =>
        guardLocation(state.uri, () => _currentSession(ref)),
    // Không khớp route nào (hoặc redirect lỗi): tới màn hình "không tìm thấy".
    onException: (context, state, router) =>
        router.go(AppPaths.notFoundFor(state.uri)),
    routes: buildRoutes(),
  );

  ref.onDispose(() {
    router.dispose();
    refresh.dispose();
  });
  return router;
});
