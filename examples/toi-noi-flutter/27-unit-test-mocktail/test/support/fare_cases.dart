import 'package:toi_noi_core/toi_noi_core.dart';

typedef FareFn = int Function(double km);

/// Ca nào được chọn theo kỹ thuật nào.
enum CaseKind {
  /// Đại diện ở giữa một phân vùng tương đương.
  partition,

  /// Giá trị sát một biên giữa hai phân vùng.
  boundary,
}

/// Một ca kiểm tra cước. [expected] là `null` nghĩa là phải bị từ chối bằng
/// [ArgumentError].
final class FareCase {
  const FareCase(this.kind, this.name, this.km, this.expected);

  final CaseKind kind;
  final String name;
  final double km;
  final int? expected;
}

/// Bốn phân vùng của quãng đường theo bảng giá: ngoài miền dưới, đồng giá
/// 0 đến 2 km, tính theo km vượt (2 đến 100 km], ngoài miền trên; thêm lớp
/// "không phải số". Giá trị kỳ vọng tính tay từ bảng giá, không gọi
/// `calculateFare`.
const fareCases = <FareCase>[
  // Đại diện ở giữa từng phân vùng.
  FareCase(CaseKind.partition, 'ngoài miền dưới: -5 km', -5, null),
  FareCase(CaseKind.partition, 'đồng giá: 1 km', 1, 12000),
  FareCase(CaseKind.partition, 'km vượt: 5 km', 5, 25500),
  FareCase(CaseKind.partition, 'km vượt: 50 km', 50, 228000),
  FareCase(CaseKind.partition, 'ngoài miền trên: 150 km', 150, null),
  FareCase(CaseKind.partition, 'không phải số: NaN', double.nan, null),
  // Sát từng biên.
  FareCase(CaseKind.boundary, 'ngay dưới 0: -0,001 km', -0.001, null),
  FareCase(CaseKind.boundary, 'biên dưới: 0 km', 0, 12000),
  FareCase(CaseKind.boundary, 'hết đồng giá: 2 km', 2, 12000),
  FareCase(CaseKind.boundary, 'vượt 1 mét: 2,001 km', 2.001, 12005),
  FareCase(CaseKind.boundary, 'biên trên: 100 km', 100, 453000),
  FareCase(CaseKind.boundary, 'ngay trên 100: 100,001 km', 100.001, null),
  FareCase(CaseKind.boundary, 'vô cực dương', double.infinity, null),
  FareCase(CaseKind.boundary, 'vô cực âm', double.negativeInfinity, null),
];

/// Chạy [cases] qua [fn], trả tên các ca mà [fn] làm sai.
///
/// Một ca sai khi giá khác kỳ vọng, hoặc khi kỳ vọng là từ chối mà [fn] không
/// ném đúng [ArgumentError] (ném kiểu khác cũng tính là sai).
List<String> fareFailures(FareFn fn, Iterable<FareCase> cases) {
  final failed = <String>[];
  for (final c in cases) {
    int? actual;
    Object? error;
    try {
      actual = fn(c.km);
    } catch (e) {
      error = e;
    }
    final ok = c.expected == null
        ? error is ArgumentError
        : error == null && actual == c.expected;
    if (!ok) {
      failed.add(c.name);
    }
  }
  return failed;
}

/// Cước theo cách tính độc lập với `calculateFareVnd`: mét nguyên và phép
/// chia số nguyên, không có `double` ở bước nhân tiền. Phần vượt tính
/// `extra * 4,5` làm tròn nửa lên, tức `(extra * 9 + 1) ~/ 2`.
int oracleFareVnd(int meters) =>
    FarePolicy.baseFareVnd +
    (meters <= FarePolicy.includedMeters
        ? 0
        : ((meters - FarePolicy.includedMeters) * 9 + 1) ~/ 2);
