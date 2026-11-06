import 'dart:math';

import 'package:toi_noi_core/toi_noi_core.dart';

/// Một chuyến kèm số phiên bản, giống `Trip.Version` của service .NET.
///
/// `version` tăng một đơn vị sau mỗi lệnh thay đổi chuyến (ở đây chỉ có nhận
/// chuyến), dùng cho `expectedVersion` khi tài xế nhận.
final class StoredTrip {
  const StoredTrip(this.trip, this.version);

  final Trip trip;
  final int version;
}

/// Biên nhận của một lần đặt chuyến, khoá bằng `Idempotency-Key`.
final class Booking {
  const Booking({
    required this.serviceKind,
    required this.distanceMeters,
    required this.tripId,
  });

  final ServiceKind serviceKind;
  final int distanceMeters;
  final String tripId;
}

/// Kho trong bộ nhớ của fake API. Mỗi `createFakeApiHandler` có một kho riêng
/// (hoặc nhận kho do test truyền vào để kiểm tra sau khi gọi).
final class FakeApiStore {
  final Map<String, StoredTrip> _trips = {};
  final Map<String, Booking> _bookings = {};

  /// Các chuyến đã tạo, theo thứ tự tạo.
  List<StoredTrip> get trips => List.unmodifiable(_trips.values);

  StoredTrip? trip(String id) => _trips[id];

  void putTrip(StoredTrip stored) => _trips[stored.trip.id] = stored;

  Booking? booking(String key) => _bookings[key];

  void putBooking(String key, Booking booking) => _bookings[key] = booking;
}

/// Sinh mã dạng UUID v4 từ [random]. Truyền `Random(seed)` để có dãy lặp lại.
String Function() uuidGenerator([Random? random]) {
  final rng = random ?? Random.secure();
  return () {
    final bytes = List<int>.generate(16, (_) => rng.nextInt(256));
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // phiên bản 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // biến thể RFC 4122
    final hex = bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return '${hex.substring(0, 8)}-${hex.substring(8, 12)}-'
        '${hex.substring(12, 16)}-${hex.substring(16, 20)}-'
        '${hex.substring(20)}';
  };
}
