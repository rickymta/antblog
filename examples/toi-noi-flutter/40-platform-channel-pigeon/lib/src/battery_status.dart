import 'package:meta/meta.dart';

/// Ảnh chụp pin của máy tài xế tại một thời điểm.
@immutable
final class BatteryStatus {
  const BatteryStatus({
    required this.levelPercent,
    required this.isCharging,
    required this.isPowerSaveMode,
  }) : assert(levelPercent >= 0 && levelPercent <= 100);

  /// Kiểm dữ liệu từ native: native là mã khác, Dart không được tin nó.
  ///
  /// Ném [FormatException] khi mức pin ngoài 0..100.
  factory BatteryStatus.checked({
    required int levelPercent,
    required bool isCharging,
    required bool isPowerSaveMode,
  }) {
    if (levelPercent < 0 || levelPercent > 100) {
      throw FormatException('Mức pin ngoài 0..100', levelPercent);
    }
    return BatteryStatus(
      levelPercent: levelPercent,
      isCharging: isCharging,
      isPowerSaveMode: isPowerSaveMode,
    );
  }

  /// Dựng từ `Map` mà `StandardMessageCodec` giải mã được từ native.
  ///
  /// Ném [FormatException] khi thiếu khoá, sai kiểu hoặc mức pin ngoài 0..100.
  factory BatteryStatus.fromMap(Map<Object?, Object?> map) {
    final level = map['levelPercent'];
    final charging = map['isCharging'];
    final powerSave = map['isPowerSaveMode'];
    if (level is! int || charging is! bool || powerSave is! bool) {
      throw FormatException('Payload pin sai khoá hoặc sai kiểu', map);
    }
    return BatteryStatus.checked(
      levelPercent: level,
      isCharging: charging,
      isPowerSaveMode: powerSave,
    );
  }

  final int levelPercent;
  final bool isCharging;
  final bool isPowerSaveMode;

  Map<String, Object> toMap() => <String, Object>{
    'levelPercent': levelPercent,
    'isCharging': isCharging,
    'isPowerSaveMode': isPowerSaveMode,
  };

  @override
  bool operator ==(Object other) =>
      other is BatteryStatus &&
      other.levelPercent == levelPercent &&
      other.isCharging == isCharging &&
      other.isPowerSaveMode == isPowerSaveMode;

  @override
  int get hashCode => Object.hash(levelPercent, isCharging, isPowerSaveMode);

  @override
  String toString() =>
      'BatteryStatus($levelPercent%, charging: $isCharging, '
      'powerSave: $isPowerSaveMode)';
}
