import 'dart:math' show max;

import 'package:test/test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/fare_cases.dart';

/// Công thức cước viết lại từ đầu, mỗi tham số là một chỗ có thể viết sai.
/// Với mặc định nó đúng bằng `calculateFareVnd`.
FareFn variant({
  bool Function(double km)? accepts,
  int includedMeters = FarePolicy.includedMeters,
  int pricePerKm = FarePolicy.pricePerKmVnd,
  int roundingBias = 500,
}) {
  final inRange = accepts ?? (km) => km >= 0 && km <= FarePolicy.maxKm;
  return (km) {
    if (!inRange(km)) {
      throw ArgumentError.value(km, 'km');
    }
    final extra = max(0, (km * 1000).round() - includedMeters);
    return FarePolicy.baseFareVnd + (extra * pricePerKm + roundingBias) ~/ 1000;
  };
}

/// Mỗi mục là một lỗi viết công thức có thật: bộ test nào bắt được nó?
final mutants = <String, FareFn>{
  'biên trên loại trừ (< thay <=)': variant(
    accepts: (km) => km >= 0 && km < FarePolicy.maxKm,
  ),
  'trần nhầm 99 km': variant(accepts: (km) => km >= 0 && km <= 99),
  'bỏ cộng 500 khi làm tròn': variant(roundingBias: 0),
  'guard viết dạng phủ định (NaN lọt)': variant(
    accepts: (km) => !(km < 0 || km > FarePolicy.maxKm),
  ),
  'giá km vượt nhầm 4.000': variant(pricePerKm: 4000),
};

/// Đây không phải test của code sản phẩm mà là test của **bộ ca**: ta cố ý cài
/// lỗi và đếm xem ca nào bắt được.
void main() {
  final partitions = fareCases.where((c) => c.kind == CaseKind.partition);
  final boundaries = fareCases.where((c) => c.kind == CaseKind.boundary);

  test('công thức viết lại với tham số mặc định qua mọi ca', () {
    expect(fareFailures(variant(), fareCases), isEmpty);
  });

  test('calculateFareVnd thật qua mọi ca', () {
    expect(fareFailures(calculateFareVnd, fareCases), isEmpty);
  });

  test('chỉ dùng đại diện giữa phân vùng thì lọt 3 trong 5 lỗi', () {
    final survivors = [
      for (final e in mutants.entries)
        if (fareFailures(e.value, partitions).isEmpty) e.key,
    ];
    expect(survivors, [
      'biên trên loại trừ (< thay <=)',
      'trần nhầm 99 km',
      'bỏ cộng 500 khi làm tròn',
    ]);
  });

  test('chỉ dùng ca biên thì lọt 1 lỗi: NaN không có biên', () {
    final survivors = [
      for (final e in mutants.entries)
        if (fareFailures(e.value, boundaries).isEmpty) e.key,
    ];
    expect(survivors, ['guard viết dạng phủ định (NaN lọt)']);
  });

  test('hợp hai kỹ thuật thì không lỗi nào lọt', () {
    for (final e in mutants.entries) {
      expect(
        fareFailures(e.value, fareCases),
        isNotEmpty,
        reason: 'lỗi "${e.key}" lọt qua cả hai bộ ca',
      );
    }
  });

  test('lỗi cắm đúng ca: biên trên chỉ bị ca 100 km bắt', () {
    final failed = fareFailures(mutants.values.first, fareCases);
    expect(failed, ['biên trên: 100 km']);
  });
}
