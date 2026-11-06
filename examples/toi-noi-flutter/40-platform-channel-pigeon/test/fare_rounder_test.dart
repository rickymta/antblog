import 'dart:ffi';
import 'dart:io';
import 'dart:math';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_40/toi_noi_40.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Kết quả biên dịch thư viện C bằng `cc` của máy.
class _Build {
  _Build.ok(this.path) : skipReason = null, failure = null;
  _Build.skipped(this.skipReason) : path = null, failure = null;
  _Build.failed(this.failure) : path = null, skipReason = null;

  final String? path;
  final String? skipReason;
  final String? failure;
}

_Build _buildNativeLibrary(Directory out) {
  if (!Platform.isMacOS && !Platform.isLinux) {
    return _Build.skipped('chỉ kiểm bằng cc trên macOS và Linux');
  }
  final extension = Platform.isMacOS ? 'dylib' : 'so';
  final output = '${out.path}/libtoi_noi_round_fare.$extension';
  final ProcessResult result;
  try {
    result = Process.runSync('cc', [
      Platform.isMacOS ? '-dynamiclib' : '-shared',
      '-fPIC',
      '-O2',
      '-o',
      output,
      'native/src/toi_noi_round_fare.c',
    ]);
  } on ProcessException {
    // Không có trình biên dịch: bỏ qua có lý do, không làm đỏ CI.
    return _Build.skipped('máy không có trình biên dịch C (cc)');
  }
  if (result.exitCode != 0) {
    // Có cc mà thoát khác 0: mã C sai, hoặc toolchain hỏng (thiếu SDK, chưa
    // chấp nhận giấy phép Xcode). Cả hai đều phải đỏ; stderr cho biết là gì.
    return _Build.failed('cc thoát ${result.exitCode}: ${result.stderr}');
  }
  return _Build.ok(output);
}

Money _fare(double km) => calculateFare(km);

void main() {
  const dart = DartFareRounder();

  group('DartFareRounder (bản chuẩn, không cần native)', () {
    test('cước tính bằng calculateFare, bước 500', () {
      expect(_fare(2.001), const Money(12005));
      expect(dart.round(_fare(2.001), step: 500), const Money(12000));
      expect(_fare(3.3), const Money(17850));
      expect(dart.round(_fare(3.3), step: 500), const Money(18000));
    });

    test('nửa bước làm tròn lên: 5 km = 25.500, bước 1.000 ra 26.000', () {
      expect(_fare(5), const Money(25500));
      expect(dart.round(_fare(5)), const Money(26000));
    });

    test('đã là bội số thì giữ nguyên: 100 km = 453.000', () {
      expect(_fare(100), const Money(453000));
      expect(dart.round(_fare(100)), const Money(453000));
      expect(dart.round(Money.zero), Money.zero);
    });

    test('step ngoài 1..2147483647 ném ArgumentError', () {
      for (final step in [0, -1, 0x80000000, 4294967297]) {
        expect(
          () => dart.round(const Money(1000), step: step),
          throwsArgumentError,
        );
      }
    });

    test('kết quả vượt int64 ném ArgumentError', () {
      expect(
        () => dart.round(const Money(0x7fffffffffffffff), step: 1000),
        throwsArgumentError,
      );
    });
  });

  final out = Directory.systemTemp.createTempSync('toi_noi_40_');
  final build = _buildNativeLibrary(out);
  tearDownAll(() => out.deleteSync(recursive: true));

  group(
    'FfiFareRounder (thư viện C biên dịch bằng cc)',
    skip: build.skipReason,
    () {
      late FfiFareRounder ffi;

      setUpAll(() {
        if (build.failure != null) fail(build.failure!);
        ffi = FfiFareRounder(DynamicLibrary.open(build.path!));
      });

      test('cùng kết quả với các ca của bản Dart', () {
        expect(ffi.round(_fare(2.001), step: 500), const Money(12000));
        expect(ffi.round(_fare(3.3), step: 500), const Money(18000));
        expect(ffi.round(_fare(5)), const Money(26000));
        expect(ffi.round(_fare(100)), const Money(453000));
      });

      test('5.000 cặp ngẫu nhiên (hạt giống cố định) giống bản Dart', () {
        final random = Random(40);
        const steps = [1, 2, 3, 7, 100, 500, 1000, 5000, 65536, 0x7fffffff];
        for (var i = 0; i < 5000; i++) {
          final fare = Money(random.nextInt(1000000000));
          final step = steps[random.nextInt(steps.length)];
          expect(
            ffi.round(fare, step: step),
            dart.round(fare, step: step),
            reason: 'vnd=${fare.vnd} step=$step',
          );
        }
      });

      test('đầu vào sai: lớp bọc ném ArgumentError trước khi gọi C', () {
        expect(
          () => ffi.round(const Money(1000), step: 0),
          throwsArgumentError,
        );
      });

      test('hàm C tự trả -1 với đầu vào sai và khi tràn int64', () {
        expect(ffi.callRawForTest(-1, 500), -1);
        expect(ffi.callRawForTest(1000, 0), -1);
        expect(ffi.callRawForTest(0x7fffffffffffffff, 1000), -1);
        expect(
          () => ffi.round(const Money(0x7fffffffffffffff), step: 1000),
          throwsArgumentError,
        );
      });

      test('Int32 bị cắt bit âm thầm: step 2^32 + 1 thành step 1', () {
        // Đây là lý do lớp bọc phải kiểm step trước: FFI không báo lỗi.
        expect(ffi.callRawForTest(12005, 4294967297), 12005);
      });

      test('thư viện không có symbol: lookup ném ArgumentError', () {
        final libc = DynamicLibrary.open(
          Platform.isMacOS ? '/usr/lib/libSystem.B.dylib' : 'libc.so.6',
        );
        expect(() => FfiFareRounder(libc), throwsArgumentError);
      });
    },
  );
}
