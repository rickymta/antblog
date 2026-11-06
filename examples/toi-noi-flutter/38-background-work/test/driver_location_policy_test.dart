import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_38/driver_location_policy.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

void main() {
  const policy = DriverLocationPolicy();

  // Bảng đặc tả viết tay, độc lập với cài đặt: bốn trạng thái có tài xế đang
  // làm việc với khách thì được theo dõi, bốn trạng thái còn lại thì không.
  const expectedTracked = <TripStatus, bool>{
    TripStatus.requested: false,
    TripStatus.searching: false,
    TripStatus.accepted: true,
    TripStatus.arriving: true,
    TripStatus.pickedUp: true,
    TripStatus.inTransit: true,
    TripStatus.completed: false,
    TripStatus.cancelled: false,
  };

  test('bảng đặc tả phủ đủ tám trạng thái của toi_noi_core', () {
    expect(TripStatus.values, hasLength(8));
    expect(expectedTracked.keys.toSet(), TripStatus.values.toSet());
  });

  group('tài xế online', () {
    for (final status in TripStatus.values) {
      final expected = expectedTracked[status]!;
      test('${status.name} -> ${expected ? 'được gửi' : 'không gửi'}', () {
        expect(
          policy.allowsBackgroundSharing(
            tripStatus: status,
            driverOnline: true,
          ),
          expected,
        );
        expect(policy.tracksStatus(status), expected);
      });
    }
  });

  group('tài xế offline: không trạng thái nào được gửi', () {
    for (final status in TripStatus.values) {
      test(status.name, () {
        expect(
          policy.allowsBackgroundSharing(
            tripStatus: status,
            driverOnline: false,
          ),
          isFalse,
        );
      });
    }
  });

  test('chưa có chuyến (null) thì không gửi, dù online', () {
    expect(
      policy.allowsBackgroundSharing(tripStatus: null, driverOnline: true),
      isFalse,
    );
  });

  test('trạng thái cuối (completed, cancelled) không bao giờ được gửi', () {
    for (final status in TripStatus.values.where((s) => s.isTerminal)) {
      expect(policy.tracksStatus(status), isFalse, reason: status.name);
    }
  });

  test('chỉ trạng thái đã có tài xế mới được theo dõi', () {
    // Chuyến chưa có tài xế (requested, searching) không có vị trí nào để chia sẻ.
    final tracked = TripStatus.values.where(policy.tracksStatus);
    expect(tracked.toSet(), {
      TripStatus.accepted,
      TripStatus.arriving,
      TripStatus.pickedUp,
      TripStatus.inTransit,
    });
  });
}
