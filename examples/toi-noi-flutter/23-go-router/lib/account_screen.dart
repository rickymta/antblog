import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'auth.dart';

/// Tab "Tài khoản". Đăng xuất chỉ đổi state; việc đưa người dùng về
/// `/login` là của `redirect`, không phải của nút này.
class AccountScreen extends ConsumerWidget {
  const AccountScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final session = ref.watch(authProvider).value;
    return Scaffold(
      appBar: AppBar(title: const Text('Tài khoản')),
      body: Padding(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Đang đăng nhập: ${session?.userName ?? '-'}'),
            const SizedBox(height: ToiNoiSpacing.md),
            OutlinedButton(
              key: const Key('sign-out'),
              onPressed: () => ref.read(authProvider.notifier).signOut(),
              child: const Text('Đăng xuất'),
            ),
          ],
        ),
      ),
    );
  }
}
