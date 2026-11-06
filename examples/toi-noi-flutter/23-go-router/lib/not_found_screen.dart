import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'paths.dart';

/// Link không hiểu được (route lạ, host lạ, thiếu mã chuyến).
class NotFoundScreen extends StatelessWidget {
  const NotFoundScreen({this.requested, super.key});

  final String? requested;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Không tìm thấy')),
      body: Padding(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('Link này không dẫn tới màn hình nào trong app.'),
            if (requested != null) Text(requested!),
            const SizedBox(height: ToiNoiSpacing.md),
            FilledButton(
              key: const Key('go-home'),
              onPressed: () => context.go(AppPaths.booking),
              child: const Text('Về trang chủ'),
            ),
          ],
        ),
      ),
    );
  }
}
