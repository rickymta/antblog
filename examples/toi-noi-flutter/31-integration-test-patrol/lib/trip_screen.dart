import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'booking_controller.dart';
import 'keys.dart';
import 'notification_permission.dart';
import 'providers.dart';

class TripScreen extends ConsumerWidget {
  const TripScreen({super.key, required this.placed});

  final BookingPlaced placed;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final id = placed.snapshot.trip.id;
    final live = ref.watch(tripProvider(id));
    return Scaffold(
      appBar: AppBar(title: const Text('Chuyến của bạn')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          // AsyncError vẫn giữ `.value` cũ, nên phải xét lỗi trước.
          switch (live) {
            AsyncError() => _PollingError(
              onRetry: () => ref.invalidate(tripProvider(id)),
            ),
            _ => _TripDetails(trip: (live.value ?? placed.snapshot).trip),
          },
          const SizedBox(height: 24),
          const _NotificationSection(),
          const SizedBox(height: 24),
          TextButton(
            onPressed: () => ref.read(bookingProvider.notifier).startOver(),
            child: const Text('Đặt chuyến khác'),
          ),
        ],
      ),
    );
  }
}

class _TripDetails extends StatelessWidget {
  const _TripDetails({required this.trip});

  final Trip trip;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final waiting = isWaitingForDriver(trip.status);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Mã chuyến', style: theme.textTheme.labelMedium),
        Text(trip.id, key: AppKeys.tripIdText),
        const SizedBox(height: 8),
        Text('Giá: ${trip.fare}'),
        const SizedBox(height: 16),
        Text(
          trip.status.label,
          key: AppKeys.tripStatusText,
          style: theme.textTheme.titleLarge,
        ),
        if (waiting) ...[
          const SizedBox(height: 8),
          const Text('Đang tìm tài xế gần bạn...'),
          const SizedBox(height: 8),
          const LinearProgressIndicator(),
        ],
        if (trip.driverId case final driverId?) ...[
          const SizedBox(height: 8),
          Text('Tài xế: $driverId', key: AppKeys.driverText),
        ],
      ],
    );
  }
}

class _PollingError extends StatelessWidget {
  const _PollingError({required this.onRetry});

  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      const Text(
        'Không cập nhật được trạng thái chuyến.',
        key: AppKeys.errorText,
      ),
      TextButton(onPressed: onRetry, child: const Text('Thử lại')),
    ],
  );
}

class _NotificationSection extends ConsumerWidget {
  const _NotificationSection();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final status = ref.watch(notificationProvider).value;
    final text = switch (status) {
      NotificationPermissionStatus.granted => 'Thông báo: đã bật',
      NotificationPermissionStatus.denied => 'Thông báo: chưa bật',
      null => 'Thông báo: đang kiểm tra',
    };
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(text, key: AppKeys.notificationStatusText),
        if (status != NotificationPermissionStatus.granted)
          FilledButton.tonal(
            key: AppKeys.enableNotificationsButton,
            onPressed: () => ref.read(notificationProvider.notifier).request(),
            child: const Text('Bật thông báo'),
          ),
      ],
    );
  }
}
