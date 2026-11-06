import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Hai bố cục của [TripCard].
enum TripCardLayout { compact, wide }

/// Thẻ chuyến: biểu tượng dịch vụ, hai địa chỉ, trạng thái và cước.
///
/// Bố cục chọn theo bề rộng **thẻ được cấp** (constraints từ cha), không theo
/// bề rộng màn hình: cùng một thẻ đặt trong ô 320 px trên máy tính bảng vẫn
/// dùng bố cục `compact`.
class TripCard extends StatelessWidget {
  const TripCard({
    super.key,
    required this.trip,
    required this.pickup,
    required this.dropoff,
  });

  /// Từ bề rộng này trở lên dùng bố cục `wide` (số minh hoạ của lab).
  static const double wideBreakpoint = 600;

  /// Key để test nhận ra bố cục đang dùng.
  static const compactKey = Key('trip-card-compact');
  static const wideKey = Key('trip-card-wide');

  final Trip trip;
  final String pickup;
  final String dropoff;

  /// Bố cục ứng với bề rộng cho phép [maxWidth].
  static TripCardLayout layoutFor(double maxWidth) =>
      maxWidth >= wideBreakpoint ? TripCardLayout.wide : TripCardLayout.compact;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final layout = layoutFor(constraints.maxWidth);
        return Card(
          margin: EdgeInsets.zero,
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: switch (layout) {
              TripCardLayout.compact => _CompactBody(this),
              TripCardLayout.wide => _WideBody(this),
            },
          ),
        );
      },
    );
  }
}

/// Điện thoại: hai hàng. Hàng trên là biểu tượng + hành trình; hàng dưới là
/// trạng thái + cước trong `Wrap`, đủ chỗ thì nằm cùng hàng, thiếu chỗ (chữ
/// lớn) thì cước xuống dòng thay vì tràn.
///
/// `stretch` cho mỗi con bề rộng chặt bằng cột. Với `start`, `Wrap` nhận
/// ràng buộc lỏng, co theo nội dung và `spaceBetween` không còn tác dụng.
class _CompactBody extends StatelessWidget {
  const _CompactBody(this.card);

  final TripCard card;

  @override
  Widget build(BuildContext context) {
    return Column(
      key: TripCard.compactKey,
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _ServiceBadge(trip: card.trip),
            const SizedBox(width: 12),
            Expanded(
              child: _Journey(
                trip: card.trip,
                pickup: card.pickup,
                dropoff: card.dropoff,
              ),
            ),
          ],
        ),
        const SizedBox(height: 12),
        Wrap(
          alignment: WrapAlignment.spaceBetween,
          crossAxisAlignment: WrapCrossAlignment.center,
          spacing: 12,
          runSpacing: 8,
          children: [
            _StatusLabel(status: card.trip.status),
            _FareBlock(trip: card.trip),
          ],
        ),
      ],
    );
  }
}

/// Máy tính bảng, xoay ngang: một hàng. Hành trình nhận phần còn lại.
class _WideBody extends StatelessWidget {
  const _WideBody(this.card);

  final TripCard card;

  @override
  Widget build(BuildContext context) {
    return Row(
      key: TripCard.wideKey,
      children: [
        _ServiceBadge(trip: card.trip),
        const SizedBox(width: 16),
        Expanded(
          child: _Journey(
            trip: card.trip,
            pickup: card.pickup,
            dropoff: card.dropoff,
          ),
        ),
        const SizedBox(width: 16),
        _StatusLabel(status: card.trip.status),
        const SizedBox(width: 24),
        _FareBlock(trip: card.trip),
      ],
    );
  }
}

/// Biểu tượng dịch vụ với chấm trạng thái ở góc: `Stack` + `Positioned`.
class _ServiceBadge extends StatelessWidget {
  const _ServiceBadge({required this.trip});

  final Trip trip;

  static IconData iconOf(ServiceKind kind) => switch (kind) {
    ServiceKind.ride => Icons.directions_car,
    ServiceKind.food => Icons.restaurant,
    ServiceKind.parcel => Icons.inventory_2,
  };

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return SizedBox(
      width: 44,
      height: 44,
      child: Stack(
        children: [
          Align(
            alignment: Alignment.center,
            child: CircleAvatar(
              radius: 20,
              backgroundColor: scheme.primaryContainer,
              child: Icon(
                iconOf(trip.serviceKind),
                color: scheme.onPrimaryContainer,
              ),
            ),
          ),
          Positioned(
            right: 0,
            bottom: 0,
            child: Container(
              key: const Key('status-dot'),
              width: 12,
              height: 12,
              decoration: BoxDecoration(
                color: statusColor(trip.status, scheme),
                shape: BoxShape.circle,
                border: Border.all(color: scheme.surface, width: 2),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// Màu theo trạng thái. Liệt kê đủ tám, không dùng `_`.
Color statusColor(TripStatus status, ColorScheme scheme) => switch (status) {
  TripStatus.requested || TripStatus.searching => scheme.tertiary,
  TripStatus.accepted ||
  TripStatus.arriving ||
  TripStatus.pickedUp ||
  TripStatus.inTransit => scheme.primary,
  TripStatus.completed => scheme.secondary,
  TripStatus.cancelled => scheme.error,
};

/// Tiêu đề chuyến và hai địa chỉ. Nằm trong `Expanded` nên có bề rộng hữu hạn.
class _Journey extends StatelessWidget {
  const _Journey({
    required this.trip,
    required this.pickup,
    required this.dropoff,
  });

  final Trip trip;
  final String pickup;
  final String dropoff;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          '${trip.serviceKind.label} · ${trip.id}',
          style: Theme.of(context).textTheme.titleSmall,
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
        ),
        const SizedBox(height: 4),
        _AddressLine(icon: Icons.trip_origin, text: pickup),
        const SizedBox(height: 2),
        _AddressLine(icon: Icons.place, text: dropoff),
      ],
    );
  }
}

class _AddressLine extends StatelessWidget {
  const _AddressLine({required this.icon, required this.text});

  final IconData icon;
  final String text;

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 2),
          child: Icon(icon, size: 16),
        ),
        const SizedBox(width: 8),
        Expanded(
          child: Text(text, maxLines: 2, overflow: TextOverflow.ellipsis),
        ),
      ],
    );
  }
}

class _StatusLabel extends StatelessWidget {
  const _StatusLabel({required this.status});

  final TripStatus status;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return DecoratedBox(
      decoration: BoxDecoration(
        color: statusColor(status, scheme).withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        child: Text(
          status.label,
          style: Theme.of(context).textTheme.labelMedium
              ?.copyWith(color: statusColor(status, scheme)),
        ),
      ),
    );
  }
}

class _FareBlock extends StatelessWidget {
  const _FareBlock({required this.trip});

  final Trip trip;

  @override
  Widget build(BuildContext context) {
    final textTheme = Theme.of(context).textTheme;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.end,
      children: [
        Text(
          trip.fare.toString(),
          style: textTheme.titleMedium?.copyWith(fontWeight: FontWeight.w600),
        ),
        Text(formatKm(trip.distanceKm), style: textTheme.bodySmall),
      ],
    );
  }
}

/// `5.0` thành `"5 km"`, `2.001` thành `"2,001 km"` (dấu phẩy thập phân).
/// Bài về `intl` sẽ thay hàm này bằng định dạng theo locale.
String formatKm(double km) {
  final text = km == km.roundToDouble() ? km.toInt().toString() : '$km';
  return '${text.replaceAll('.', ',')} km';
}
