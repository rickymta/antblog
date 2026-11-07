/// Ranh giới tới plugin mở link ngoài app. Test dùng bản giả.
abstract interface class UrlOpener {
  /// Trả `true` nếu hệ điều hành nhận yêu cầu mở. Có thể ném lỗi.
  Future<bool> open(Uri uri);
}
