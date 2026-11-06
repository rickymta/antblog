import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'providers.dart';
import 'quote_state.dart';

/// Màn hình báo giá: chọn dịch vụ, bấm hỏi giá 5 km.
class QuoteScreen extends ConsumerStatefulWidget {
  const QuoteScreen({super.key});

  @override
  ConsumerState<QuoteScreen> createState() => _QuoteScreenState();
}

class _QuoteScreenState extends ConsumerState<QuoteScreen> {
  static const _distanceKm = 5.0;

  ServiceKind _kind = ServiceKind.ride;

  QuoteInput get _input =>
      QuoteInput(serviceKind: _kind, distanceKm: _distanceKm);

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(quoteControllerProvider);
    final busy = state is QuoteLoading;

    return Scaffold(
      appBar: AppBar(title: const Text('Báo giá')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            SegmentedButton<ServiceKind>(
              segments: [
                for (final kind in ServiceKind.values)
                  ButtonSegment(value: kind, label: Text(kind.label)),
              ],
              selected: {_kind},
              onSelectionChanged: busy
                  ? null
                  : (selection) => setState(() => _kind = selection.single),
            ),
            const SizedBox(height: 16),
            FilledButton(
              onPressed: busy
                  ? null
                  : () => ref
                        .read(quoteControllerProvider.notifier)
                        .fetch(_input),
              child: const Text('Báo giá 5 km'),
            ),
            const SizedBox(height: 16),
            // Vét cạn bốn nhánh: thiếu một nhánh là lỗi biên dịch.
            switch (state) {
              QuoteIdle() => const Text('Chọn dịch vụ rồi bấm báo giá.'),
              QuoteLoading() => const LinearProgressIndicator(),
              QuoteReady(:final fare) => Text(
                fare.toString(),
                style: Theme.of(context).textTheme.headlineMedium,
              ),
              QuoteFailed(:final failure) => Text(_describe(failure)),
            },
          ],
        ),
      ),
    );
  }

  String _describe(ApiFailure failure) => switch (failure) {
    NetworkFailure() => 'Không có kết nối tới máy chủ. Hãy thử lại.',
    TimeoutFailure() => 'Máy chủ phản hồi quá chậm. Hãy thử lại.',
    ServerFailure() => 'Máy chủ đang gặp sự cố. Hãy thử lại sau.',
    ClientFailure() => 'Yêu cầu không hợp lệ.',
    ParseFailure() => 'Dữ liệu từ máy chủ không đọc được.',
  };
}
