import 'dart:ffi';

/// Làm tròn [vnd] tới bội số gần nhất của [step] (nửa bước làm tròn lên).
///
/// Trả -1 khi [vnd] âm, [step] không dương hoặc kết quả vượt `int64`; lớp bọc
/// ở lab chính đổi -1 thành [ArgumentError].
///
/// Không có đường dẫn thư viện ở đây: assetId mặc định là URI của file này,
/// khớp `assetName` trong `hook/build.dart`. `symbol` là tên hàm trong C;
/// bỏ nó đi thì Dart tìm symbol tên `toiNoiRoundFare` và không thấy.
@Native<Int64 Function(Int64, Int32)>(
  symbol: 'toi_noi_round_fare',
  isLeaf: true,
)
external int toiNoiRoundFare(int vnd, int step);
