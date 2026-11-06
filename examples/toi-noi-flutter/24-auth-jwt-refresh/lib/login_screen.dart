import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'auth/auth_api.dart';
import 'auth/auth_controller.dart';

/// Câu hiển thị cho lỗi khi đăng nhập. Không đưa `title` tiếng Anh của server
/// lên giao diện.
String loginFailureText(Object error) => switch (error) {
  ClientFailure(statusCode: 429) =>
    'Bạn nhập sai quá số lần cho phép. Hãy xin mã mới.',
  ClientFailure(statusCode: 422, :final fieldErrors)
      when fieldErrors.containsKey('phone') =>
    'Số điện thoại chưa đúng định dạng quốc tế, ví dụ +84000000001.',
  ClientFailure(statusCode: 422) => 'Mã không đúng hoặc đã hết hạn.',
  NetworkFailure() ||
  TimeoutFailure() => 'Không tới được máy chủ. Kiểm tra mạng rồi thử lại.',
  _ => 'Có lỗi xảy ra. Thử lại sau ít phút.',
};

/// Hai bước: nhập số điện thoại, rồi nhập mã OTP. Số mẫu là số giả không có
/// thật; mã OTP của server giả là hằng cố định, chỉ dành cho fake.
class LoginScreen extends ConsumerStatefulWidget {
  const LoginScreen({super.key});

  @override
  ConsumerState<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends ConsumerState<LoginScreen> {
  final _phone = TextEditingController(text: '+84000000001');
  final _code = TextEditingController();
  OtpChallenge? _challenge;
  String? _error;
  bool _busy = false;

  @override
  void dispose() {
    _phone.dispose();
    _code.dispose();
    super.dispose();
  }

  Future<void> _run(Future<void> Function() action) async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      await action();
    } on ApiFailure catch (failure) {
      if (!mounted) return;
      setState(() {
        _error = loginFailureText(failure);
        // Quá số lần: mã cũ đã bị huỷ, quay về bước nhập số.
        if (failure is ClientFailure && failure.statusCode == 429) {
          _challenge = null;
          _code.clear();
        }
      });
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _request() => _run(() async {
    final challenge = await ref
        .read(authControllerProvider.notifier)
        .requestOtp(_phone.text.trim());
    if (mounted) setState(() => _challenge = challenge);
  });

  Future<void> _verify() => _run(
    () => ref
        .read(authControllerProvider.notifier)
        .verifyOtp(_challenge!, _code.text.trim()),
  );

  @override
  Widget build(BuildContext context) {
    final challenge = _challenge;
    return Scaffold(
      appBar: AppBar(title: const Text('Đăng nhập')),
      body: Padding(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            TextField(
              key: const Key('phone'),
              controller: _phone,
              enabled: challenge == null && !_busy,
              keyboardType: TextInputType.phone,
              decoration: const InputDecoration(labelText: 'Số điện thoại'),
            ),
            const SizedBox(height: ToiNoiSpacing.md),
            if (challenge == null)
              FilledButton(
                key: const Key('request-otp'),
                onPressed: _busy ? null : _request,
                child: const Text('Gửi mã OTP'),
              )
            else ...[
              TextField(
                key: const Key('code'),
                controller: _code,
                enabled: !_busy,
                keyboardType: TextInputType.number,
                decoration: const InputDecoration(labelText: 'Mã OTP'),
              ),
              const SizedBox(height: ToiNoiSpacing.md),
              FilledButton(
                key: const Key('verify-otp'),
                onPressed: _busy ? null : _verify,
                child: const Text('Xác nhận'),
              ),
            ],
            if (_error != null) ...[
              const SizedBox(height: ToiNoiSpacing.md),
              Text(
                _error!,
                key: const Key('login-error'),
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
            ],
            const SizedBox(height: ToiNoiSpacing.lg),
            const Text(
              'Bản thử dùng server giả: mã OTP là 123456 (chỉ dành cho fake).',
            ),
          ],
        ),
      ),
    );
  }
}
