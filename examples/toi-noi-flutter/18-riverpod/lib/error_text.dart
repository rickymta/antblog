import 'quote_repository.dart';

/// Câu hiển thị cho người dùng; không bao giờ đưa `toString()` của lỗi thô
/// lên giao diện.
String describeError(Object error) => switch (error) {
  QuoteUnavailable() => 'Dịch vụ báo giá đang bận',
  TripNotFound(:final id) => 'Không tìm thấy chuyến $id',
  _ => 'Có lỗi không mong đợi',
};
