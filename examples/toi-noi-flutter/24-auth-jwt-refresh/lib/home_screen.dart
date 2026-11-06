import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'auth/auth_controller.dart';
import 'providers.dart';

/// Màn hình sau đăng nhập. `session.claims` chỉ để hiện tên và vai trò; không
/// có chỗ nào ở đây dùng `role` để mở hay khoá một chức năng.
class HomeScreen extends ConsumerWidget {
  const HomeScreen({required this.session, super.key});

  final AuthSession session;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final claims = session.claims;
    final booking = ref.watch(bookingControllerProvider);
    return Scaffold(
      appBar: AppBar(title: const Text('Tới Nơi')),
      body: Padding(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              claims == null
                  ? 'Đã đăng nhập'
                  : 'Tài khoản ${claims.subject} (vai trò ${claims.role ?? '?'}, '
                        'chỉ để hiển thị)',
              key: const Key('who'),
            ),
            const SizedBox(height: ToiNoiSpacing.md),
            FilledButton(
              key: const Key('book'),
              onPressed: booking.isLoading
                  ? null
                  : ref.read(bookingControllerProvider.notifier).book,
              child: const Text('Đặt chuyến mẫu 5 km'),
            ),
            const SizedBox(height: ToiNoiSpacing.md),
            switch (booking) {
              AsyncLoading() => const CircularProgressIndicator(),
              AsyncError(:final error) => Text(
                error is ClientFailure && error.statusCode == 401
                    ? 'Phiên đăng nhập đã hết hạn.'
                    : 'Chưa đặt được chuyến. Thử lại sau.',
                key: const Key('booking-error'),
              ),
              AsyncData(:final value) when value != null => Row(
                children: [
                  Text('Chuyến ${value.id}: '),
                  MoneyText(value.fare, key: const Key('fare')),
                ],
              ),
              _ => const SizedBox.shrink(),
            },
            const Spacer(),
            OutlinedButton(
              key: const Key('sign-out'),
              onPressed: ref.read(authControllerProvider.notifier).signOut,
              child: const Text('Đăng xuất'),
            ),
          ],
        ),
      ),
    );
  }
}
