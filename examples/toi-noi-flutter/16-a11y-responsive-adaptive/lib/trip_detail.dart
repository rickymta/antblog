import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'accessible.dart';
import 'platform_adaptive.dart';

/// Nội dung chi tiết một chuyến, không có `Scaffold`: dùng được cả trong
/// trang riêng (điện thoại) lẫn khung bên phải (máy tính bảng).
class TripDetailView extends StatelessWidget {
  const TripDetailView({
    super.key,
    required this.trip,
    required this.onCancelled,
  });

  final Trip trip;

  /// Gọi với chuyến đã chuyển sang `cancelled` sau khi khách xác nhận.
  final ValueChanged<Trip> onCancelled;

  Future<void> _cancel(BuildContext context) async {
    final confirmed = await confirmCancelTrip(context, trip);
    // Hộp thoại mở bằng await: màn hình có thể đã bị gỡ trong lúc chờ.
    if (!confirmed || !context.mounted) return;
    onCancelled(trip.apply(const Cancelled(by: CancelledBy.customer)));
  }

  @override
  Widget build(BuildContext context) {
    final textTheme = Theme.of(context).textTheme;
    return ListView(
      padding: const EdgeInsets.all(ToiNoiSpacing.md),
      children: [
        Text(trip.serviceKind.label, style: textTheme.headlineSmall),
        const SizedBox(height: ToiNoiSpacing.sm),
        Align(
          alignment: Alignment.centerLeft,
          child: AccessibleStatus(trip.status, liveRegion: true),
        ),
        const SizedBox(height: ToiNoiSpacing.md),
        Text('Mã chuyến ${trip.id}'),
        Text('Quãng đường ${formatKm(trip.distanceKm)}'),
        Text('Tài xế: ${trip.driverId ?? 'chưa có'}'),
        const SizedBox(height: ToiNoiSpacing.md),
        AccessibleMoney(
          trip.fare,
          prefix: 'Cước',
          style: textTheme.headlineSmall,
        ),
        const SizedBox(height: ToiNoiSpacing.lg),
        if (trip.status.canCancel)
          OutlinedButton(
            onPressed: () => _cancel(context),
            child: const Text('Huỷ chuyến'),
          ),
      ],
    );
  }
}

/// Trang chi tiết đẩy lên `Navigator` ở bố cục một cột.
///
/// Giữ bản sao chuyến trong `State` để sau khi huỷ trang tự cập nhật, và báo
/// ngược cho nơi mở qua [onChanged].
class TripDetailPage extends StatefulWidget {
  const TripDetailPage({
    super.key,
    required this.trip,
    required this.onChanged,
  });

  final Trip trip;
  final ValueChanged<Trip> onChanged;

  @override
  State<TripDetailPage> createState() => _TripDetailPageState();
}

class _TripDetailPageState extends State<TripDetailPage> {
  late Trip _trip = widget.trip;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('Chuyến ${_trip.id}')),
      body: SafeArea(
        child: TripDetailView(
          trip: _trip,
          onCancelled: (cancelled) {
            setState(() => _trip = cancelled);
            widget.onChanged(cancelled);
          },
        ),
      ),
    );
  }
}
