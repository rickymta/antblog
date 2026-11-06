import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Một vị trí tài xế báo về. Thời điểm là UTC.
final class DriverPosition {
  const DriverPosition({
    required this.lat,
    required this.lng,
    required this.at,
  });

  final double lat;
  final double lng;
  final DateTime at;

  @override
  bool operator ==(Object other) =>
      other is DriverPosition &&
      other.lat == lat &&
      other.lng == lng &&
      other.at == at;

  @override
  int get hashCode => Object.hash(lat, lng, at);

  @override
  String toString() => 'DriverPosition($lat, $lng)';
}

/// Nguồn vị trí tài xế. Bài 36 cắm WebSocket vào đây; test cắm
/// `StreamController`.
abstract interface class DriverLocationSource {
  Stream<DriverPosition> watch(String driverId);
}

/// Quá lâu không có vị trí mới thì coi là mất tín hiệu (minh hoạ).
const positionStaleAfter = Duration(seconds: 10);

/// Phải được ghi đè bằng nguồn thật hoặc nguồn giả.
final driverLocationSourceProvider = Provider<DriverLocationSource>(
  (ref) => throw UnimplementedError('ghi đè driverLocationSourceProvider'),
);

/// Vị trí mới nhất của một tài xế.
///
/// Giữa hai vị trí mà im quá [positionStaleAfter] thì state thành `AsyncError`
/// mang `TimeoutException` (và vẫn giữ vị trí cũ); có vị trí mới thì quay về
/// `AsyncData`. Bỏ provider là huỷ subscription tới nguồn.
///
/// Tắt `retry`: mất tín hiệu không phải lý do để Riverpod tự nghe lại nguồn từ
/// đầu (mặc định nó sẽ làm vậy với mọi `Exception`, kể cả `TimeoutException`).
/// Nối lại kết nối là việc của nguồn (bài 36).
final driverPositionProvider = StreamProvider.autoDispose
    .family<DriverPosition, String>(
      (ref, driverId) => ref
          .watch(driverLocationSourceProvider)
          .watch(driverId)
          .timeout(positionStaleAfter),
      retry: (_, _) => null,
    );
