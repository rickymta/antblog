import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_40/toi_noi_40.dart';

void main() {
  test('fromMap đọc đúng payload native gửi lên', () {
    final status = BatteryStatus.fromMap(<Object?, Object?>{
      'levelPercent': 42,
      'isCharging': true,
      'isPowerSaveMode': false,
    });
    expect(
      status,
      const BatteryStatus(
        levelPercent: 42,
        isCharging: true,
        isPowerSaveMode: false,
      ),
    );
    expect(BatteryStatus.fromMap(status.toMap()), status);
  });

  test('biên 0 và 100 hợp lệ, -1 và 101 bị từ chối', () {
    for (final level in [0, 100]) {
      expect(
        BatteryStatus.checked(
          levelPercent: level,
          isCharging: false,
          isPowerSaveMode: false,
        ).levelPercent,
        level,
      );
    }
    for (final level in [-1, 101]) {
      expect(
        () => BatteryStatus.checked(
          levelPercent: level,
          isCharging: false,
          isPowerSaveMode: false,
        ),
        throwsFormatException,
      );
    }
  });

  test('thiếu khoá hoặc sai kiểu ném FormatException', () {
    expect(
      () => BatteryStatus.fromMap(<Object?, Object?>{'levelPercent': 50}),
      throwsFormatException,
    );
    expect(
      () => BatteryStatus.fromMap(<Object?, Object?>{
        'levelPercent': '50',
        'isCharging': false,
        'isPowerSaveMode': false,
      }),
      throwsFormatException,
    );
    expect(
      () => BatteryStatus.fromMap(<Object?, Object?>{
        'levelPercent': 50.0,
        'isCharging': false,
        'isPowerSaveMode': false,
      }),
      throwsFormatException,
    );
  });
}
