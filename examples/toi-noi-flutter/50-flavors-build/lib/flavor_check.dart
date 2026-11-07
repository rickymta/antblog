/// Kết quả đối chiếu hai nguồn cho biết bản build này thuộc flavor nào.
enum FlavorCheck {
  /// Build bằng `--flavor` và file cấu hình trùng tên.
  consistent,

  /// Không có `--flavor`: bản build mang ID mặc định của runner, không có flavor native.
  nativeMissing,

  /// Có `--flavor` nhưng không có khoá `FLAVOR` từ `--dart-define-from-file`.
  configMissing,

  /// Hai nguồn nói hai flavor khác nhau: ID của `prod` nhưng cấu hình của `dev`, hoặc ngược lại.
  mismatch,
}

/// [native] là `appFlavor` (công cụ Flutter đặt từ `--flavor`), [config] là khoá
/// `FLAVOR` trong file JSON. Chuỗi rỗng và `null` đều là "không đặt".
FlavorCheck checkFlavors({required String? native, required String? config}) {
  final nativeFlavor = (native == null || native.isEmpty) ? null : native;
  final configFlavor = (config == null || config.isEmpty) ? null : config;
  if (nativeFlavor == null) return FlavorCheck.nativeMissing;
  if (configFlavor == null) return FlavorCheck.configMissing;
  return nativeFlavor == configFlavor
      ? FlavorCheck.consistent
      : FlavorCheck.mismatch;
}
