import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

/// Gốc ứng dụng. `ProviderScope` đặt ở `main()` hoặc ở test, bên trên widget
/// này; router truyền từ ngoài để test giữ được tay cầm mà đọc vị trí.
class ToiNoiApp extends StatelessWidget {
  const ToiNoiApp({required this.router, super.key});

  final GoRouter router;

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      title: 'Tới Nơi',
      theme: ToiNoiTheme.light(),
      darkTheme: ToiNoiTheme.dark(),
      routerConfig: router,
    );
  }
}
