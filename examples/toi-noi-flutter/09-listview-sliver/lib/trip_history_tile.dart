import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Định dạng `dd/MM/yyyy HH:mm` từ [time] (đã ở giờ địa phương).
///
/// Viết tay để chưa cần `intl`; bài 14 sẽ thay bằng định dạng theo locale.
String formatTripTime(DateTime time) {
  String two(int n) => n.toString().padLeft(2, '0');
  return '${two(time.day)}/${two(time.month)}/${time.year} '
      '${two(time.hour)}:${two(time.minute)}';
}

IconData _iconOf(ServiceKind kind) => switch (kind) {
  ServiceKind.ride => Icons.two_wheeler,
  ServiceKind.food => Icons.ramen_dining,
  ServiceKind.parcel => Icons.inventory_2_outlined,
};

/// Một dòng lịch sử. Chạm vào dòng để mở hoặc gập chi tiết.
///
/// Đây là `StatefulWidget` vì nó phải nhớ một điều mà dữ liệu chuyến không
/// chứa: dòng đang mở hay gập. Chi tiết về `State` ở bài 10.
class TripHistoryTile extends StatefulWidget {
  const TripHistoryTile({super.key, required this.trip, this.onPin});

  final Trip trip;

  /// Khác null thì chi tiết có nút "Ghim lên đầu".
  final VoidCallback? onPin;

  @override
  State<TripHistoryTile> createState() => _TripHistoryTileState();
}

class _TripHistoryTileState extends State<TripHistoryTile> {
  bool _expanded = false;

  @override
  Widget build(BuildContext context) {
    final trip = widget.trip;
    final completed = trip.status == TripStatus.completed;
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        ListTile(
          leading: Icon(_iconOf(trip.serviceKind)),
          title: Text('${trip.serviceKind.label} · ${trip.id}'),
          subtitle: Text(
            '${formatTripTime(trip.requestedAt.toLocal())} · '
            '${trip.distanceKm} km',
          ),
          trailing: Text(completed ? trip.fare.toString() : trip.status.label),
          onTap: () => setState(() => _expanded = !_expanded),
        ),
        if (_expanded)
          Padding(
            padding: const EdgeInsets.fromLTRB(72, 0, 16, 8),
            child: Row(
              children: [
                Expanded(
                  child: Text(
                    'Tài xế: ${trip.driverId ?? 'chưa có'}\n'
                    'Trạng thái: ${trip.status.label}',
                  ),
                ),
                if (widget.onPin case final onPin?)
                  TextButton(
                    onPressed: onPin,
                    child: const Text('Ghim lên đầu'),
                  ),
              ],
            ),
          ),
      ],
    );
  }
}
