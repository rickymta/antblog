import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'paths.dart';
import 'trips.dart';

/// `/trip/:id`. Mở được từ danh sách (có màn hình bên dưới) lẫn từ deep link
/// (ngăn xếp chỉ có màn hình này), nên nút Back phải xử lý cả hai.
class TripScreen extends ConsumerWidget {
  const TripScreen({required this.id, super.key});

  final String id;

  void _back(BuildContext context) {
    if (context.canPop()) {
      context.pop();
    } else {
      context.go(AppPaths.history);
    }
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final trip = ref.watch(tripByIdProvider(id));
    return Scaffold(
      appBar: AppBar(
        leading: BackButton(onPressed: () => _back(context)),
        title: Text('Chuyến $id'),
      ),
      body: Padding(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        child: switch (trip) {
          AsyncData(:final value) => Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              TripStatusChip(status: value.status),
              const SizedBox(height: ToiNoiSpacing.md),
              Text('${value.serviceKind.label}, ${value.distanceKm} km'),
              MoneyText(value.fare),
            ],
          ),
          AsyncError(:final error) when error is TripNotFound => Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('Không tìm thấy chuyến ${error.id}.'),
              const SizedBox(height: ToiNoiSpacing.md),
              FilledButton(
                key: const Key('back-to-history'),
                onPressed: () => context.go(AppPaths.history),
                child: const Text('Về lịch sử'),
              ),
            ],
          ),
          AsyncError(:final error) => Text('Không tải được chuyến: $error'),
          _ => const Center(child: CircularProgressIndicator()),
        },
      ),
    );
  }
}
