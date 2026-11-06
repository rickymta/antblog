import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Thứ khách đang chọn trên màn hình đặt chuyến, chưa gửi đi đâu.
///
/// Bất biến và có `==`: [TripDraftNotifier] chỉ báo cho widget khi bản nháp
/// mới thật sự khác bản cũ.
final class TripDraft {
  const TripDraft({required this.serviceKind, required this.distanceKm});

  final ServiceKind serviceKind;
  final double distanceKm;

  TripDraft copyWith({ServiceKind? serviceKind, double? distanceKm}) =>
      TripDraft(
        serviceKind: serviceKind ?? this.serviceKind,
        distanceKm: distanceKm ?? this.distanceKm,
      );

  @override
  bool operator ==(Object other) =>
      other is TripDraft &&
      other.serviceKind == serviceKind &&
      other.distanceKm == distanceKm;

  @override
  int get hashCode => Object.hash(serviceKind, distanceKm);

  @override
  String toString() => 'TripDraft(${serviceKind.name}, $distanceKm km)';
}

/// State đồng bộ của bản nháp. `build` trả về giá trị khởi đầu; mọi thay đổi
/// đi qua method của notifier.
class TripDraftNotifier extends Notifier<TripDraft> {
  @override
  TripDraft build() =>
      const TripDraft(serviceKind: ServiceKind.ride, distanceKm: 5);

  void selectService(ServiceKind kind) {
    state = state.copyWith(serviceKind: kind);
  }

  /// Đặt quãng đường; ném [ArgumentError] khi ngoài miền 0..100 km của bảng
  /// giá (kể cả NaN) và giữ nguyên bản nháp.
  void setDistanceKm(double km) {
    final isValid = km >= 0 && km <= FarePolicy.maxKm;
    if (!isValid) {
      throw ArgumentError.value(
        km,
        'km',
        'phải nằm trong khoảng 0..${FarePolicy.maxKm} km',
      );
    }
    state = state.copyWith(distanceKm: km);
  }
}

final tripDraftProvider = NotifierProvider<TripDraftNotifier, TripDraft>(
  TripDraftNotifier.new,
);
