import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'auth.dart';

/// Màn hình đăng nhập giả (bài 24 làm OTP thật). Nó không biết `from` và
/// không tự điều hướng: đăng nhập xong, `refreshListenable` làm router đánh
/// giá lại `/login?from=...` và `redirect` đưa người dùng tới `from`.
class LoginScreen extends ConsumerStatefulWidget {
  const LoginScreen({super.key});

  @override
  ConsumerState<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends ConsumerState<LoginScreen> {
  final _name = TextEditingController(text: 'Khách Tới Nơi');

  @override
  void dispose() {
    _name.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Đăng nhập')),
      body: Padding(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            TextField(
              key: const Key('user-name'),
              controller: _name,
              decoration: const InputDecoration(labelText: 'Tên hiển thị'),
            ),
            const SizedBox(height: ToiNoiSpacing.md),
            FilledButton(
              key: const Key('sign-in'),
              onPressed: () =>
                  ref.read(authProvider.notifier).signIn(_name.text),
              child: const Text('Đăng nhập'),
            ),
          ],
        ),
      ),
    );
  }
}
