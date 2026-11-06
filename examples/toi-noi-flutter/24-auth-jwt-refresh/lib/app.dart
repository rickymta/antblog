import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'auth/auth_controller.dart';
import 'home_screen.dart';
import 'login_screen.dart';

class ToiNoiApp extends StatelessWidget {
  const ToiNoiApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi',
      theme: ToiNoiTheme.light(),
      darkTheme: ToiNoiTheme.dark(),
      home: const AuthGate(),
    );
  }
}

/// Chọn màn hình theo trạng thái đăng nhập. Không có `Navigator.push` nào để
/// quên: đăng xuất ở đâu (kể cả do refresh bị từ chối giữa chừng) thì state
/// thành `null` và màn hình đăng nhập hiện ra. Với go_router, đây là chỗ nối
/// `refreshListenable` của bài 23.
class AuthGate extends ConsumerWidget {
  const AuthGate({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final auth = ref.watch(authControllerProvider);
    return switch (auth) {
      AsyncData(:final value) =>
        value == null ? const LoginScreen() : HomeScreen(session: value),
      _ => const Scaffold(body: Center(child: CircularProgressIndicator())),
    };
  }
}
