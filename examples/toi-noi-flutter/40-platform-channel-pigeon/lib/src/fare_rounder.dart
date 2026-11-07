import 'dart:ffi';

import 'package:toi_noi_core/toi_noi_core.dart';

/// Làm tròn cước để hiển thị hoặc thu tiền mặt.
abstract interface class FareRounder {
  /// Làm tròn [fare] tới bội số gần nhất của [step] (nửa bước làm tròn lên).
  ///
  /// Ném [ArgumentError] khi [step] ngoài `1..2147483647` hoặc kết quả vượt
  /// `int64`.
  Money round(Money fare, {int step = 1000});
}

const int _maxInt32 = 0x7fffffff;
const int _maxInt64 = 0x7fffffffffffffff;

void _checkStep(int step) {
  if (step < 1 || step > _maxInt32) {
    throw ArgumentError.value(step, 'step', 'phải nằm trong 1..$_maxInt32');
  }
}

/// Bản Dart thuần: dùng khi không có thư viện native, và làm chuẩn so sánh
/// cho bản FFI.
final class DartFareRounder implements FareRounder {
  const DartFareRounder();

  @override
  Money round(Money fare, {int step = 1000}) {
    _checkStep(step);
    final rest = fare.vnd % step;
    final down = fare.vnd - rest;
    if (rest * 2 < step) return Money(down);
    if (down > _maxInt64 - step) {
      throw ArgumentError.value(fare, 'fare', 'kết quả vượt int64');
    }
    return Money(down + step);
  }
}

typedef _RoundFareNative = Int64 Function(Int64 vnd, Int32 step);
typedef _RoundFareDart = int Function(int vnd, int step);

/// Gọi `toi_noi_round_fare` trong thư viện C đã nạp bằng [DynamicLibrary].
final class FfiFareRounder implements FareRounder {
  /// Ném [ArgumentError] nếu [library] không có symbol `toi_noi_round_fare`.
  FfiFareRounder(DynamicLibrary library)
    : _round = library
          .lookup<NativeFunction<_RoundFareNative>>('toi_noi_round_fare')
          .asFunction<_RoundFareDart>(isLeaf: true);

  final _RoundFareDart _round;

  /// Gọi thẳng hàm C, không kiểm gì: chỉ để test chứng minh việc Dart không
  /// tự kiểm kiểu số nguyên khi truyền qua FFI.
  int callRawForTest(int vnd, int step) => _round(vnd, step);

  @override
  Money round(Money fare, {int step = 1000}) {
    // `Int32` ở phía C: số lớn hơn bị cắt bit chứ không báo lỗi, nên Dart
    // phải kiểm trước khi gọi.
    _checkStep(step);
    final result = _round(fare.vnd, step);
    if (result < 0) {
      throw ArgumentError.value(fare, 'fare', 'native từ chối đầu vào');
    }
    return Money(result);
  }
}
