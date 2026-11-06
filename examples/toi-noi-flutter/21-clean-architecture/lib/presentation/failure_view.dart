import 'package:flutter/material.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'failure_text.dart';

/// Hiển thị một lỗi; nút Thử lại chỉ hiện khi domain nói thử lại có ích.
class FailureView extends StatelessWidget {
  const FailureView({super.key, required this.error, required this.onRetry});

  final Object error;

  /// Làm lại thao tác vừa thất bại.
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    final info = describeFailure(error);
    final color = Theme.of(context).colorScheme.error;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(info.message, style: TextStyle(color: color)),
        if (info.supportCode != null) Text('Mã hỗ trợ: ${info.supportCode}'),
        if (info.canRetry)
          Padding(
            padding: const EdgeInsets.only(top: ToiNoiSpacing.xs),
            child: FilledButton.tonal(
              onPressed: onRetry,
              child: const Text('Thử lại'),
            ),
          ),
      ],
    );
  }
}
