import 'package:flutter/material.dart';

import 'url_opener.dart';

/// Màn hình chặn: không có nút đóng, nút Back hệ thống không thoát được, chỉ
/// có một việc là mở cửa hàng.
class ForceUpdateScreen extends StatelessWidget {
  const ForceUpdateScreen({
    super.key,
    required this.message,
    required this.storeUri,
    required this.opener,
  });

  final String message;
  final Uri storeUri;
  final UrlOpener opener;

  static const String openFailedText =
      'Không mở được cửa hàng. Hãy mở App Store hoặc Google Play và tìm "Tới Nơi".';

  Future<void> _openStore(BuildContext context) async {
    final messenger = ScaffoldMessenger.of(context);
    var opened = false;
    try {
      opened = await opener.open(storeUri);
    } on Object {
      opened = false;
    }
    if (!opened) {
      messenger.showSnackBar(const SnackBar(content: Text(openFailedText)));
    }
  }

  @override
  Widget build(BuildContext context) {
    return PopScope(
      canPop: false,
      child: Scaffold(
        body: SafeArea(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                const Icon(Icons.system_update, size: 64),
                const SizedBox(height: 24),
                Text(
                  'Cần cập nhật Tới Nơi',
                  style: Theme.of(context).textTheme.headlineSmall,
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 12),
                Text(message, textAlign: TextAlign.center),
                const SizedBox(height: 32),
                FilledButton(
                  onPressed: () => _openStore(context),
                  child: const Text('Mở cửa hàng'),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
