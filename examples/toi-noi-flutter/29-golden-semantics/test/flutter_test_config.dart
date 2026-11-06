import 'dart:async';

import 'package:flutter_test/flutter_test.dart';

import 'support/golden_harness.dart';
import 'support/tolerant_comparator.dart';

/// Chạy trước mọi file test trong thư mục này (và thư mục con).
///
/// Hai việc: nạp font icon thật, và thay comparator mặc định (khớp từng
/// pixel) bằng bản có ngưỡng nhỏ, vẫn đọc ảnh chuẩn cạnh file test như
/// mặc định.
Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  TestWidgetsFlutterBinding.ensureInitialized();
  await loadMaterialIcons();

  final current = goldenFileComparator;
  if (current is LocalFileComparator) {
    goldenFileComparator = TolerantGoldenComparator(
      current.basedir.resolve('golden_test.dart'),
    );
  }
  await testMain();
}
