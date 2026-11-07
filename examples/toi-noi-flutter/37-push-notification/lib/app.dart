import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import 'push_message.dart';
import 'push_registrar.dart';

/// `trip` lồng trong `/` để một đường dẫn `/trip/T-1002` mở được hai màn hình
/// xếp chồng (Trang chủ, rồi Chuyến): nút Back từ thông báo về Trang chủ
/// thay vì thoát app.
GoRouter buildRouter({
  required PushRegistrar registrar,
  String initialLocation = '/',
}) {
  return GoRouter(
    initialLocation: initialLocation,
    routes: [
      GoRoute(
        path: '/',
        builder: (context, state) => HomeScreen(registrar: registrar),
        routes: [
          GoRoute(
            path: 'trip/:id',
            builder: (context, state) =>
                TripScreen(tripId: state.pathParameters['id']!),
          ),
        ],
      ),
    ],
  );
}

class ToiNoiPushApp extends StatelessWidget {
  const ToiNoiPushApp({super.key, required this.router});

  final GoRouter router;

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(title: 'Tới Nơi', routerConfig: router);
  }
}

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key, required this.registrar});

  final PushRegistrar registrar;

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  PushPermission? _permission;

  Future<void> _enable() async {
    final permission = await widget.registrar.enable();
    if (!mounted) return;
    setState(() => _permission = permission);
  }

  String _label(PushStatus status) => switch (status) {
    PushStatus.off => 'Thông báo: tắt',
    PushStatus.waiting => 'Thông báo: đang đăng ký',
    PushStatus.synced => 'Thông báo: đã bật',
    PushStatus.failed => 'Thông báo: chưa báo được cho máy chủ',
  };

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Tới Nơi')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            ValueListenableBuilder<PushStatus>(
              valueListenable: widget.registrar.status,
              builder: (context, status, _) =>
                  Text(_label(status), key: const Key('push-status')),
            ),
            // Android 13+ phân biệt denied và deniedPermanently, iOS chỉ có
            // denied (kể cả khi hệ điều hành sẽ không hỏi nữa), nên cả hai
            // đều chỉ đường tới Cài đặt.
            if (_permission == PushPermission.denied ||
                _permission == PushPermission.deniedForever)
              const Text(
                'Thông báo đang tắt. Nếu hộp thoại không hiện lại, hãy bật '
                'trong Cài đặt của máy.',
                key: Key('push-denied-hint'),
              ),
            const SizedBox(height: 12),
            FilledButton(
              key: const Key('enable-push'),
              onPressed: _enable,
              child: const Text('Bật thông báo'),
            ),
            const SizedBox(height: 12),
            OutlinedButton(
              key: const Key('sign-out'),
              onPressed: () => widget.registrar.signOut(),
              child: const Text('Đăng xuất'),
            ),
          ],
        ),
      ),
    );
  }
}

/// Màn hình chuyến. Chỉ nhận mã chuyến: trạng thái, tài xế và giá phải lấy từ
/// máy chủ theo mã này, không lấy từ nội dung thông báo.
class TripScreen extends StatelessWidget {
  const TripScreen({super.key, required this.tripId});

  final String tripId;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('Chuyến $tripId')),
      body: Center(
        child: Text('Đang tải chuyến $tripId...', key: const Key('trip-body')),
      ),
    );
  }
}
