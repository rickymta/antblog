/// Kết quả phân loại tập file thay đổi trước khi chọn đường phát hành.
enum PatchVerdict {
  /// Chỉ đổi code Dart (và test): ứng viên cho patch.
  patchCandidate,

  /// Có đổi cấu hình dependency hoặc asset: cần người xem; CLI của công cụ
  /// là nơi quyết định cuối cùng.
  needsReview,

  /// Có đổi code native hoặc cấu hình runner: phải phát hành bản mới qua store.
  needsStoreRelease,
}

/// Bộ lọc **bảo thủ**, dựa trên đường dẫn, chạy trước `shorebird patch` để
/// chọn đường. Không thay thế công cụ: `shorebird patch` so artifact thật của
/// bản build với bản phát hành và cảnh báo khi có khác biệt native hay asset.
abstract final class PatchPreflight {
  static PatchVerdict classify(Iterable<String> changedPaths) {
    var verdict = PatchVerdict.patchCandidate;
    for (final raw in changedPaths) {
      final path = raw.replaceAll(r'\', '/');
      final next = _classify(path);
      if (next.index > verdict.index) verdict = next;
    }
    return verdict;
  }

  static PatchVerdict _classify(String path) {
    final segments = path.split('/');
    const native = {'android', 'ios', 'macos', 'linux', 'windows'};
    if (segments.any(native.contains)) return PatchVerdict.needsStoreRelease;
    final isDart =
        path.endsWith('.dart') &&
        (segments.contains('lib') || segments.contains('test'));
    return isDart ? PatchVerdict.patchCandidate : PatchVerdict.needsReview;
  }
}
