import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'connectivity.dart';
import 'failure_text.dart';
import 'providers.dart';

/// Banner mỏng báo thiết bị không có đường truyền. Chỉ là thông tin: mọi nút
/// vẫn bấm được, vì connectivity báo "none" cũng có thể trễ hoặc sai.
class OfflineBanner extends ConsumerWidget {
  const OfflineBanner({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final link = ref.watch(linkProvider).value;
    if (link != LinkType.none) {
      return const SizedBox.shrink();
    }
    final scheme = Theme.of(context).colorScheme;
    return Material(
      color: scheme.errorContainer,
      child: Padding(
        padding: const EdgeInsets.all(ToiNoiSpacing.sm),
        child: Row(
          children: [
            Icon(Icons.wifi_off, color: scheme.onErrorContainer),
            const SizedBox(width: ToiNoiSpacing.sm),
            Expanded(
              child: Text(
                'Thiết bị đang không có kết nối mạng.',
                style: TextStyle(color: scheme.onErrorContainer),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Hiển thị một lỗi cùng các nút hành động do [describeFailure] chọn.
class FailureView extends ConsumerWidget {
  const FailureView({super.key, required this.error, required this.onRetry});

  final Object error;

  /// Làm lại thao tác vừa thất bại.
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Huỷ request không phải thất bại: không có gì để hiện.
    if (isCancellation(error)) {
      return const SizedBox.shrink();
    }
    final link = ref.watch(linkProvider).value;
    final info = describeFailure(error, link: link);
    final color = Theme.of(context).colorScheme.error;
    final settings = ref.read(systemSettingsProvider);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(info.message, style: TextStyle(color: color)),
        if (info.supportCode != null) Text('Mã hỗ trợ: ${info.supportCode}'),
        if (info.actions.isNotEmpty)
          Wrap(
            spacing: ToiNoiSpacing.sm,
            children: [
              if (info.actions.contains(FailureAction.retry))
                FilledButton.tonal(
                  onPressed: onRetry,
                  child: const Text('Thử lại'),
                ),
              if (info.actions.contains(FailureAction.openSettings))
                TextButton(
                  onPressed: settings.openNetworkSettings,
                  child: Text(settings.buttonLabel),
                ),
            ],
          ),
      ],
    );
  }
}
