/// Giá trị lớn nhất Google Play cho phép của `versionCode`
/// (developer.android.com/studio/publish/versioning, kiểm tại 11/2026).
const maxVersionCode = 2100000000;

/// Build number của một run: `run_number + offset`, giống phép tính trong
/// bước "Compute build number" của workflow mẫu.
///
/// `run_number` tăng với mỗi run mới của cùng một workflow và KHÔNG đổi khi
/// chạy lại (re-run) cùng run, nên hai lần chạy lại cho cùng một số. [offset]
/// dành cho trường hợp store đã nhận các build số cao hơn từ trước.
///
/// Ném [RangeError] khi kết quả ngoài `1..maxVersionCode`, và
/// [ArgumentError] khi đầu vào âm.
int computeBuildNumber({required int runNumber, int offset = 0}) {
  if (runNumber < 0 || offset < 0) {
    throw ArgumentError('runNumber và offset phải không âm');
  }
  final value = runNumber + offset;
  if (value < 1 || value > maxVersionCode) {
    throw RangeError.range(value, 1, maxVersionCode, 'buildNumber');
  }
  return value;
}

/// `true` nếu [numbers] không bao giờ giảm theo thứ tự thời gian (cho phép
/// bằng nhau: run chạy lại).
bool isNonDecreasing(Iterable<int> numbers) {
  int? previous;
  for (final n in numbers) {
    if (previous != null && n < previous) return false;
    previous = n;
  }
  return true;
}

/// `true` nếu [numbers] tăng ngặt: điều store cần giữa hai bản phát hành.
bool isStrictlyIncreasing(Iterable<int> numbers) {
  int? previous;
  for (final n in numbers) {
    if (previous != null && n <= previous) return false;
    previous = n;
  }
  return true;
}
